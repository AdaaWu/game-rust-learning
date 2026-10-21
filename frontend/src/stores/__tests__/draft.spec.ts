import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { DRAFT_DEBOUNCE_MS, needsResetConfirm, useDraftStore } from '../draft'
import { useToastStore } from '../toast'

const ID = '01-hello-ferris'
const KEY = 'rustquest.draft.01-hello-ferris'
const STARTER = 'fn main() {}\n'

describe('F2 草稿自動儲存', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  it('debounce 為 500 ms', () => {
    expect(DRAFT_DEBOUNCE_MS).toBe(500)
  })

  it('沒有草稿時載入 starter', () => {
    expect(useDraftStore().load(ID, STARTER)).toBe(STARTER)
  })

  it('有草稿時載入草稿', () => {
    localStorage.setItem(KEY, 'fn main() { draft }')
    expect(useDraftStore().load(ID, STARTER)).toBe('fn main() { draft }')
  })

  it('空字串草稿也算草稿', () => {
    localStorage.setItem(KEY, '')
    expect(useDraftStore().load(ID, STARTER)).toBe('')
  })

  it('停止輸入 500 ms 後才寫入', () => {
    const store = useDraftStore()
    store.scheduleSave(ID, 'a')
    vi.advanceTimersByTime(499)
    expect(localStorage.getItem(KEY)).toBeNull()
    vi.advanceTimersByTime(1)
    expect(localStorage.getItem(KEY)).toBe('a')
  })

  it('連續輸入只寫最後一次', () => {
    const store = useDraftStore()
    const spy = vi.spyOn(Storage.prototype, 'setItem')
    store.scheduleSave(ID, 'a')
    vi.advanceTimersByTime(300)
    store.scheduleSave(ID, 'ab')
    vi.advanceTimersByTime(300)
    store.scheduleSave(ID, 'abc')
    vi.advanceTimersByTime(500)
    expect(spy).toHaveBeenCalledTimes(1)
    expect(localStorage.getItem(KEY)).toBe('abc')
  })

  it('每關一個 key，不併入進度', () => {
    const store = useDraftStore()
    store.scheduleSave('02-mut-matters', 'x')
    vi.advanceTimersByTime(500)
    expect(localStorage.getItem('rustquest.draft.02-mut-matters')).toBe('x')
    expect(localStorage.getItem('rustquest.progress.v1')).toBeNull()
  })

  it('clear 刪除該關草稿', () => {
    localStorage.setItem(KEY, 'x')
    useDraftStore().clear(ID)
    expect(localStorage.getItem(KEY)).toBeNull()
  })

  it('clear 取消尚未寫入的 debounce', () => {
    const store = useDraftStore()
    store.scheduleSave(ID, 'pending')
    store.clear(ID)
    vi.advanceTimersByTime(500)
    expect(localStorage.getItem(KEY)).toBeNull()
  })

  it('寫入失敗不提示、不丟例外', () => {
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('QuotaExceededError')
    })
    const store = useDraftStore()
    store.scheduleSave(ID, 'x')
    expect(() => vi.advanceTimersByTime(500)).not.toThrow()
    expect(useToastStore().messages).toEqual([])
  })

  it('讀取丟例外時退回 starter', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('SecurityError')
    })
    expect(useDraftStore().load(ID, STARTER)).toBe(STARTER)
  })

  it('重設確認：程式碼與 starter 不同才需要確認', () => {
    expect(needsResetConfirm('changed', STARTER)).toBe(true)
    expect(needsResetConfirm(STARTER, STARTER)).toBe(false)
  })
})
