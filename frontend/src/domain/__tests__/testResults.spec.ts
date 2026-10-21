import { describe, expect, it } from 'vitest'
import { splitAssertion, testCounts } from '../testResults'

describe('F6 測試結果拆解', () => {
  it('assert_eq! 加自訂訊息：標題、實際、預期', () => {
    const msg = 'assertion `left == right` failed: Ferris 沒有停在終點\n  left: (0, 1)\n right: (4, 1)'
    expect(splitAssertion(msg)).toEqual({ title: 'Ferris 沒有停在終點', actual: '(0, 1)', expected: '(4, 1)' })
  })

  it('assert_eq! 沒有自訂訊息時標題為 null', () => {
    expect(splitAssertion('assertion `left == right` failed\n  left: 0\n right: 5050')).toEqual({
      title: null,
      actual: '0',
      expected: '5050',
    })
  })

  it('拆不出 left／right 時回 null', () => {
    expect(splitAssertion('RefCell already borrowed')).toBeNull()
  })

  it('只有 left 沒有 right 時回 null', () => {
    expect(splitAssertion('assertion failed\n  left: 1')).toBeNull()
  })

  it('testCounts 由 tests 計數', () => {
    expect(
      testCounts([
        { name: 'a', status: 'failed', message: 'x' },
        { name: 'b', status: 'failed', message: 'y' },
        { name: 'c', status: 'passed', message: null },
      ]),
    ).toEqual({ passed: 1, total: 3 })
  })

  it('testCounts 空陣列', () => {
    expect(testCounts([])).toEqual({ passed: 0, total: 0 })
  })
})
