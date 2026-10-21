import { describe, expect, it } from 'vitest'
import type { GridDef } from '../../../api/types'
import { validateGrid } from '../validate'
import { sampleGrid } from '../../../test/fixtures'

function g(overrides: Partial<GridDef>): GridDef {
  return { ...sampleGrid, ...overrides }
}

describe('F3 validateGrid 地圖載入驗證', () => {
  it('合法地圖沒有問題', () => {
    expect(validateGrid(sampleGrid)).toEqual([])
  })

  it.each([
    [2, 2],
    [12, 12],
  ])('寬高邊界 %i×%i 合法', (width, height) => {
    expect(validateGrid({ width, height, start: [0, 0], goal: [1, 1], walls: [], gems: [], doors: [] })).toEqual([])
  })

  it.each([
    [1, 5],
    [13, 5],
    [6, 1],
    [6, 13],
  ])('寬高 %i×%i 超出 2–12 為 size', (width, height) => {
    const issues = validateGrid(g({ width, height, start: [0, 0], goal: [0, 0], walls: [], gems: [], doors: [] }))
    expect(issues.map((i) => i.kind)).toContain('size')
  })

  it('座標超出範圍為 out_of_bounds', () => {
    expect(validateGrid(g({ gems: [[6, 0]] }))).toContainEqual({ kind: 'out_of_bounds', point: [6, 0] })
    expect(validateGrid(g({ walls: [[0, -1]] }))).toContainEqual({ kind: 'out_of_bounds', point: [0, -1] })
  })

  it('start 與 goal 相同為 overlap', () => {
    expect(validateGrid(g({ goal: [0, 0] }))).toContainEqual({ kind: 'overlap', point: [0, 0] })
  })

  it('牆與寶石重疊為 overlap', () => {
    expect(validateGrid(g({ gems: [[1, 0]] }))).toContainEqual({ kind: 'overlap', point: [1, 0] })
  })

  it('門與起點重疊為 overlap', () => {
    expect(validateGrid(g({ doors: [[0, 0]] }))).toContainEqual({ kind: 'overlap', point: [0, 0] })
  })

  it('同一陣列內重複座標為 duplicate', () => {
    expect(validateGrid(g({ walls: [[1, 0], [1, 0]] }))).toContainEqual({ kind: 'duplicate', point: [1, 0] })
  })
})
