import { describe, expect, it } from 'vitest'
import { markerHover, MarkerSeverity, panicLocation, panicMarker, toMarkers } from '../markers'
import type { MarkerModel } from '../markers'
import { diag } from '../../../test/fixtures'

// 依 01 starter 的實際行內容模擬 Monaco model；第 4 行 41 字元、第 6 行空行
const LINES = [
  '// 回傳 Ferris 要執行的指令',
  'fn route() -> Vec<String> {',
  '    let mut cmds = Vec::new();',
  '    cmds.push(String::from("MOVE DOWN"));',
  '    cmds',
  '',
]

function modelOf(lines: string[]): MarkerModel {
  return {
    getLineMaxColumn: (line: number) => (lines[line - 1] ?? ''.padEnd(40, 'x')).length + 1,
    getWordAtPosition: ({ lineNumber, column }) => {
      const text = lines[lineNumber - 1] ?? ''
      for (const m of text.matchAll(/\w+/g)) {
        const start = m.index + 1
        const end = start + m[0].length
        if (column >= start && column <= end) return { word: m[0], startColumn: start, endColumn: end }
      }
      return null
    },
  }
}

const lineMax = modelOf(LINES)

function range(m: { startLineNumber: number; startColumn: number; endLineNumber: number; endColumn: number }) {
  return [m.startLineNumber, m.startColumn, m.endLineNumber, m.endColumn]
}

describe('F6 Diagnostic 轉 Monaco marker', () => {
  it('範圍直接對應，error 為 Error，訊息為人話並帶錯誤碼連結', () => {
    expect(toMarkers([diag()], lineMax)).toEqual([
      {
        startLineNumber: 10,
        startColumn: 17,
        endLineNumber: 10,
        endColumn: 20,
        severity: MarkerSeverity.Error,
        message: '`bag` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
        code: 'E0382',
        explanationUrl: 'https://doc.rust-lang.org/error_codes/E0382.html',
      },
    ])
  })

  it('warning 為 Warning', () => {
    const [m] = toMarkers([diag({ level: 'warning', code: 'unused_variables', message: 'unused variable: `x`', explanation_url: null })], lineMax)
    expect(m.severity).toBe(MarkerSeverity.Warning)
    expect(m.explanationUrl).toBeNull()
  })

  it('note 與 help 為 Info', () => {
    const ms = toMarkers([diag({ level: 'note' }), diag({ level: 'help' })], lineMax)
    expect(ms.map((m) => m.severity)).toEqual([MarkerSeverity.Info, MarkerSeverity.Info])
  })

  it('line = null 不畫 marker', () => {
    expect(toMarkers([diag({ line: null, column: null, end_line: null, end_column: null })], lineMax)).toEqual([])
  })

  it('end_column = null 時標到 end_line 行尾（第 4 行 41 字元 → 第 42 欄）', () => {
    expect(LINES[3]).toHaveLength(41)
    const [m] = toMarkers([diag({ line: 3, column: 5, end_line: 4, end_column: null })], lineMax)
    expect([m.startLineNumber, m.startColumn, m.endLineNumber, m.endColumn]).toEqual([3, 5, 4, 42])
  })

  it('多筆混合時只畫有位置的', () => {
    const ms = toMarkers(
      [diag({ line: null, column: null, end_line: null, end_column: null }), diag({ line: 2 }), diag({ line: 7 })],
      lineMax,
    )
    expect(ms).toHaveLength(2)
  })

  it('沒有診斷時為空陣列', () => {
    expect(toMarkers([], lineMax)).toEqual([])
  })
})

describe('F6 零寬度 span（architecture §7.4）', () => {
  const zero = (line: number, column: number) => diag({ line, column, end_line: line, end_column: column })

  it('(2, 4) 在 token 開頭：擴到 token 結尾 → 4..9', () => {
    const [m] = toMarkers([zero(2, 4)], lineMax)
    expect(range(m)).toEqual([2, 4, 2, 9])
  })

  it('(3, 1) 在空白上：endColumn = column + 1 → 1..2', () => {
    const [m] = toMarkers([zero(3, 1)], lineMax)
    expect(range(m)).toEqual([3, 1, 3, 2])
  })

  it('token 結尾不大於 column 時視同不在 token 內', () => {
    const model: MarkerModel = { ...lineMax, getWordAtPosition: () => ({ word: 'x', startColumn: 10, endColumn: 12 }) }
    const [m] = toMarkers([zero(3, 12)], model)
    expect(range(m)).toEqual([3, 12, 3, 13])
  })

  it('(3, 31) 在非空行行尾：改標前一字元 → 30..31', () => {
    expect(LINES[2]).toHaveLength(30)
    const [m] = toMarkers([zero(3, 31)], lineMax)
    expect(range(m)).toEqual([3, 30, 3, 31])
  })

  it('空行：照原值不另處理', () => {
    const [m] = toMarkers([zero(6, 1)], lineMax)
    expect(range(m)).toEqual([6, 1, 6, 1])
  })

  it('非零寬度的 span 不改寫', () => {
    const [m] = toMarkers([diag({ line: 2, column: 5, end_line: 2, end_column: 7 })], lineMax)
    expect(range(m)).toEqual([2, 5, 2, 7])
  })
})

describe('F6 panic 位置', () => {
  const stderr = "\nthread 'main' (12) panicked at /tmp/main.rs:15:11:\nRefCell already borrowed\n"

  it('從 stderr 取出 main.rs:行:欄', () => {
    expect(panicLocation(stderr)).toEqual({ line: 15, column: 11 })
  })

  it('沒有位置時回 null', () => {
    expect(panicLocation('Killed\n')).toBeNull()
  })

  it('panic 行畫紅色 marker', () => {
    const m = panicMarker(stderr, lineMax)
    expect(m?.severity).toBe(MarkerSeverity.Error)
    expect(m?.startLineNumber).toBe(15)
    expect(m?.endLineNumber).toBe(15)
  })

  it('panic marker 標整行：第 1 欄到行尾', () => {
    const m = panicMarker(stderr, lineMax)
    expect([m?.startLineNumber, m?.startColumn, m?.endLineNumber, m?.endColumn]).toEqual([15, 1, 15, 41])
    expect(lineMax.getLineMaxColumn(15)).toBe(41)
  })

  it('沒有位置時不畫 marker', () => {
    expect(panicMarker('', lineMax)).toBeNull()
  })
})

describe('F6 marker hover 內容', () => {
  it('字典人話的反引號保留為 markdown 行內程式碼，並附錯誤碼連結', () => {
    const [marker] = toMarkers([diag()], lineMax)
    expect(markerHover(marker)).toEqual([
      '`bag` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
      '[E0382](https://doc.rust-lang.org/error_codes/E0382.html)',
    ])
  })

  it('字典外的 message 照字面顯示，反引號不轉成程式碼', () => {
    const [marker] = toMarkers([diag({ code: null, message: 'expected `;`, found `println`', explanation_url: null })], lineMax)
    expect(markerHover(marker)).toEqual(['expected \\`;\\`, found \\`println\\`'])
  })

  it('沒有說明網址的錯誤碼以純文字附上', () => {
    const [marker] = toMarkers([diag({ level: 'warning', code: 'unused_variables', message: 'unused variable: `x`', explanation_url: null })], lineMax)
    expect(markerHover(marker)).toEqual(['unused variable: \\`x\\`', 'unused\\_variables'])
  })
})
