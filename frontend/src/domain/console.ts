import type { RequestFailure } from '../api/client'
import type { JudgeStatus, RunResponse, SubmitResponse } from '../api/types'

export type ConsoleResult =
  | { action: 'run'; data: RunResponse }
  | { action: 'submit'; data: SubmitResponse }
  | { action: 'run' | 'submit'; failure: RequestFailure }

export type ConsoleTab = 'output' | 'diagnostics' | 'tests'

// 主控台預設分頁只看結果狀態，不看 kind
export function defaultTab(action: 'run' | 'submit', status: JudgeStatus): ConsoleTab {
  if (status === 'compile_error') return 'diagnostics'
  if (status === 'test_failed') return 'tests'
  if (action === 'submit' && status === 'ok') return 'tests'
  return 'output'
}
