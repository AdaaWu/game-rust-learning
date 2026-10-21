import { describe, expect, it } from 'vitest'
import type { LevelSummary } from '../../api/types'
import {
  blockingLevel,
  chapterStars,
  computeAttemptStars,
  countsAsFailedSubmit,
  currentLevelId,
  emptyLevelProgress,
  emptyProgress,
  isUnlocked,
  mergeStars,
  nextLevelId,
  nextTheme,
  nodeState,
  parsePrefs,
  parseProgress,
  starCount,
} from '../progress'
import type { Progress, StarSet } from '../progress'
import { levels } from '../../test/fixtures'

const ALL: StarSet = { pass: true, noHint: true, firstTry: true }
const ONLY_PASS: StarSet = { pass: true, noHint: false, firstTry: false }

function progressWith(passed: Record<string, Partial<StarSet>>): Progress {
  const p = emptyProgress()
  for (const [id, stars] of Object.entries(passed)) {
    p.levels[id] = { ...emptyLevelProgress(), stars: { pass: true, noHint: false, firstTry: false, ...stars } }
  }
  return p
}

describe('F2 星等計算', () => {
  it('沒看提示也沒失敗：三顆星', () => {
    expect(computeAttemptStars({ hintsRevealed: 0, failedSubmits: 0 })).toEqual(ALL)
  })

  it('看過提示：沒有 noHint', () => {
    expect(computeAttemptStars({ hintsRevealed: 1, failedSubmits: 0 })).toEqual({ pass: true, noHint: false, firstTry: true })
  })

  it('失敗過一次：沒有 firstTry', () => {
    expect(computeAttemptStars({ hintsRevealed: 0, failedSubmits: 1 })).toEqual({ pass: true, noHint: true, firstTry: false })
  })

  it('看提示又失敗：只有 pass', () => {
    expect(computeAttemptStars({ hintsRevealed: 3, failedSubmits: 5 })).toEqual(ONLY_PASS)
  })

  it('歷史聯集：★1★3 加 ★1★2 等於三顆星', () => {
    const first: StarSet = { pass: true, noHint: false, firstTry: true }
    const second: StarSet = { pass: true, noHint: true, firstTry: false }
    expect(mergeStars(first, second)).toEqual(ALL)
  })

  it('歷史聯集不會被較低成績覆蓋', () => {
    expect(mergeStars(ALL, ONLY_PASS)).toEqual(ALL)
  })

  it('從未通關與本次合併', () => {
    expect(mergeStars(emptyLevelProgress().stars, ONLY_PASS)).toEqual(ONLY_PASS)
  })

  it('starCount 計算亮星數', () => {
    expect(starCount(ALL)).toBe(3)
    expect(starCount(ONLY_PASS)).toBe(1)
    expect(starCount(emptyLevelProgress().stars)).toBe(0)
  })

  it.each(['compile_error', 'test_failed', 'timeout', 'memory_limit', 'output_limit'] as const)(
    '%s 計入失敗的 Submit',
    (status) => {
      expect(countsAsFailedSubmit(status)).toBe(true)
    },
  )

  it.each(['ok', 'runtime_panic'] as const)('%s 不計入失敗', (status) => {
    expect(countsAsFailedSubmit(status)).toBe(false)
  })
})

describe('F2 localStorage 解析', () => {
  it('合法 JSON 原樣解析', () => {
    const p = progressWith({ '01-hello-ferris': { noHint: true } })
    expect(parseProgress(JSON.stringify(p))).toEqual(p)
  })

  it('沒有資料時回 null（由呼叫端用空進度）', () => {
    expect(parseProgress(null)).toBeNull()
  })

  it('JSON 壞掉回 null', () => {
    expect(parseProgress('{not json')).toBeNull()
  })

  it('version 不符回 null', () => {
    expect(parseProgress(JSON.stringify({ version: 2, levels: {} }))).toBeNull()
  })

  it('結構不對（levels 不是物件）回 null', () => {
    expect(parseProgress(JSON.stringify({ version: 1, levels: 3 }))).toBeNull()
  })

  it('Prefs 合法值照用', () => {
    expect(parsePrefs(JSON.stringify({ theme: 'light' }))).toEqual({ theme: 'light' })
  })

  it.each([null, '{bad', JSON.stringify({}), JSON.stringify({ theme: 'neon' }), JSON.stringify({ theme: 123 }), JSON.stringify({ theme: null }), 'null', '[]', JSON.stringify({ theme: 'Dark' })])('Prefs 缺欄位或不合法 %j 用 system', (raw) => {
    expect(parsePrefs(raw)).toEqual({ theme: 'system' })
  })

  it('Prefs 不保存風格或播放速度欄位', () => {
    expect(parsePrefs(JSON.stringify({ theme: 'dark', style: 'b', playbackSpeed: 2 }))).toEqual({ theme: 'dark' })
  })

  it('深淺色依 系統 → 淺色 → 深色 循環', () => {
    expect(nextTheme('system')).toBe('light')
    expect(nextTheme('light')).toBe('dark')
    expect(nextTheme('dark')).toBe('system')
  })
})

describe('F2 解鎖規則', () => {
  it('第一章第 1 關預設解鎖', () => {
    expect(isUnlocked('01-hello-ferris', levels, emptyProgress())).toBe(true)
  })

  it('前一關沒過時同章下一關鎖定', () => {
    expect(isUnlocked('02-mut-matters', levels, emptyProgress())).toBe(false)
  })

  it('前一關只有 ★1 也解鎖下一關', () => {
    expect(isUnlocked('02-mut-matters', levels, progressWith({ '01-hello-ferris': {} }))).toBe(true)
  })

  it('前一章還有關卡沒過時，下一章第 1 關鎖定', () => {
    const p = progressWith({ '01-hello-ferris': {} })
    expect(isUnlocked('05-delta', levels, p)).toBe(false)
  })

  it('只過 02、沒過 01 時下一章第 1 關仍鎖定', () => {
    const p = progressWith({ '02-mut-matters': {} })
    expect(isUnlocked('05-delta', levels, p)).toBe(false)
  })

  it('前一章全部至少 ★1 時，下一章第 1 關解鎖', () => {
    const p = progressWith({ '01-hello-ferris': {}, '02-mut-matters': {} })
    expect(isUnlocked('05-delta', levels, p)).toBe(true)
  })

  it('跨空章節：第 4 章沒有關卡，第 5 章看第 3 章', () => {
    const p = progressWith({
      '01-hello-ferris': {},
      '02-mut-matters': {},
      '05-delta': {},
      '09-move-the-gem': {},
    })
    expect(isUnlocked('19-lighthouse', levels, p)).toBe(false)
    p.levels['11-borrow'] = { ...emptyLevelProgress(), stars: ONLY_PASS }
    expect(isUnlocked('19-lighthouse', levels, p)).toBe(true)
  })

  it('order 跳號不影響「下一關」判定', () => {
    const p = progressWith({ '01-hello-ferris': {}, '02-mut-matters': {}, '05-delta': {}, '09-move-the-gem': {} })
    expect(isUnlocked('11-borrow', levels, p)).toBe(true)
  })

  it('關卡陣列順序打亂時仍依 (chapter, order) 判定', () => {
    const shuffled: LevelSummary[] = [...levels].reverse()
    expect(isUnlocked('01-hello-ferris', shuffled, emptyProgress())).toBe(true)
    expect(isUnlocked('02-mut-matters', shuffled, emptyProgress())).toBe(false)
  })

  it('不存在的關卡視為鎖定', () => {
    expect(isUnlocked('99-unknown', levels, emptyProgress())).toBe(false)
  })

  it('節點狀態：鎖定、可挑戰、已通過', () => {
    const p = progressWith({ '01-hello-ferris': {} })
    expect(nodeState('01-hello-ferris', levels, p)).toBe('passed')
    expect(nodeState('02-mut-matters', levels, p)).toBe('available')
    expect(nodeState('05-delta', levels, p)).toBe('locked')
  })

  it('鎖定的同章關卡，提示要先完成前一關', () => {
    expect(blockingLevel('02-mut-matters', levels, emptyProgress())?.id).toBe('01-hello-ferris')
  })

  it('鎖定的章節第 1 關，提示前一章第一個未通過的關卡', () => {
    const p = progressWith({ '01-hello-ferris': {} })
    expect(blockingLevel('05-delta', levels, p)?.id).toBe('02-mut-matters')
  })

  it('已解鎖的關卡沒有阻擋關卡', () => {
    expect(blockingLevel('01-hello-ferris', levels, emptyProgress())).toBeNull()
  })
})

describe('F2 目前所在、下一關、章節星數', () => {
  it('目前所在是可挑戰節點中排序最前的一個', () => {
    const p = progressWith({ '01-hello-ferris': {} })
    expect(currentLevelId(levels, p)).toBe('02-mut-matters')
  })

  it('全新進度時目前所在為第一關', () => {
    expect(currentLevelId(levels, emptyProgress())).toBe('01-hello-ferris')
  })

  it('全部通過時目前所在為最後一關', () => {
    const all = Object.fromEntries(levels.map((l) => [l.id, {}]))
    expect(currentLevelId(levels, progressWith(all))).toBe('19-lighthouse')
  })

  it('下一關已解鎖時回下一關 id', () => {
    const p = progressWith({ '01-hello-ferris': {} })
    expect(nextLevelId('01-hello-ferris', levels, p)).toBe('02-mut-matters')
  })

  it('章節最後一關但同章還有關卡沒過：下一關仍鎖定，回 null', () => {
    const p = progressWith({ '02-mut-matters': {} })
    expect(nextLevelId('02-mut-matters', levels, p)).toBeNull()
  })

  it('章節最後一關且整章通過：下一關為下一章第 1 關', () => {
    const p = progressWith({ '01-hello-ferris': {}, '02-mut-matters': {} })
    expect(nextLevelId('02-mut-matters', levels, p)).toBe('05-delta')
  })

  it('最後一關沒有下一關，回 null', () => {
    const all = Object.fromEntries(levels.map((l) => [l.id, {}]))
    expect(nextLevelId('19-lighthouse', levels, progressWith(all))).toBeNull()
  })

  it('章節星數分母為該章已載入關卡數 × 3', () => {
    const p = progressWith({ '01-hello-ferris': { noHint: true, firstTry: true }, '02-mut-matters': {} })
    expect(chapterStars(1, levels, p)).toEqual({ earned: 4, max: 6 })
  })

  it('沒進度的章節為 0 / n×3', () => {
    expect(chapterStars(3, levels, emptyProgress())).toEqual({ earned: 0, max: 6 })
  })

  it('只有一關的章節分母為 3', () => {
    expect(chapterStars(5, levels, emptyProgress())).toEqual({ earned: 0, max: 3 })
  })
})
