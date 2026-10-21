import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import type { Locator } from '@playwright/test'
import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'
import { MVP_IDS, ok } from '../support/mock'
import type { MockApi } from '../support/mock'
import { runResponse } from '../support/responses'
import { setPlatform } from './common'

type Json = any

// ---------- 資料來源 ----------

const read = (rel: string) => readFileSync(fileURLToPath(new URL(rel, import.meta.url)), 'utf8')

const SEASON2 = JSON.parse(read('../fixtures/levels.json')).season2Timeline as {
  levels: Json[]
  details: Record<string, Json>
  season1Starter31: string
}

// ui_timeline.feature 檔尾附錄：「名稱」下方每行「# > 」之後為 stdout 原文，「#   stderr」之後為 stderr
function parseAppendix(text: string) {
  const stdout = new Map<string, string>()
  const stderr = new Map<string, string>()
  const lines = text.split('\n')
  let name: string | null = null
  let target: Map<string, string> | null = null
  let buf: string[] = []
  const flush = () => {
    if (name && target && buf.length > 0) target.set(name, buf.join('\n') + '\n')
    buf = []
  }
  for (const line of lines) {
    const title = /^# 「(.+?)」：/.exec(line)
    if (title) {
      flush()
      name = title[1]
      target = stdout
    } else if (/^#\s+stderr/.test(line)) {
      flush()
      target = stderr
    } else if (line.startsWith('# > ') || line === '# >') {
      buf.push(line.slice(4))
    } else {
      flush()
    }
  }
  flush()
  return { stdout, stderr }
}

const APPENDIX = parseAppendix(read('../../../features/ui_timeline.feature'))
// 體積大的兩個範例另存：對 127.0.0.1:3000 實際 Run 的輸出
const LARGE = JSON.parse(read('../fixtures/timeline-stdout.json')) as Record<string, string>
for (const [k, v] of Object.entries(LARGE)) APPENDIX.stdout.set(k, v)

function stdoutOf(name: string): string {
  const s = APPENDIX.stdout.get(name)
  if (s === undefined) throw new Error(`附錄沒有 stdout 範例「${name}」`)
  return s
}

const escapeRe = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
const NAMES = `(${[...APPENDIX.stdout.keys()].map(escapeRe).join('|')})`
const quotes = (text: string) => [...text.matchAll(/「([^」]+)」/g)].map((m) => m[1])
const names = (text: string) => [...text.matchAll(/"([^"]+)"/g)].map((m) => m[1])

function setRun(mock: MockApi, reply: Json, delayMs = 0) {
  mock.runReplies = [ok(reply)]
  mock.runDelayMs = delayMs
}

// 暫停頁面時鐘，自動播放停在回應後的第 1 步，不受主機快慢影響（同 ci/setup 的 pauseClock）
async function pauseClock(world: World) {
  await world.installClock()
  // pauseAt 不能回到過去，取頁面現在時間再往後一點，主機太慢時重取
  for (let i = 0; ; i++) {
    const now = await world.page.evaluate(() => Date.now())
    try {
      await world.page.clock.pauseAt(now + 200)
      return
    } catch (e) {
      if (i >= 4) throw e
    }
  }
}

const pauseBeforeRun = new WeakSet<World>()

// ---------- 畫面元素 ----------

function tl(world: World): Locator {
  return world.page.getByTestId('timeline-view')
}

function laneLabel(world: World, lane: string): Locator {
  return tl(world).locator(`[data-testid=tl-lane-label][data-lane="${lane}"]`)
}

async function laneNames(world: World): Promise<string[]> {
  return tl(world)
    .getByTestId('tl-lane-label')
    .evaluateAll((els) => els.map((el) => el.getAttribute('data-lane') ?? ''))
}

interface Geometry {
  colW: number
  lanes: { name: string; top: number; bottom: number }[]
  events: { step: number; kind: string; lane: string; icon: string; title: string; tabbable: boolean }[]
  segments: { lane: string; kind: string; target: string; deadlocked: boolean; layer: number; from: number; to: number | null; x1: number; x2: number; y: number }[]
  messages: { pending: boolean; from: { lane: string; step: number }; to: { lane: string; step: number } | null; text: string }[]
  arrows: { x1: number; y1: number; x2: number; y2: number; text: string }[]
  unfinished: string[]
}

// 從 SVG 座標反推步數與泳道：事件標記的 x 即該步的欄中心，泳道列的上下緣與標籤列對齊
async function geometry(world: World): Promise<Geometry> {
  return tl(world).evaluate((root) => {
    const svg = root.querySelector('svg.tl-svg')!
    // 泳道底色列依泳道順序排在 SVG 最前面
    const names = [...root.querySelectorAll('[data-testid=tl-lane-label]')].map((el) => el.getAttribute('data-lane') ?? '')
    const bands = [...svg.querySelectorAll(':scope > g:not([data-testid]) > rect')]
    const lanes = names.map((name, i) => {
      const top = Number(bands[i].getAttribute('y'))
      return { name, top, bottom: top + Number(bands[i].getAttribute('height')) }
    })
    const laneAt = (y: number) => lanes.find((l) => y >= l.top && y < l.bottom)?.name ?? ''
    const num = (el: Element | null, attr: string) => Number(el?.getAttribute(attr))
    const xs = new Map<number, number>()
    const events = [...root.querySelectorAll('[data-testid=tl-event]')].map((g) => {
      const shape = g.querySelector('circle') ?? g.querySelector('text')!
      const x = shape.tagName === 'circle' ? num(shape, 'cx') : num(shape, 'x')
      const y = shape.tagName === 'circle' ? num(shape, 'cy') : num(shape, 'y') - 4
      const step = Number(g.getAttribute('data-step'))
      xs.set(step, x)
      return {
        step,
        kind: g.getAttribute('data-kind') ?? '',
        lane: laneAt(y),
        icon: g.querySelector('text')?.textContent ?? '',
        title: g.querySelector('title')?.textContent ?? '',
        tabbable: g.getAttribute('tabindex') === '0',
      }
    })
    const colW = xs.has(2) ? xs.get(2)! - xs.get(1)! : 0
    const stepAt = (x: number) => {
      for (const [s, sx] of xs) if (Math.abs(sx - x) < 0.01) return s
      return null
    }
    const segments = [...root.querySelectorAll('[data-testid=tl-segment]')].map((g) => {
      const rect = g.querySelector('rect:not(.tl-dead-frame)')!
      const x1 = num(rect, 'x')
      const x2 = x1 + num(rect, 'width')
      return {
        lane: g.getAttribute('data-lane') ?? '',
        kind: g.getAttribute('data-kind') ?? '',
        target: g.getAttribute('data-target') ?? '',
        deadlocked: g.getAttribute('data-deadlocked') === 'true',
        layer: Number(g.getAttribute('data-layer')),
        from: stepAt(x1) ?? NaN,
        to: stepAt(x2),
        x1,
        x2,
        y: num(rect, 'y') + 6,
      }
    })
    const pathPoints = (d: string) => {
      const n = (d.match(/-?\d+(?:\.\d+)?/g) ?? []).map(Number)
      return { x1: n[0], y1: n[1], x2: n[n.length - 2], y2: n[n.length - 1] }
    }
    const messages = [...root.querySelectorAll('[data-testid=tl-message]')].map((g) => {
      const p = pathPoints(g.querySelector('path')!.getAttribute('d') ?? '')
      const pending = g.getAttribute('data-pending') === 'true'
      return {
        pending,
        from: { lane: laneAt(p.y1), step: stepAt(p.x1) ?? NaN },
        to: pending ? null : { lane: laneAt(p.y2), step: stepAt(p.x2) ?? NaN },
        text: g.textContent ?? '',
      }
    })
    const arrows = [...root.querySelectorAll('[data-testid=tl-deadlock-arrow]')].map((g) => ({
      ...pathPoints(g.querySelector('path')!.getAttribute('d') ?? ''),
      text: g.querySelector('text')?.textContent ?? '',
    }))
    const unfinished = [...root.querySelectorAll('[data-testid=tl-lane-unfinished]')].map((el) => el.getAttribute('data-lane') ?? '')
    return { colW, lanes, events, segments, messages, arrows, unfinished }
  })
}

const nullable = (v: string) => (v === 'null' ? null : Number(v))

async function laneOpacity(world: World, lane: string): Promise<number> {
  return laneLabel(world, lane).evaluate((el) => Number(getComputedStyle(el).opacity))
}

async function messageRowBox(world: World) {
  return (await tl(world).locator('.tl-message-row').boundingBox())!
}

// ---------- Background ----------

Step('mock API 回傳測試用 8 關 fixture 的關卡列表與詳細資料，並套用第二季 timeline 調整', async ({ world, mock }) => {
  const ids = SEASON2.levels.map((l) => l.id)
  const levels = [...mock.list.levels.filter((l: Json) => !ids.includes(l.id)), ...structuredClone(SEASON2.levels)]
  levels.sort((a: Json, b: Json) => a.order - b.order)
  mock.list = { ...mock.list, levels }
  for (const id of ids) mock.details[id] = structuredClone(SEASON2.details[id])
  await setPlatform(world, 'Windows')
  // 自動播放靠快轉時鐘等待
  await world.installClock()
  const clickRun = world.clickRun.bind(world)
  world.clickRun = async () => {
    if (pauseBeforeRun.delete(world)) await pauseClock(world)
    await clickRun()
  }
})

Step(/^玩家進度中測試用 8 關 fixture 與 30-send-the-crabs、33-boss-great-migration 都已拿到 ★1$/, async ({ world }) => {
  for (const id of [...MVP_IDS, '30-send-the-crabs', '33-boss-great-migration']) world.levelProgress(id).stars.pass = true
})

Step(/^玩家進度中 "([^"]+)" 已解鎖$/, async ({ world }, id: string) => {
  // 真實後端：排在該關之前的關卡都拿到 ★1
  const res = await world.page.request.get('http://127.0.0.1:3000/api/levels')
  const levels = (await res.json()).levels as Json[]
  const order = levels.find((l) => l.id === id).order
  for (const l of levels) if (l.order < order) world.levelProgress(l.id).stars.pass = true
})

Step(/^localStorage 的 "([^"]+)" 為第一季 31-shared-counter 的 starter\.rs 全文$/, async ({ world }, key: string) => {
  await world.setStorage(key, SEASON2.season1Starter31)
})

// ---------- mock Run ----------

Step(
  new RegExp(
    `^mock API 的 Run (?:改為延遲 ([0-9]+) 秒後)?回 status "(\\w+)"(?:（execution\\.timed_out 為 true）)?，stdout 為「${NAMES}」` +
      `(?:，stdout_truncated 為 (true))?(?:，stderr 為「(.+)」的 stderr)?$`,
  ),
  async ({ mock, world }, delay: string | undefined, status: string, name: string, truncated: string | undefined, stderrName: string | undefined) => {
    const body = runResponse(status, stdoutOf(name), truncated === 'true')
    if (stderrName) body.execution.stderr = APPENDIX.stderr.get(stderrName)
    if (status === 'timeout') expect(body.execution.timed_out).toBe(true)
    setRun(mock, body, delay ? Number(delay) * 1000 : 0)
    // 延遲回應的 Run：回來後要比對第 1 步，先停住時鐘
    if (delay) pauseBeforeRun.add(world)
  },
)

// ---------- 中央提示與泳道 ----------

Step(/^時間軸中央顯示「(.+)」$/, async ({ world }, text: string) => {
  await world.waitIdle()
  await expect(tl(world).getByTestId('tl-empty')).toHaveText(text.replace(/`/g, ''))
})

Step('時間軸沒有任何泳道', async ({ world }) => {
  await expect(tl(world).getByTestId('tl-lane-label')).toHaveCount(0)
})

Step(/^時間軸圖例下方顯示「(.+)」$/, async ({ world }, text: string) => {
  const note = tl(world).getByTestId('tl-legend').getByTestId('tl-order-note')
  await expect(note).toHaveText(text)
  await expect(note).toBeVisible()
})

Step('播放控制列為 disabled', async ({ world }) => {
  const bar = world.visual()
  for (const name of ['從頭', '播放', '單步前進']) await expect(bar.getByRole('button', { name, exact: true })).toBeDisabled()
  await expect(bar.getByLabel('進度')).toBeDisabled()
})

Step(/^泳道由上而下依序為 (".+")$/, async ({ world }, list: string) => {
  await expect.poll(() => laneNames(world)).toEqual(names(list))
})

Step(/^泳道第一條為 "([^"]+)"，其餘恰好為 (".+")（順序不限）$/, async ({ world }, first: string, list: string) => {
  const lanes = await laneNames(world)
  expect(lanes[0]).toBe(first)
  expect([...lanes.slice(1)].sort()).toEqual(names(list).sort())
})

Step(/^泳道標籤依序為：$/, async ({ world }, table: any) => {
  const rows = table.hashes() as Record<string, string>[]
  expect(await laneNames(world)).toEqual(rows.map((r) => r.lane))
  for (const r of rows) {
    const label = laneLabel(world, r.lane)
    await expect(label.locator('.tl-lane-icon')).toHaveText(r.icon)
    if (r.text) await expect(label.locator('.tl-lane-dead')).toHaveText(r.text)
    else await expect(label.locator('.tl-lane-dead')).toHaveCount(0)
  }
})

Step(/^泳道 "([^"]+)" 的標籤圖示為「([^」]+)」$/, async ({ world }, lane: string, icon: string) => {
  await expect(laneLabel(world, lane).locator('.tl-lane-icon')).toHaveText(icon)
})

Step(/^泳道 "([^"]+)" 的標籤圖示為「([^」]+)」，標籤旁顯示「(.+)」$/, async ({ world }, lane: string, icon: string, note: string) => {
  await expect(laneLabel(world, lane).locator('.tl-lane-icon')).toHaveText(icon)
  await expect(tl(world).locator(`[data-testid=tl-lane-note][data-lane="${lane}"]`)).toHaveText(note)
})

Step(/^泳道 "([^"]+)" 標籤旁沒有「(.+)」$/, async ({ world }, lane: string, note: string) => {
  await expect(tl(world).locator(`[data-testid=tl-lane-note][data-lane="${lane}"]`)).toHaveCount(0)
  await expect(laneLabel(world, lane).locator('..')).not.toContainText(note)
})

Step(/^泳道 (".+") 的標籤顯示「✗」與「死鎖」$/, async ({ world }, list: string) => {
  for (const lane of names(list)) {
    const label = laneLabel(world, lane)
    await expect(label.locator('.tl-lane-icon')).toHaveText('✗')
    await expect(label.locator('.tl-lane-dead')).toHaveText('死鎖')
  }
})

Step(/^泳道 "([^"]+)" 的標籤同時有「✗」圖示、「死鎖」文字與 "(--[\w-]+)" 文字色$/, async ({ world }, lane: string, color: string) => {
  const label = laneLabel(world, lane)
  await expect(label.locator('.tl-lane-icon')).toHaveText('✗')
  await expect(label.locator('.tl-lane-dead')).toHaveText('死鎖')
  expect(await label.evaluate((el) => getComputedStyle(el).color)).toBe(await world.cssVarColor(color))
})

Step(/^泳道標籤按鈕 "([^"]+)" 的 aria-label 為「(.+)」$/, async ({ world }, lane: string, label: string) => {
  const btn = laneLabel(world, lane)
  expect(await btn.evaluate((el) => el.tagName)).toBe('BUTTON')
  await expect(btn).toHaveAttribute('aria-label', label)
})

Step(/^泳道 "([^"]+)" 標籤的 title 為 "([^"]+)"，aria-label 含 "([^"]+)"$/, async ({ world }, lane: string, title: string, part: string) => {
  const label = laneLabel(world, lane)
  await expect(label).toHaveAttribute('title', title)
  expect(await label.getAttribute('aria-label')).toContain(part)
})

Step(/^(?:4 條泳道|泳道 (".+") )的標籤透明度(?:都)?(?:是|為) ([0-9]+)%$/, async ({ world }, list: string | undefined, pct: string) => {
  const lanes = list ? names(list) : await laneNames(world)
  if (!list) expect(lanes).toHaveLength(4)
  for (const lane of lanes) await expect.poll(() => laneOpacity(world, lane)).toBeCloseTo(Number(pct) / 100, 2)
})

Step(/^玩家點泳道標籤 "([^"]+)"$/, async ({ world }, lane: string) => {
  await laneLabel(world, lane).click()
})

// ---------- 訊息列與圖例 ----------

Step(/^時間軸訊息列第一行為「(.+)」$/, async ({ world }, text: string) => {
  await expect(tl(world).getByTestId('tl-sentence')).toHaveText(text)
})

Step(/^時間軸訊息列第二行為「(.+)」$/, async ({ world }, text: string) => {
  await expect(tl(world).getByTestId('tl-conclusion')).toHaveText(text)
})

Step(/^時間軸訊息列第二行(依序)?含(「.+」)$/, async ({ world }, ordered: string | undefined, list: string) => {
  const parts = quotes(list)
  const conclusion = tl(world).getByTestId('tl-conclusion')
  for (const p of parts) await expect(conclusion).toContainText(p)
  if (ordered) {
    const text = (await conclusion.textContent()) ?? ''
    const at = parts.map((p) => text.indexOf(p))
    expect(at).toEqual([...at].sort((a, b) => a - b))
  }
})

Step(/^時間軸圖例依序為：(.+)$/, async ({ world }, spec: string) => {
  const locks = [...spec.matchAll(/鎖代號「(.)」對 "([^"]+)"/g)].map((m) => `${m[1]} ${m[2]}`)
  const channels = [...spec.matchAll(/channel「(.)」對 "([^"]+)"/g)].map((m) => `${m[1]} ${m[2]}`)
  const texts = (id: string) => tl(world).getByTestId(id).evaluateAll((els) => els.map((el) => (el.textContent ?? '').trim()))
  expect(await texts('tl-legend-lock')).toEqual(locks)
  expect(await texts('tl-legend-channel')).toEqual(channels)
})

Step(/^時間軸圖例第二行含(「.+」)$/, async ({ world }, list: string) => {
  const row = tl(world).locator('.tl-legend-row').nth(1)
  for (const q of quotes(list)) await expect(row).toContainText(q)
})

Step(/^時間軸刻度標示 (".+")$/, async ({ world }, list: string) => {
  const ticks = await tl(world).getByTestId('tl-tick').allTextContents()
  expect(ticks).toEqual(names(list))
})

Step(/^「(.+)」顯示在時間軸訊息列下方$/, async ({ world }, text: string) => {
  const note = tl(world).getByTestId('tl-order-note')
  await expect(note).toHaveCount(1)
  await expect(note).toHaveText(text)
  await expect(tl(world).getByTestId('tl-legend').getByTestId('tl-order-note')).toHaveCount(0)
  const [n, row] = [(await note.boundingBox())!, await messageRowBox(world)]
  expect(n.y).toBeGreaterThanOrEqual(row.y + row.height - 1)
})

Step(/^時間軸訊息列上方顯示黃色提示「(.+)」$/, async ({ world }, text: string) => {
  const note = world.visual().locator('.visual-stale-note')
  await expect(note).toHaveText(text)
  expect(await note.evaluate((el) => getComputedStyle(el).color)).toBe(await world.cssVarColor('--warning'))
  expect((await note.boundingBox())!.y).toBeLessThan((await messageRowBox(world)).y)
})

Step(/^時間軸沒有顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(tl(world)).not.toContainText(text)
})

// ---------- 區段 ----------

Step(/^時間軸區段依泳道與起點排列為：$/, async ({ world }, table: any) => {
  const g = await geometry(world)
  const order = g.lanes.map((l) => l.name)
  const actual = g.segments
    .map((s) => ({ lane: s.lane, kind: s.kind, target: s.target, from: s.from, to: s.to }))
    .sort((a, b) => order.indexOf(a.lane) - order.indexOf(b.lane) || a.from - b.from)
  const expected = (table.hashes() as Record<string, string>[]).map((r) => ({ lane: r.lane, kind: r.kind, target: r.target, from: Number(r.from), to: nullable(r.to) }))
  expect(actual).toEqual(expected)
})

Step(/^deadlocked 的區段恰好為：$/, async ({ world }, table: any) => {
  const g = await geometry(world)
  const key = (s: { lane: string; kind: string; target: string; from: number }) => `${s.lane}|${s.kind}|${s.target}|${s.from}`
  const actual = g.segments.filter((s) => s.deadlocked).map(key).sort()
  const expected = (table.hashes() as Record<string, string>[]).map((r) => key({ ...r, from: Number(r.from) } as any)).sort()
  expect(actual).toEqual(expected)
})

Step(/^已結束的等待區段 (.+)不是 deadlocked$/, async ({ world }, spec: string) => {
  const g = await geometry(world)
  const items = [...spec.matchAll(/"([^"]+)" (\w+) (\S+ \d+)（第 (\d+)–(\d+) 步）/g)]
  expect(items.length).toBeGreaterThan(0)
  for (const [, lane, kind, target, from, to] of items) {
    const seg = g.segments.find((s) => s.lane === lane && s.kind === kind && s.target === target && s.from === Number(from))
    expect(seg, `${lane} ${kind} ${target} 第 ${from} 步`).toBeTruthy()
    expect(seg!.to).toBe(Number(to))
    expect(seg!.deadlocked).toBe(false)
  }
})

Step(/^泳道 "([^"]+)" 的區段都不是 deadlocked$/, async ({ world }, lane: string) => {
  const segs = (await geometry(world)).segments.filter((s) => s.lane === lane)
  expect(segs.length).toBeGreaterThan(0)
  expect(segs.every((s) => !s.deadlocked)).toBe(true)
})

Step(/^每個 deadlocked 區段都有 3px "(--[\w-]+)" 實線外框與條尾 ✗，且保留原本的填色或虛線$/, async ({ world }, color: string) => {
  const danger = await world.cssVarColor(color)
  const segs = tl(world).locator('[data-testid=tl-segment][data-deadlocked=true]')
  expect(await segs.count()).toBeGreaterThan(0)
  for (const seg of await segs.all()) {
    const frame = await seg.locator('.tl-dead-frame').evaluate((el) => {
      const s = getComputedStyle(el)
      return { stroke: s.stroke, width: s.strokeWidth, dash: s.strokeDasharray }
    })
    expect(frame).toEqual({ stroke: danger, width: '3px', dash: 'none' })
    await expect(seg.locator('.tl-dead-mark')).toHaveText('✗')
    const body = seg.locator('rect:not(.tl-dead-frame)').first()
    const kind = await seg.getAttribute('data-kind')
    if (kind === 'hold') expect(await body.getAttribute('fill')).not.toBe('none')
    else expect(await body.getAttribute('stroke-dasharray')).toBeTruthy()
  }
})

Step(/^泳道 "([^"]+)" 疊成 ([0-9]+) 層：持有 (.) 的區段與等待 (.) 的區段$/, async ({ world }, lane: string, n: string, holdCode: string, waitCode: string) => {
  const segs = tl(world).locator(`[data-testid=tl-segment][data-lane="${lane}"]`)
  const head = (text: string) => ({ has: world.page.locator('.tl-seg-head', { hasText: new RegExp(`^${text}$`) }) })
  const hold = tl(world).locator(`[data-testid=tl-segment][data-lane="${lane}"][data-kind=hold]`).filter(head(holdCode))
  const wait = tl(world).locator(`[data-testid=tl-segment][data-lane="${lane}"][data-kind=wait_lock]`).filter(head(`⏳${waitCode}`))
  await expect(hold).toHaveCount(1)
  await expect(wait).toHaveCount(1)
  const layers = await segs.evaluateAll((els) => [...new Set(els.map((el) => el.getAttribute('data-layer')))])
  expect(layers).toHaveLength(Number(n))
  expect(await hold.getAttribute('data-layer')).not.toBe(await wait.getAttribute('data-layer'))
})

Step(/^泳道 "([^"]+)" 的 hold 區段為實心條，條頭標「(.)」$/, async ({ world }, lane: string, code: string) => {
  const seg = tl(world).locator(`[data-testid=tl-segment][data-lane="${lane}"][data-kind=hold]`)
  await expect(seg).toHaveCount(1)
  const body = seg.locator('rect:not(.tl-dead-frame)').first()
  expect(await body.getAttribute('fill')).not.toBe('none')
  expect(await body.getAttribute('stroke-dasharray')).toBeNull()
  await expect(seg.locator('.tl-seg-head')).toHaveText(code)
})

Step(/^泳道 "([^"]+)" 第 ([0-9]+) 步起的 (wait_lock|wait_msg) 區段為(虛線|點線)外框，條頭標「(.+)」$/, async ({ world }, lane: string, from: string, kind: string, line: string, head: string) => {
  const g = await geometry(world)
  const index = g.segments.findIndex((s) => s.lane === lane && s.kind === kind && s.from === Number(from))
  expect(index).toBeGreaterThanOrEqual(0)
  const seg = tl(world).getByTestId('tl-segment').nth(index)
  const body = seg.locator('rect:not(.tl-dead-frame)').first()
  expect(await body.getAttribute('fill')).toBe('none')
  expect(await body.getAttribute('stroke-dasharray')).toBe(line === '虛線' ? '4 3' : '1 3')
  await expect(seg.locator('.tl-seg-head')).toHaveText(head)
})

Step('時間軸沒有任何區段', async ({ world }) => {
  await expect(tl(world).getByTestId('tl-segment')).toHaveCount(0)
})

// ---------- 死鎖標記與箭頭 ----------

Step('時間軸沒有任何 ✗ 標記', async ({ world }) => {
  await expect(tl(world).locator('.tl-dead-mark')).toHaveCount(0)
  await expect(tl(world).getByTestId('tl-deadlock-arrow')).toHaveCount(0)
  await expect(tl(world).locator('[data-testid=tl-lane-label][data-state=deadlocked]')).toHaveCount(0)
  await expect(tl(world).getByTestId('tl-conclusion')).not.toContainText('✗')
})

Step(/^時間軸有 ([0-9]+) 條 "(--[\w-]+)" 虛線箭頭，各標 ✗：(.+)$/, async ({ world }, n: string, color: string, spec: string) => {
  const arrows = tl(world).getByTestId('tl-deadlock-arrow')
  await expect(arrows).toHaveCount(Number(n))
  const danger = await world.cssVarColor(color)
  for (const a of await arrows.all()) {
    const style = await a.locator('path').evaluate((el) => ({ stroke: getComputedStyle(el).stroke, dash: getComputedStyle(el).strokeDasharray }))
    expect(style.stroke).toBe(danger)
    expect(style.dash).not.toBe('none')
    await expect(a.locator('text')).toHaveText('✗')
  }
  const g = await geometry(world)
  const pairs = [...spec.matchAll(/([^\s：、]+ \d+) 的等待區段指向([^\s：、]+ \d+) 的 hold 區段/g)]
  expect(pairs).toHaveLength(Number(n))
  for (const [, waiter, holder] of pairs) {
    const wait = g.segments.find((s) => s.lane === waiter && s.kind === 'wait_lock' && s.to === null)!
    const hold = g.segments.find((s) => s.lane === holder && s.kind === 'hold' && s.to === null && s.target === wait.target)!
    const hit = g.arrows.find((a) => Math.abs(a.x1 - wait.x2) < 0.5 && Math.abs(a.y1 - wait.y) < 0.5 && Math.abs(a.y2 - hold.y) < 0.5)
    expect(hit, `${waiter} → ${holder}`).toBeTruthy()
  }
})

// ---------- 訊息箭頭 ----------

Step(/^時間軸有 ([0-9]+) 條訊息箭頭、([0-9]+) 條尾端為 ✉ 的虛線殘箭，殘箭從泳道 "([^"]+)" 第 ([0-9]+) 步出發$/, async ({ world }, arrows: string, pending: string, lane: string, step: string) => {
  const g = await geometry(world)
  expect(g.messages.filter((m) => !m.pending)).toHaveLength(Number(arrows))
  const stubs = g.messages.filter((m) => m.pending)
  expect(stubs).toHaveLength(Number(pending))
  expect(stubs[0].from).toEqual({ lane, step: Number(step) })
  expect(stubs[0].text).toContain('✉')
  const dash = await tl(world).locator('[data-testid=tl-message][data-pending=true] path').evaluate((el) => getComputedStyle(el).strokeDasharray)
  expect(dash).not.toBe('none')
})

Step(/^時間軸有 ([0-9]+) 條訊息箭頭：從泳道 "([^"]+)" 第 ([0-9]+) 步指到泳道 "([^"]+)" 第 ([0-9]+) 步$/, async ({ world }, n: string, fromLane: string, fromStep: string, toLane: string, toStep: string) => {
  const arrows = (await geometry(world)).messages.filter((m) => !m.pending)
  expect(arrows).toHaveLength(Number(n))
  expect(arrows[0].from).toEqual({ lane: fromLane, step: Number(fromStep) })
  expect(arrows[0].to).toEqual({ lane: toLane, step: Number(toStep) })
})

Step(/^時間軸有 ([0-9]+) 條訊息箭頭(?:，都指到泳道 "([^"]+)")?$/, async ({ world }, n: string, lane: string | undefined) => {
  const arrows = (await geometry(world)).messages.filter((m) => !m.pending)
  expect(arrows).toHaveLength(Number(n))
  if (lane) for (const a of arrows) expect(a.to?.lane).toBe(lane)
})

Step('時間軸沒有殘箭', async ({ world }) => {
  await expect(tl(world).locator('[data-testid=tl-message][data-pending=true]')).toHaveCount(0)
})

// ---------- 事件標記 ----------

Step(/^泳道 "([^"]+)" 第 ([0-9]+) 步有 (.) 標記$/, async ({ world }, lane: string, step: string, icon: string) => {
  const e = (await geometry(world)).events.find((x) => x.step === Number(step))!
  expect(e.lane).toBe(lane)
  expect(e.icon).toBe(icon)
})

Step(/^泳道 (".+") 各有 1 個 ● 與 1 個 ■$/, async ({ world }, list: string) => {
  const g = await geometry(world)
  for (const lane of names(list)) {
    const icons = g.events.filter((e) => e.lane === lane).map((e) => e.icon)
    expect(icons.filter((i) => i === '●'), lane).toHaveLength(1)
    expect(icons.filter((i) => i === '■'), lane).toHaveLength(1)
  }
})

Step(/^泳道 "([^"]+)" 沒有 ●、■ 與「…」$/, async ({ world }, lane: string) => {
  const g = await geometry(world)
  expect(g.events.filter((e) => e.lane === lane && (e.icon === '●' || e.icon === '■'))).toEqual([])
  expect(g.unfinished).not.toContain(lane)
})

Step(/^時間軸的 🚩 標記依序為：$/, async ({ world }, table: any) => {
  const g = await geometry(world)
  const notes = g.events.filter((e) => e.kind === 'note')
  const rows = table.hashes() as Record<string, string>[]
  expect(notes.map((e) => ({ lane: e.lane, step: e.step, icon: e.icon, title: e.title }))).toEqual(
    rows.map((r) => ({ lane: r.lane, step: Number(r.step), icon: '🚩', title: `${r.lane}：${r.text}` })),
  )
  for (const r of rows) {
    const [from, to] = r['hold 區段'].split('–').map(Number)
    const hold = g.segments.find((s) => s.lane === r.lane && s.kind === 'hold' && s.from === from)
    expect(hold?.to, `${r.lane} hold ${from}–${to}`).toBe(to)
    expect(Number(r.step)).toBeGreaterThan(to)
  }
})

Step(/^泳道 (".+") 尾端畫「…」$/, async ({ world }, list: string) => {
  expect((await geometry(world)).unfinished.sort()).toEqual(names(list).sort())
})

Step(/^泳道 (".+") 尾端沒有「…」，標籤圖示為「(.)」$/, async ({ world }, list: string, icon: string) => {
  const g = await geometry(world)
  for (const lane of names(list)) {
    expect(g.unfinished).not.toContain(lane)
    await expect(laneLabel(world, lane).locator('.tl-lane-icon')).toHaveText(icon)
  }
})

Step('時間軸沒有任何泳道尾端畫「…」', async ({ world }) => {
  await expect(tl(world).getByTestId('tl-lane-unfinished')).toHaveCount(0)
})

Step(/^播放中玩家點泳道 "([^"]+)" 第 ([0-9]+) 步的事件標記$/, async ({ world, page }, lane: string, step: string) => {
  await expect(world.playButton()).toHaveAttribute('aria-label', '暫停')
  const e = (await geometry(world)).events.find((x) => x.step === Number(step))!
  expect(e.lane).toBe(lane)
  // 記錄點擊之後步數標籤出現過的值
  await world.stepLabel().evaluate((el) => {
    const seen: string[] = []
    let clicked = false
    ;(window as any).__tlSeen = seen
    document.addEventListener('click', () => (clicked = true), { capture: true, once: true })
    new MutationObserver(() => clicked && seen.push(el.textContent ?? '')).observe(el, { childList: true, characterData: true, subtree: true })
  })
  await tl(world).locator(`[data-testid=tl-event][data-step="${step}"]`).click()
})

Step(/^第 ([0-9]+) 步到第 ([0-9]+) 步之間沒有播放中間動畫$/, async ({ page }, _from: string, to: string) => {
  const seen = (await page.evaluate(() => (window as any).__tlSeen as string[])).map((t) => Number(/第 (\d+) \//.exec(t)?.[1]))
  // 點擊之後步數直接變成目標步，沒有經過中間步
  expect([...new Set(seen)]).toEqual([Number(to)])
})

Step(/^時間軸可用 Tab 走訪的事件標記恰好 ([0-9]+) 個，依第 1 到第 ([0-9]+) 步排列$/, async ({ world }, n: string, last: string) => {
  const g = await geometry(world)
  const tabbable = g.events.filter((e) => e.tabbable).map((e) => e.step)
  expect(tabbable).toHaveLength(Number(n))
  expect(tabbable).toEqual(Array.from({ length: Number(last) }, (_, i) => i + 1))
})

Step(/^玩家用 Tab 移到第 ([0-9]+) 步的事件標記並按 "(\w+)"$/, async ({ world, page }, step: string, key: string) => {
  await tl(world).focus()
  for (let i = 0; i < 20; i++) {
    await page.keyboard.press('Tab')
    const at = await page.evaluate(() => document.activeElement?.getAttribute('data-step') ?? null)
    if (at === step) break
  }
  expect(await page.evaluate(() => document.activeElement?.getAttribute('data-step'))).toBe(step)
  await page.keyboard.press(key)
})

// ---------- 版面 ----------

Step(/^時間軸每步欄寬為 ([0-9]+)px$/, async ({ world }, w: string) => {
  expect((await geometry(world)).colW).toBeCloseTo(Number(w), 2)
})

Step(/^時間軸區(可|不能)橫向捲動$/, async ({ world }, can: string) => {
  const scroll = tl(world).locator('.tl-scroll')
  const [sw, cw, overflow] = await scroll.evaluate((el) => [el.scrollWidth, el.clientWidth, getComputedStyle(el).overflowX] as const)
  if (can === '可') {
    expect(sw).toBeGreaterThan(cw)
    expect(['auto', 'scroll']).toContain(overflow)
  } else expect(sw).toBeLessThanOrEqual(cw)
})

Step('頁面本身沒有橫向捲軸', async ({ page }) => {
  const [sw, cw] = await page.evaluate(() => [document.documentElement.scrollWidth, document.documentElement.clientWidth])
  expect(sw).toBeLessThanOrEqual(cw)
})

Step(/^第 ([0-9]+) 步的欄位在時間軸區的可見範圍內$/, async ({ world }, step: string) => {
  const scroll = tl(world).locator('.tl-scroll')
  const marker = tl(world).locator(`[data-testid=tl-event][data-step="${step}"]`)
  await expect
    .poll(async () => {
      const [s, m] = [(await scroll.boundingBox())!, (await marker.boundingBox())!]
      return m.x >= s.x && m.x + m.width <= s.x + s.width
    })
    .toBe(true)
})

Step(/^時間軸容器寬度小於 ([0-9]+)px$/, async ({ world }, w: string) => {
  expect((await tl(world).boundingBox())!.width).toBeLessThan(Number(w))
})

Step(/^時間軸標籤欄寬度為 ([0-9]+)px$/, async ({ world }, w: string) => {
  await expect.poll(async () => (await tl(world).locator('.tl-labels').boundingBox())!.width).toBeCloseTo(Number(w), 0)
})

// ---------- 可及性與動畫 ----------

Step(/^時間軸 SVG 的 role 為 "(\w+)"，aria-label 為「(.+)」$/, async ({ world }, role: string, label: string) => {
  const svg = tl(world).locator('svg.tl-svg')
  await expect(svg).toHaveAttribute('role', role)
  await expect(svg).toHaveAttribute('aria-label', label)
})

Step('時間軸的訊息箭頭與區段都沒有畫線動畫', async ({ world }) => {
  const motions = await tl(world)
    .locator('[data-testid=tl-message], [data-testid=tl-message] path, [data-testid=tl-segment], [data-testid=tl-segment] rect')
    .evaluateAll((els) =>
      els.map((el) => {
        const s = getComputedStyle(el)
        return { animation: s.animationName, transition: s.transitionDuration.split(',').every((d) => parseFloat(d) === 0) }
      }),
    )
  expect(motions.length).toBeGreaterThan(0)
  for (const m of motions) expect(m).toEqual({ animation: 'none', transition: true })
})

// ---------- 主控台 ----------

function outputPanel(world: World) {
  return world.console().locator('[role=tabpanel]').nth(0)
}

Step(/^主控台「輸出」分頁內容為 "(.+)"$/, async ({ world }, text: string) => {
  await world.waitIdle()
  expect(await outputPanel(world).getByTestId('stdout-line').allTextContents()).toEqual([text])
})

Step(/^主控台「輸出」分頁沒有「已隱藏 n 行時間軸事件」提示$/, async ({ world }) => {
  await world.waitIdle()
  await expect(outputPanel(world).getByTestId('tl-hidden-note')).toHaveCount(0)
})

Step(/^主控台「輸出」分頁頂部顯示淡色小字「(.+)」$/, async ({ world }, text: string) => {
  await world.waitIdle()
  const note = outputPanel(world).getByTestId('tl-hidden-note')
  await expect(note).toHaveText(text)
  expect(await note.evaluate((el) => getComputedStyle(el).color)).toBe(await world.cssVarColor('--text-muted'))
  const stdout = outputPanel(world).getByTestId('stdout')
  if ((await stdout.count()) > 0) expect((await note.boundingBox())!.y).toBeLessThan((await stdout.boundingBox())!.y)
})

Step(/^主控台「輸出」分頁的 stdout 依序為：$/, async ({ world }, table: any) => {
  await world.waitIdle()
  const expected = (table.hashes() as Record<string, string>[]).map((r) => r.line)
  expect(await outputPanel(world).getByTestId('stdout-line').allTextContents()).toEqual(expected)
})

Step(/^主控台「輸出」分頁沒有任何以 "(.+)" 開頭的行$/, async ({ world }, prefix: string) => {
  const lines = await outputPanel(world).getByTestId('stdout-line').allTextContents()
  expect(lines.filter((l) => l.startsWith(prefix))).toEqual([])
})

Step(/^主控台「輸出」分頁的 stdout 共 ([0-9]+) 行，第 1 行為 "(.+)"$/, async ({ world }, n: string, first: string) => {
  await world.waitIdle()
  const lines = await outputPanel(world).getByTestId('stdout-line').allTextContents()
  expect(lines).toHaveLength(Number(n))
  expect(lines[0]).toBe(first)
})

Step(/^主控台「輸出」分頁第 ([0-9]+) 行為 "(.+)"（行首一個空白）$/, async ({ world }, n: string, text: string) => {
  const lines = await outputPanel(world).getByTestId('stdout-line').allTextContents()
  expect(text.startsWith(' ') && !text.startsWith('  ')).toBe(true)
  expect(lines[Number(n) - 1]).toBe(text)
})

// ---------- @docker ----------

Step(/^玩家把編輯器內容換成該關 solution\.rs 接上 content\/shared\/timeline\.rs（architecture\.md §2\.2 組法）$/, async ({ world }) => {
  const solution = read(`../../../content/levels/${world.currentLevel}/solution.rs`)
  await world.setEditorValue(`${solution}\n${read('../../../content/shared/timeline.rs')}`)
})
