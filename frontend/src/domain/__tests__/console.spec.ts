import { describe, expect, it } from 'vitest'
import { defaultTab } from '../console'
import type { JudgeStatus } from '../../api/types'

describe('F6 主控台預設分頁（只看結果狀態）', () => {
  it.each([
    ['run', 'ok', 'output'],
    ['run', 'compile_error', 'diagnostics'],
    ['run', 'runtime_panic', 'output'],
    ['run', 'timeout', 'output'],
    ['run', 'memory_limit', 'output'],
    ['run', 'output_limit', 'output'],
    ['submit', 'ok', 'tests'],
    ['submit', 'compile_error', 'diagnostics'],
    ['submit', 'test_failed', 'tests'],
    ['submit', 'timeout', 'output'],
    ['submit', 'memory_limit', 'output'],
    ['submit', 'output_limit', 'output'],
  ] as ['run' | 'submit', JudgeStatus, string][])('%s %s → %s', (action, status, tab) => {
    expect(defaultTab(action, status)).toBe(tab)
  })
})
