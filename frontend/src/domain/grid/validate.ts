import type { GridDef, Point } from '../../api/types'

export type GridIssueKind = 'size' | 'out_of_bounds' | 'overlap' | 'duplicate'

export interface GridIssue {
  kind: GridIssueKind
  point: Point | null
}

const inRange = (n: number) => Number.isInteger(n) && n >= 2 && n <= 12

// ux.md §4.2 載入時驗證
export function validateGrid(grid: GridDef): GridIssue[] {
  const issues: GridIssue[] = []
  if (!inRange(grid.width) || !inRange(grid.height)) issues.push({ kind: 'size', point: null })
  const groups: Point[][] = [[grid.start], [grid.goal], grid.walls, grid.gems, grid.doors]
  const owner = new Map<string, number>()
  groups.forEach((points, group) => {
    const seen = new Set<string>()
    for (const p of points) {
      const key = `${p[0]},${p[1]}`
      if (p[0] < 0 || p[0] >= grid.width || p[1] < 0 || p[1] >= grid.height) issues.push({ kind: 'out_of_bounds', point: p })
      if (seen.has(key)) {
        issues.push({ kind: 'duplicate', point: p })
        continue
      }
      seen.add(key)
      if (owner.has(key) && owner.get(key) !== group) issues.push({ kind: 'overlap', point: p })
      else owner.set(key, group)
    }
  })
  return issues
}
