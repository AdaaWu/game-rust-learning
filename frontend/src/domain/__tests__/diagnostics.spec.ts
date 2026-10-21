import { describe, expect, it } from 'vitest'
import { diagnosticCards, errorCount, locationLabel, VISIBLE_CARD_LIMIT } from '../diagnostics'
import { diag } from '../../test/fixtures'

const noPos = { line: null, column: null, end_line: null, end_column: null }

describe('F6 錯誤卡片位置與排序', () => {
  it('有位置：「第 L 行第 C 欄」', () => {
    expect(locationLabel(diag())).toBe('第 10 行第 17 欄')
  })

  it('line 與 rendered 都是 null：「隱藏測試」', () => {
    expect(locationLabel(diag({ ...noPos, rendered: null }))).toBe('隱藏測試')
  })

  it('line 為 null 但 rendered 有值：「無位置」', () => {
    expect(locationLabel(diag({ ...noPos, rendered: 'error: linking with `cc` failed' }))).toBe('無位置')
  })

  it('error 在前、warning 在後；同級依行號；line = null 排在同級最後', () => {
    const cards = diagnosticCards([
      diag({ code: 'E0425', ...noPos, rendered: null }),
      diag({ code: null, level: 'warning', line: 2 }),
      diag({ code: 'E0384', line: 7 }),
      diag({ code: 'E0382', line: 3 }),
    ])
    expect(cards.map((c) => c.code ?? c.level)).toEqual(['E0382', 'E0384', 'E0425', 'warning'])
  })

  it('note 與 help 不另開卡片', () => {
    const cards = diagnosticCards([diag(), diag({ level: 'note', code: null }), diag({ level: 'help', code: null })])
    expect(cards).toHaveLength(1)
  })

  it('空陣列沒有卡片', () => {
    expect(diagnosticCards([])).toEqual([])
  })

  it('同行依欄位穩定排序（保持原順序）', () => {
    const a = diag({ code: 'E0001', line: 5 })
    const b = diag({ code: 'E0002', line: 5 })
    expect(diagnosticCards([a, b]).map((c) => c.code)).toEqual(['E0001', 'E0002'])
  })

  it('預設展開前 5 筆', () => {
    expect(VISIBLE_CARD_LIMIT).toBe(5)
  })

  it('errorCount 只算 error', () => {
    expect(errorCount([diag(), diag(), diag({ level: 'warning' }), diag({ level: 'note' })])).toBe(2)
  })
})
