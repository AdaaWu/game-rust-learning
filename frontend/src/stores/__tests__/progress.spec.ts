import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProgressStore } from '../progress'
import { useToastStore } from '../toast'
import { PREFS_KEY, PROGRESS_KEY } from '../../domain/progress'

const ID = '01-hello-ferris'

function stored() {
  return JSON.parse(localStorage.getItem(PROGRESS_KEY) ?? 'null')
}

describe('F2 useProgressStore', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('沒有 current 時視為 0 / 0', () => {
    const store = useProgressStore()
    store.load()
    expect(store.levelProgress(ID).current).toEqual({ hintsRevealed: 0, failedSubmits: 0 })
  })

  it('從 localStorage 載入既有進度', () => {
    localStorage.setItem(
      PROGRESS_KEY,
      JSON.stringify({ version: 1, levels: { [ID]: { stars: { pass: true, noHint: false, firstTry: true }, current: { hintsRevealed: 2, failedSubmits: 0 } } } }),
    )
    const store = useProgressStore()
    store.load()
    expect(store.levelProgress(ID).stars).toEqual({ pass: true, noHint: false, firstTry: true })
    expect(store.levelProgress(ID).current.hintsRevealed).toBe(2)
  })

  it('揭露提示累加並立即寫入 localStorage', () => {
    const store = useProgressStore()
    store.load()
    store.revealHint(ID)
    store.revealHint(ID)
    expect(store.levelProgress(ID).current.hintsRevealed).toBe(2)
    expect(stored().levels[ID].current.hintsRevealed).toBe(2)
  })

  it.each(['compile_error', 'test_failed', 'timeout', 'memory_limit', 'output_limit'] as const)(
    'Submit %s 累加 failedSubmits 並寫入',
    (status) => {
      const store = useProgressStore()
      store.load()
      store.recordSubmit(ID, status)
      expect(store.levelProgress(ID).current.failedSubmits).toBe(1)
      expect(stored().levels[ID].current.failedSubmits).toBe(1)
    },
  )

  it('Submit ok 不累加失敗次數', () => {
    const store = useProgressStore()
    store.load()
    store.recordSubmit(ID, 'ok')
    expect(store.levelProgress(ID).current.failedSubmits).toBe(0)
  })

  it('通關結算：回傳本次星等、與歷史聯集、current 歸零', () => {
    const store = useProgressStore()
    store.load()
    store.revealHint(ID)
    const earned = store.recordPass(ID)
    expect(earned).toEqual({ pass: true, noHint: false, firstTry: true })
    expect(store.levelProgress(ID).current).toEqual({ hintsRevealed: 0, failedSubmits: 0 })
    expect(stored().levels[ID]).toEqual({
      stars: { pass: true, noHint: false, firstTry: true },
      current: { hintsRevealed: 0, failedSubmits: 0 },
    })
  })

  it('重玩取得的星與歷史取聯集', () => {
    const store = useProgressStore()
    store.load()
    store.revealHint(ID)
    store.recordPass(ID) // ★1★3
    store.recordSubmit(ID, 'test_failed')
    const second = store.recordPass(ID) // ★1★2
    expect(second).toEqual({ pass: true, noHint: true, firstTry: false })
    expect(store.levelProgress(ID).stars).toEqual({ pass: true, noHint: true, firstTry: true })
  })

  it('重新載入（模擬重新整理）不歸零本次遊玩計數', () => {
    const store = useProgressStore()
    store.load()
    store.revealHint(ID)
    store.recordSubmit(ID, 'test_failed')
    setActivePinia(createPinia())
    const reloaded = useProgressStore()
    reloaded.load()
    expect(reloaded.levelProgress(ID).current).toEqual({ hintsRevealed: 1, failedSubmits: 1 })
  })

  it('JSON 損壞時以空進度啟動並 toast「進度無法讀取，已從頭開始」', () => {
    localStorage.setItem(PROGRESS_KEY, '{broken')
    const store = useProgressStore()
    store.load()
    expect(store.progress).toEqual({ version: 1, levels: {} })
    expect(useToastStore().messages).toContain('進度無法讀取，已從頭開始')
  })

  it('localStorage 讀取丟例外（隱私模式）時以空進度啟動並 toast', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('SecurityError')
    })
    const store = useProgressStore()
    expect(() => store.load()).not.toThrow()
    expect(store.progress.levels).toEqual({})
    expect(useToastStore().messages).toContain('進度無法讀取，已從頭開始')
  })

  it('第一次進站沒有任何資料時不 toast', () => {
    const store = useProgressStore()
    store.load()
    expect(useToastStore().messages).toEqual([])
  })

  it('version 不符時以空進度啟動', () => {
    localStorage.setItem(PROGRESS_KEY, JSON.stringify({ version: 2, levels: { [ID]: {} } }))
    const store = useProgressStore()
    store.load()
    expect(store.progress).toEqual({ version: 1, levels: {} })
  })

  it('寫入失敗時 toast「進度無法儲存」，遊戲照常（記憶體內仍更新）', () => {
    const store = useProgressStore()
    store.load()
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('QuotaExceededError')
    })
    expect(() => store.recordPass(ID)).not.toThrow()
    expect(store.levelProgress(ID).stars.pass).toBe(true)
    expect(useToastStore().messages).toContain('進度無法儲存')
  })

  it('深淺色偏好寫入 rustquest.prefs.v1 且只有 theme', () => {
    const store = useProgressStore()
    store.load()
    store.setTheme('dark')
    expect(store.prefs.theme).toBe('dark')
    expect(JSON.parse(localStorage.getItem(PREFS_KEY) ?? 'null')).toEqual({ theme: 'dark' })
  })

  it('load 時把深淺色套用到 <html data-theme>', () => {
    localStorage.setItem(PREFS_KEY, JSON.stringify({ theme: 'dark' }))
    useProgressStore().load()
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  it('setTheme 立即套用到 <html data-theme>', () => {
    const store = useProgressStore()
    store.load()
    store.setTheme('light')
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('載入時讀取深淺色偏好，不合法時用 system', () => {
    localStorage.setItem(PREFS_KEY, JSON.stringify({ theme: 'light' }))
    const store = useProgressStore()
    store.load()
    expect(store.prefs.theme).toBe('light')
  })
})
