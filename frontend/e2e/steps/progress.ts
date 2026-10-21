import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'

const quotes = (text: string) => [...text.matchAll(/「([^」]+)」/g)].map((m) => m[1])
const STAR_KEY: Record<string, string> = { 通過: 'pass', 沒看提示: 'noHint', 一次就過: 'firstTry' }
const NODE_STATE: Record<string, string> = { 可挑戰: 'available', 鎖定: 'locked', 已通過: 'passed' }

// ---------- 通關動畫與結算 ----------

// 動畫只有 1.5 秒，以頁面內紀錄確認曾在結算前出現
Step('播放通關動畫', async ({ world, page }) => {
  await world.waitIdle()
  await expect.poll(() => page.evaluate(() => window.__rqTrace?.clearShownAlone ?? false)).toBe(true)
})

Step(/^通關動畫結束後顯示通關結算 Modal，按鈕有(「.+」)$/, async ({ world, page }, list: string) => {
  await expect(page.getByTestId('clear-animation')).toHaveCount(0, { timeout: 5000 })
  await expect(world.completeDialog()).toBeVisible()
  for (const name of quotes(list)) await expect(world.completeDialog().getByRole('button', { name, exact: true })).toBeVisible()
})

Step(/^通關動畫播放中玩家按 "(.+)"$/, async ({ world, page }, key: string) => {
  await world.waitIdle()
  await expect(page.getByTestId('clear-animation')).toBeVisible()
  await page.keyboard.press(key)
})

Step('立即顯示通關結算 Modal', async ({ world, page }) => {
  await expect(world.completeDialog()).toBeVisible({ timeout: 500 })
  await expect(page.getByTestId('clear-animation')).toHaveCount(0)
})

Step('不播放通關動畫', async ({ world, page }) => {
  let seen = false
  const deadline = Date.now() + 1000
  await world.waitIdle()
  while (Date.now() < deadline) {
    if ((await page.getByTestId('clear-animation').count()) > 0) seen = true
    await page.waitForTimeout(50)
  }
  expect(seen).toBe(false)
})

Step('顯示通關結算 Modal', async ({ world }) => {
  await world.waitIdle()
  await expect(world.completeDialog()).toBeVisible({ timeout: world.mock.real ? 30_000 : 5000 })
})

Step('不出現通關結算 Modal', async ({ world, page }) => {
  await world.waitIdle()
  await page.waitForTimeout(500)
  await expect(world.completeDialog()).toHaveCount(0)
  await expect(page.getByTestId('clear-animation')).toHaveCount(0)
})

Step(/^通關結算顯示 (★.+)$/, async ({ world }, list: string) => {
  await world.waitIdle()
  const dialog = world.completeDialog()
  await expect(dialog).toBeVisible()
  for (const m of list.matchAll(/★(\S+?) (true|false)/g)) {
    await expect(dialog.locator(`[data-testid=star-row][data-star=${STAR_KEY[m[1]]}]`)).toHaveAttribute('data-earned', m[2])
  }
})

Step('通關結算中每顆星旁顯示達成條件與是否達成', async ({ world }) => {
  const rows = world.completeDialog().getByTestId('star-row')
  await expect(rows).toHaveCount(3)
  for (const [i, label] of ['通過', '沒看提示', '一次就過'].entries()) {
    const row = rows.nth(i)
    await expect(row).toContainText(label)
    const earned = (await row.getAttribute('data-earned')) === 'true'
    await expect(row.locator('.star-status')).toHaveText(earned ? '達成' : '未達成')
  }
})

Step(/^「沒看提示」星旁顯示一句怎麼拿到，例如「(.+)」$/, async ({ world }, tip: string) => {
  await expect(world.completeDialog().locator('[data-star=noHint]').getByTestId('star-tip')).toHaveText(tip)
})

Step('亮星為實心、暗星為空心外框', async ({ world }) => {
  const rows = world.completeDialog().getByTestId('star-row')
  for (let i = 0; i < 3; i++) {
    const row = rows.nth(i)
    const earned = (await row.getAttribute('data-earned')) === 'true'
    await expect(row.getByTestId('star-icon')).toHaveText(earned ? '★' : '☆')
  }
})

Step('通關結算有按鈕「下一關」「再玩一次」「回地圖」，「下一關」為主要按鈕', async ({ world }) => {
  const d = world.completeDialog()
  for (const name of ['下一關', '再玩一次', '回地圖']) await expect(d.getByRole('button', { name, exact: true })).toBeVisible()
  await expect(d.getByRole('button', { name: '下一關', exact: true })).toHaveClass(/is-primary/)
  await expect(d.locator('.btn.is-primary')).toHaveCount(1)
})

// ---------- Submit 直到通關 ----------

Step('玩家依序點「送出」直到通關', async ({ world }) => {
  for (let i = 0; i < 5; i++) {
    await world.clickSubmit()
    await world.waitIdle()
    if ((await world.page.locator('.complete-dialog, [data-testid=clear-animation]').count()) > 0) return
  }
  throw new Error('送出 5 次仍未通關')
})

// ---------- 進度 ----------

Step(/^進度中 "([^"]+)" 的 stars 為 pass (true|false)、noHint (true|false)、firstTry (true|false)$/, async ({ world }, id: string, pass: string, noHint: string, firstTry: string) => {
  await expect
    .poll(async () => (await world.storedProgress())?.levels?.[id]?.stars)
    .toEqual({ pass: pass === 'true', noHint: noHint === 'true', firstTry: firstTry === 'true' })
})

Step(/^進度中 "([^"]+)" 的 current 為 hintsRevealed ([0-9]+)、failedSubmits ([0-9]+)$/, async ({ world }, id: string, hints: string, failed: string) => {
  await world.waitIdle().catch(() => {})
  await expect
    .poll(async () => (await world.storedProgress())?.levels?.[id]?.current)
    .toEqual({ hintsRevealed: Number(hints), failedSubmits: Number(failed) })
})

Step(/^進度中 "([^"]+)" 的 current\.(hintsRevealed|failedSubmits) 為 ([0-9]+)$/, async ({ world }, id: string, field: string, n: string) => {
  await world.waitIdle()
  await expect.poll(async () => (await world.storedProgress())?.levels?.[id]?.current?.[field] ?? 0).toBe(Number(n))
})

Step(/^進度中 "([^"]+)" 的 key 恰好為 "(.+)"$/, async ({ world }, id: string, keys: string) => {
  await world.waitIdle()
  await expect.poll(async () => Object.keys((await world.storedProgress())?.levels?.[id] ?? {}).sort()).toEqual(keys.split(',').map((k) => k.trim()).sort())
})

// ---------- 提示 ----------

async function revealHints(world: World, n: number) {
  for (let i = 0; i < n; i++) {
    await world.page.getByTestId('hint-button').click()
    if ((await world.page.getByTestId('confirm-dialog').count()) > 0) await world.page.getByTestId('confirm-ok').click()
    await expect(world.page.getByTestId('hint-item')).toHaveCount(i + 1)
  }
}

Step(/^玩家揭露 ([0-9]+) 則提示$/, async ({ world }, n: string) => {
  await revealHints(world, Number(n))
})

Step(/^玩家(?:再)?點提示按鈕$/, async ({ page }) => {
  await page.getByTestId('hint-button').click()
})

Step(/^(?:提示區)?顯示第 ([\d、]+) 則提示$/, async ({ world, mock, page }, list: string) => {
  const nums = list.split('、').map(Number)
  const hints = mock.details[world.currentLevel].hints as string[]
  await expect(page.getByTestId('hint-item')).toHaveText(nums.map((n) => hints[n - 1]))
})

Step(/^提示區顯示按鈕「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByTestId('hint-button')).toHaveText(text)
})

Step('提示區沒有提示按鈕', async ({ page }) => {
  await expect(page.getByTestId('hint-section')).toBeVisible()
  await expect(page.getByTestId('hint-button')).toHaveCount(0)
})

Step('沒有顯示任何提示內容', async ({ page }) => {
  await expect(page.getByTestId('hint-item')).toHaveCount(0)
})

Step('左欄沒有提示區', async ({ page }) => {
  await expect(page.getByTestId('col-info')).toBeVisible()
  await expect(page.getByTestId('hint-section')).toHaveCount(0)
})

// ---------- 章節地圖 ----------

Step(/^關卡 "([^"]+)" 的節點為 ?(可挑戰|鎖定|已通過)$/, async ({ world }, id: string, state: string) => {
  await expect(world.node(id)).toHaveAttribute('data-state', NODE_STATE[state])
})

Step(/^章節地圖沒有第 ([0-9]+) 章與第 ([0-9]+) 章的區塊$/, async ({ page }, a: string, b: string) => {
  const titles = await page.getByTestId('chapter-title').allInnerTexts()
  for (const n of [a, b]) expect(titles.some((t) => t.startsWith(`第 ${n} 章`))).toBe(false)
})

Step(/^章節區塊標題依序為(「.+」)$/, async ({ page }, list: string) => {
  const titles = await page.locator('[data-testid=chapter-title] > span:first-child').allInnerTexts()
  expect(titles).toEqual(quotes(list))
})

Step(/^第 ([0-9]+) 章顯示「(.+)」$/, async ({ page }, chapter: string, text: string) => {
  const title = page.getByTestId('chapter-title').filter({ hasText: new RegExp(`^第 ${chapter} 章`) })
  await expect(title.getByTestId('chapter-stars')).toHaveText(text)
})

Step(/^關卡 "([^"]+)" 的節點為實心，下方 ([0-9]+) 顆亮星、([0-9]+) 顆暗星$/, async ({ world }, id: string, on: string, off: string) => {
  const node = world.node(id)
  await expect(node).toHaveAttribute('data-state', 'passed')
  await expect(node.locator('.star-icon.is-on')).toHaveCount(Number(on))
  await expect(node.locator('.star-icon:not(.is-on)')).toHaveCount(Number(off))
  const [disc, stars] = [await node.locator('.node-disc').boundingBox(), await node.locator('.node-stars').boundingBox()]
  expect(stars!.y).toBeGreaterThan(disc!.y)
})

Step(/^關卡 "([^"]+)" 的節點為可挑戰，且 Ferris 圖示站在這裡$/, async ({ world }, id: string) => {
  await expect(world.node(id)).toHaveAttribute('data-state', 'available')
  await expect(world.node(id).getByTestId('map-ferris')).toBeVisible()
  await expect(world.page.getByTestId('map-ferris')).toHaveCount(1)
})

Step(/^Ferris 圖示站在關卡 "([^"]+)" 的節點$/, async ({ world }, id: string) => {
  await expect(world.node(id).getByTestId('map-ferris')).toBeVisible()
  await expect(world.page.getByTestId('map-ferris')).toHaveCount(1)
})

Step(/^關卡 "([^"]+)" 的節點有鎖頭圖示，路徑為虛線$/, async ({ world }, id: string) => {
  const node = world.node(id)
  await expect(node.locator('.node-lock')).toHaveText('🔒')
  const path = node.locator('xpath=..').locator('.trail-path')
  await expect(path).toHaveClass(/is-dashed/)
})

Step(/^關卡 "([^"]+)" 的節點下方顯示「(.+)」與 kind 圖示「(.+)」$/, async ({ world }, id: string, label: string, icon: string) => {
  const node = world.node(id)
  const number = (await node.locator('.node-number').innerText()).trim()
  const title = (await node.locator('.node-title').innerText()).replace(icon, '').trim()
  expect(`${number} ${title}`).toBe(label)
  await expect(node.locator('.node-kind-icon')).toHaveText(icon)
})

Step(/^頂部列顯示「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByTestId('level-title')).toHaveText(text)
})

Step(/^玩家點關卡 "([^"]+)" 的節點$/, async ({ world }, id: string) => {
  await world.node(id).click()
})

Step(/^節點旁顯示 tooltip「(.+)」$/, async ({ page }, text: string) => {
  await expect(page.getByRole('tooltip')).toHaveText(text)
})
