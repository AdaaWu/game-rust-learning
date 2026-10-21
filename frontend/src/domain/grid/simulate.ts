import type { GridDef, Point } from '../../api/types'
import type { Direction, GridCommand } from './commands'

export const MAX_STEPS = 1000

export type StopReason = 'wall' | 'door' | 'out_of_bounds' | 'invalid' | 'pick_empty' | 'open_nothing' | 'step_limit'

export interface GridState {
  pos: Point
  gemsLeft: Point[]
  openDoors: Point[]
}

// step 為 1 起算；target 為撞到的格子（牆、門、地圖外）
export interface GridStop {
  reason: StopReason
  step: number
  target: Point | null
  command: GridCommand | null
}

// totalSteps：進度條總步數（指令數，上限 1000）
// states[k]：第 k 步後的狀態，states[0] 為初始地圖；遇到停止時只算到停止那一步
export interface SimResult {
  totalSteps: number
  states: GridState[]
  stop: GridStop | null
  cleared: boolean
}

export const DELTAS: Record<Direction, Point> = {
  UP: [0, -1],
  DOWN: [0, 1],
  LEFT: [-1, 0],
  RIGHT: [1, 0],
}

export const samePoint = (a: Point, b: Point) => a[0] === b[0] && a[1] === b[1]
const has = (list: Point[], p: Point) => list.some((q) => samePoint(q, p))

export function initialState(grid: GridDef): GridState {
  return { pos: grid.start, gemsLeft: [...grid.gems], openDoors: [] }
}

function neighbor(pos: Point, dir: Direction): Point {
  const [dx, dy] = DELTAS[dir]
  return [pos[0] + dx, pos[1] + dy]
}

function inBounds(grid: GridDef, p: Point): boolean {
  return p[0] >= 0 && p[0] < grid.width && p[1] >= 0 && p[1] < grid.height
}

type StepOutcome = { state: GridState } | { stop: Omit<GridStop, 'step' | 'command'> }

function apply(grid: GridDef, state: GridState, command: GridCommand): StepOutcome {
  switch (command.kind) {
    case 'invalid':
      return { stop: { reason: 'invalid', target: null } }
    case 'pick':
      if (!has(state.gemsLeft, state.pos)) return { stop: { reason: 'pick_empty', target: null } }
      return { state: { ...state, gemsLeft: state.gemsLeft.filter((g) => !samePoint(g, state.pos)) } }
    case 'open': {
      const target = neighbor(state.pos, command.dir)
      if (!has(grid.doors, target) || has(state.openDoors, target)) return { stop: { reason: 'open_nothing', target: null } }
      return { state: { ...state, openDoors: [...state.openDoors, target] } }
    }
    case 'move': {
      const target = neighbor(state.pos, command.dir)
      if (!inBounds(grid, target)) return { stop: { reason: 'out_of_bounds', target } }
      if (has(grid.walls, target)) return { stop: { reason: 'wall', target } }
      if (has(grid.doors, target) && !has(state.openDoors, target)) return { stop: { reason: 'door', target } }
      return { state: { ...state, pos: target } }
    }
  }
}

export function simulate(grid: GridDef, commands: GridCommand[]): SimResult {
  const totalSteps = Math.min(commands.length, MAX_STEPS)
  const states: GridState[] = [initialState(grid)]
  let stop: GridStop | null = null
  for (let i = 0; i < totalSteps; i++) {
    const current = states[states.length - 1]
    const outcome = apply(grid, current, commands[i])
    if ('stop' in outcome) {
      states.push(current)
      stop = { ...outcome.stop, step: i + 1, command: commands[i] }
      break
    }
    states.push(outcome.state)
  }
  if (stop === null && commands.length > MAX_STEPS) {
    stop = { reason: 'step_limit', step: MAX_STEPS, target: null, command: null }
  }
  const last = states[states.length - 1]
  const cleared = stop === null && samePoint(last.pos, grid.goal) && last.gemsLeft.length === 0
  return { totalSteps, states, stop, cleared }
}
