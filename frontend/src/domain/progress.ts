import type { JudgeStatus, LevelSummary } from '../api/types'

// ux.md §6.5
export const PROGRESS_KEY = 'rustquest.progress.v1'
export const PREFS_KEY = 'rustquest.prefs.v1'
export const DRAFT_KEY_PREFIX = 'rustquest.draft.'

export interface StarSet {
  pass: boolean
  noHint: boolean
  firstTry: boolean
}

export interface AttemptCounters {
  hintsRevealed: number
  failedSubmits: number
}

export interface LevelProgress {
  stars: StarSet
  current: AttemptCounters
}

export interface Progress {
  version: 1
  levels: Record<string, LevelProgress>
}

export type Theme = 'system' | 'light' | 'dark'

export interface Prefs {
  theme: Theme
}

export type NodeState = 'locked' | 'available' | 'passed'

export function emptyProgress(): Progress {
  return { version: 1, levels: {} }
}

export function emptyLevelProgress(): LevelProgress {
  return {
    stars: { pass: false, noHint: false, firstTry: false },
    current: { hintsRevealed: 0, failedSubmits: 0 },
  }
}

// 本次遊玩通過時拿到的星
export function computeAttemptStars(current: AttemptCounters): StarSet {
  return { pass: true, noHint: current.hintsRevealed === 0, firstTry: current.failedSubmits === 0 }
}

// 歷史聯集
export function mergeStars(a: StarSet, b: StarSet): StarSet {
  return { pass: a.pass || b.pass, noHint: a.noHint || b.noHint, firstTry: a.firstTry || b.firstTry }
}

export function starCount(stars: StarSet): number {
  return Number(stars.pass) + Number(stars.noHint) + Number(stars.firstTry)
}

const FAILED_SUBMIT_STATUSES: JudgeStatus[] = ['compile_error', 'test_failed', 'timeout', 'memory_limit', 'output_limit']

// 計入失敗嘗試的 Submit 狀態
export function countsAsFailedSubmit(status: JudgeStatus): boolean {
  return FAILED_SUBMIT_STATUSES.includes(status)
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

// 解析 localStorage 內容；壞掉或版本不符回 null
export function parseProgress(raw: string | null): Progress | null {
  if (raw === null) return null
  let value: unknown
  try {
    value = JSON.parse(raw)
  } catch {
    return null
  }
  if (!isRecord(value) || value.version !== 1 || !isRecord(value.levels)) return null
  return value as unknown as Progress
}

const THEMES: Theme[] = ['system', 'light', 'dark']

export function parsePrefs(raw: string | null): Prefs {
  if (raw === null) return { theme: 'system' }
  try {
    const value: unknown = JSON.parse(raw)
    if (isRecord(value) && THEMES.includes(value.theme as Theme)) return { theme: value.theme as Theme }
  } catch {
    // 壞掉時用預設值
  }
  return { theme: 'system' }
}

// 系統 → 淺色 → 深色 循環
export function nextTheme(theme: Theme): Theme {
  return THEMES[(THEMES.indexOf(theme) + 1) % THEMES.length]
}

function sortLevels(levels: LevelSummary[]): LevelSummary[] {
  return [...levels].sort((a, b) => a.chapter - b.chapter || a.order - b.order)
}

function isPassed(levelId: string, progress: Progress): boolean {
  return progress.levels[levelId]?.stars.pass === true
}

// 鎖定時擋住它的關卡；解鎖或不存在回 null
function blocker(levelId: string, levels: LevelSummary[], progress: Progress): LevelSummary | null {
  const sorted = sortLevels(levels)
  const index = sorted.findIndex((l) => l.id === levelId)
  if (index <= 0) return null
  const level = sorted[index]
  const previous = sorted[index - 1]
  if (previous.chapter === level.chapter) return isPassed(previous.id, progress) ? null : previous
  return sorted.find((l) => l.chapter === previous.chapter && !isPassed(l.id, progress)) ?? null
}

export function isUnlocked(levelId: string, levels: LevelSummary[], progress: Progress): boolean {
  if (!levels.some((l) => l.id === levelId)) return false
  return blocker(levelId, levels, progress) === null
}

export function nodeState(levelId: string, levels: LevelSummary[], progress: Progress): NodeState {
  if (isPassed(levelId, progress)) return 'passed'
  return isUnlocked(levelId, levels, progress) ? 'available' : 'locked'
}

// 鎖定節點 tooltip 要提到的關卡
export function blockingLevel(levelId: string, levels: LevelSummary[], progress: Progress): LevelSummary | null {
  return blocker(levelId, levels, progress)
}

// 地圖「目前所在」
export function currentLevelId(levels: LevelSummary[], progress: Progress): string | null {
  const sorted = sortLevels(levels)
  const available = sorted.find((l) => nodeState(l.id, levels, progress) === 'available')
  if (available) return available.id
  return sorted.length > 0 ? sorted[sorted.length - 1].id : null
}

// 通關後「下一關」：下一關已解鎖回其 id，否則回 null（回地圖）
export function nextLevelId(levelId: string, levels: LevelSummary[], progress: Progress): string | null {
  const sorted = sortLevels(levels)
  const index = sorted.findIndex((l) => l.id === levelId)
  if (index < 0 || index + 1 >= sorted.length) return null
  const next = sorted[index + 1]
  return isUnlocked(next.id, levels, progress) ? next.id : null
}

export function chapterStars(chapter: number, levels: LevelSummary[], progress: Progress): { earned: number; max: number } {
  const inChapter = levels.filter((l) => l.chapter === chapter)
  const earned = inChapter.reduce((sum, l) => {
    const stars = progress.levels[l.id]?.stars
    return sum + (stars ? starCount(stars) : 0)
  }, 0)
  return { earned, max: inChapter.length * 3 }
}
