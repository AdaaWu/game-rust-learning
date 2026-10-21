import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory } from 'vue-router'
import type { Router } from 'vue-router'
import type { LevelDetail } from '../../api/types'
import { chapters, clearRoute, diag, gridLevel, levels, memoryLevel, runCompileError, runOk, submitResult } from '../../test/fixtures'
import { PROGRESS_KEY } from '../../domain/progress'
import { timelineLevel, tl33Starter } from '../../test/timelineFixtures'
import type { RunResponse } from '../../api/types'

const api = vi.hoisted(() => ({
  listLevels: vi.fn(),
  getLevel: vi.fn(),
  run: vi.fn(),
  submit: vi.fn(),
  health: vi.fn(),
}))
vi.mock('../../api', () => ({ api }))

// Monaco 在 jsdom 無法運作：以 textarea 代替 CodeEditor
const editorSpy = vi.hoisted(() => ({ revealPosition: vi.fn() }))
vi.mock('../../components/CodeEditor.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      name: 'CodeEditor',
      props: ['modelValue', 'diagnostics', 'panicStderr', 'highlightLine', 'theme'],
      emits: ['update:modelValue', 'run', 'submit', 'gutter-click'],
      setup(props, { emit, expose }) {
        expose({ revealPosition: editorSpy.revealPosition })
        return () =>
          h('div', [
            h('textarea', {
              'data-testid': 'editor',
              'data-markers': String((props.diagnostics as unknown[] | undefined)?.length ?? 0),
              'data-highlight-line': props.highlightLine == null ? '' : String(props.highlightLine),
              value: props.modelValue,
              onInput: (e: Event) => emit('update:modelValue', (e.target as HTMLTextAreaElement).value),
            }),
            // 模擬點第 10 行行號左側 ▶ 區域
            h('button', { 'data-testid': 'editor-gutter-10', onClick: () => emit('gutter-click', 10) }),
          ])
      },
    }),
  }
})

import LevelView from '../LevelView.vue'
import { createAppRouter } from '../../router'

const ID = '01-hello-ferris'
const DRAFT_KEY = `rustquest.draft.${ID}`

let reducedMotion = false

function seed(levelsProgress: Record<string, { pass?: boolean; noHint?: boolean; firstTry?: boolean; hintsRevealed?: number; failedSubmits?: number }>) {
  const out: Record<string, unknown> = {}
  for (const [id, p] of Object.entries(levelsProgress)) {
    out[id] = {
      stars: { pass: p.pass ?? false, noHint: p.noHint ?? false, firstTry: p.firstTry ?? false },
      current: { hintsRevealed: p.hintsRevealed ?? 0, failedSubmits: p.failedSubmits ?? 0 },
    }
  }
  localStorage.setItem(PROGRESS_KEY, JSON.stringify({ version: 1, levels: out }))
}

function details(...list: LevelDetail[]) {
  api.getLevel.mockImplementation(async (id: string) => {
    const found = list.find((l) => l.id === id)
    return found ? { ok: true, data: found } : { ok: false, failure: { kind: 'level_not_found' } }
  })
}

async function openLevel(id: string): Promise<{ w: VueWrapper; router: Router }> {
  // 09 關在第 3 章，需先通過前兩章才解鎖
  if (id === '09-move-the-gem' && !localStorage.getItem(PROGRESS_KEY)) {
    seed({ '01-hello-ferris': { pass: true }, '02-mut-matters': { pass: true }, '05-delta': { pass: true } })
  }
  const pinia = createPinia()
  setActivePinia(pinia)
  const router = createAppRouter(createMemoryHistory())
  await router.push(`/level/${id}`)
  const w = mount(LevelView, { global: { plugins: [pinia, router] }, attachTo: document.body })
  await flushPromises()
  return { w, router }
}

const btn = (w: VueWrapper, name: 'run' | 'submit' | 'reset') => w.get(`[data-testid="btn-${name}"]`)
const confirmText = (w: VueWrapper) => w.get('[data-testid="confirm-dialog"] [data-testid="confirm-message"]').text()
const isDisabled = (w: VueWrapper, name: 'run' | 'submit' | 'reset') => btn(w, name).attributes('disabled') !== undefined

describe('F7 LevelView 關卡頁', () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    localStorage.clear()
    document.body.innerHTML = ''
    reducedMotion = false
    Object.values(api).forEach((f) => f.mockReset())
    api.listLevels.mockResolvedValue({ ok: true, data: { chapters, levels } })
    api.health.mockResolvedValue({ ok: true, data: { status: 'ok', judge: { available: true, reason: null } } })
    details(gridLevel(), memoryLevel(), gridLevel({ id: '02-mut-matters', order: 2, title: '黏住的腳', kind: 'fix-compile' }))
    window.matchMedia = vi.fn((query: string) => ({
      matches: reducedMotion && query.includes('reduce'),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })) as unknown as typeof window.matchMedia
    Element.prototype.scrollIntoView = vi.fn()
  })
  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
    delete document.documentElement.dataset.theme
  })

  describe('版面與內容', () => {
    it('桌機三欄：左資訊、中編輯器、右結果', async () => {
      const { w } = await openLevel(ID)
      expect(w.find('[data-testid="col-info"]').exists()).toBe(true)
      expect(w.find('[data-testid="col-editor"]').exists()).toBe(true)
      expect(w.find('[data-testid="col-result"]').exists()).toBe(true)
    })

    it('頂部列：第 N 章 章節名稱 ▸ 編號 關卡名，附回地圖連結', async () => {
      const { w } = await openLevel(ID)
      expect(w.get('[data-testid="level-title"]').text()).toBe('第 1 章 沙灘甦醒 ▸ 1-1 第一步')
      expect(w.get('a[data-testid="back-to-map"]').attributes('href')).toBe('/')
    })

    it('頂部列顯示本關歷史星等', async () => {
      seed({ [ID]: { pass: true, firstTry: true } })
      const { w } = await openLevel(ID)
      expect(w.findAll('[data-testid="level-stars"] [data-testid="star-icon"]').map((s) => s.text())).toEqual(['★', '☆', '★'])
    })

    it('Ferris 泡泡顯示 story（markdown、過濾 XSS），沒有「下一句」', async () => {
      details(gridLevel({ story: '**Ferris** 醒來 <img src=x onerror="alert(1)">' }))
      const { w } = await openLevel(ID)
      const bubble = w.get('[data-testid="ferris-bubble"]')
      expect(bubble.find('strong').text()).toBe('Ferris')
      expect(bubble.html()).not.toContain('onerror')
      expect(bubble.text()).not.toContain('下一句')
    })

    it('concept 以 Markdown 渲染', async () => {
      const { w } = await openLevel(ID)
      expect(w.get('[data-testid="concept"] h2').text()).toBe('函式與回傳值')
    })

    it.each([
      ['build', '任務：完成下面的程式'],
      ['fix-compile', '這段程式編不過，先按執行看錯誤訊息，再修好它'],
      ['debug', '程式能跑，但結果不對，找出 bug'],
    ] as const)('%s 任務橫幅', async (kind, text) => {
      details(gridLevel({ kind }))
      const { w } = await openLevel(ID)
      expect(w.get('[data-testid="kind-banner"]').text()).toBe(text)
    })

    it('fix-compile 關卡開啟時不自動 Run', async () => {
      details(gridLevel({ kind: 'fix-compile' }))
      await openLevel(ID)
      expect(api.run).not.toHaveBeenCalled()
    })

    it('進入關卡頁時檢查一次 health', async () => {
      await openLevel(ID)
      expect(api.health).toHaveBeenCalledTimes(1)
    })

    it('visual = grid 顯示 GridWorld', async () => {
      const { w } = await openLevel(ID)
      expect(w.find('[data-testid="grid-world"]').exists()).toBe(true)
    })

    it('visual = memory 顯示 MemoryView 第 1 步', async () => {
      const { w } = await openLevel('09-move-the-gem')
      expect(w.get('[data-testid="mem-caption"]').text()).toBe('第 1 步 · 第 8 行')
    })

    it('visual = none 時可視化區隱藏', async () => {
      details(gridLevel({ visual: 'none', grid: null }))
      const { w } = await openLevel(ID)
      expect(w.find('[data-testid="visual-panel"]').exists()).toBe(false)
      expect(w.find('[data-testid="console"]').exists()).toBe(true)
    })

    it('關卡不存在：顯示找不到這關與回地圖連結', async () => {
      const { w } = await openLevel('99-unknown')
      const nf = w.get('[data-testid="not-found"]')
      expect(nf.text()).toContain('找不到這關')
      expect(nf.get('a').attributes('href')).toBe('/')
    })
  })

  describe('提示', () => {
    it('hints 為空陣列時不顯示提示區', async () => {
      details(gridLevel({ hints: [] }))
      const { w } = await openLevel(ID)
      expect(w.find('[data-testid="hint-section"]').exists()).toBe(false)
    })

    it('預設只顯示按鈕「需要提示嗎？（0 / 3）」', async () => {
      const { w } = await openLevel(ID)
      expect(w.get('[data-testid="hint-button"]').text()).toBe('需要提示嗎？（0 / 3）')
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(0)
    })

    it('第一次看提示先彈出確認框；取消則不揭露', async () => {
      const { w } = await openLevel(ID)
      await w.get('[data-testid="hint-button"]').trigger('click')
      expect(confirmText(w)).toBe('看提示會拿不到『沒看提示』這顆星，確定要看嗎？')
      await w.get('[data-testid="confirm-cancel"]').trigger('click')
      expect(w.find('[data-testid="confirm-dialog"]').exists()).toBe(false)
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(0)
    })

    it('不使用瀏覽器原生 confirm', async () => {
      const native = vi.spyOn(window, 'confirm')
      const { w } = await openLevel(ID)
      await w.get('[data-testid="hint-button"]').trigger('click')
      expect(native).not.toHaveBeenCalled()
    })

    it('確認後揭露第一則並寫入計數；之後不再確認', async () => {
      const { w } = await openLevel(ID)
      await w.get('[data-testid="hint-button"]').trigger('click')
      await w.get('[data-testid="confirm-ok"]').trigger('click')
      await w.get('[data-testid="hint-button"]').trigger('click')
      expect(w.find('[data-testid="confirm-dialog"]').exists()).toBe(false)
      expect(w.findAll('[data-testid="hint-item"]').map((h) => h.text())).toEqual(['提示一', '提示二'])
      expect(w.get('[data-testid="hint-button"]').text()).toBe('需要提示嗎？（2 / 3）')
      expect(JSON.parse(localStorage.getItem(PROGRESS_KEY)!).levels[ID].current.hintsRevealed).toBe(2)
    })

    it('全部揭露後按鈕隱藏', async () => {
      seed({ [ID]: { hintsRevealed: 2 } })
      const { w } = await openLevel(ID)
      await w.get('[data-testid="hint-button"]').trigger('click')
      expect(w.find('[data-testid="hint-button"]').exists()).toBe(false)
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(3)
    })

    it('進入關卡時依 current.hintsRevealed 還原已揭露提示', async () => {
      seed({ [ID]: { hintsRevealed: 2 } })
      const { w } = await openLevel(ID)
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(2)
    })
  })

  describe('草稿與重設', () => {
    it('有草稿時載入草稿', async () => {
      localStorage.setItem(DRAFT_KEY, 'fn main() { draft }')
      const { w } = await openLevel(ID)
      expect((w.get('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('fn main() { draft }')
    })

    it('沒有草稿時載入 starter', async () => {
      const { w } = await openLevel(ID)
      expect((w.get('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('fn main() {}\n')
    })

    it('編輯後 500ms 寫入草稿', async () => {
      const { w } = await openLevel(ID)
      await w.get('[data-testid="editor"]').setValue('fn main() { x }')
      vi.advanceTimersByTime(500)
      expect(localStorage.getItem(DRAFT_KEY)).toBe('fn main() { x }')
    })

    it('重設：先確認，確定後改回 starter 並刪除草稿，不影響本次遊玩計數', async () => {
      seed({ [ID]: { hintsRevealed: 1, failedSubmits: 2 } })
      localStorage.setItem(DRAFT_KEY, 'changed')
      const { w } = await openLevel(ID)
      await btn(w, 'reset').trigger('click')
      expect(confirmText(w)).toBe('重設會把程式碼改回起始版本，目前的修改會消失，確定嗎？')
      expect((w.get('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('changed')
      await w.get('[data-testid="confirm-ok"]').trigger('click')
      expect((w.get('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('fn main() {}\n')
      vi.advanceTimersByTime(1000)
      expect(localStorage.getItem(DRAFT_KEY)).toBeNull()
      expect(JSON.parse(localStorage.getItem(PROGRESS_KEY)!).levels[ID].current).toEqual({ hintsRevealed: 1, failedSubmits: 2 })
    })

    it('重設取消時不變', async () => {
      localStorage.setItem(DRAFT_KEY, 'changed')
      const { w } = await openLevel(ID)
      await btn(w, 'reset').trigger('click')
      await w.get('[data-testid="confirm-cancel"]').trigger('click')
      expect(localStorage.getItem(DRAFT_KEY)).toBe('changed')
      expect((w.get('[data-testid="editor"]').element as HTMLTextAreaElement).value).toBe('changed')
    })

    it('程式碼已與 starter 相同時重設不彈確認', async () => {
      const { w } = await openLevel(ID)
      await btn(w, 'reset').trigger('click')
      expect(w.find('[data-testid="confirm-dialog"]').exists()).toBe(false)
    })
  })

  describe('Run／Submit', () => {
    it('Run 請求中：按鈕顯示判題中、其他按鈕 disabled、編輯器可編輯、主控台骨架列', async () => {
      api.run.mockReturnValue(new Promise(() => {}))
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      expect(btn(w, 'run').text()).toContain('判題中…')
      expect(isDisabled(w, 'submit')).toBe(true)
      expect(isDisabled(w, 'reset')).toBe(true)
      expect(w.get('[data-testid="editor"]').attributes('readonly')).toBeUndefined()
      expect(w.find('[data-testid="console-skeleton"]').exists()).toBe(true)
      expect(api.run).toHaveBeenCalledWith(ID, 'fn main() {}\n', expect.any(AbortSignal))
    })

    it('Run 成功：提示按送出驗收、GridWorld 自動播放', async () => {
      api.run.mockResolvedValue({ ok: true, data: runOk(clearRoute) })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="run-ok-hint"]').text()).toBe('看起來可以了，按『送出』驗收')
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 1 / 20 步')
      expect(w.find('[role="dialog"]').exists()).toBe(false)
    })

    it('compile_error：把 marker 交給編輯器', async () => {
      api.run.mockResolvedValue({ ok: true, data: runCompileError([diag(), diag({ line: 2 })]) })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="editor"]').attributes('data-markers')).toBe('2')
    })

    it('修改程式碼後清除 marker、主控台顯示程式已修改', async () => {
      api.run.mockResolvedValue({ ok: true, data: runCompileError() })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await w.get('[data-testid="editor"]').setValue('fn main() {} ')
      expect(w.get('[data-testid="editor"]').attributes('data-markers')).toBe('0')
      expect(w.get('[data-testid="console"]').text()).toContain('程式已修改')
    })

    it('Run 結果不影響 MemoryView', async () => {
      api.run.mockResolvedValue({ ok: true, data: runCompileError() })
      const { w } = await openLevel('09-move-the-gem')
      await w.get('button[aria-label="單步前進"]').trigger('click')
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="mem-caption"]').text()).toBe('第 2 步 · 第 3 行')
    })

    it('Submit 失敗：留在關卡頁、計一次失敗、不出現結算', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('test_failed', [{ name: 'a', status: 'failed', message: 'x' }]) })
      const { w } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      expect(w.find('[role="dialog"]').exists()).toBe(false)
      expect(JSON.parse(localStorage.getItem(PROGRESS_KEY)!).levels[ID].current.failedSubmits).toBe(1)
    })

    it('Submit 通過：先播通關動畫約 1.5 秒，再顯示通關結算並寫入進度', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      expect(w.find('[data-testid="clear-animation"]').exists()).toBe(true)
      expect(w.find('[role="dialog"]').exists()).toBe(false)
      vi.advanceTimersByTime(1500)
      await flushPromises()
      expect(w.find('[role="dialog"]').exists()).toBe(true)
      expect(JSON.parse(localStorage.getItem(PROGRESS_KEY)!).levels[ID].stars).toEqual({ pass: true, noHint: true, firstTry: true })
    })

    it('通關動畫可按 Esc 跳過', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      await flushPromises()
      expect(w.find('[role="dialog"]').exists()).toBe(true)
    })

    it('通關動畫可點任意處跳過', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      await w.get('[data-testid="clear-animation"]').trigger('click')
      expect(w.find('[role="dialog"]').exists()).toBe(true)
    })

    it('減少動態效果時直接顯示通關結算', async () => {
      reducedMotion = true
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      expect(w.find('[data-testid="clear-animation"]').exists()).toBe(false)
      expect(w.find('[role="dialog"]').exists()).toBe(true)
    })

    it('結算「下一關」：下一關已解鎖時進入下一關', async () => {
      reducedMotion = true
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w, router } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      await w.get('[role="dialog"] button.is-primary').trigger('click')
      await flushPromises()
      expect(router.currentRoute.value.path).toBe('/level/02-mut-matters')
    })

    it('結算「下一關」：沒有下一關時回地圖', async () => {
      reducedMotion = true
      details(gridLevel({ id: '19-lighthouse', chapter: 5, order: 19, title: '燈塔', kind: 'debug', visual: 'none', grid: null }))
      seed(Object.fromEntries(levels.filter((l) => l.id !== '19-lighthouse').map((l) => [l.id, { pass: true }])))
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w, router } = await openLevel('19-lighthouse')
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      await w.get('[role="dialog"] button.is-primary').trigger('click')
      await flushPromises()
      expect(router.currentRoute.value.path).toBe('/')
    })

    it('結算「回地圖」', async () => {
      reducedMotion = true
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w, router } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      const back = w.findAll('[role="dialog"] button').find((b) => b.text() === '回地圖')
      await back!.trigger('click')
      await flushPromises()
      expect(router.currentRoute.value.path).toBe('/')
    })

    it('結算「再玩一次」：關閉結算並留在本關，提示區收起', async () => {
      reducedMotion = true
      seed({ [ID]: { hintsRevealed: 1 } })
      api.submit.mockResolvedValue({ ok: true, data: submitResult('ok') })
      const { w, router } = await openLevel(ID)
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      const replay = w.findAll('[role="dialog"] button').find((b) => b.text() === '再玩一次')
      await replay!.trigger('click')
      await flushPromises()
      expect(w.find('[role="dialog"]').exists()).toBe(false)
      expect(router.currentRoute.value.path).toBe(`/level/${ID}`)
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(0)
    })
  })

  describe('請求失敗與健康檢查', () => {
    it('server_busy：兩顆按鈕一起 disabled 並各自倒數，重設不受影響，倒數後不自動重送', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'server_busy', retryAfterSeconds: 2 } })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(btn(w, 'run').text()).toBe('執行（2 秒後可再試）')
      expect(btn(w, 'submit').text()).toBe('送出（2 秒後可再試）')
      expect(isDisabled(w, 'run')).toBe(true)
      expect(isDisabled(w, 'submit')).toBe(true)
      expect(isDisabled(w, 'reset')).toBe(false)
      vi.advanceTimersByTime(1000)
      await flushPromises()
      expect(btn(w, 'run').text()).toBe('執行（1 秒後可再試）')
      vi.advanceTimersByTime(1000)
      await flushPromises()
      expect(isDisabled(w, 'run')).toBe(false)
      expect(api.run).toHaveBeenCalledTimes(1)
    })

    it('judge_unavailable：不倒數、不 disabled', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'judge_unavailable' } })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(isDisabled(w, 'run')).toBe(false)
      expect(w.get('[data-testid="console"]').text()).toContain('判題服務還沒準備好，關卡內容仍可瀏覽')
    })

    it('network_error：不倒數、不 disabled', async () => {
      api.run.mockResolvedValue({ ok: false, failure: { kind: 'network_error', status: null, code: null, message: null } })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(isDisabled(w, 'run')).toBe(false)
      expect(btn(w, 'run').text()).toBe('執行')
      expect(w.get('[data-testid="console"]').text()).toContain('連不上伺服器，檢查網路後再試')
    })

    it('health degraded：Run／Submit 直接 disabled 並顯示說明', async () => {
      api.health.mockResolvedValue({ ok: true, data: { status: 'degraded', judge: { available: false, reason: 'docker_unreachable' } } })
      const { w } = await openLevel(ID)
      expect(isDisabled(w, 'run')).toBe(true)
      expect(isDisabled(w, 'submit')).toBe(true)
      expect(w.get('[data-testid="degraded-note"]').text()).toBe('判題服務還沒準備好，關卡內容仍可瀏覽')
    })
  })

  describe('元件接線', () => {
    const highlight = (w: VueWrapper) => w.get('[data-testid="editor"]').attributes('data-highlight-line')

    it('進入 memory 關卡即高亮第 1 步對應行', async () => {
      const { w } = await openLevel('09-move-the-gem')
      expect(highlight(w)).toBe('8')
    })

    it('MemoryView 換步時編輯器高亮跟著換', async () => {
      const { w } = await openLevel('09-move-the-gem')
      await w.get('button[aria-label="單步前進"]').trigger('click')
      expect(highlight(w)).toBe('3')
    })

    it('MemoryView 播完保留最後一步的高亮', async () => {
      const { w } = await openLevel('09-move-the-gem')
      await w.get('button[aria-label="播放"]').trigger('click')
      await vi.advanceTimersByTimeAsync(10_000)
      expect(highlight(w)).toBe('10')
    })

    it('點編輯器行號左側 ▶ 跳到對應步驟', async () => {
      const { w } = await openLevel('09-move-the-gem')
      await w.get('[data-testid="editor-gutter-10"]').trigger('click')
      expect(w.get('[data-testid="mem-caption"]').text()).toBe('第 3 步 · 第 10 行')
    })

    it('程式碼與 starter 不同時 MemoryView 提示行號僅供參考，重設後消失', async () => {
      const { w } = await openLevel('09-move-the-gem')
      expect(w.text()).not.toContain('程式碼已修改，行號高亮僅供參考')
      await w.get('[data-testid="editor"]').setValue('fn main() { changed }')
      expect(w.text()).toContain('程式碼已修改，行號高亮僅供參考')
      await btn(w, 'reset').trigger('click')
      await w.get('[data-testid="confirm-ok"]').trigger('click')
      expect(w.text()).not.toContain('程式碼已修改，行號高亮僅供參考')
    })

    it('改回與 starter 相同時提示消失', async () => {
      const { w } = await openLevel('09-move-the-gem')
      await w.get('[data-testid="editor"]').setValue('x')
      await w.get('[data-testid="editor"]').setValue('fn main() {}\n')
      expect(w.text()).not.toContain('程式碼已修改，行號高亮僅供參考')
    })

    it('點錯誤卡片呼叫編輯器 revealPosition(行, 欄)', async () => {
      editorSpy.revealPosition.mockReset()
      api.run.mockResolvedValue({ ok: true, data: runCompileError() })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await w.get('[data-testid="diag-card"]').trigger('click')
      expect(editorSpy.revealPosition).toHaveBeenCalledWith(10, 17)
    })

    it('grid 關卡主控台標示無效指令行', async () => {
      api.run.mockResolvedValue({ ok: true, data: runOk('MOVE DOWN\nMOVE FORWARD\n') })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.findAll('[data-testid="stdout-line"].is-invalid')).toHaveLength(1)
    })

    it('同一份程式碼連續 Run 兩次，第二次也從第 1 步重播', async () => {
      api.run.mockResolvedValue({ ok: true, data: runOk(clearRoute) })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await vi.advanceTimersByTimeAsync(60_000)
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 20 / 20 步')
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 1 / 20 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
    })
  })

  describe('頂部列：深淺色與快捷鍵說明', () => {
    it('☀/☾ 按鈕依 系統 → 淺色 → 深色 循環，並設定 <html data-theme> 與偏好', async () => {
      const { w } = await openLevel(ID)
      const toggle = () => w.get('[data-testid="theme-toggle"]').trigger('click')
      await toggle()
      expect(document.documentElement.getAttribute('data-theme')).toBe('light')
      expect(JSON.parse(localStorage.getItem('rustquest.prefs.v1')!)).toEqual({ theme: 'light' })
      await toggle()
      expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
      await toggle()
      expect(JSON.parse(localStorage.getItem('rustquest.prefs.v1')!)).toEqual({ theme: 'system' })
      // matchMedia mock 不偏好深色
      expect(document.documentElement.getAttribute('data-theme')).toBe('light')
    })

    it('⌨ 快捷鍵按鈕開啟說明面板，列出全部按鍵與 Ctrl+M', async () => {
      const { w } = await openLevel(ID)
      const button = w.get('[data-testid="shortcut-help-button"]')
      expect(button.text()).toContain('⌨')
      await button.trigger('click')
      const panel = w.get('[data-testid="shortcut-help"]')
      expect(panel.attributes('role')).toBe('dialog')
      for (const key of ['Ctrl+Enter', 'Ctrl+Shift+Enter', 'Space', '→', 'Home', 'Esc', 'Ctrl+M']) {
        expect(panel.text()).toContain(key)
      }
    })

    it('快捷鍵說明面板按 Esc 關閉，焦點回到 ⌨ 按鈕', async () => {
      const { w } = await openLevel(ID)
      const button = w.get('[data-testid="shortcut-help-button"]')
      ;(button.element as HTMLButtonElement).focus()
      await button.trigger('click')
      await w.get('[data-testid="shortcut-help"]').trigger('keydown', { key: 'Escape' })
      expect(w.find('[data-testid="shortcut-help"]').exists()).toBe(false)
      expect(document.activeElement).toBe(button.element)
    })

    it('確認框關閉後焦點回到觸發按鈕', async () => {
      const { w } = await openLevel(ID)
      const hint = w.get('[data-testid="hint-button"]')
      ;(hint.element as HTMLButtonElement).focus()
      await hint.trigger('click')
      await w.get('[data-testid="confirm-cancel"]').trigger('click')
      await flushPromises()
      expect(document.activeElement).toBe(w.get('[data-testid="hint-button"]').element)
    })

    it('確認框按 Esc 等同取消', async () => {
      const { w } = await openLevel(ID)
      await w.get('[data-testid="hint-button"]').trigger('click')
      await w.get('[data-testid="confirm-dialog"]').trigger('keydown', { key: 'Escape' })
      expect(w.find('[data-testid="confirm-dialog"]').exists()).toBe(false)
      expect(w.findAll('[data-testid="hint-item"]')).toHaveLength(0)
    })
  })

  describe('全域快捷鍵', () => {
    function press(init: KeyboardEventInit) {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, ...init }))
    }

    it('焦點在編輯器外 Ctrl+Enter 觸發 Run', async () => {
      api.run.mockResolvedValue({ ok: true, data: runOk('') })
      await openLevel(ID)
      press({ ctrlKey: true })
      await flushPromises()
      expect(api.run).toHaveBeenCalledTimes(1)
    })

    it('焦點在編輯器外 Ctrl+Shift+Enter 觸發 Submit', async () => {
      api.submit.mockResolvedValue({ ok: true, data: submitResult('test_failed') })
      await openLevel(ID)
      press({ ctrlKey: true, shiftKey: true })
      await flushPromises()
      expect(api.submit).toHaveBeenCalledTimes(1)
    })

    it('請求中再按快捷鍵不重送', async () => {
      api.run.mockReturnValue(new Promise(() => {}))
      await openLevel(ID)
      press({ ctrlKey: true })
      press({ ctrlKey: true })
      press({ ctrlKey: true, shiftKey: true })
      await flushPromises()
      expect(api.run).toHaveBeenCalledTimes(1)
      expect(api.submit).not.toHaveBeenCalled()
    })
  })
  describe('S2-3 timeline 關卡', () => {
    const highlight = (w: VueWrapper) => w.get('[data-testid="editor"]').attributes('data-highlight-line')
    const deadlockRun = (): RunResponse => ({
      ...runOk(tl33Starter),
      status: 'timeout',
      execution: { ...runOk(tl33Starter).execution!, exit_code: null, timed_out: true },
    })
    const playToEnd = async () => {
      await vi.advanceTimersByTimeAsync(600_000)
      await flushPromises()
    }

    beforeEach(() => {
      details(timelineLevel())
    })

    it('visual = timeline 顯示 TimelineView，不顯示 GridWorld、MemoryView', async () => {
      const { w } = await openLevel(ID)
      expect(w.find('[data-testid="timeline-view"]').exists()).toBe(true)
      expect(w.find('[data-testid="grid-world"]').exists()).toBe(false)
      expect(w.find('[data-testid="mem-caption"]').exists()).toBe(false)
    })

    it('Run 逾時（33 starter）：時間軸自動播放，主控台隱藏 @tl| 行並顯示隱藏行數', async () => {
      api.run.mockResolvedValue({ ok: true, data: deadlockRun() })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      const timeline = w.find('[data-testid="timeline-view"]')
      expect(timeline.exists()).toBe(true)
      expect(timeline.find('[data-testid="step-label"]').exists() && timeline.get('[data-testid="step-label"]').text()).toBe('第 1 / 14 步')
      const consoleText = w.get('[data-testid="console"]').text()
      expect(consoleText).not.toContain('@tl|')
      expect(consoleText).toContain('已隱藏 14 行時間軸事件')
      expect(consoleText).toContain('分身 0 站上窄橋 0')
    })

    it('時間軸 line-change 接到編輯器高亮：死鎖最後一步高亮循環第一條泳道的等待行', async () => {
      api.run.mockResolvedValue({ ok: true, data: deadlockRun() })
      const { w } = await openLevel(ID)
      expect(highlight(w)).toBe('')
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(highlight(w)).toBe('58')
      await playToEnd()
      expect(highlight(w)).toBe('39')
    })

    it('Run 之後修改程式碼：可視化區提示「程式碼已修改，行號高亮僅供參考」', async () => {
      api.run.mockResolvedValue({ ok: true, data: deadlockRun() })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="visual-panel"]').text()).not.toContain('程式碼已修改，行號高亮僅供參考')
      await w.get('[data-testid="editor"]').setValue('fn main() { changed }')
      expect(w.find('[data-testid="timeline-view"]').exists()).toBe(true)
      expect(w.get('[data-testid="visual-panel"]').text()).toContain('程式碼已修改，行號高亮僅供參考')
    })

    it('Submit 不驅動時間軸：還原上次 Run 的最後一步，不重播', async () => {
      api.run.mockResolvedValue({ ok: true, data: deadlockRun() })
      api.submit.mockResolvedValue({ ok: true, data: submitResult('timeout', [{ name: 'great_migration_reaches_the_tablet', status: 'not_run', message: null }]) })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await playToEnd()
      await btn(w, 'submit').trigger('click')
      await flushPromises()
      const timeline = w.find('[data-testid="timeline-view"]')
      expect(timeline.exists()).toBe(true)
      expect(timeline.find('[data-testid="step-label"]').exists() && timeline.get('[data-testid="step-label"]').text()).toBe('第 14 / 14 步')
      expect(timeline.find('button[aria-label="暫停"]').exists()).toBe(false)
    })

    it('同一份程式碼連續 Run 兩次結果相同，第二次也從第 1 步重播', async () => {
      api.run.mockResolvedValue({ ok: true, data: deadlockRun() })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await playToEnd()
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 14 / 14 步')
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 1 / 14 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
    })

    it('Run 請求失敗：還原上次 Run 的最後一步，不重播', async () => {
      api.run.mockResolvedValueOnce({ ok: true, data: deadlockRun() })
      api.run.mockResolvedValueOnce({ ok: false, failure: { kind: 'network_error', status: null, code: null, message: null } })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      await playToEnd()
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="step-label"]').text()).toBe('第 14 / 14 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(false)
    })

    it('grid 關卡主控台不隱藏 @tl| 行', async () => {
      details(gridLevel())
      api.run.mockResolvedValue({ ok: true, data: runOk('@tl|start|1|甲|\nMOVE DOWN\n') })
      const { w } = await openLevel(ID)
      await btn(w, 'run').trigger('click')
      await flushPromises()
      expect(w.get('[data-testid="console"]').text()).toContain('@tl|start|1|甲|')
    })
  })
})
