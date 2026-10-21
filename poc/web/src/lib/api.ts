export type Pos = [number, number]

export interface GridDef {
  width: number
  height: number
  start: Pos
  goal: Pos
  walls: Pos[]
  gems: Pos[]
  doors: Pos[]
}

export interface LevelDetail {
  id: string
  chapter: number
  order: number
  title: string
  kind: string
  visual: string
  story: string
  concept: string
  hints: string[]
  starter: string
  grid: GridDef
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
  duration_ms: number
  diagnostics: Diagnostic[]
}

export interface ExecutionResult {
  exit_code: number | null
  stdout: string
  stderr: string
  timed_out: boolean
  stdout_truncated: boolean
  stderr_truncated: boolean
  duration_ms: number
}

export type JudgeStatus =
  | 'ok' | 'compile_error' | 'runtime_panic' | 'test_failed'
  | 'timeout' | 'memory_limit' | 'output_limit'

export interface RunResponse {
  status: JudgeStatus
  compile: CompileResult
  execution: ExecutionResult | null
}

export interface TestResult {
  name: string
  status: 'passed' | 'failed' | 'not_run'
  message: string | null
}

export interface SubmitResponse extends RunResponse {
  passed: boolean
  tests: TestResult[]
  summary: { total: number; passed: number; failed: number; not_run: number }
  star_inputs: { warning_count: number }
}

export class ApiError extends Error {
  constructor(public code: string, message: string) {
    super(message)
  }
}

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  let res: Response
  try {
    res = await fetch(url, init)
  } catch {
    throw new ApiError('NETWORK', '連不上伺服器，檢查網路後再試')
  }
  const body = await res.json().catch(() => null)
  if (!res.ok) {
    throw new ApiError(body?.error?.code ?? 'INTERNAL_ERROR', body?.error?.message ?? '伺服器發生錯誤')
  }
  return body as T
}

export const fetchLevel = () => request<LevelDetail>('/api/levels/02')

const post = <T>(path: string, code: string) =>
  request<T>(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ code }),
  })

export const runCode = (code: string) => post<RunResponse>('/api/levels/02/run', code)
export const submitCode = (code: string) => post<SubmitResponse>('/api/levels/02/submit', code)
