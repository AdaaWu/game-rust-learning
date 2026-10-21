import type { Diagnostic } from '../api/types'

export const VISIBLE_CARD_LIMIT = 5

// 卡片位置欄：「第 L 行第 C 欄」「隱藏測試」「無位置」
export function locationLabel(diag: Diagnostic): string {
  if (diag.line !== null) return `第 ${diag.line} 行第 ${diag.column ?? 1} 欄`
  return diag.rendered === null ? '隱藏測試' : '無位置'
}

const LEVEL_RANK: Record<string, number> = { error: 0, warning: 1 }

// 只留 error 與 warning，error 在前、同級依行號、line = null 排最後
export function diagnosticCards(diags: Diagnostic[]): Diagnostic[] {
  return diags
    .filter((d) => d.level === 'error' || d.level === 'warning')
    .map((d, index) => ({ d, index }))
    .sort((a, b) => {
      const rank = LEVEL_RANK[a.d.level] - LEVEL_RANK[b.d.level]
      if (rank !== 0) return rank
      const la = a.d.line ?? Number.POSITIVE_INFINITY
      const lb = b.d.line ?? Number.POSITIVE_INFINITY
      if (la !== lb) return la - lb
      return a.index - b.index
    })
    .map(({ d }) => d)
}

export function errorCount(diags: Diagnostic[]): number {
  return diags.filter((d) => d.level === 'error').length
}
