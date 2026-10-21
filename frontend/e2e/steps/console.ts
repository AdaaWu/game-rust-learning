import { expect, Step } from '../support/fixtures'
import type { World } from '../support/fixtures'

const quotes = (text: string) => [...text.matchAll(/「([^」]+)」/g)].map((m) => m[1])

function selectedTab(world: World) {
  return world.console().locator('.console-tab[aria-selected=true]')
}

function outputPanel(world: World) {
  return world.console().locator('[role=tabpanel]').nth(0)
}

function card(world: World, n: string | number) {
  return world.diagCards().nth(Number(n) - 1)
}

// ---------- 分頁 ----------

Step(/^主控台切到「(.+)」分頁$/, async ({ world }, name: string) => {
  await world.waitIdle()
  await expect(selectedTab(world)).toHaveText(new RegExp(`^\\s*${name.replace(/[()]/g, '\\$&')}`))
})

Step(/^玩家切到「(.+)」分頁$/, async ({ world }, name: string) => {
  await world.waitIdle()
  await world.console().locator('.console-tab', { hasText: name }).click()
})

Step(/^「測試結果」分頁標題顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.console().locator('.console-tab[data-tab=tests]')).toContainText(text)
})

Step(/^主控台的 aria-live 區朗讀「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.console().locator('[aria-live]')).toHaveText(text)
})

// ---------- 輸出分頁 ----------

Step('「輸出」分頁頂部顯示綠色列「✓ 執行完成」', async ({ world }) => {
  const bar = outputPanel(world).getByTestId('run-success')
  await expect(bar).toHaveText('✓ 執行完成')
  const first = await outputPanel(world).evaluate((el) => (el.firstElementChild as HTMLElement | null)?.dataset.testid)
  expect(first).toBe('run-success')
  expect(await bar.evaluate((el) => getComputedStyle(el).color)).toBe(await world.cssVarColor('--success'))
})

Step(/^「輸出」分頁顯示 ([0-9]+) 行 stdout，第一行為 "(.+)"$/, async ({ world }, n: string, first: string) => {
  const lines = outputPanel(world).getByTestId('stdout-line')
  await expect(lines).toHaveCount(Number(n))
  await expect(lines.first()).toHaveText(first)
})

Step(/^(?:「輸出」分頁|卡片上方)顯示 stdout "([^"]+)"$/, async ({ world }, text: string) => {
  await expect(outputPanel(world).getByTestId('stdout')).toContainText(text)
  await expectStdoutAboveCard(world)
})

Step(/^卡片上方顯示 stdout ([0-9]+) 行 "([^"]+)"$/, async ({ world }, n: string, text: string) => {
  const lines = outputPanel(world).getByTestId('stdout-line')
  await expect(lines).toHaveCount(Number(n))
  for (const t of await lines.allInnerTexts()) expect(t).toBe(text)
  await expectStdoutAboveCard(world)
})

async function expectStdoutAboveCard(world: World) {
  const cardEl = outputPanel(world).getByTestId('status-card')
  if ((await cardEl.count()) === 0) return
  const s = await outputPanel(world).getByTestId('stdout').boundingBox()
  const c = await cardEl.boundingBox()
  expect(s!.y).toBeLessThan(c!.y)
}

Step(/^主控台「輸出」分頁(?:第一行為|顯示) (".+")$/, async ({ world }, list: string) => {
  await world.waitIdle()
  const texts = await outputPanel(world).getByTestId('stdout-line').allInnerTexts()
  const expected = [...list.matchAll(/"([^"]+)"/g)].map((m) => m[1])
  if (expected.length === 1) expect(texts[0]).toBe(expected[0])
  else for (const e of expected) expect(texts).toContain(e)
})

Step(/^「輸出」分頁頂部有可收合的 warning 卡片，第一行含(「.+」)$/, async ({ world }, list: string) => {
  const cards = outputPanel(world).getByTestId('warning-cards')
  await expect(cards).toBeVisible()
  expect(await cards.evaluate((el) => el.tagName)).toBe('DETAILS')
  const head = await cards.locator('.diag-head').first().textContent()
  for (const q of quotes(list)) expect(head).toContain(q)
})

Step(/^頂部列下方顯示「(.+)」$/, async ({ world, page }, text: string) => {
  await world.waitIdle()
  await expect(page.getByTestId('run-ok-hint')).toHaveText(text)
})

Step(/^主控台 stdout 結尾標「(.+)」$/, async ({ world }, text: string) => {
  const stdout = outputPanel(world).getByTestId('stdout')
  expect((await stdout.innerText()).trim().endsWith(text)).toBe(true)
})

Step(/^主控台 "([^"]+)" 那一行有紅色左邊框與「✗」圖示$/, async ({ world }, text: string) => {
  const line = outputPanel(world).getByTestId('stdout-line').filter({ hasText: text })
  await expect(line).toHaveCount(1)
  await expect(line).toContainText('✗')
  const style = await line.evaluate((el) => {
    const s = getComputedStyle(el)
    return { width: s.borderLeftWidth, color: s.borderLeftColor, style: s.borderLeftStyle }
  })
  expect(style.style).toBe('solid')
  expect(parseFloat(style.width)).toBeGreaterThan(0)
  expect(style.color).toBe(await world.cssVarColor('--danger'))
})

// ---------- 狀態卡片 ----------

Step(/^主控台顯示(灰色)?卡片「(.+)」$/, async ({ world }, gray: string | undefined, title: string) => {
  await world.waitIdle()
  const c = world.console().getByTestId('status-card')
  await expect(c.locator('strong')).toHaveText(title)
  if (gray) await expect(c).toHaveClass(/is-muted/)
})

Step(/^卡片含「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.console().getByTestId('status-card')).toContainText(text)
})

Step(/^卡片不含「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.console().getByTestId('status-card')).not.toContainText(text)
})

Step(/^「輸出」分頁最後一張卡片標題為「(.+)」$/, async ({ world }, title: string) => {
  await world.waitIdle()
  const last = outputPanel(world).locator('article').last()
  await expect(last.locator('strong')).toHaveText(title)
})

Step(/^該卡片顯示 stderr 原文，含 "(.+)"$/, async ({ world }, text: string) => {
  await expect(outputPanel(world).locator('article').last().locator('pre')).toContainText(text)
})

Step(/^主控台顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.console()).toContainText(text)
})

Step(/^主控台頂部顯示「(.+)」$/, async ({ world }, text: string) => {
  const body = world.console().getByTestId('console-body')
  const first = body.locator('> *').first()
  await expect(first).toHaveText(text)
})

Step('主控台顯示骨架列', async ({ world }) => {
  await expect(world.console().getByTestId('console-skeleton').locator('.skeleton-row').first()).toBeVisible()
})

Step(/^主控台仍顯示 ([0-9]+) 張錯誤卡片，並淡化顯示「(.+)」$/, async ({ world }, n: string, text: string) => {
  await expect(world.diagCards()).toHaveCount(Number(n))
  const body = world.console().getByTestId('console-body')
  await expect(body).toHaveClass(/is-stale/)
  await expect(body).toContainText(text)
  const opacity = await body.locator('.console-panel').first().evaluate((el) => Number(getComputedStyle(el).opacity))
  expect(opacity).toBeLessThan(1)
})

// ---------- 錯誤卡片 ----------

Step(/^第 ([0-9]+) 張錯誤卡片第一行含(「.+」)$/, async ({ world }, n: string, list: string) => {
  const head = card(world, n).locator('.diag-head')
  for (const q of quotes(list)) await expect(head).toContainText(q)
})

Step(/^第 ([0-9]+) 張錯誤卡片的人話層文字為「(.+)」$/, async ({ world }, n: string, text: string) => {
  world.focusRow = n
  await expect(card(world, n).getByTestId('diag-hint')).toHaveText(text)
})

Step(/^第 ([0-9]+) 張錯誤卡片的人話為 "(.+)"$/, async ({ world }, n: string, text: string) => {
  await expect(card(world, n).getByTestId('diag-hint')).toHaveText(text)
})

async function expectCode(world: World, names: string[]) {
  const hint = card(world, world.focusRow ?? 1).getByTestId('diag-hint')
  const codes = await hint.locator('code').allInnerTexts()
  for (const name of names) expect(codes).toContain(name)
  expect(await hint.innerText()).not.toContain('`')
}

Step(/^該人話層中 "([^"]+)" 與 "([^"]+)" 以行內程式碼 <code> 呈現，畫面不顯示反引號字元$/, async ({ world }, a: string, b: string) => {
  await expectCode(world, [a, b])
})

Step(/^該人話層中 "([^"]+)" 以行內程式碼 <code> 呈現，畫面不顯示反引號字元$/, async ({ world }, a: string) => {
  await expectCode(world, [a])
})

Step(/^該人話層中 "([^"]+)" 不在 <code> 內，"([^"]+)" 以行內程式碼 <code> 呈現$/, async ({ world }, plain: string, code: string) => {
  const hint = card(world, world.focusRow ?? 1).getByTestId('diag-hint')
  const codes = await hint.locator('code').allInnerTexts()
  expect(codes).toContain(code)
  expect(codes.some((c) => c.includes(plain))).toBe(false)
  await expect(hint).toContainText(plain)
})

Step(/^第 ([0-9]+) 張錯誤卡片的「rustc 原文」為收合狀態$/, async ({ world }, n: string) => {
  const raw = card(world, n).getByTestId('diag-raw')
  expect(await raw.evaluate((el) => (el as HTMLDetailsElement).open)).toBe(false)
  await expect(raw.locator('pre')).toBeHidden()
})

Step(/^第 ([0-9]+) 張錯誤卡片有「(\S+) 官方說明」連結，指向 "([^"]+)" 並在新分頁開啟$/, async ({ world }, n: string, code: string, url: string) => {
  const link = card(world, n).getByTestId('diag-doc')
  await expect(link).toContainText(`${code} 官方說明`)
  await expect(link).toHaveAttribute('href', url)
  await expect(link).toHaveAttribute('target', '_blank')
})

Step(/^第 ([0-9]+) 張錯誤卡片的錯誤碼「(\S+)」本身是指向同一網址的連結$/, async ({ world }, n: string, code: string) => {
  const c = card(world, n)
  const href = await c.getByTestId('diag-doc').getAttribute('href')
  const link = c.getByTestId('diag-code')
  await expect(link).toHaveText(code)
  await expect(link).toHaveAttribute('href', href!)
})

Step(/^第 ([0-9]+) 張錯誤卡片沒有「官方說明」連結$/, async ({ world }, n: string) => {
  await expect(card(world, n).getByTestId('diag-doc')).toHaveCount(0)
  await expect(card(world, n)).not.toContainText('官方說明')
})

Step(/^([0-9]+) 張錯誤卡片的位置欄都顯示「(.+)」$/, async ({ world }, n: string, text: string) => {
  await expect(world.diagCards()).toHaveCount(Number(n))
  for (let i = 0; i < Number(n); i++) await expect(world.diagCards().nth(i).locator('.diag-head span').nth(1)).toHaveText(text)
})

Step(/^第 ([0-9]+) 張錯誤卡片的位置欄顯示「(.+)」$/, async ({ world }, n: string, text: string) => {
  await expect(card(world, n).locator('.diag-head span').nth(1)).toHaveText(text)
})

Step(/^玩家點第 ([0-9]+) 張錯誤卡片$/, async ({ world }, n: string) => {
  await world.waitIdle()
  world.before.cursor = await world.cursor()
  await card(world, n).locator('.diag-hint').click()
})

Step(/^玩家點第 ([0-9]+) 張錯誤卡片的「顯示 rustc 原文」$/, async ({ world }, n: string) => {
  await world.waitIdle()
  world.focusRow = n
  await card(world, n).getByTestId('diag-raw').locator('summary').click()
})

Step('卡片以等寬字型顯示該筆 Diagnostic 的 rendered 全文', async ({ world, mock }) => {
  const pre = card(world, world.focusRow ?? 1).getByTestId('diag-raw').locator('pre')
  await expect(pre).toBeVisible()
  const rendered = mock.runReplies[0].json.compile.diagnostics[0].rendered
  expect(await pre.textContent()).toBe(rendered)
  const font = await pre.evaluate((el) => getComputedStyle(el).fontFamily)
  expect(font).toMatch(/mono/i)
})

Step(/^原文含 "(.+)"$/, async ({ world }, text: string) => {
  await expect(card(world, world.focusRow ?? 1).getByTestId('diag-raw').locator('pre')).toContainText(text)
})

Step(/^第 ([0-9]+) 張錯誤卡片的原文區顯示 "(.+)"$/, async ({ world }, n: string, text: string) => {
  const pre = card(world, n).getByTestId('diag-raw').locator('pre')
  await expect(pre).toBeVisible()
  await expect(pre).toHaveText(text)
})

Step(/^錯誤卡片依序為 "(.+)"$/, async ({ world }, list: string) => {
  const expected = list.split(',').map((s) => s.trim())
  await expect(world.diagCards()).toHaveCount(expected.length)
  const actual: string[] = []
  for (let i = 0; i < expected.length; i++) {
    const c = world.diagCards().nth(i)
    const code = await c.getAttribute('data-code')
    actual.push(code || ((await c.locator('.diag-head').textContent())?.includes('警告') ? 'warning' : 'error'))
  }
  expect(actual).toEqual(expected)
})

Step(/^展開顯示 ([0-9]+) 張錯誤卡片$/, async ({ world }, n: string) => {
  await expect(world.diagCards().locator('visible=true')).toHaveCount(Number(n))
})

Step(/^顯示「還有 ([0-9]+) 筆」$/, async ({ world }, n: string) => {
  await expect(world.console().locator('.more-cards > summary')).toHaveText(`還有 ${n} 筆`)
})

Step(/^只顯示 ([0-9]+) 張錯誤卡片$/, async ({ world }, n: string) => {
  await world.waitIdle()
  await expect(world.diagCards()).toHaveCount(Number(n))
})

// ---------- 測試結果 ----------

Step(/^測試結果依序為：$/, async ({ world }, table: any) => {
  await world.waitIdle()
  const rows = world.console().getByTestId('test-row')
  const expected = table.hashes() as Record<string, string>[]
  await expect(rows).toHaveCount(expected.length)
  for (let i = 0; i < expected.length; i++) {
    const row = rows.nth(i)
    await expect(row).toHaveAttribute('data-name', expected[i].name)
    await expect(row.locator('.test-icon')).toHaveText(expected[i].icon)
    await expect(row.locator('.test-status')).toHaveText(expected[i].text)
  }
})

Step(/^只有 "([^"]+)" 那一列預設展開$/, async ({ world }, name: string) => {
  const rows = world.console().getByTestId('test-row')
  const n = await rows.count()
  for (let i = 0; i < n; i++) {
    const row = rows.nth(i)
    const expanded = (await row.getAttribute('data-name')) === name
    await expect(row).toHaveAttribute('data-expanded', String(expanded))
    await expect(row.locator('.test-detail')).toHaveCount(expanded ? 1 : 0)
  }
})

Step(/^玩家展開 "([^"]+)" 那一列$/, async ({ world }, name: string) => {
  world.focusRow = name
  await world.testRow(name).getByTestId('test-toggle').click()
  await expect(world.testRow(name)).toHaveAttribute('data-expanded', 'true')
})

Step(/^該列標題顯示「(.+)」$/, async ({ world }, text: string) => {
  await expect(world.testRow(world.focusRow!).getByTestId('assert-title')).toHaveText(text)
})

Step(/^該列並排顯示「實際（left）」為 "(.+)"、「預期（right）」為 "(.+)"$/, async ({ world }, actual: string, expected: string) => {
  const row = world.testRow(world.focusRow!)
  const a = row.getByTestId('assert-actual')
  const e = row.getByTestId('assert-expected')
  await expect(a).toContainText('實際（left）')
  await expect(a.locator('code')).toHaveText(actual)
  await expect(e).toContainText('預期（right）')
  await expect(e.locator('code')).toHaveText(expected)
  const [ba, be] = [await a.boundingBox(), await e.boundingBox()]
  expect(Math.abs(ba!.y - be!.y)).toBeLessThan(4)
  expect(ba!.x).toBeLessThan(be!.x)
})

Step('差異處反白', async ({ world }) => {
  const diff = world.testRow(world.focusRow!).locator('.is-diff')
  await expect(diff).toHaveCount(1)
  const bg = await diff.evaluate((el) => getComputedStyle(el).backgroundColor)
  expect(bg).not.toBe('rgba(0, 0, 0, 0)')
})

Step(/^"([^"]+)" 那一列顯示 "(.+)"$/, async ({ world }, name: string, text: string) => {
  world.focusRow = name
  await world.waitIdle()
  await expect(world.testRow(name)).toContainText(text)
})

Step('該列沒有「實際（left）」「預期（right）」欄', async ({ world }) => {
  const row = world.testRow(world.focusRow!)
  await expect(row.getByTestId('assert-actual')).toHaveCount(0)
  await expect(row.getByTestId('assert-expected')).toHaveCount(0)
})

// ---------- 測試程式中途當掉 ----------

function testsPanel(world: World) {
  return world.console().locator('[role=tabpanel]').nth(2)
}

Step(/^「測試結果」分頁頂部顯示紅色卡片「(.+)」$/, async ({ world }, title: string) => {
  await world.waitIdle()
  const c = testsPanel(world).getByTestId('test-crash-card')
  await expect(c.locator('strong')).toHaveText(title)
  await expect(c).toHaveClass(/is-danger/)
  const first = await testsPanel(world).evaluate((el) => (el.firstElementChild as HTMLElement | null)?.dataset.testid)
  expect(first).toBe('test-crash-card')
})

Step(/^該卡片以等寬字型顯示 stderr 原文，含 "(.+)" 與 "(.+)"$/, async ({ world, mock }, a: string, b: string) => {
  const pre = testsPanel(world).getByTestId('test-crash-card').locator('pre')
  expect((await pre.textContent())?.trim()).toBe(mock.submitReplies[0].json.execution.stderr.trim())
  await expect(pre).toContainText(a)
  await expect(pre).toContainText(b)
  expect(await pre.evaluate((el) => getComputedStyle(el).fontFamily)).toMatch(/mono/i)
})

Step('沒有任何測試列展開', async ({ world }) => {
  const rows = world.console().getByTestId('test-row')
  expect(await rows.count()).toBeGreaterThan(0)
  for (const row of await rows.all()) await expect(row).toHaveAttribute('data-expanded', 'false')
  await expect(world.console().locator('.test-detail')).toHaveCount(0)
})

Step(/^「測試結果」分頁沒有「(.+)」卡片$/, async ({ world }, title: string) => {
  await world.waitIdle()
  await expect(testsPanel(world).getByText(title)).toHaveCount(0)
  await expect(testsPanel(world).getByTestId('test-crash-card')).toHaveCount(0)
})

Step(/^"([^"]+)" 那一列顯示「(.)」圖示與文字「(.+)」，為灰色$/, async ({ world }, name: string, icon: string, text: string) => {
  const row = world.testRow(name)
  await expect(row.locator('.test-icon')).toHaveText(icon)
  await expect(row.locator('.test-status')).toHaveText(text)
  const color = await row.locator('.test-status').evaluate((el) => getComputedStyle(el).color)
  expect(color).toBe(await world.cssVarColor('--text-muted'))
})

Step(/^點擊 "([^"]+)" 那一列不會展開$/, async ({ world }, name: string) => {
  const row = world.testRow(name)
  await expect(row.getByTestId('test-toggle')).toHaveCount(0)
  await row.click()
  await expect(row).toHaveAttribute('data-expanded', 'false')
  await expect(row.locator('.test-detail')).toHaveCount(0)
})
