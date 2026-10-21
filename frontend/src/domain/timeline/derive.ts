// ux.md §11.5、§11.6 時間軸狀態推導
import type { RunStatus } from '../../api/types'
import type { TimelineEvent } from './parse'

export type LaneState = 'not_started' | 'running' | 'waiting_lock' | 'waiting_msg' | 'ended' | 'deadlocked'

export interface TimelineLane {
  name: string
  state: LaneState
  // 該泳道到目前這一步為止，最近一個有行號的事件
  line: number | null
}

export interface TimelineSegment {
  lane: string
  kind: 'hold' | 'wait_lock' | 'wait_msg'
  // 鎖名或 channel 名
  target: string
  from: number
  // null 表示到目前仍未結束
  to: number | null
  deadlocked: boolean
}

export interface TimelineMessage {
  channel: string
  from: { lane: string; step: number }
  // null 表示還沒被收到
  to: { lane: string; step: number } | null
}

export interface TimelineMark {
  lane: string
  step: number
  kind: 'start' | 'end' | 'recv_closed' | 'note'
  // note 為標註文字、recv_closed 為 channel 名、start／end 為空字串
  text: string
}

export interface DeadlockLink {
  lane: string
  waits: string
  heldBy: string
}

export interface TimelineState {
  // main 置頂、其餘依第一個事件出現的先後；全部事件的泳道一開始就配置好
  lanes: TimelineLane[]
  segments: TimelineSegment[]
  messages: TimelineMessage[]
  marks: TimelineMark[]
  // 從循環中泳道排序最前者開始，沿等待關係排列
  deadlock: DeadlockLink[] | null
  // 目前這一步的事件句；第 0 步為空字串
  sentence: string
  // 播到這一步時的結論
  conclusion: string
}

// main 置頂，其餘依第一個事件出現的先後
function laneOrder(events: TimelineEvent[]): string[] {
  const seen: string[] = []
  for (const e of events) if (!seen.includes(e.thread)) seen.push(e.thread)
  return seen.includes('main') ? ['main', ...seen.filter((n) => n !== 'main')] : seen
}

interface LaneTrack {
  last: TimelineEvent | null
  line: number | null
  named: boolean
  ended: boolean
}

interface Send {
  lane: string
  step: number
}

const held = (by: string, lane: string) => (by === lane ? '（被自己佔著）' : `（被 ${by} 佔著）`)

export function deriveTimeline(events: TimelineEvent[], step: number, status: RunStatus): TimelineState {
  const order = laneOrder(events)
  const tracks = new Map<string, LaneTrack>(order.map((n) => [n, { last: null, line: null, named: false, ended: false }]))
  const segments: TimelineSegment[] = []
  const marks: TimelineMark[] = []
  const sends = new Map<string, Send[]>()
  const recvs = new Map<string, Send[]>()
  const upto = Math.max(0, Math.min(step, events.length))

  // 鎖目前的持有者：未結束 hold 中最近開始的
  const holderOf = (lock: string): TimelineSegment | null => {
    let found: TimelineSegment | null = null
    for (const g of segments) if (g.kind === 'hold' && g.target === lock && g.to === null && (!found || g.from > found.from)) found = g
    return found
  }

  let sentence = ''
  for (let i = 0; i < upto; i++) {
    const e = events[i]
    const s = i + 1
    const track = tracks.get(e.thread)!
    for (const g of segments) if (g.lane === e.thread && g.to === null && g.kind !== 'hold') g.to = s

    let text = ''
    switch (e.kind) {
      case 'start':
        track.named = true
        marks.push({ lane: e.thread, step: s, kind: 'start', text: '' })
        text = `${e.thread} 開始`
        break
      case 'end':
        track.ended = true
        marks.push({ lane: e.thread, step: s, kind: 'end', text: '' })
        text = `${e.thread} 結束`
        break
      case 'lock_wait': {
        const h = holderOf(e.target)
        segments.push({ lane: e.thread, kind: 'wait_lock', target: e.target, from: s, to: null, deadlocked: false })
        text = `${e.thread} 想拿 ${e.target}${h ? held(h.lane, e.thread) : ''}`
        break
      }
      case 'lock_got':
        segments.push({ lane: e.thread, kind: 'hold', target: e.target, from: s, to: null, deadlocked: false })
        text = `${e.thread} 拿到 ${e.target}`
        break
      case 'lock_free': {
        const open = segments.filter((g) => g.lane === e.thread && g.kind === 'hold' && g.target === e.target && g.to === null)
        if (open.length > 0) open[open.length - 1].to = s
        text = `${e.thread} 放開 ${e.target}`
        break
      }
      case 'send':
        if (!sends.has(e.target)) sends.set(e.target, [])
        sends.get(e.target)!.push({ lane: e.thread, step: s })
        text = `${e.thread} 往 ${e.target} 送出一則訊息`
        break
      case 'recv_wait':
        segments.push({ lane: e.thread, kind: 'wait_msg', target: e.target, from: s, to: null, deadlocked: false })
        text = `${e.thread} 等 ${e.target} 的訊息`
        break
      case 'recv': {
        if (!recvs.has(e.target)) recvs.set(e.target, [])
        const list = recvs.get(e.target)!
        list.push({ lane: e.thread, step: s })
        const sender = sends.get(e.target)?.[list.length - 1]
        text = `${e.thread} 從 ${e.target} 收到一則訊息${sender ? `（${sender.lane} 送的）` : ''}`
        break
      }
      case 'recv_closed':
        marks.push({ lane: e.thread, step: s, kind: 'recv_closed', text: e.target })
        text = `${e.target} 已關閉，${e.thread} 不會再收到訊息`
        break
      case 'note':
        marks.push({ lane: e.thread, step: s, kind: 'note', text: e.target })
        text = `${e.thread}：${e.target}`
        break
    }
    track.last = e
    if (e.line !== null) track.line = e.line
    if (s === upto) sentence = `第 ${s} 步${e.line !== null ? ` · 第 ${e.line} 行` : ''}：${text}`
  }

  // 訊息：每個 channel 第 k 個 send 對第 k 個 recv
  const messages: TimelineMessage[] = []
  const allSends = [...sends.entries()].flatMap(([channel, list]) => list.map((from, k) => ({ channel, from, k })))
  allSends.sort((a, b) => a.from.step - b.from.step)
  for (const { channel, from, k } of allSends) {
    const to = recvs.get(channel)?.[k] ?? null
    if (to && to.step < from.step) continue
    messages.push({ channel, from, to })
  }

  // 等待關係：最後一個事件是 lock_wait 且該鎖有持有者
  const waitsFor = new Map<string, { lock: string; hold: TimelineSegment }>()
  for (const [name, t] of tracks) {
    if (t.last?.kind !== 'lock_wait') continue
    const h = holderOf(t.last.target)
    if (h) waitsFor.set(name, { lock: t.last.target, hold: h })
  }

  // 找循環；每個循環從其中泳道排序最前者開始
  const inCycle = new Set<string>()
  let deadlock: DeadlockLink[] | null = null
  for (const start of order) {
    if (inCycle.has(start)) continue
    const path: string[] = []
    let cur: string | undefined = start
    while (cur !== undefined && !path.includes(cur) && !inCycle.has(cur)) {
      path.push(cur)
      cur = waitsFor.get(cur)?.hold.lane
    }
    if (cur === undefined || inCycle.has(cur)) continue
    const cycle = path.slice(path.indexOf(cur))
    const first = cycle.reduce((a, b) => (order.indexOf(b) < order.indexOf(a) ? b : a))
    const links: DeadlockLink[] = []
    let lane = first
    do {
      const w = waitsFor.get(lane)!
      links.push({ lane, waits: w.lock, heldBy: w.hold.lane })
      inCycle.add(lane)
      w.hold.deadlocked = true
      for (const g of segments) if (g.lane === lane && g.kind === 'wait_lock' && g.to === null) g.deadlocked = true
      lane = w.hold.lane
    } while (lane !== first)
    deadlock = [...(deadlock ?? []), ...links]
  }

  const lanes: TimelineLane[] = order.map((name) => {
    const t = tracks.get(name)!
    let state: LaneState = 'running'
    if (t.last === null) state = 'not_started'
    else if (t.ended) state = 'ended'
    else if (inCycle.has(name)) state = 'deadlocked'
    else if (t.last.kind === 'lock_wait') state = 'waiting_lock'
    else if (t.last.kind === 'recv_wait') state = 'waiting_msg'
    return { name, state, line: t.line }
  })

  const conclusion = conclude(lanes, tracks, deadlock, status)
  return { lanes, segments, messages, marks, deadlock, sentence, conclusion }
}

function conclude(lanes: TimelineLane[], tracks: Map<string, LaneTrack>, deadlock: DeadlockLink[] | null, status: RunStatus): string {
  const named = lanes.filter((l) => tracks.get(l.name)!.named)
  if (deadlock && (status === 'ok' || status === 'timeout')) {
    return `✗ 死鎖：${deadlock.map((d) => `${d.lane} 等 ${d.waits}${held(d.heldBy, d.lane)}`).join('；')}`
  }
  switch (status) {
    case 'ok': {
      const unfinished = named.filter((l) => l.state !== 'ended')
      if (unfinished.length === 0) return '全部執行緒都結束了'
      return `程式結束時，還有 ${unfinished.length} 條執行緒沒做完（${unfinished.map((l) => l.name).join('、')}）`
    }
    case 'timeout': {
      const waiting = (l: TimelineLane) => l.state === 'waiting_lock' || l.state === 'waiting_msg'
      const parts: string[] = []
      const count = named.filter(waiting).length
      if (count > 0) parts.push(`還有 ${count} 條執行緒在等`)
      for (const l of lanes) {
        if (tracks.get(l.name)!.named || !waiting(l)) continue
        parts.push(`${l.name} ${l.state === 'waiting_msg' ? '等訊息' : '等鎖'}`)
      }
      return parts.length > 0 ? `時間到：${parts.join('；')}` : '時間到'
    }
    case 'runtime_panic':
      return '程式在這裡結束（panic）'
    case 'memory_limit':
      return '記憶體不足'
    case 'output_limit':
      return '輸出過長已截斷'
    default:
      return ''
  }
}
