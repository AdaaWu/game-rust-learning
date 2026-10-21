import type { HeapBlock, MemorySnapshot, Pointer, StackVar } from '../api/types'

export type BorrowState = 'mut' | 'shared' | null

// 錯誤標記：badge 為錯誤碼或「執行期錯誤」
export interface ErrorMark {
  badge: string
}

export interface VarView extends StackVar {
  borrow: BorrowState
  error: ErrorMark | null
  changed: boolean
}

export interface FrameView {
  id: string
  fn: string
  vars: VarView[]
  current: boolean
  changed: boolean
}

export interface HeapView extends HeapBlock {
  error: ErrorMark | null
  changed: boolean
}

export interface PointerView extends Pointer {
  danger: boolean
}

export interface SnapshotView {
  line: number
  note: string
  frames: FrameView[]
  heap: HeapView[]
  pointers: PointerView[]
}

// 由 pointers 推導借用狀態；mut 優先
export function borrowStateOf(id: string, pointers: Pointer[]): BorrowState {
  const incoming = pointers.filter((p) => p.to === id)
  if (incoming.some((p) => p.kind === 'mut')) return 'mut'
  if (incoming.some((p) => p.kind === 'shared')) return 'shared'
  return null
}

function errorMarkFor(id: string, snapshot: MemorySnapshot): ErrorMark | null {
  if (snapshot.error === null || snapshot.error.target !== id) return null
  return { badge: snapshot.error.code ?? '執行期錯誤' }
}

function previousStates(snapshot: MemorySnapshot | undefined): Map<string, string> {
  const states = new Map<string, string>()
  if (!snapshot) return states
  for (const frame of snapshot.frames) for (const v of frame.vars) states.set(v.id, v.state)
  for (const block of snapshot.heap) states.set(block.id, block.state)
  return states
}

// index 為陣列索引（0 起算）
export function deriveSnapshotView(snapshots: MemorySnapshot[], index: number): SnapshotView {
  const snapshot = snapshots[index]
  const previous = index > 0 ? previousStates(snapshots[index - 1]) : null
  // 上一步已是 dropped／freed 的項目從畫面移除
  const isGone = (id: string, state: string) =>
    (state === 'dropped' || state === 'freed') && previous !== null && previous.get(id) === state
  const changed = (id: string, state: string) => previous !== null && previous.get(id) !== state
  const previousFrames = index > 0 ? new Set(snapshots[index - 1].frames.map((f) => f.id)) : null

  const frames: FrameView[] = snapshot.frames.map((frame, i) => ({
    id: frame.id,
    fn: frame.fn,
    current: i === snapshot.frames.length - 1,
    changed: previousFrames !== null && !previousFrames.has(frame.id),
    vars: frame.vars
      .filter((v) => !isGone(v.id, v.state))
      .map((v) => ({
        ...v,
        borrow: borrowStateOf(v.id, snapshot.pointers),
        error: errorMarkFor(v.id, snapshot),
        changed: changed(v.id, v.state),
      })),
  }))
  const heap: HeapView[] = snapshot.heap
    .filter((h) => !isGone(h.id, h.state))
    .map((h) => ({ ...h, error: errorMarkFor(h.id, snapshot), changed: changed(h.id, h.state) }))
  const errorVar = snapshot.error?.target
  const isVarTarget = snapshot.frames.some((f) => f.vars.some((v) => v.id === errorVar))
  const pointers: PointerView[] = snapshot.pointers.map((p) => ({ ...p, danger: isVarTarget && p.from === errorVar }))
  return { line: snapshot.line, note: snapshot.note, frames, heap, pointers }
}

// 「第 n 步 · 第 L 行」
export function stepCaption(snapshots: MemorySnapshot[], index: number): string {
  return `第 ${index + 1} 步 · 第 ${snapshots[index].line} 行`
}

// 反向同步：第一個 line 等於該行的步驟索引
export function findStepForLine(snapshots: MemorySnapshot[], line: number): number | null {
  const index = snapshots.findIndex((s) => s.line === line)
  return index >= 0 ? index : null
}
