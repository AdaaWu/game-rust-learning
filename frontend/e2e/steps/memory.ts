import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'

const squash = (text: string | null) => (text ?? '').replace(/\s+/g, '')

function mem(world: World) {
  return world.page.getByTestId('memory-view')
}

// 標籤去掉旁邊的圖示（👁、✎）
const labelText = (text: string | null) => (text ?? '').replace(/[👁✎]/gu, '').replace(/\s+/g, ' ').trim()

Step(/^說明列顯示「(.+)」(?:與 "(.+)")?$/, async ({ world }, caption: string, note: string | undefined) => {
  await expect(mem(world).getByTestId('mem-caption')).toHaveText(caption)
  if (note) await expect(mem(world).getByTestId('mem-note')).toHaveText(note)
})

Step(/^說明列中 "(.+)" 以粗體顯示$/, async ({ world }, text: string) => {
  await expect(mem(world).getByTestId('mem-note').locator('strong, b', { hasText: text })).toBeVisible()
})

Step(/^Stack 區有 ([0-9]+) 個 frame "(.+)"，含變數 "(.+)"，值 "(.+)"$/, async ({ world }, n: string, fn: string, variable: string, value: string) => {
  const frames = mem(world).getByTestId('mem-frame')
  await expect(frames).toHaveCount(Number(n))
  await expect(frames.first().locator('.mem-frame-name')).toHaveText(fn)
  const v = frames.first().getByTestId('mem-var').first()
  const head = squash(await v.locator('.mem-var-name').innerText()) + squash(await v.locator('.mem-var-type').innerText())
  expect(head).toBe(squash(variable))
  await expect(v.locator('.mem-var-value')).toContainText(value)
})

Step(/^Heap 區有 ([0-9]+) 個區塊，型別 "(.+)"，值 "(.+)"$/, async ({ world }, n: string, type: string, value: string) => {
  const blocks = mem(world).getByTestId('mem-heap')
  await expect(blocks).toHaveCount(Number(n))
  await expect(blocks.first().locator('.mem-var-type')).toHaveText(type)
  await expect(blocks.first().locator('.mem-block-value')).toHaveText(value)
})

Step(/^Stack 區由上而下為 (.+)$/, async ({ world }, list: string) => {
  const names = [...list.matchAll(/"([^"]+)"/g)].map((m) => m[1])
  await expect(mem(world).locator('[data-testid=mem-frame] .mem-frame-name')).toHaveText(names)
})

function frame(world: World, fn: string) {
  return mem(world).getByTestId('mem-frame').filter({ has: world.page.locator('.mem-frame-name', { hasText: new RegExp(`^${fn}$`) }) })
}

Step(/^frame "(.+)" 為加粗外框，frame "(.+)" 不是$/, async ({ world }, bold: string, normal: string) => {
  const width = (fn: string) => frame(world, fn).evaluate((el) => parseFloat(getComputedStyle(el).borderTopWidth))
  await expect(frame(world, bold)).toHaveClass(/is-current/)
  await expect(frame(world, normal)).not.toHaveClass(/is-current/)
  expect(await width(bold)).toBeGreaterThan(await width(normal))
})

Step(/^MemoryView 不是半透明(?:，也沒有顯示「(.+)」)?$/, async ({ world }, text: string | undefined) => {
  const opacity = await mem(world).evaluate((el) => {
    let o = 1
    for (let e: Element | null = el; e; e = e.parentElement) o *= Number(getComputedStyle(e).opacity)
    return o
  })
  expect(opacity).toBe(1)
  if (text) await expect(world.visual()).not.toContainText(text)
})

// ---------- 變數與 heap ----------

Step(/^變數 "(.+)" 整格灰階並有斜線網底$/, async ({ world }, id: string) => {
  const s = await world.memVar(id).evaluate((el) => {
    const c = getComputedStyle(el)
    return { filter: c.filter, bg: c.backgroundImage }
  })
  expect(s.filter).toContain('grayscale(1)')
  expect(s.bg).toContain('repeating-linear-gradient')
})

Step(/^變數 "(.+)" 的名稱有刪除線$/, async ({ world }, id: string) => {
  const deco = await world.memVar(id).locator('.mem-var-name').evaluate((el) => getComputedStyle(el).textDecorationLine)
  expect(deco).toContain('line-through')
})

Step(/^變數 "(.+)" 右側有徽章「(.+)」$/, async ({ world }, id: string, badge: string) => {
  await expect(world.memVar(id).locator('.mem-badge', { hasText: badge })).toBeVisible()
})

Step(/^heap 區塊 "(.+)" 為虛線外框、透明度 40%，並有徽章「(.+)」$/, async ({ world }, id: string, badge: string) => {
  const block = mem(world).locator(`[data-testid=mem-heap][data-id="${id}"]`)
  const s = await block.evaluate((el) => {
    const c = getComputedStyle(el)
    return { style: c.borderTopStyle, opacity: c.opacity }
  })
  expect(s.style).toBe('dashed')
  expect(Number(s.opacity)).toBeCloseTo(0.4)
  await expect(block.locator('.mem-badge', { hasText: badge })).toBeVisible()
})

Step(/^Heap 區沒有區塊 "(.+)"$/, async ({ world }, id: string) => {
  await expect(mem(world).locator(`[data-testid=mem-heap][data-id="${id}"]`)).toHaveCount(0)
})

// ---------- 指標 ----------

async function pointerStyle(world: World, from: string, to: string) {
  const p = world.pointer(from, to)
  await expect(p).toHaveCount(1)
  return p.evaluate((el) => {
    const path = el.querySelector('path')!
    const s = getComputedStyle(path)
    const markerUrl = path.getAttribute('marker-end') ?? ''
    const id = /#([^)]+)/.exec(markerUrl)?.[1] ?? ''
    const head = document.getElementById(id)?.querySelector('path')
    const hs = head ? getComputedStyle(head) : null
    return {
      width: parseFloat(s.strokeWidth),
      dash: s.strokeDasharray,
      stroke: s.stroke,
      headFilled: !!hs && hs.fill !== 'none',
      label: el.querySelector('text')?.textContent ?? '',
    }
  })
}

Step(/^從 "(.+)" 到 "(.+)" 有一條(.+)的(?:指標|虛線實心箭頭)$/, async ({ world }, from: string, to: string, desc: string) => {
  world.focusPointer = { from, to }
  const s = await pointerStyle(world, from, to)
  if (desc.includes('實線')) expect(s.dash).toBe('none')
  if (desc.includes('實心')) expect(s.headFilled).toBe(true)
  if (desc.includes('空心')) expect(s.headFilled).toBe(false)
  if (desc.includes('細')) expect(s.width).toBeLessThan(2)
  const w = /線寬 ([0-9]+)px/.exec(desc)
  if (w) expect(s.width).toBe(Number(w[1]))
  const dash = /dash "(.+)"/.exec(desc)
  if (dash) expect(s.dash.replace(/px|,/g, '').replace(/\s+/g, ' ')).toBe(dash[1])
})

Step(/^該指標的線上標籤為 "(.+?)"(?:，旁邊有「(.+)」圖示)?$/, async ({ world }, label: string, icon: string | undefined) => {
  const { from, to } = world.focusPointer!
  const text = await world.pointer(from, to).getByTestId('mem-pointer-label').textContent()
  expect(labelText(text)).toBe(label)
  if (icon) expect(text).toContain(icon)
})

Step(/^從 "(.+)" 到 "(.+)" 的指標為 --danger 色$/, async ({ world }, from: string, to: string) => {
  world.focusPointer = { from, to }
  const s = await pointerStyle(world, from, to)
  expect(s.stroke).toBe(await world.cssVarColor('--danger'))
})

Step('該指標仍為細實線、空心箭頭', async ({ world }) => {
  const { from, to } = world.focusPointer!
  const s = await pointerStyle(world, from, to)
  expect(s.dash).toBe('none')
  expect(s.width).toBeLessThan(2)
  expect(s.headFilled).toBe(false)
})

Step('其他指標都不是 --danger 色', async ({ world }) => {
  const danger = await world.cssVarColor('--danger')
  const { from, to } = world.focusPointer!
  const others = await mem(world)
    .getByTestId('mem-pointer')
    .evaluateAll((els) => els.map((el) => ({ from: el.getAttribute('data-from'), to: el.getAttribute('data-to'), stroke: getComputedStyle(el.querySelector('path')!).stroke })))
  for (const o of others) if (!(o.from === from && o.to === to)) expect(o.stroke).not.toBe(danger)
})

Step(/^每一條指標都有文字標籤，且標籤屬於 "(.+)"$/, async ({ world }, list: string) => {
  const allowed = list.split(',').map((s) => s.trim())
  const labels = await mem(world).getByTestId('mem-pointer-label').allTextContents()
  expect(labels.length).toBeGreaterThan(0)
  expect(await mem(world).getByTestId('mem-pointer').count()).toBe(labels.length)
  for (const l of labels) expect(allowed).toContain(labelText(l))
})

// ---------- 借用標示 ----------

Step(/^變數 "(.+)" 為(橘|藍)色(粗|細)外框(.*)$/, async ({ world }, id: string, color: string, weight: string, rest: string) => {
  const v = world.memVar(id)
  const s = await v.evaluate((el) => {
    const c = getComputedStyle(el)
    return { color: c.borderTopColor, width: parseFloat(c.borderTopWidth) }
  })
  expect(s.color).toBe(await world.cssVarColor(color === '橘' ? '--borrow-mut' : '--borrow-shared'))
  if (weight === '粗') expect(s.width).toBeGreaterThanOrEqual(2.5)
  else expect(s.width).toBeLessThan(2.5)
  const lock = /「(.+)」圖示與文字「(.+)」/.exec(rest)
  if (lock) {
    await expect(v).toContainText(lock[1])
    await expect(v).toContainText(lock[2])
  }
  const badge = /右上角有「(.+)」徽章/.exec(rest)
  if (badge) {
    const b = v.locator('.mem-badge', { hasText: badge[1] })
    await expect(b).toBeVisible()
    const [vb, bb] = [await v.boundingBox(), await b.boundingBox()]
    expect(bb!.x + bb!.width).toBeGreaterThan(vb!.x + vb!.width * 0.6)
  }
})

Step(/^變數 "(.+)" 顯示「(.+)」$/, async ({ world }, id: string, text: string) => {
  await expect(world.memVar(id)).toContainText(text)
})

// ---------- 錯誤標記 ----------

Step(/^"(.+)" 的外框為 --danger 色、3px 粗實線$/, async ({ world }, id: string) => {
  const s = await world.memTarget(id).evaluate((el) => {
    const c = getComputedStyle(el)
    return { color: c.borderTopColor, width: c.borderTopWidth, style: c.borderTopStyle }
  })
  expect(s).toEqual({ color: await world.cssVarColor('--danger'), width: '3px', style: 'solid' })
})

Step(/^"(.+)" 有「✗」圖示與徽章「(.+)」$/, async ({ world }, id: string, badge: string) => {
  const target = world.memTarget(id)
  await expect(target).toContainText('✗')
  const texts = await target.getByTestId('mem-error-badge').allInnerTexts()
  expect(texts.map((t) => t.replace('✗', '').trim())).toEqual([badge])
})

// 錯誤目標框與箭頭重疊處，最上層的元素屬於錯誤目標
Step('錯誤標記圖層位於所有指標箭頭之上', async ({ world }) => {
  const result = await mem(world).evaluate((root) => {
    const target = root.querySelector<HTMLElement>('.mem-var.is-error, .mem-block.is-error')
    if (!target) return { error: 'no target' }
    target.scrollIntoView({ block: 'center' })
    const t = target.getBoundingClientRect()
    const b = root.querySelector('.mem-board')!.getBoundingClientRect()
    // 只取目標框在可視區域內的部分
    const rect = { left: Math.max(t.left, b.left), right: Math.min(t.right, b.right), top: Math.max(t.top, b.top), bottom: Math.min(t.bottom, b.bottom) }
    const svg = root.querySelector<SVGElement>('.mem-arrows')!
    const paths = Array.from(root.querySelectorAll<SVGPathElement>('[data-testid=mem-pointer] path'))
    const prev = svg.style.pointerEvents
    svg.style.pointerEvents = 'auto'
    paths.forEach((p) => (p.style.pointerEvents = 'stroke'))
    let checked = 0
    let covered = 0
    for (const p of paths) {
      const len = p.getTotalLength()
      const ctm = p.getScreenCTM()!
      for (let d = 0; d <= len; d += 2) {
        const pt = p.getPointAtLength(d).matrixTransform(ctm)
        if (pt.x <= rect.left + 2 || pt.x >= rect.right - 2 || pt.y <= rect.top + 2 || pt.y >= rect.bottom - 2) continue
        checked++
        const top = document.elementFromPoint(pt.x, pt.y)
        if (!top || !target.contains(top)) covered++
      }
    }
    svg.style.pointerEvents = prev
    paths.forEach((p) => (p.style.pointerEvents = ''))
    return { checked, covered }
  })
  expect(result).not.toHaveProperty('error')
  expect(result.covered).toBe(0)
})

Step(/^變數 "(.+)" 同時有徽章「(.+)」與「(.+)」$/, async ({ world }, id: string, a: string, b: string) => {
  const badges = (await world.memVar(id).locator('.mem-badge').allInnerTexts()).map((t) => t.replace('✗', '').trim())
  expect(badges).toContain(a)
  expect(badges).toContain(b)
})

Step(/^變數 "(.+)" 仍有斜線網底與刪除線$/, async ({ world }, id: string) => {
  const s = await world.memVar(id).evaluate((el) => ({
    bg: getComputedStyle(el).backgroundImage,
    deco: getComputedStyle(el.querySelector('.mem-var-name')!).textDecorationLine,
  }))
  expect(s.bg).toContain('repeating-linear-gradient')
  expect(s.deco).toContain('line-through')
})

Step('MemoryView 中沒有 --danger 色外框、「✗」圖示或錯誤碼徽章', async ({ world }) => {
  const danger = await world.cssVarColor('--danger')
  const colors = await mem(world)
    .locator('[data-testid=mem-var], [data-testid=mem-heap]')
    .evaluateAll((els) => els.map((el) => getComputedStyle(el).borderTopColor))
  expect(colors).not.toContain(danger)
  await expect(mem(world).locator('.mem-board')).not.toContainText('✗')
  await expect(mem(world).getByTestId('mem-error-badge')).toHaveCount(0)
})

// ---------- 步驟變化提示 ----------

Step(/^frame "(.+)" 與變數 "(.+)" 的外框閃一次（300ms）$/, async ({ world }, fn: string, id: string) => {
  for (const el of [frame(world, fn), world.memVar(id)]) {
    const s = await el.evaluate((e) => {
      const c = getComputedStyle(e)
      return { name: c.animationName, duration: c.animationDuration, count: c.animationIterationCount }
    })
    expect(s.name).not.toBe('none')
    expect(s.duration).toBe('0.3s')
    expect(s.count).toBe('1')
  }
})

Step(/^變數 "(.+)" 旁有常駐小圓點「(.+)」$/, async ({ world, page }, id: string, text: string) => {
  const dot = world.memVar(id).getByTestId('mem-new-dot')
  await expect(dot).toHaveText(text)
  await page.waitForTimeout(600)
  await expect(dot).toBeVisible()
})

// ---------- 行號同步 ----------

Step(/^編輯器第 ([0-9]+) 行有整行背景高亮與(?:行號左側)?「▶」圖示$/, async ({ world, page }, line: string) => {
  await expect.poll(() => world.decoratedLines('rq-line-active')).toEqual([Number(line)])
  await expect.poll(() => world.decoratedLines('rq-line-glyph', true)).toEqual([Number(line)])
  await expect(page.locator('[data-testid=col-editor] .rq-line-active')).toBeAttached()
  await expect(page.locator('[data-testid=col-editor] .rq-line-glyph')).toBeAttached()
})

Step(/^編輯器第 ([0-9]+) 行沒有高亮與「▶」圖示$/, async ({ world }, line: string) => {
  expect(await world.decoratedLines('rq-line-active')).not.toContain(Number(line))
  expect(await world.decoratedLines('rq-line-glyph', true)).not.toContain(Number(line))
})

Step(/^編輯器只有 ([0-9]+) 行有行高亮$/, async ({ world }, n: string) => {
  expect(await world.decoratedLines('rq-line-active')).toHaveLength(Number(n))
})

Step('編輯器沒有任何行高亮與「▶」圖示', async ({ world, page }) => {
  await expect.poll(() => world.decoratedLines('rq-line-active')).toEqual([])
  expect(await world.decoratedLines('rq-line-glyph', true)).toEqual([])
  await expect(page.locator('[data-testid=col-editor] .rq-line-glyph')).toHaveCount(0)
})

Step('編輯器高度只能顯示 3 行，且已捲到第 1 行', async ({ page }) => {
  await page.evaluate(() => {
    const ed = window.__rqMonaco.editor.getEditors()[0]
    const host = document.querySelector<HTMLElement>('[data-testid=col-editor] .editor-host')!
    const lineHeight = ed.getOption(window.__rqMonaco.editor.EditorOption.lineHeight)
    host.style.flex = 'none'
    host.style.height = `${lineHeight * 3}px`
    ed.layout()
    ed.setScrollTop(0)
  })
  await page.waitForTimeout(100)
})

Step(/^玩家點編輯器第 ([0-9]+) 行行號左側的 ▶ 區域$/, async ({ world, page }, line: string) => {
  const p = await world.glyphPoint(Number(line))
  await page.mouse.click(p.x, p.y)
})

Step(/^說明列上方顯示黃色提示「(.+)」$/, async ({ world }, text: string) => {
  const note = mem(world).locator('.mem-modified')
  await expect(note).toHaveText(text)
  expect(await note.evaluate((el) => getComputedStyle(el).color)).toBe(await world.cssVarColor('--warning'))
  const [n, c] = [await note.boundingBox(), await mem(world).getByTestId('mem-caption').boundingBox()]
  expect(n!.y).toBeLessThan(c!.y)
})

Step(/^不顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.page.getByText(text)).toHaveCount(0)
})

// ---------- 版面 ----------

const STACK_THRESHOLD = 360

Step(/^可視化區寬度為 ([0-9]+)px$/, async ({ page }, width: string) => {
  // 桌機右欄佔 32fr、平板右欄佔 45fr：依版面反推視窗寬度
  const w = Number(width)
  const desktop = Math.round(w / 0.32 + 48)
  await page.setViewportSize({ width: desktop >= 1200 ? desktop : Math.round(w / 0.45 + 36), height: 800 })
})

async function stackHeap(world: World) {
  const width = await world.visual().evaluate((el) => el.getBoundingClientRect().width)
  const [s, h] = [await mem(world).locator('.mem-stack').boundingBox(), await mem(world).locator('.mem-heap').boundingBox()]
  return { width, s: s!, h: h! }
}

Step('Stack 區位於 Heap 區上方', async ({ world }) => {
  const { width, s, h } = await stackHeap(world)
  expect(width).toBeLessThan(STACK_THRESHOLD)
  expect(s.y + s.height).toBeLessThanOrEqual(h.y + 1)
})

Step('Stack 區與 Heap 區左右並排', async ({ world }) => {
  const { width, s, h } = await stackHeap(world)
  expect(width).toBeGreaterThanOrEqual(STACK_THRESHOLD)
  expect(s.x + s.width).toBeLessThanOrEqual(h.x)
  expect(Math.abs(s.y - h.y)).toBeLessThan(4)
})

Step(/^「單步」按鈕的 aria-label 為「(.+)」$/, async ({ world }, label: string) => {
  await expect(world.visual().locator('.pb-btn').nth(2)).toHaveAttribute('aria-label', label)
})
