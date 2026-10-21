// ux.md §11.4 時間軸事件解析

export type TimelineEventKind =
  | 'start'
  | 'end'
  | 'lock_wait'
  | 'lock_got'
  | 'lock_free'
  | 'send'
  | 'recv_wait'
  | 'recv'
  | 'recv_closed'
  | 'note'

export interface TimelineEvent {
  kind: TimelineEventKind
  // 編輯器行號；lock_free、end 為 null
  line: number | null
  // 泳道名稱（去除頭尾空白）
  thread: string
  // 鎖名、channel 名、標註文字；start、end 為空字串
  target: string
}

export interface ParsedTimeline {
  events: TimelineEvent[]
  // 以 @tl| 開頭但格式不對的行數
  malformed: number
  // 合法事件超過上限被截掉
  capped: boolean
}

export const MAX_TIMELINE_EVENTS = 1000

const PREFIX = '@tl|'
const KINDS = new Set<string>(['start', 'end', 'lock_wait', 'lock_got', 'lock_free', 'send', 'recv_wait', 'recv', 'recv_closed', 'note'])
const LINE_RE = /^[1-9][0-9]*$/

// 去掉前綴後的一行；格式不對回傳 null
function parseLine(body: string): TimelineEvent | null {
  const parts = body.split('|')
  if (parts.length < 4) return null
  const [kind, line, rawThread] = parts
  const rest = parts.slice(3).join('|')
  if (!KINDS.has(kind)) return null
  if (line !== '' && !LINE_RE.test(line)) return null
  const thread = rawThread.trim()
  if (thread === '') return null
  const bare = kind === 'start' || kind === 'end'
  if (!bare && rest === '') return null
  return { kind: kind as TimelineEventKind, line: line === '' ? null : Number(line), thread, target: bare ? '' : rest }
}

export function parseTimelineEvents(stdout: string, truncated: boolean): ParsedTimeline {
  const lines = stdout.split(/\r?\n/)
  if (truncated) {
    // 結尾換行後的空字串不算一行
    if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop()
    lines.pop()
  }
  const events: TimelineEvent[] = []
  let malformed = 0
  let capped = false
  for (const raw of lines) {
    if (!raw.startsWith(PREFIX)) continue
    const event = parseLine(raw.slice(PREFIX.length))
    if (events.length >= MAX_TIMELINE_EVENTS) {
      // 上限後的格式錯誤不計入
      if (event === null) continue
      capped = true
      break
    }
    if (event === null) malformed += 1
    else events.push(event)
  }
  return { events, malformed, capped }
}
