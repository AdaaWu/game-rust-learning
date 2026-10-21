import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'

function grid(world: World) {
  return world.page.getByTestId('grid-world')
}

const at = (x: string, y: string) => `[data-x="${x}"][data-y="${y}"]`

// ---------- 播放控制（GridWorld 與 MemoryView 共用）----------

Step(/^播放控制列顯示「(第 \d+ \/ \d+ 步)」$/, async ({ world }, text: string) => {
  await expect(world.stepLabel()).toHaveText(text)
})

Step(/^約 500 ms 後播放控制列顯示「(第 \d+ \/ \d+ 步)」$/, async ({ world }, text: string) => {
  await expect(world.stepLabel()).toHaveText(text, { timeout: 1000 })
})

Step(/^播放按鈕顯示為「(播放|暫停)」$/, async ({ world }, label: string) => {
  await expect(world.playButton()).toHaveAttribute('aria-label', label)
})

Step('播放已暫停', async ({ world, page }) => {
  await expect(world.playButton()).toHaveAttribute('aria-label', '播放')
  const step = await world.currentStep()
  if (world.clockInstalled) await page.clock.runFor(1500)
  else await page.waitForTimeout(800)
  expect(await world.currentStep()).toBe(step)
})

Step('步數比按「單步」前多 1', async ({ world }) => {
  expect(await world.currentStep()).toBe(world.before.step + 1)
})

Step(/^玩家點進度條跳到第 ([0-9]+) 步$/, async ({ world }, step: string) => {
  await world.waitIdle()
  await world.seekTo(Number(step))
})

Step(/^玩家把速度改為 "(.+)"$/, async ({ world }, speed: string) => {
  await world.visual().getByLabel('速度').selectOption({ label: speed })
})

Step(/^速度選單顯示 "(.+)"$/, async ({ world }, speed: string) => {
  const select = world.visual().getByLabel('速度')
  expect(await select.evaluate((el) => (el as HTMLSelectElement).selectedOptions[0].text)).toBe(speed)
})

Step('焦點在可視化區', async ({ world }) => {
  await world.visual().locator('[tabindex="0"]').first().focus()
})

Step(/^播放停在第 ([0-9]+) 步(?:，進度條總步數為 ([0-9]+))?$/, async ({ world }, step: string, total: string | undefined) => {
  await expect(world.playButton()).toHaveAttribute('aria-label', '播放')
  expect(await world.currentStep()).toBe(Number(step))
  if (total) await expect(world.stepLabel()).toHaveText(`第 ${step} / ${total} 步`)
})

Step(/^進度條總步數為 ([0-9]+)$/, async ({ world }, total: string) => {
  await expect(world.stepLabel()).toHaveText(new RegExp(`/ ${total} 步$`))
  await expect(world.visual().locator('input[type=range]')).toHaveAttribute('max', total)
})

// ---------- 地圖 ----------

Step(/^Ferris 位於 \(([0-9]+), ([0-9]+)\)$/, async ({ world }, x: string, y: string) => {
  await world.waitIdle()
  const f = grid(world).getByTestId('ferris')
  await expect(f).toHaveAttribute('data-x', x)
  await expect(f).toHaveAttribute('data-y', y)
})

Step(/^地圖有 ([0-9]+) 個牆格、([0-9]+) 顆寶石、([0-9]+) 扇關閉的門、([0-9]+) 個終點旗子$/, async ({ world }, walls: string, gems: string, doors: string, goals: string) => {
  await expect(grid(world).getByTestId('wall')).toHaveCount(Number(walls))
  await expect(grid(world).getByTestId('gem')).toHaveCount(Number(gems))
  await expect(grid(world).locator('[data-testid=door][data-open=false]')).toHaveCount(Number(doors))
  await expect(grid(world).getByTestId('goal')).toHaveCount(Number(goals))
})

Step(/^地圖右上角顯示「(.+)」$/, async ({ world }, text: string) => {
  const counter = grid(world).getByTestId('gem-counter')
  await expect(counter).toHaveText(text)
  const [c, stage] = [await counter.boundingBox(), await grid(world).locator('.grid-stage').boundingBox()]
  expect(c!.y - stage!.y).toBeLessThan(20)
  expect(stage!.x + stage!.width - (c!.x + c!.width)).toBeLessThan(20)
})

Step(/^\(([0-9]+), ([0-9]+)\) 的寶石已消失$/, async ({ world }, x: string, y: string) => {
  await expect(grid(world).locator(`[data-testid=gem]${at(x, y)}`)).toHaveCount(0)
})

Step(/^\(([0-9]+), ([0-9]+)\) 的門顯示為開啟狀態、格子為可通行底色$/, async ({ world }, x: string, y: string) => {
  const door = grid(world).locator(`[data-testid=door]${at(x, y)}`)
  await expect(door).toHaveAttribute('data-open', 'true')
  // 開啟後只剩虛線門框，沒有實心門板
  await expect(door.locator('path')).toHaveCount(0)
  await expect(door).not.toContainText('🔒')
})

Step(/^\(([0-9]+), ([0-9]+)\) 的牆格閃紅框 ([0-9]+) 次$/, async ({ world }, x: string, y: string, times: string) => {
  const wall = grid(world).locator(`[data-testid=wall]${at(x, y)}`)
  await expect(wall).toHaveAttribute('data-hit', 'true')
  const frame = wall.locator('.hit-frame')
  const style = await frame.evaluate((el) => {
    const s = getComputedStyle(el)
    return { stroke: s.stroke, count: s.animationIterationCount, name: s.animationName }
  })
  expect(style.stroke).toBe(await world.cssVarColor('--danger'))
  expect(style.count).toBe(times)
  expect(style.name).not.toBe('none')
})

Step(/^\(([0-9]+), ([0-9]+)\) 的門格上顯示「(.+)」$/, async ({ world }, x: string, y: string, icon: string) => {
  await expect(grid(world).locator(`[data-testid=door]${at(x, y)}`)).toContainText(icon)
})

Step(/^Ferris 頭上顯示「(.+)」$/, async ({ world }, icon: string) => {
  await expect(grid(world).getByTestId('ferris-badge')).toHaveText(icon)
})

Step(/^訊息列顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(grid(world).getByTestId('grid-message')).toHaveText(text)
})

Step(/^訊息列含「(.+)」$/, async ({ world }, text: string) => {
  await expect(grid(world).getByTestId('grid-message')).toContainText(text)
})

Step(/^可視化區(?:標示|顯示)「(.+)」$/, async ({ world }, text: string) => {
  await world.waitIdle()
  await expect(grid(world).getByTestId('grid-message')).toHaveText(text)
})

Step('Ferris 跳一下、終點旗幟揮動', async ({ world }) => {
  await expect(grid(world).locator('[data-testid=ferris] .ferris-fx')).toHaveClass(/is-jumping/)
  await expect(grid(world).getByTestId('goal')).toHaveAttribute('data-waving', 'true')
})

Step('Ferris 沒有跳一下、終點旗幟沒有揮動', async ({ world }) => {
  await expect(grid(world).locator('[data-testid=ferris] .ferris-fx')).not.toHaveClass(/is-jumping/)
  await expect(grid(world).getByTestId('goal')).toHaveAttribute('data-waving', 'false')
})

Step('沒有無效指令提示', async ({ world }) => {
  await expect(world.console().locator('.stdout-line.is-invalid')).toHaveCount(0)
  await expect(grid(world).getByTestId('grid-message')).not.toContainText('看不懂')
  await expect(grid(world).getByTestId('ferris-badge').filter({ hasText: '❓' })).toHaveCount(0)
})

Step(/^可視化區(為|不是)半透明$/, async ({ world }, state: string) => {
  const stage = grid(world).locator('.grid-stage')
  if (state === '為') await expect.poll(() => stage.evaluate((el) => Number(getComputedStyle(el).opacity))).toBeLessThan(1)
  else {
    await world.waitIdle()
    await expect.poll(() => stage.evaluate((el) => Number(getComputedStyle(el).opacity))).toBe(1)
  }
})

Step(/^Ferris 從 \(([0-9]+), ([0-9]+)\) 到 \(([0-9]+), ([0-9]+)\) 沒有位移補間動畫$/, async ({ world }, _x1: string, _y1: string, x2: string, y2: string) => {
  const f = grid(world).getByTestId('ferris')
  await expect(f).toHaveAttribute('data-x', x2)
  await expect(f).toHaveAttribute('data-y', y2)
  const duration = await f.evaluate((el) => getComputedStyle(el).transitionDuration)
  expect(duration.split(',').every((d) => parseFloat(d) === 0)).toBe(true)
})

Step(/^aria-live 區含 ?(?:「(.+)」|"(.+)")$/, async ({ page }, a: string | undefined, b: string | undefined) => {
  const text = (a ?? b)!
  await expect(page.locator('[aria-live]', { hasText: text }).first()).toBeAttached()
})
