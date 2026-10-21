import { describe, expect, it } from 'vitest'
import type { GridDef } from '../../../api/types'
import { parseGridCommands } from '../commands'
import { MAX_STEPS, simulate } from '../simulate'
import { clearRoute, sampleGrid } from '../../../test/fixtures'

function run(stdout: string, grid: GridDef = sampleGrid) {
  return simulate(grid, parseGridCommands(stdout, false))
}

function lines(...cmds: string[]) {
  return cmds.join('\n') + '\n'
}

describe('F3 simulate 移動與通關', () => {
  it('沒有指令時停在起點，總步數 0', () => {
    const r = run('')
    expect(r.totalSteps).toBe(0)
    expect(r.states).toEqual([{ pos: [0, 0], gemsLeft: sampleGrid.gems, openDoors: [] }])
    expect(r.stop).toBeNull()
    expect(r.cleared).toBe(false)
  })

  it('MOVE 方向：UP y−1、DOWN y+1、LEFT x−1、RIGHT x+1', () => {
    const open: GridDef = { width: 3, height: 3, start: [1, 1], goal: [2, 2], walls: [], gems: [], doors: [] }
    expect(run('MOVE UP\n', open).states[1].pos).toEqual([1, 0])
    expect(run('MOVE DOWN\n', open).states[1].pos).toEqual([1, 2])
    expect(run('MOVE LEFT\n', open).states[1].pos).toEqual([0, 1])
    expect(run('MOVE RIGHT\n', open).states[1].pos).toEqual([2, 1])
  })

  it('通關路線：撿完全部寶石並停在終點', () => {
    const r = run(clearRoute)
    expect(r.totalSteps).toBe(20)
    expect(r.states).toHaveLength(21)
    expect(r.stop).toBeNull()
    expect(r.cleared).toBe(true)
    expect(r.states[20]).toEqual({ pos: [5, 4], gemsLeft: [], openDoors: [[3, 4]] })
  })

  it('跳到第 11 步可直接取得該步狀態：撿起 (2, 0) 的寶石', () => {
    const r = run(clearRoute)
    expect(r.states[11].pos).toEqual([2, 0])
    expect(r.states[11].gemsLeft).toEqual([[4, 4]])
  })

  it('第 16 步開門後門為開啟', () => {
    const r = run(clearRoute)
    expect(r.states[15].openDoors).toEqual([])
    expect(r.states[16].openDoors).toEqual([[3, 4]])
  })

  it('到終點但寶石沒撿完不算通關', () => {
    const r = run(lines('MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE RIGHT', 'MOVE RIGHT', 'OPEN RIGHT', 'MOVE RIGHT', 'MOVE RIGHT', 'PICK', 'MOVE RIGHT'))
    expect(r.states[r.states.length - 1].pos).toEqual([5, 4])
    expect(r.cleared).toBe(false)
    expect(r.stop).toBeNull()
  })

  it('沒有寶石的地圖只要到終點就通關', () => {
    const g: GridDef = { width: 2, height: 2, start: [0, 0], goal: [1, 0], walls: [], gems: [], doors: [] }
    expect(run('MOVE RIGHT\n', g).cleared).toBe(true)
  })

  it('中途經過終點但最後不在終點不算通關', () => {
    const g: GridDef = { width: 3, height: 1, start: [0, 0], goal: [1, 0], walls: [], gems: [], doors: [] }
    expect(run(lines('MOVE RIGHT', 'MOVE RIGHT'), g).cleared).toBe(false)
  })

  it('開啟的門可以通過', () => {
    const r = run(lines('MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE RIGHT', 'MOVE RIGHT', 'OPEN RIGHT', 'MOVE RIGHT'))
    expect(r.stop).toBeNull()
    expect(r.states[8].pos).toEqual([3, 4])
  })
})

describe('F3 simulate 錯誤立即停止', () => {
  it('撞牆：停在該步、位置不變、後續指令不執行，總步數仍為指令數', () => {
    const r = run(lines('MOVE RIGHT', 'MOVE DOWN'))
    expect(r.totalSteps).toBe(2)
    expect(r.stop).toMatchObject({ reason: 'wall', step: 1, target: [1, 0] })
    expect(r.states).toHaveLength(2)
    expect(r.states[1].pos).toEqual([0, 0])
    expect(r.cleared).toBe(false)
  })

  it('走出地圖：out_of_bounds，target 為地圖外座標', () => {
    const r = run(lines('MOVE UP', 'MOVE DOWN'))
    expect(r.stop).toMatchObject({ reason: 'out_of_bounds', step: 1, target: [0, -1] })
    expect(r.states[1].pos).toEqual([0, 0])
  })

  it('右邊界外也是 out_of_bounds', () => {
    const g: GridDef = { width: 2, height: 2, start: [1, 1], goal: [0, 0], walls: [], gems: [], doors: [] }
    expect(simulate(g, parseGridCommands('MOVE RIGHT\n', false)).stop).toMatchObject({ reason: 'out_of_bounds', target: [2, 1] })
  })

  it('撞到關閉的門：door，停在第 7 步', () => {
    const r = run(lines('MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE RIGHT', 'MOVE RIGHT', 'MOVE RIGHT', 'MOVE RIGHT'))
    expect(r.stop).toMatchObject({ reason: 'door', step: 7, target: [3, 4] })
    expect(r.states[7].pos).toEqual([2, 4])
    expect(r.totalSteps).toBe(8)
  })

  it('腳下沒寶石卻 PICK：pick_empty', () => {
    const r = run(lines('PICK', 'MOVE DOWN'))
    expect(r.stop).toMatchObject({ reason: 'pick_empty', step: 1 })
  })

  it('同一顆寶石撿第二次：pick_empty', () => {
    const r = run(lines('MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE RIGHT', 'MOVE RIGHT', 'MOVE UP', 'MOVE UP', 'MOVE UP', 'MOVE UP', 'PICK', 'PICK'))
    expect(r.stop).toMatchObject({ reason: 'pick_empty', step: 12 })
  })

  it('該方向沒有門卻 OPEN：open_nothing', () => {
    const r = run(lines('OPEN DOWN', 'MOVE DOWN'))
    expect(r.stop).toMatchObject({ reason: 'open_nothing', step: 1 })
  })

  it('對已開啟的門再 OPEN：open_nothing', () => {
    const r = run(lines('MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE DOWN', 'MOVE RIGHT', 'MOVE RIGHT', 'OPEN RIGHT', 'OPEN RIGHT'))
    expect(r.stop).toMatchObject({ reason: 'open_nothing', step: 8 })
  })

  it('無效指令：invalid，Ferris 不移動', () => {
    const r = run(lines('MOVE DOWN', 'MOVE FORWARD', 'MOVE DOWN'))
    expect(r.stop).toMatchObject({ reason: 'invalid', step: 2, command: { kind: 'invalid', raw: 'MOVE FORWARD' } })
    expect(r.states[2].pos).toEqual([0, 1])
    expect(r.totalSteps).toBe(3)
  })

  it('非指令行不算步數', () => {
    const r = run('debug: x = 1\nmove up\n\n  MOVE   DOWN\n箱內 2 樣\nMOVE DOWN\n')
    expect(r.totalSteps).toBe(2)
    expect(r.stop).toBeNull()
    expect(r.states[2].pos).toEqual([0, 2])
  })
})

describe('F3 simulate 步數上限 1,000', () => {
  function alternating(n: number) {
    return Array.from({ length: n }, (_, i) => (i % 2 === 0 ? 'MOVE DOWN' : 'MOVE UP')).join('\n') + '\n'
  }

  it('上限常數為 1000', () => {
    expect(MAX_STEPS).toBe(1000)
  })

  it('剛好 1000 步：全部播放，沒有 step_limit', () => {
    const r = run(alternating(1000))
    expect(r.totalSteps).toBe(1000)
    expect(r.stop).toBeNull()
    expect(r.states).toHaveLength(1001)
  })

  it('1001 步：只保留前 1000 步，停在第 1000 步，原因 step_limit', () => {
    const r = run(alternating(1001))
    expect(r.totalSteps).toBe(1000)
    expect(r.stop).toMatchObject({ reason: 'step_limit', step: 1000 })
    expect(r.states).toHaveLength(1001)
  })

  it('無效指令也計入步數上限', () => {
    const r = run(alternating(1000) + 'JUMP\n')
    expect(r.totalSteps).toBe(1000)
    expect(r.stop).toMatchObject({ reason: 'step_limit', step: 1000 })
  })

  it('第 1000 步本身是錯誤時以錯誤為準', () => {
    const r = run(alternating(999) + 'JUMP\nMOVE DOWN\n')
    expect(r.stop).toMatchObject({ reason: 'invalid', step: 1000 })
  })

  it('上限前就撞牆時以撞牆為準', () => {
    const r = run('MOVE RIGHT\n' + alternating(2000))
    expect(r.stop).toMatchObject({ reason: 'wall', step: 1 })
    expect(r.totalSteps).toBe(1000)
  })
})
