// 對照 api.md §6

export type LevelKind = 'build' | 'fix-compile' | 'debug'
export type LevelVisual = 'grid' | 'memory' | 'timeline' | 'none'

export interface ChapterSummary {
  chapter: number
  title: string
}

export interface LevelSummary {
  id: string
  chapter: number
  order: number
  title: string
  kind: LevelKind
  visual: LevelVisual
}

export interface LevelListResponse {
  chapters: ChapterSummary[]
  levels: LevelSummary[]
}

export interface LevelDetail extends LevelSummary {
  story: string
  concept: string
  hints: string[]
  starter: string
  grid: GridDef | null
  memory_snapshots: MemorySnapshot[] | null
}

export type Point = [number, number]

export interface GridDef {
  width: number
  height: number
  start: Point
  goal: Point
  walls: Point[]
  gems: Point[]
  doors: Point[]
}

export interface MemorySnapshot {
  line: number
  note: string
  frames: StackFrame[]
  heap: HeapBlock[]
  pointers: Pointer[]
  error: SnapshotError | null
}

export interface SnapshotError {
  code: string | null
  target: string
}

export interface StackFrame {
  id: string
  fn: string
  vars: StackVar[]
}

export interface StackVar {
  id: string
  name: string
  type: string
  value: string | null
  state: 'live' | 'moved' | 'dropped'
}

export interface HeapBlock {
  id: string
  type: string
  value: string
  state: 'live' | 'freed'
}

export interface Pointer {
  from: string
  to: string
  kind: 'own' | 'shared' | 'mut'
}

export interface Diagnostic {
  code: string | null
  message: string
  line: number | null
  column: number | null
  end_line: number | null
  end_column: number | null
  level: 'error' | 'warning' | 'note' | 'help'
  explanation_url: string | null
  rendered: string | null
}

export interface CompileResult {
  success: boolean
  timed_out: boolean
  diagnostics: Diagnostic[]
}

export interface ExecutionResult {
  exit_code: number | null
  stdout: string
  stderr: string
  timed_out: boolean
  stdout_truncated: boolean
  stderr_truncated: boolean
}

export type JudgeStatus =
  | 'ok'
  | 'compile_error'
  | 'runtime_panic'
  | 'test_failed'
  | 'timeout'
  | 'memory_limit'
  | 'output_limit'

export type RunStatus = Exclude<JudgeStatus, 'test_failed'>
export type SubmitStatus = Exclude<JudgeStatus, 'runtime_panic'>

export interface RunResponse {
  status: RunStatus
  compile: CompileResult
  execution: ExecutionResult | null
}

export type TestStatus = 'passed' | 'failed' | 'not_run'

export interface TestResult {
  name: string
  status: TestStatus
  message: string | null
}

export interface SubmitResponse {
  status: SubmitStatus
  passed: boolean
  compile: CompileResult
  execution: ExecutionResult | null
  tests: TestResult[]
}

export interface HealthResponse {
  status: 'ok' | 'degraded'
  judge: { available: boolean; reason: 'docker_unreachable' | 'runner_image_missing' | null }
}

export interface ApiError {
  error: { code: string; message: string }
}
