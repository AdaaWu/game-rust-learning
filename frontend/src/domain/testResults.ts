import type { TestResult } from '../api/types'

export interface AssertionSplit {
  title: string | null
  actual: string
  expected: string
}

// assert_eq! 訊息拆成實際（left）與預期（right）；拆不出來回 null
export function splitAssertion(message: string): AssertionSplit | null {
  const left = /^\s*left:\s?(.*)$/m.exec(message)
  const right = /^\s*right:\s?(.*)$/m.exec(message)
  if (!left || !right) return null
  const firstLine = message.split('\n')[0]
  const failedAt = firstLine.indexOf('failed:')
  const title = failedAt >= 0 ? firstLine.slice(failedAt + 'failed:'.length).trim() || null : null
  return { title, actual: left[1], expected: right[1] }
}

export function testCounts(tests: TestResult[]): { passed: number; total: number } {
  return { passed: tests.filter((t) => t.status === 'passed').length, total: tests.length }
}
