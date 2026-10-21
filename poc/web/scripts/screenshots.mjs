// 三種風格各跑一次完整流程並截圖：起始碼 Run → 錯誤卡片 → 加 mut → Run 動畫 → Submit 通關
import { chromium } from '@playwright/test'
import { mkdirSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const OUT = join(dirname(fileURLToPath(import.meta.url)), '../../screenshots')
const BASE = process.env.POC_URL ?? 'http://127.0.0.1:5180'
mkdirSync(OUT, { recursive: true })

const PLAN = [
  { style: 'A', mode: 'light', extraMode: 'dark' },
  { style: 'B', mode: 'light', extraMode: 'dark' },
  { style: 'C', mode: 'dark', extraMode: 'light' },
]

async function addMut(page, re) {
  const line = page.locator('.monaco-editor .view-line').filter({ hasText: re })
  await line.click()
  await page.keyboard.press('Home')
  for (let i = 0; i < 4; i++) await page.keyboard.press('ArrowRight')
  await page.keyboard.type('mut ')
}

const browser = await chromium.launch()
const failures = []
for (const p of PLAN) {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 })
  page.on('pageerror', (e) => failures.push(`${p.style} pageerror: ${e.message}`))
  await page.goto(`${BASE}/?style=${p.style}&mode=${p.mode}`)
  await page.locator('.monaco-editor .view-line').first().waitFor()
  await page.evaluate(() => document.fonts.ready)

  // ① 起始碼 Run：應出現 E0384、E0596 錯誤卡片與波浪線
  await page.locator('.monaco-editor textarea').focus()
  await page.keyboard.press('ControlOrMeta+Enter')
  await page.locator('.err-card').nth(1).waitFor({ timeout: 30000 })
  const codes = await page.locator('.err-card .code').allTextContents()
  const squiggles = await page.locator('.monaco-editor .squiggly-error').count()
  if (!codes.includes('E0384') || !codes.includes('E0596')) failures.push(`${p.style}: 錯誤碼不符 ${codes}`)
  if (squiggles < 2) failures.push(`${p.style}: 波浪線數量 ${squiggles}`)
  await page.screenshot({ path: join(OUT, `${p.style}-errors.png`) })

  // ② 加上 mut 後 Run：Ferris 動畫走到終點
  await addMut(page, /let\s+cmds/)
  await addMut(page, /let\s+steps/)
  await page.getByRole('button', { name: /執行/ }).click()
  await page.locator('.gw-msg.good').waitFor({ timeout: 30000 })
  await page.waitForTimeout(600)
  await page.screenshot({ path: join(OUT, `${p.style}.png`) })

  // ③ 另一種深淺色
  await page.goto(`${BASE}/?style=${p.style}&mode=${p.extraMode}`)
  await page.locator('.monaco-editor .view-line').first().waitFor()
  await addMut(page, /let\s+cmds/)
  await addMut(page, /let\s+steps/)
  await page.getByRole('button', { name: /執行/ }).click()
  await page.locator('.gw-msg.good').waitFor({ timeout: 30000 })
  await page.waitForTimeout(600)
  await page.screenshot({ path: join(OUT, `${p.style}-${p.extraMode}.png`) })

  // ④ Submit 通關
  await page.locator('.monaco-editor textarea').focus()
  await page.keyboard.press('ControlOrMeta+Shift+Enter')
  await page.getByRole('dialog', { name: '通關結算' }).waitFor({ timeout: 30000 })
  await page.waitForTimeout(1600)
  await page.screenshot({ path: join(OUT, `${p.style}-clear.png`) })
  await page.close()
}
await browser.close()
if (failures.length) {
  console.error('FAIL\n' + failures.join('\n'))
  process.exit(1)
}
console.log('OK，截圖輸出到', OUT)
