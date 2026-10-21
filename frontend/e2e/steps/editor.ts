import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'
import { levelDetail } from '../support/mock'
import { unescape } from '../support/responses'

const QUOTED = '"((?:[^"\\\\]|\\\\.)*)"'
const quotes = (text: string) => [...text.matchAll(/「([^」]+)」/g)].map((m) => m[1])
const squash = (text: string | null) => (text ?? '').replace(/\s+/g, '')

// ---------- 點擊 ----------

async function clickNamed(world: World, name: string) {
  const { page } = world
  switch (name) {
    case '執行':
      return world.clickRun()
    case '送出':
      return world.clickSubmit()
    case '重設':
      return page.getByTestId('btn-reset').click()
    case '播放':
    case '暫停':
      await expect(world.playButton()).toHaveAttribute('aria-label', name)
      return world.playButton().click()
    case '單步':
      world.before.step = await world.currentStep()
      return world.visual().getByRole('button', { name: '單步前進' }).click()
    case '從頭':
      return world.visual().getByRole('button', { name: '從頭' }).click()
    case '下一關':
    case '再玩一次':
    case '回地圖':
      return world.completeDialog().getByRole('button', { name, exact: true }).click()
    default:
      throw new Error(`不知道怎麼點「${name}」`)
  }
}

Step(/^(?:播放中|請求中)?玩家點「([^」]+)」$/, async ({ world }, name: string) => {
  if (name === '執行') world.before.bubble = await world.page.getByTestId('ferris-bubble').innerText()
  await clickNamed(world, name)
})

Step(/^玩家點「(執行|送出)」([0-9]+) 次$/, async ({ world }, name: string, times: string) => {
  for (let i = 0; i < Number(times); i++) {
    await clickNamed(world, name)
    await world.waitIdle()
  }
})

Step('玩家點「執行」並等待 3 秒', async ({ world, page }) => {
  await world.clickRun()
  await page.waitForTimeout(3200)
})

Step(/^玩家點「(執行|播放)」並等待播放結束$/, async ({ world }, name: string) => {
  await clickNamed(world, name)
  await world.waitPlaybackEnd()
})

Step('玩家點「重設」並確認', async ({ world, page }) => {
  await page.getByTestId('btn-reset').click()
  await page.getByTestId('confirm-ok').click()
})

Step(/^玩家在編輯器輸入 "([^"]+)" 後點「重設」並確認$/, async ({ world, page }, text: string) => {
  await world.typeInEditor(text)
  await page.getByTestId('btn-reset').click()
  await page.getByTestId('confirm-ok').click()
})

Step('玩家確認', async ({ page }) => {
  await page.getByTestId('confirm-ok').click()
})

Step('玩家取消', async ({ page }) => {
  await page.getByTestId('confirm-cancel').click()
})

// ---------- 泡泡、橫幅、找不到 ----------

Step(/^左欄頂部的 Ferris 對話泡泡顯示關卡 "([^"]+)" 的 story 全文$/, async ({ page }, id: string) => {
  const bubble = page.getByTestId('col-info').getByTestId('ferris-bubble')
  await expect(bubble).toBeVisible()
  expect(squash(await bubble.innerText())).toBe(squash(levelDetail(id).story))
})

Step('泡泡沒有「下一句」按鈕', async ({ page }) => {
  await expect(page.getByRole('button', { name: '下一句' })).toHaveCount(0)
})

Step('Ferris 對話泡泡內容與執行前相同', async ({ world, page }) => {
  await world.waitIdle()
  expect(await page.getByTestId('ferris-bubble').innerText()).toBe(world.before.bubble)
})

Step(/^任務橫幅顯示「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByTestId('kind-banner')).toHaveText(text)
})

Step(/^畫面顯示「找不到這關」$/, async ({ page }) => {
  await expect(page.getByTestId('not-found')).toContainText('找不到這關')
})

Step('畫面有回地圖的連結', async ({ page }) => {
  const link = page.getByTestId('not-found').getByRole('link', { name: '回地圖' })
  await expect(link).toHaveAttribute('href', '/')
})

// ---------- 按鈕狀態 ----------

const BUTTON_ID: Record<string, string> = { 執行: 'btn-run', 送出: 'btn-submit', 重設: 'btn-reset' }

Step(/^「(執行|送出)」按鈕顯示 spinner 與文字「判題中…」$/, async ({ page }, name: string) => {
  const btn = page.getByTestId(BUTTON_ID[name])
  await expect(btn).toHaveText('判題中…')
  await expect(btn).toHaveClass(/is-loading/)
  const spinner = await btn.evaluate((el) => getComputedStyle(el, '::before').animationName)
  expect(spinner).toMatch(/^spin/)
})

Step(/^「(.+?)」與「(.+?)」按鈕為 disabled$/, async ({ page }, a: string, b: string) => {
  await expect(page.getByTestId(BUTTON_ID[a])).toBeDisabled()
  await expect(page.getByTestId(BUTTON_ID[b])).toBeDisabled()
})

Step(/^「(執行|送出)」按鈕顯示「(.+)」並為 disabled$/, async ({ page }, name: string, text: string) => {
  await expect(page.getByTestId(BUTTON_ID[name])).toHaveText(text)
  await expect(page.getByTestId(BUTTON_ID[name])).toBeDisabled()
})

Step(/^「(執行|送出|重設)」按鈕為可點擊$/, async ({ page }, name: string) => {
  await expect(page.getByTestId(BUTTON_ID[name])).toBeEnabled()
})

Step(/^「執行」與「送出」按鈕(?:恢復)?為可點擊(，且不顯示倒數)?$/, async ({ page }, noCountdown: string | undefined) => {
  await expect(page.getByTestId('btn-run')).toBeEnabled()
  await expect(page.getByTestId('btn-submit')).toBeEnabled()
  if (noCountdown) {
    await expect(page.getByTestId('btn-run')).toHaveText('執行')
    await expect(page.getByTestId('btn-submit')).toHaveText('送出')
  }
})

Step(/^經過 ([0-9]+) 秒$/, async ({ world }, secs: string) => {
  await world.elapse(Number(secs) * 1000)
})

Step(/^按鈕旁顯示「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByTestId('degraded-note')).toHaveText(text)
})

Step('左欄故事與提示區仍可瀏覽', async ({ page }) => {
  await expect(page.getByTestId('ferris-bubble')).toBeVisible()
  await expect(page.getByTestId('hint-section')).toBeVisible()
  await expect(page.getByTestId('hint-button')).toBeEnabled()
})

// ---------- 編輯器 ----------

Step('編輯器仍可輸入文字', async ({ world }) => {
  const before = await world.editorValue()
  await world.typeInEditor('x')
  expect(await world.editorValue()).toBe(before + 'x')
})

Step(/^玩家(?:在請求中於|在)編輯器輸入 "([^"]+)"$/, async ({ world }, text: string) => {
  await world.typeInEditor(text)
})

Step('玩家在編輯器輸入一個空白', async ({ world }) => {
  await world.typeInEditor(' ')
})

Step('玩家刪掉剛輸入的空白', async ({ page }) => {
  await page.keyboard.press('Backspace')
})

Step(/^玩家在編輯器輸入 "([^"]+)" 並停止輸入 500 ms$/, async ({ world, page }, text: string) => {
  await world.typeInEditor(text)
  await page.waitForTimeout(700)
})

Step(/^焦點在編輯器(內|外)$/, async ({ world, page }, where: string) => {
  if (where === '內') await world.focusEditorEnd()
  else {
    await page.getByTestId('kind-banner').click()
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur())
  }
  world.before.editor = await world.editorValue()
})

Step('編輯器內容與按鍵前相同', async ({ world }) => {
  await world.page.waitForTimeout(200)
  expect(await world.editorValue()).toBe(world.before.editor)
})

Step(new RegExp(`^編輯器內容為 ${QUOTED}$`), async ({ world }, text: string) => {
  await expect.poll(() => world.editorValue()).toBe(unescape(text))
})

Step(/^編輯器內容等於 mock API 關卡 "([^"]+)" 的 starter$/, async ({ world, mock }, id: string) => {
  await expect.poll(() => world.editorValue()).toBe(mock.details[id].starter)
})

Step(/^編輯器內容含 "([^"]+)"$/, async ({ world }, text: string) => {
  expect(await world.editorValue()).toContain(text)
})

// ---------- marker ----------

const SEVERITY: Record<string, number> = { error: 8, warning: 4 }
const SQUIGGLY: Record<string, string> = { error: '.squiggly-error', warning: '.squiggly-warning' }

async function expectMarker(world: World, level: string, range: [number, number, number, number]) {
  await expect
    .poll(async () =>
      (await world.markers()).some(
        (m) =>
          m.severity === SEVERITY[level] &&
          m.startLineNumber === range[0] &&
          m.startColumn === range[1] &&
          m.endLineNumber === range[2] &&
          m.endColumn === range[3],
      ),
    )
    .toBe(true)
  await expect(world.page.locator(`[data-testid=col-editor] ${SQUIGGLY[level]}`).first()).toBeAttached()
}

Step(
  /^編輯器第 ([0-9]+) 行第 ([0-9]+) 欄到第 ([0-9]+) 欄有 (error|warning) 等級的(?:紅|黃)色波浪線 marker$/,
  async ({ world }, line: string, start: string, end: string, level: string) => {
    await expectMarker(world, level, [Number(line), Number(start), Number(line), Number(end)])
  },
)

Step(
  /^編輯器第 ([0-9]+) 行第 ([0-9]+) 欄到第 ([0-9]+) 行第 ([0-9]+) 欄有 (error|warning) 等級的(?:紅|黃)色波浪線 marker$/,
  async ({ world }, l1: string, c1: string, l2: string, c2: string, level: string) => {
    await expectMarker(world, level, [Number(l1), Number(c1), Number(l2), Number(c2)])
  },
)

Step(/^編輯器第 ([0-9]+) 行整行有 error 等級的紅色波浪線 marker$/, async ({ world, page }, line: string) => {
  const l = Number(line)
  const max = await page.evaluate((n) => window.__rqMonaco.editor.getEditors()[0].getModel().getLineMaxColumn(n), l)
  await expectMarker(world, 'error', [l, 1, l, max])
})

Step('編輯器沒有 error 等級的 marker', async ({ world }) => {
  await world.waitIdle()
  expect((await world.markers()).filter((m) => m.severity === 8)).toEqual([])
})

Step('編輯器沒有任何 marker', async ({ world }) => {
  await world.waitIdle()
  await expect.poll(async () => (await world.markers()).length).toBe(0)
})

Step(/^編輯器共有 ([0-9]+) 個 marker$/, async ({ world }, n: string) => {
  await expect.poll(async () => (await world.markers()).length).toBe(Number(n))
})

// ---------- 游標、捲動、閃爍 ----------

Step(/^編輯器第 ([0-9]+) 行在可見範圍內$/, async ({ world }, line: string) => {
  await expect.poll(() => world.lineVisible(Number(line))).toBe(true)
})

Step(/^編輯器游標位於第 ([0-9]+) 行第 ([0-9]+) 欄$/, async ({ world }, line: string, col: string) => {
  await expect.poll(() => world.cursor()).toEqual({ lineNumber: Number(line), column: Number(col) })
})

Step(/^編輯器第 ([0-9]+) 行背景閃一下$/, async ({ world }, line: string) => {
  expect(await world.decoratedLines('rq-line-flash')).toEqual([Number(line)])
  await expect(world.page.locator('[data-testid=col-editor] .rq-line-flash')).toBeAttached()
  // 閃完即移除
  await expect.poll(() => world.decoratedLines('rq-line-flash'), { timeout: 3000 }).toEqual([])
})

Step('編輯器游標位置與點擊前相同', async ({ world }) => {
  expect(await world.cursor()).toEqual(world.before.cursor)
})

// ---------- hover ----------

Step(/^玩家把滑鼠移到第 ([0-9]+) 行第 ([0-9]+) 欄的波浪線上$/, async ({ world, page }, line: string, col: string) => {
  await world.waitIdle()
  await expect.poll(async () => (await world.markers()).length).toBeGreaterThan(0)
  const p = await world.editorPoint(Number(line), Number(col) + 1)
  await page.mouse.move(p.x - 30, p.y)
  await page.mouse.move(p.x, p.y, { steps: 5 })
  await expect(page.locator('.monaco-hover:not(.hidden)').first()).toBeVisible({ timeout: 5000 })
})

function hover(world: World) {
  return world.page.locator('.monaco-hover:not(.hidden)').first()
}

Step(/^hover 內容的文字含「(.+)」$/, async ({ world }, text: string) => {
  await expect(hover(world)).toContainText(text)
})

Step(/^hover 內容中 "([^"]+)" 以行內程式碼 <code> 呈現，不顯示反引號字元$/, async ({ world }, name: string) => {
  await expect(hover(world).locator('code', { hasText: name }).first()).toBeVisible()
  expect(await hover(world).innerText()).not.toContain('`')
})

Step(/^hover 內容含指向 "([^"]+)" 的錯誤碼連結$/, async ({ world }, url: string) => {
  const link = hover(world).locator('a:visible', { hasText: 'E0382' }).first()
  await expect(link).toBeVisible()
  const href = (await link.getAttribute('href')) ?? (await link.getAttribute('data-href'))
  expect(href).toBe(url)
})

// ---------- 草稿 ----------

// 以頁面內時間比對最後輸入與首次寫入，不受主機快慢影響
Step(/^輸入後 300 ms 時 localStorage 沒有 key "([^"]+)"$/, async ({ page }, key: string) => {
  await page.waitForFunction(() => performance.now() >= window.__rqTrace!.lastInput + 300)
  const { lastInput, written } = await page.evaluate((k) => ({ lastInput: window.__rqTrace!.lastInput, written: window.__rqTrace!.writes[k] }), key)
  if (written !== undefined) expect(written - lastInput).toBeGreaterThanOrEqual(300)
})

Step(/^停止輸入 500 ms 後 localStorage 的 "([^"]+)" 等於編輯器目前內容$/, async ({ world }, key: string) => {
  const code = await world.editorValue()
  await expect.poll(() => world.storageItem(key)).toBe(code)
})

// ---------- 確認對話框 ----------

Step(/^出現確認「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByTestId('confirm-message')).toHaveText(text)
})

Step('出現頁面內的確認對話框，沒有觸發瀏覽器原生 confirm', async ({ world, page }) => {
  await expect(page.getByTestId('confirm-dialog')).toBeVisible()
  await expect(page.getByTestId('confirm-dialog')).toHaveAttribute('aria-modal', 'true')
  expect(world.nativeDialogs).toBe(0)
})

Step('焦點在確認對話框內', async ({ page }) => {
  await expect.poll(() => page.evaluate(() => !!document.activeElement?.closest('[data-testid=confirm-dialog]'))).toBe(true)
})

Step('確認對話框關閉，焦點回到「重設」按鈕', async ({ page }) => {
  await expect(page.getByTestId('confirm-dialog')).toHaveCount(0)
  await expect(page.getByTestId('btn-reset')).toBeFocused()
})

Step(/^沒有出現確認(?:對話框)?$/, async ({ world, page }) => {
  await page.waitForTimeout(200)
  await expect(page.getByTestId('confirm-dialog')).toHaveCount(0)
  expect(world.nativeDialogs).toBe(0)
})

Step('畫面沒有出現關於草稿的 toast', async ({ page }) => {
  await page.waitForTimeout(300)
  await expect(page.locator('.toast')).toHaveCount(0)
})

// ---------- 快捷鍵說明 ----------

Step('玩家點頂部列「⌨ 快捷鍵」', async ({ page }) => {
  await page.locator('.topbar').getByRole('button', { name: '快捷鍵說明' }).click()
})

Step(/^快捷鍵說明面板列出(「.+」)$/, async ({ page }, list: string) => {
  const keys = await page.getByTestId('shortcut-help').locator('kbd').allInnerTexts()
  for (const k of quotes(list)) expect(keys).toContain(k)
})

Step('快捷鍵說明面板含「Ctrl+M」切換 Tab 移動焦點的說明', async ({ page }) => {
  const help = page.getByTestId('shortcut-help')
  await expect(help.locator('kbd', { hasText: 'Ctrl+M' })).toBeVisible()
  await expect(help).toContainText('Tab 移動焦點')
})

Step('快捷鍵說明面板關閉', async ({ page }) => {
  await expect(page.getByTestId('shortcut-help')).toHaveCount(0)
})

// ---------- 深淺色 ----------

Step(/^HTML 根元素的 data-theme 為 "(light|dark)"$/, async ({ page }, theme: string) => {
  await expect(page.locator('html')).toHaveAttribute('data-theme', theme)
})

const MONACO_BG: Record<string, string> = { 'ocean-light': 'rgb(255, 253, 248)', 'ocean-dark': 'rgb(22, 56, 75)' }

Step(/^Monaco 主題為 "(ocean-light|ocean-dark)"$/, async ({ page }, theme: string) => {
  const bg = page.locator('[data-testid=col-editor] .monaco-editor-background').first()
  await expect(bg).toHaveCSS('background-color', MONACO_BG[theme])
})

Step('玩家點頂部列深淺色切換按鈕', async ({ page }) => {
  await page.getByTestId('theme-toggle').click()
})

// ---------- 版面 ----------

async function box(world: World, testId: string) {
  const b = await world.page.getByTestId(testId).boundingBox()
  if (!b) throw new Error(`${testId} 不可見`)
  return b
}

Step('關卡頁同時顯示左欄（故事）、中欄（編輯器）、右欄（可視化與主控台）', async ({ world, page }) => {
  for (const id of ['col-info', 'col-editor', 'col-result', 'visual-panel', 'console']) await expect(page.getByTestId(id)).toBeVisible()
  const [a, b, c] = [await box(world, 'col-info'), await box(world, 'col-editor'), await box(world, 'col-result')]
  expect(a.x + a.width).toBeLessThanOrEqual(b.x)
  expect(b.x + b.width).toBeLessThanOrEqual(c.x)
  expect(a.x).toBeGreaterThanOrEqual(0)
})

Step('編輯器與可視化並排顯示', async ({ world }) => {
  const [e, r] = [await box(world, 'col-editor'), await box(world, 'col-result')]
  expect(e.x + e.width).toBeLessThanOrEqual(r.x)
  expect(Math.abs(e.y - r.y)).toBeLessThan(40)
})

async function drawerOpen(world: World): Promise<boolean> {
  const expanded = await world.page.locator('.topbar-drawer').getAttribute('aria-expanded')
  const b = await world.page.getByTestId('col-info').boundingBox()
  return expanded === 'true' && !!b && b.x >= 0
}

Step(/^「☰ 故事」抽屜為(展開|關閉)(?:，內容含關卡 "([^"]+)" 的 story)?$/, async ({ world, page }, state: string, id: string | undefined) => {
  await expect(page.locator('.topbar-drawer')).toBeVisible()
  await expect.poll(() => drawerOpen(world)).toBe(state === '展開')
  if (id) expect(squash(await page.getByTestId('ferris-bubble').innerText())).toBe(squash(levelDetail(id).story))
})

const TAB_NAME: Record<string, string> = { 故事: 'story', 程式: 'code', 結果: 'result' }

Step(/^畫面有「故事」「程式」「結果」三個分頁，目前為「(.+)」$/, async ({ page }, current: string) => {
  const tabs = page.locator('.mobile-tabs [role=tab]')
  await expect(tabs).toHaveCount(3)
  expect((await tabs.allInnerTexts()).map((t) => t.replace(/•/g, '').trim())).toEqual(['故事', '程式', '結果'])
  await expect(page.locator('.mobile-tabs [role=tab][aria-selected=true]')).toHaveText(current)
  await expect(page.locator(`.level-page.mtab-${TAB_NAME[current]}`)).toBeVisible()
})

Step(/^目前分頁為「(.+)」$/, async ({ page }, current: string) => {
  await expect(page.locator('.mobile-tabs [role=tab][aria-selected=true]')).toHaveText(new RegExp(`^\\s*${current}`))
})

Step('底部操作列有「重設」「執行」「送出」，每顆按鈕觸控區至少 44×44px', async ({ world }) => {
  for (const id of ['btn-reset', 'btn-run', 'btn-submit']) {
    const b = await box(world, id)
    expect(b.width).toBeGreaterThanOrEqual(44)
    expect(b.height).toBeGreaterThanOrEqual(44)
    expect(b.y + b.height).toBeGreaterThan(800 - 60)
  }
})
