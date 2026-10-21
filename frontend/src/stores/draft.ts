import { defineStore } from 'pinia'
import { DRAFT_KEY_PREFIX } from '../domain/progress'

export const DRAFT_DEBOUNCE_MS = 500

// 重設是否需要確認：程式碼已等於 starter 時不需要
export function needsResetConfirm(code: string, starter: string): boolean {
  return code !== starter
}

const timers = new Map<string, ReturnType<typeof setTimeout>>()

function cancel(levelId: string) {
  const timer = timers.get(levelId)
  if (timer !== undefined) clearTimeout(timer)
  timers.delete(levelId)
}

export const useDraftStore = defineStore('draft', {
  actions: {
    // 有草稿回草稿，沒有回 starter
    load(levelId: string, starter: string): string {
      try {
        return localStorage.getItem(DRAFT_KEY_PREFIX + levelId) ?? starter
      } catch {
        return starter
      }
    },
    // debounce 後寫入
    scheduleSave(levelId: string, code: string) {
      cancel(levelId)
      timers.set(
        levelId,
        setTimeout(() => {
          timers.delete(levelId)
          try {
            localStorage.setItem(DRAFT_KEY_PREFIX + levelId, code)
          } catch {
            // 草稿寫入失敗不提示
          }
        }, DRAFT_DEBOUNCE_MS),
      )
    },
    clear(levelId: string) {
      cancel(levelId)
      try {
        localStorage.removeItem(DRAFT_KEY_PREFIX + levelId)
      } catch {
        // 刪除失敗不提示
      }
    },
  },
})
