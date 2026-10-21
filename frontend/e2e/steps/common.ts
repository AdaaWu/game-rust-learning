import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { expect, PROGRESS_KEY, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'
import { MVP_IDS } from '../support/mock'
import { unescape } from '../support/responses'

const QUOTED = '"((?:[^"\\\\]|\\\\.)*)"'

const MAC_UA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36'
const WIN_UA = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36'

async function setPlatform(world: World, platform: 'Windows' | 'macOS') {
  await world.page.addInitScript(
    ([p, ua]) => {
      Object.defineProperty(Navigator.prototype, 'platform', { configurable: true, get: () => p })
      Object.defineProperty(Navigator.prototype, 'userAgent', { configurable: true, get: () => ua })
    },
    platform === 'macOS' ? (['MacIntel', MAC_UA] as const) : (['Win32', WIN_UA] as const),
  )
}

// ---------- Background：mock 與進度 ----------

Step(/^mock API 回傳測試用 8 關 fixture 的關卡列表(?:（含 chapters）)?與詳細資料$/, async ({ world }) => {
  // 預設即為測試用 8 關 fixture；預設平台為 Windows
  await setPlatform(world, 'Windows')
})

Step(/^mock API 只有一章（第 1 章）一關 "fixture-grid"，visual 為 "grid"，地圖為 ux\.md §4\.2 範例$/, async ({ world, mock }) => {
  mock.useFixtureGrid()
  await setPlatform(world, 'Windows')
  // 指令播放靠快轉時鐘等待
  await world.installClock()
})

Step(/^mock API 的 GET "\/api\/health" (?:改為)?回 status "(ok|degraded)"$/, async ({ mock }, status: string) => {
  mock.health = status as 'ok' | 'degraded'
})

Step(/^(?:玩家)?進度中測試用 8 關 fixture 都已拿到 ★1$/, async ({ world }) => {
  for (const id of MVP_IDS) world.levelProgress(id).stars.pass = true
})

Step('進度為空', async ({ world }) => {
  world.progress = { version: 1, levels: {} }
})

Step(/^進度中已拿到 ★1 的關卡為 "([^"]*)"$/, async ({ world }, list: string) => {
  world.progress = { version: 1, levels: {} }
  for (const id of list.split(',').map((s) => s.trim()).filter(Boolean)) world.levelProgress(id).stars.pass = true
})

function bool(text: string): boolean {
  return text === 'true'
}

Step(
  /^進度中 "([^"]+)" 的 stars 為 pass (true|false)、noHint (true|false)、firstTry (true|false)，current 為 hintsRevealed ([0-9]+)、failedSubmits ([0-9]+)$/,
  async ({ world }, id: string, pass: string, noHint: string, firstTry: string, hints: string, failed: string) => {
    const level = world.levelProgress(id)
    level.stars = { pass: bool(pass), noHint: bool(noHint), firstTry: bool(firstTry) }
    level.current = { hintsRevealed: Number(hints), failedSubmits: Number(failed) }
  },
)

Step(
  /^進度中 "([^"]+)" 的 stars 全為 false，current 為 hintsRevealed ([0-9]+)、failedSubmits ([0-9]+)$/,
  async ({ world }, id: string, hints: string, failed: string) => {
    const level = world.levelProgress(id)
    level.stars = { pass: false, noHint: false, firstTry: false }
    level.current = { hintsRevealed: Number(hints), failedSubmits: Number(failed) }
  },
)

Step(/^mock API 關卡 "([^"]+)" 的 hints 為空陣列$/, async ({ mock }, id: string) => {
  mock.details[id].hints = []
})

Step('前端連到真實後端，判題服務可用', async ({ mock, world }) => {
  mock.real = true
  await setPlatform(world, 'Windows')
  const res = await world.page.request.get('http://127.0.0.1:3000/api/health')
  expect((await res.json()).status).toBe('ok')
})

// ---------- 瀏覽器環境 ----------

Step(/^瀏覽器平台為 "(Windows|macOS)"$/, async ({ world }, platform: string) => {
  await setPlatform(world, platform as 'Windows' | 'macOS')
})

Step(/^瀏覽器設定 prefers-reduced-motion 為 "reduce"$/, async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
})

Step(/^瀏覽器設定 prefers-color-scheme 為 "(light|dark)"$/, async ({ page }, scheme: string) => {
  await page.emulateMedia({ colorScheme: scheme as 'light' | 'dark' })
})

Step(/^瀏覽器視窗寬度為 ([0-9]+)px$/, async ({ page }, width: string) => {
  await page.setViewportSize({ width: Number(width), height: 800 })
})

Step('瀏覽器以 Achromatopsia 模擬色彩', async ({ page }) => {
  const cdp = await page.context().newCDPSession(page)
  await cdp.send('Emulation.setEmulatedVisionDeficiency', { type: 'achromatopsia' })
})

// ---------- localStorage ----------

async function storageIs(world: World, key: string, value: string) {
  if (!world.originReady) await world.setStorage(key, value)
  else expect(await world.storageItem(key)).toBe(value)
}

Step(new RegExp(`^localStorage 的 "([^"]+)" 為 ${QUOTED}$`), async ({ world }, key: string, value: string) => {
  await storageIs(world, key, unescape(value))
})

Step(/^localStorage 的 "([^"]+)" 為 `(.*)`$/, async ({ world }, key: string, value: string) => {
  await storageIs(world, key, value)
})

Step(/^localStorage 沒有 key "([^"]+)"$/, async ({ world }, key: string) => {
  if (!world.originReady) await world.setStorage(key, null)
  else await expect.poll(() => world.storageItem(key)).toBeNull()
})

Step('localStorage 寫入會失敗', async ({ world, page }) => {
  world.failWrites = true
  await page.addInitScript(() => {
    if (location.pathname === '/favicon.svg') return
    Storage.prototype.setItem = function () {
      throw new DOMException('寫入失敗', 'QuotaExceededError')
    }
  })
})

Step(/^localStorage 的 "([^"]+)" 的 theme 為 "(\w+)"$/, async ({ world }, key: string, theme: string) => {
  await expect.poll(async () => JSON.parse((await world.storageItem(key)) ?? '{}').theme).toBe(theme)
})

Step(/^localStorage 的 "([^"]+)" 不含 "(\w+)"$/, async ({ world }, key: string, field: string) => {
  expect((await world.storageItem(key)) ?? '').not.toContain(field)
})

Step(/^localStorage 除了 "rustquest\.progress\.v1"、"rustquest\.prefs\.v1" 與 "rustquest\.draft\." 開頭的 key 以外沒有其他 key$/, async ({ page }) => {
  const keys = await page.evaluate(() => Object.keys(localStorage))
  const others = keys.filter((k) => k !== PROGRESS_KEY && k !== 'rustquest.prefs.v1' && !k.startsWith('rustquest.draft.'))
  expect(others).toEqual([])
})

// ---------- 導覽 ----------

Step(/^玩家開啟關卡 "([^"]+)"$/, async ({ world }, id: string) => {
  await world.open(`/level/${id}`)
})

Step(/^玩家開啟關卡 "([^"]+)"，編輯器第 ([0-9]+) 行長度為 ([0-9]+) 個字元（01 starter 第 4 行）$/, async ({ world }, id: string, line: string, len: string) => {
  await world.open(`/level/${id}`)
  const length = await world.page.evaluate((l) => window.__rqMonaco.editor.getEditors()[0].getModel().getLineLength(l), Number(line))
  expect(length).toBe(Number(len))
})

Step(/^玩家直接開啟網址 "([^"]+)"$/, async ({ world }, path: string) => {
  await world.open(path)
})

Step('玩家開啟章節地圖', async ({ world }) => {
  await world.open('/')
})

Step('玩家重新整理頁面', async ({ world }) => {
  await world.reload()
})

async function backToMapAndEnter(world: World, id: string) {
  await world.page.getByTestId('back-to-map').click()
  await expect(world.node(id)).toBeVisible()
  await world.node(id).click()
  await world.page.waitForURL(`**/level/${id}`)
  await world.waitReady()
}

Step(/^玩家回章節地圖後再點節點進入關卡 "([^"]+)"$/, async ({ world }, id: string) => {
  await backToMapAndEnter(world, id)
})

Step('玩家回章節地圖後再點節點進入同一關', async ({ world }) => {
  await backToMapAndEnter(world, world.currentLevel)
})

Step(/^玩家從章節地圖進入關卡 "([^"]+)"$/, async ({ world }, id: string) => {
  await world.open('/')
  await world.node(id).click()
  await world.page.waitForURL(`**/level/${id}`)
  await world.waitReady()
})

Step(/^網址(?:仍)?為 "([^"]+)"$/, async ({ page }, path: string) => {
  await expect.poll(() => new URL(page.url()).pathname).toBe(path)
})

// ---------- 鍵盤 ----------

Step(/^玩家按 "([^"]+)"$/, async ({ world, page }, keys: string) => {
  world.before.editor = await world.editorValue().catch(() => null)
  await page.keyboard.press(keys)
})

Step(/^玩家在請求中按 "([^"]+)" 與 "([^"]+)"$/, async ({ page }, a: string, b: string) => {
  await page.keyboard.press(a)
  await page.keyboard.press(b)
})

// ---------- toast ----------

Step(/^顯示 toast「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.locator('.toast', { hasText: text })).toBeVisible()
})

Step(/^沒有顯示 toast「(.+)」$/, async ({ page }, text: string) => {
  await page.waitForTimeout(300)
  await expect(page.locator('.toast', { hasText: text })).toHaveCount(0)
})

// ---------- @docker ----------

const SOLUTIONS = (id: string) => fileURLToPath(new URL(`../../../content/levels/${id}/solution.rs`, import.meta.url))

Step('玩家把編輯器內容換成該關 solution.rs', async ({ world }) => {
  await world.setEditorValue(readFileSync(SOLUTIONS(world.currentLevel), 'utf8'))
})
