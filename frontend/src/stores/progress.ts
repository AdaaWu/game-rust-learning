import { defineStore } from 'pinia'
import type { SubmitStatus } from '../api/types'
import {
  computeAttemptStars,
  countsAsFailedSubmit,
  emptyLevelProgress,
  emptyProgress,
  mergeStars,
  parsePrefs,
  parseProgress,
  PREFS_KEY,
  PROGRESS_KEY,
} from '../domain/progress'
import type { LevelProgress, Prefs, StarSet, Theme } from '../domain/progress'
import { applyTheme } from '../domain/theme'
import { useToastStore } from './toast'

const READ_FAILED = '進度無法讀取，已從頭開始'
const WRITE_FAILED = '進度無法儲存'

export const useProgressStore = defineStore('progress', {
  state: () => ({
    progress: emptyProgress(),
    prefs: { theme: 'system' } as Prefs,
  }),
  actions: {
    // 讀 localStorage；失敗以空進度啟動並 toast
    load() {
      let raw: string | null = null
      let readFailed = false
      try {
        raw = localStorage.getItem(PROGRESS_KEY)
      } catch {
        readFailed = true
      }
      const parsed = parseProgress(raw)
      this.progress = parsed ?? emptyProgress()
      if (readFailed || (raw !== null && parsed === null)) useToastStore().show(READ_FAILED)

      let rawPrefs: string | null = null
      try {
        rawPrefs = localStorage.getItem(PREFS_KEY)
      } catch {
        rawPrefs = null
      }
      this.prefs = parsePrefs(rawPrefs)
      applyTheme(this.prefs.theme)
    },
    levelProgress(levelId: string): LevelProgress {
      return this.progress.levels[levelId] ?? emptyLevelProgress()
    },
    ensureLevel(levelId: string): LevelProgress {
      if (!this.progress.levels[levelId]) this.progress.levels[levelId] = emptyLevelProgress()
      return this.progress.levels[levelId]
    },
    save() {
      try {
        localStorage.setItem(PROGRESS_KEY, JSON.stringify(this.progress))
      } catch {
        useToastStore().show(WRITE_FAILED)
      }
    },
    revealHint(levelId: string) {
      this.ensureLevel(levelId).current.hintsRevealed += 1
      this.save()
    },
    // 判題結果的 Submit；失敗狀態累加 failedSubmits
    recordSubmit(levelId: string, status: SubmitStatus) {
      if (!countsAsFailedSubmit(status)) return
      this.ensureLevel(levelId).current.failedSubmits += 1
      this.save()
    },
    // 通關結算：回傳本次拿到的星，與歷史聯集後歸零 current
    recordPass(levelId: string): StarSet {
      const level = this.ensureLevel(levelId)
      const earned = computeAttemptStars(level.current)
      level.stars = mergeStars(level.stars, earned)
      level.current = { hintsRevealed: 0, failedSubmits: 0 }
      this.save()
      return earned
    },
    setTheme(theme: Theme) {
      this.prefs = { theme }
      applyTheme(theme)
      try {
        localStorage.setItem(PREFS_KEY, JSON.stringify(this.prefs))
      } catch {
        // 偏好寫入失敗不影響遊戲
      }
    },
  },
})
