import type { Diagnostic } from '../../api/types'
import { ERROR_HINTS, humanize } from '../errorHints'

// 與 monaco.MarkerSeverity 數值相同
export const MarkerSeverity = { Info: 2, Warning: 4, Error: 8 } as const
export type MarkerSeverityValue = (typeof MarkerSeverity)[keyof typeof MarkerSeverity]

export interface EditorMarker {
  startLineNumber: number
  startColumn: number
  endLineNumber: number
  endColumn: number
  severity: MarkerSeverityValue
  message: string
  code: string | null
  explanationUrl: string | null
}

// Monaco ITextModel 用到的部分
export interface MarkerModel {
  getLineMaxColumn(line: number): number
  getWordAtPosition(position: { lineNumber: number; column: number }): { word: string; startColumn: number; endColumn: number } | null
}

const SEVERITY: Record<Diagnostic['level'], MarkerSeverityValue> = {
  error: MarkerSeverity.Error,
  warning: MarkerSeverity.Warning,
  note: MarkerSeverity.Info,
  help: MarkerSeverity.Info,
}

type Range = Pick<EditorMarker, 'startLineNumber' | 'startColumn' | 'endLineNumber' | 'endColumn'>

// 零寬度 span 改寫：擴到 token 結尾、至少 1 字元、行尾改標前一字元
function widenZeroWidth(range: Range, model: MarkerModel): Range {
  const line = range.startLineNumber
  const column = range.startColumn
  const maxColumn = model.getLineMaxColumn(line)
  if (maxColumn <= 1) return range
  const word = model.getWordAtPosition({ lineNumber: line, column })
  if (word && word.endColumn > column) return { ...range, endColumn: word.endColumn }
  if (column < maxColumn) return { ...range, endColumn: column + 1 }
  return { ...range, startColumn: column - 1 }
}

// Diagnostic 轉 marker；line = null 不畫；end_column = null 標到 end_line 行尾；零寬度 span 依 architecture §7.4 改寫
export function toMarkers(diags: Diagnostic[], model: MarkerModel): EditorMarker[] {
  const markers: EditorMarker[] = []
  for (const d of diags) {
    if (d.line === null) continue
    const startColumn = d.column ?? 1
    const endLineNumber = d.end_line ?? d.line
    const endColumn = d.end_column ?? model.getLineMaxColumn(endLineNumber)
    let range: Range = { startLineNumber: d.line, startColumn, endLineNumber, endColumn }
    if (range.startLineNumber === range.endLineNumber && range.startColumn === range.endColumn) {
      range = widenZeroWidth(range, model)
    }
    markers.push({
      ...range,
      severity: SEVERITY[d.level],
      message: humanize(d),
      code: d.code,
      explanationUrl: d.explanation_url,
    })
  }
  return markers
}

// stderr 中 main.rs:行:欄 的 panic 位置
export function panicLocation(stderr: string): { line: number; column: number } | null {
  const match = /main\.rs:(\d+):(\d+)/.exec(stderr)
  return match ? { line: Number(match[1]), column: Number(match[2]) } : null
}

// 整行 error marker
export function panicMarker(stderr: string, model: MarkerModel): EditorMarker | null {
  const location = panicLocation(stderr)
  if (!location) return null
  return {
    startLineNumber: location.line,
    startColumn: 1,
    endLineNumber: location.line,
    endColumn: model.getLineMaxColumn(location.line),
    severity: MarkerSeverity.Error,
    message: '程式在執行途中當掉了（panic）',
    code: null,
    explanationUrl: null,
  }
}

const escapeMarkdown = (text: string) => text.replace(/[\\`*_{}[\]()#+\-.!<>|~]/g, '\\$&')

// hover 內容（markdown）：字典人話的反引號為行內程式碼，其餘照字面；附錯誤碼連結
export function markerHover(marker: EditorMarker): string[] {
  const fromHints = marker.code !== null && ERROR_HINTS[marker.code] !== undefined
  const parts = [fromHints ? marker.message : escapeMarkdown(marker.message)]
  if (marker.code !== null) {
    parts.push(marker.explanationUrl ? `[${marker.code}](${marker.explanationUrl})` : escapeMarkdown(marker.code))
  }
  return parts
}
