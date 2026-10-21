import { expect } from '@playwright/test'
import { test as base } from 'playwright-bdd'
import type { Locator, Page } from '@playwright/test'
import { createBdd } from 'playwright-bdd'
import { MockApi } from './mock'

type Json = any

export const PROGRESS_KEY = 'rustquest.progress.v1'

declare global {
  interface Window {
    __rqMonaco?: any
  }
}

// 單一情境的狀態：待寫入的 localStorage、比對用的前值
export class World {
  storageOps: [string, string | null][] = []
  progress: Json | null = null
  originReady = false
  clockInstalled = false
  // Run 送出後要快轉的毫秒數（模擬 60 秒逾時）
  fastForwardMs = 0
  failWrites = false
  currentLevel = ''
  nativeDialogs = 0
  before: Record<string, Json> = {}
  // 「該卡片」「該列」「該指標」指涉的對象
  focusRow: string | null = null
  focusPointer: { from: string; to: string } | null = null

  constructor(
    readonly page: Page,
    readonly mock: MockApi,
  ) {
    page.on('dialog', (d) => {
      this.nativeDialogs++
      d.dismiss().catch(() => {})
    })
  }

  // ---------- localStorage 與導覽 ----------

  async setStorage(key: string, value: string | null) {
    if (this.originReady) {
      await this.page.evaluate(([k, v]) => (v === null ? localStorage.removeItem(k) : localStorage.setItem(k, v)), [key, value] as const)
    } else {
      this.storageOps.push([key, value])
    }
  }

  ensureProgress(): Json {
    if (!this.progress) this.progress = { version: 1, levels: {} }
    return this.progress
  }

  levelProgress(id: string): Json {
    const p = this.ensureProgress()
    if (!p.levels[id]) p.levels[id] = { stars: { pass: false, noHint: false, firstTry: false }, current: { hintsRevealed: 0, failedSubmits: 0 } }
    return p.levels[id]
  }

  async installClock() {
    if (this.clockInstalled) return
    await this.page.clock.install()
    this.clockInstalled = true
  }

  async prepareOrigin() {
    if (this.originReady) return
    await this.page.goto('/favicon.svg')
    const ops = [...this.storageOps]
    if (this.progress) ops.push([PROGRESS_KEY, JSON.stringify(this.progress)])
    await this.page.evaluate((list) => {
      for (const [k, v] of list) {
        if (v === null) localStorage.removeItem(k)
        else localStorage.setItem(k, v)
      }
    }, ops)
    this.storageOps = []
    this.originReady = true
  }

  async open(path: string) {
    await this.prepareOrigin()
    await this.page.goto(path)
    await this.waitReady()
  }

  async waitReady(timeout = 15_000) {
    await this.page.waitForFunction(
      () => {
        if (location.pathname.startsWith('/level/')) {
          if (document.querySelector('[data-testid=not-found]')) return true
          return !!document.querySelector('[data-testid=col-editor] .monaco-editor .view-lines') && !!window.__rqMonaco
        }
        return document.querySelectorAll('[data-testid=level-node]').length > 0 || !!document.querySelector('.map-error')
      },
      undefined,
      { timeout },
    )
    const m = /^\/level\/([^/]+)/.exec(new URL(this.page.url()).pathname)
    if (m) this.currentLevel = decodeURIComponent(m[1])
  }

  async reload() {
    await this.page.reload()
    await this.waitReady()
  }

  async storageItem(key: string): Promise<string | null> {
    return this.page.evaluate((k) => localStorage.getItem(k), key)
  }

  async storedProgress(): Promise<Json> {
    const raw = await this.storageItem(PROGRESS_KEY)
    return raw === null ? null : JSON.parse(raw)
  }

  // ---------- 請求 ----------

  async waitIdle(timeout = this.mock.real ? 60_000 : 15_000) {
    if (!this.mock.real) await expect.poll(() => this.mock.inflight, { timeout }).toBe(0)
    await expect(this.page.getByTestId('btn-run')).not.toHaveText('判題中…', { timeout })
    await expect(this.page.getByTestId('btn-submit')).not.toHaveText('判題中…', { timeout })
  }

  async clickRun() {
    await this.page.getByTestId('btn-run').click()
    await this.afterRequestClick()
  }

  async clickSubmit() {
    await this.page.getByTestId('btn-submit').click()
    await this.afterRequestClick()
  }

  private async afterRequestClick() {
    if (this.fastForwardMs > 0 && this.clockInstalled) {
      await this.page.waitForTimeout(100)
      await this.page.clock.runFor(this.fastForwardMs)
    }
  }

  // ---------- Monaco ----------

  async editorValue(): Promise<string> {
    return this.page.evaluate(() => window.__rqMonaco.editor.getEditors()[0].getValue())
  }

  async setEditorValue(value: string) {
    await this.page.evaluate((v) => window.__rqMonaco.editor.getEditors()[0].setValue(v), value)
  }

  async focusEditorEnd() {
    await this.page.evaluate(() => {
      const ed = window.__rqMonaco.editor.getEditors()[0]
      const model = ed.getModel()
      const line = model.getLineCount()
      ed.focus()
      ed.setPosition({ lineNumber: line, column: model.getLineMaxColumn(line) })
    })
  }

  async typeInEditor(text: string) {
    await this.focusEditorEnd()
    await this.page.keyboard.type(text)
  }

  async markers(): Promise<Json[]> {
    return this.page.evaluate(() => window.__rqMonaco.editor.getModelMarkers({ owner: 'rustc' }))
  }

  async cursor(): Promise<{ lineNumber: number; column: number }> {
    return this.page.evaluate(() => {
      const p = window.__rqMonaco.editor.getEditors()[0].getPosition()
      return { lineNumber: p.lineNumber, column: p.column }
    })
  }

  // 有指定 className 裝飾的行號
  async decoratedLines(className: string, glyph = false): Promise<number[]> {
    return this.page.evaluate(
      ([cls, useGlyph]) => {
        const model = window.__rqMonaco.editor.getEditors()[0].getModel()
        return model
          .getAllDecorations()
          .filter((d: any) => (useGlyph ? d.options.glyphMarginClassName : d.options.className) === cls)
          .map((d: any) => d.range.startLineNumber)
      },
      [className, glyph] as const,
    )
  }

  async lineVisible(line: number): Promise<boolean> {
    return this.page.evaluate((l) => {
      const ranges = window.__rqMonaco.editor.getEditors()[0].getVisibleRanges()
      return ranges.some((r: any) => r.startLineNumber <= l && l <= r.endLineNumber)
    }, line)
  }

  // 編輯器內某位置的頁面座標
  async editorPoint(line: number, column: number): Promise<{ x: number; y: number }> {
    return this.page.evaluate(
      ([l, c]) => {
        const ed = window.__rqMonaco.editor.getEditors()[0]
        ed.revealLineInCenterIfOutsideViewport(l)
        const pos = ed.getScrolledVisiblePosition({ lineNumber: l, column: c })
        const rect = ed.getDomNode().getBoundingClientRect()
        return { x: rect.left + pos.left + 3, y: rect.top + pos.top + pos.height / 2 }
      },
      [line, column] as const,
    )
  }

  async glyphPoint(line: number): Promise<{ x: number; y: number }> {
    return this.page.evaluate((l) => {
      const ed = window.__rqMonaco.editor.getEditors()[0]
      ed.revealLineInCenterIfOutsideViewport(l)
      const layout = ed.getLayoutInfo()
      const top = ed.getTopForLineNumber(l) - ed.getScrollTop()
      const rect = ed.getDomNode().getBoundingClientRect()
      return { x: rect.left + layout.glyphMarginLeft + layout.glyphMarginWidth / 2, y: rect.top + top + 9 }
    }, line)
  }

  // ---------- 畫面元素 ----------

  console(): Locator {
    return this.page.getByTestId('console')
  }

  visual(): Locator {
    return this.page.getByTestId('visual-panel')
  }

  stepLabel(): Locator {
    return this.visual().getByTestId('step-label')
  }

  playButton(): Locator {
    return this.visual().locator('.pb-btn.is-main')
  }

  diagCards(): Locator {
    return this.console().getByTestId('diag-card')
  }

  testRow(name: string): Locator {
    return this.console().locator(`[data-testid=test-row][data-name="${name}"]`)
  }

  memVar(id: string): Locator {
    return this.visual().locator(`[data-testid=mem-var][data-id="${id}"]`)
  }

  memTarget(id: string): Locator {
    return this.visual().locator(`[data-testid=mem-var][data-id="${id}"], [data-testid=mem-heap][data-id="${id}"]`)
  }

  pointer(from: string, to: string): Locator {
    return this.visual().locator(`[data-testid=mem-pointer][data-from="${from}"][data-to="${to}"]`)
  }

  node(id: string): Locator {
    return this.page.locator(`[data-testid=level-node][data-id="${id}"]`)
  }

  completeDialog(): Locator {
    return this.page.locator('.complete-dialog')
  }

  // CSS 變數解析成 rgb() 字串
  async cssVarColor(name: string): Promise<string> {
    return this.page.evaluate((n) => {
      const el = document.createElement('span')
      el.style.color = `var(${n})`
      document.body.appendChild(el)
      const color = getComputedStyle(el).color
      el.remove()
      return color
    }, name)
  }

  async seekTo(step: number) {
    const range = this.visual().locator('input[type=range]')
    await range.evaluate((el, s) => {
      const input = el as HTMLInputElement
      input.value = String(s)
      input.dispatchEvent(new Event('input', { bubbles: true }))
    }, step)
    await expect(this.stepLabel()).toContainText(`第 ${step} /`)
  }

  async currentStep(): Promise<number> {
    const text = (await this.stepLabel().textContent()) ?? ''
    return Number(/第 (\d+) \//.exec(text)?.[1] ?? NaN)
  }

  // 播放結束：請求完成且播放按鈕回到「播放」
  async waitPlaybackEnd() {
    await this.waitIdle()
    for (let i = 0; i < 2000; i++) {
      const label = await this.playButton().getAttribute('aria-label')
      if (label === '播放') {
        // 確認不是剛好在兩步之間
        await this.page.waitForTimeout(50)
        if ((await this.playButton().getAttribute('aria-label')) === '播放') return
      }
      if (this.clockInstalled) await this.page.clock.runFor(1000)
      else await this.page.waitForTimeout(200)
    }
    throw new Error('播放沒有結束')
  }
}

export const test = base.extend<{ mock: MockApi; world: World }>({
  mock: async ({ page }, use) => {
    const mock = new MockApi()
    await page.route(
      (url) => url.pathname.startsWith('/api/'),
      (route) => mock.handle(route),
    )
    await use(mock)
  },
  world: async ({ page, mock }, use) => {
    await use(new World(page, mock))
  },
})

export const { Given, When, Then, Step } = createBdd(test)
export { expect }
