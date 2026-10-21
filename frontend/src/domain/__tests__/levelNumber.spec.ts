import { describe, expect, it } from 'vitest'
import { levelNumber } from '../levelNumber'
import { levels } from '../../test/fixtures'

describe('F2 關卡編號「章-章內序」', () => {
  it.each([
    ['01-hello-ferris', '1-1'],
    ['02-mut-matters', '1-2'],
    ['05-delta', '2-1'],
    ['09-move-the-gem', '3-1'],
    ['11-borrow', '3-2'],
    ['19-lighthouse', '5-1'],
  ])('%s 顯示為 %s', (id, expected) => {
    expect(levelNumber(id, levels)).toBe(expected)
  })

  it('章內序依 order 排序，與陣列順序無關', () => {
    expect(levelNumber('11-borrow', [...levels].reverse())).toBe('3-2')
  })

  it('補關後編號跟著改變', () => {
    const withNew = [...levels, { id: '10-new', chapter: 3, order: 10, title: '新關', kind: 'build' as const, visual: 'none' as const }]
    expect(levelNumber('11-borrow', withNew)).toBe('3-3')
  })

  it('找不到關卡時回空字串', () => {
    expect(levelNumber('99-unknown', levels)).toBe('')
  })
})
