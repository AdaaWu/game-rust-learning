import type { GridDef, Pos } from './api'

export type Dir = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT'
export type Halt = 'wall' | 'door' | 'out_of_bounds' | 'invalid' | 'pick_empty' | 'open_nothing'

export interface Command {
  kind: 'move' | 'pick' | 'open' | 'invalid'
  dir: Dir | null
  raw: string
  line: number // stdout 第幾行（0 起算）
}

export interface Frame {
  pos: Pos
  facing: 'left' | 'right'
  picked: string[]
  opened: string[]
  event: null | 'move' | 'pick' | 'open' | Halt
  bumpDir: Dir | null
  message: string
  line: number | null
}

const DELTA: Record<Dir, Pos> = { UP: [0, -1], DOWN: [0, 1], LEFT: [-1, 0], RIGHT: [1, 0] }
const DIR_ZH: Record<Dir, string> = { UP: '上', DOWN: '下', LEFT: '左', RIGHT: '右' }
const ATTEMPT = /^[A-Z_]+( +[A-Z_]+)*$/
export const STEP_LIMIT = 200

export const key = (p: Pos) => `${p[0]},${p[1]}`

// 只有整行大寫的才算指令嘗試，其他當成一般輸出
export function parseCommands(stdout: string): Command[] {
  const cmds: Command[] = []
  stdout.split('\n').forEach((rawLine, line) => {
    const raw = rawLine.trim()
    if (!raw || !ATTEMPT.test(raw)) return
    const parts = raw.split(/ +/)
    const [verb, arg, ...rest] = parts
    const isDir = (d?: string): d is Dir => d === 'UP' || d === 'DOWN' || d === 'LEFT' || d === 'RIGHT'
    if ((verb === 'MOVE' || verb === 'OPEN') && isDir(arg) && rest.length === 0) {
      cmds.push({ kind: verb === 'MOVE' ? 'move' : 'open', dir: arg, raw, line })
    } else if (verb === 'PICK' && parts.length === 1) {
      cmds.push({ kind: 'pick', dir: null, raw, line })
    } else {
      cmds.push({ kind: 'invalid', dir: null, raw, line })
    }
  })
  return cmds.slice(0, STEP_LIMIT)
}

export function initialFrame(grid: GridDef): Frame {
  return { pos: grid.start, facing: 'right', picked: [], opened: [], event: null, bumpDir: null, message: '準備出發', line: null }
}

// 由指令推出每一步的畫面；遇到錯誤就停
export function buildFrames(grid: GridDef, cmds: Command[]): Frame[] {
  const frames = [initialFrame(grid)]
  const walls = new Set(grid.walls.map(key))
  const gems = new Set(grid.gems.map(key))
  const doors = new Set(grid.doors.map(key))
  for (let i = 0; i < cmds.length; i++) {
    const c = cmds[i]
    const prev = frames[frames.length - 1]
    const n = i + 1
    const base: Frame = { ...prev, event: null, bumpDir: null, line: c.line }
    if (c.kind === 'invalid') {
      frames.push({ ...base, event: 'invalid', message: `第 ${n} 步：看不懂「${c.raw}」，指令只能是 MOVE/PICK/OPEN 加方向` })
      break
    }
    if (c.kind === 'pick') {
      const k = key(prev.pos)
      if (!gems.has(k) || prev.picked.includes(k)) {
        frames.push({ ...base, event: 'pick_empty', message: `第 ${n} 步：這格沒有寶石` })
        break
      }
      frames.push({ ...base, picked: [...prev.picked, k], event: 'pick', message: `第 ${n} 步：撿到寶石！` })
      continue
    }
    const d = c.dir as Dir
    const target: Pos = [prev.pos[0] + DELTA[d][0], prev.pos[1] + DELTA[d][1]]
    const tk = key(target)
    const facing = d === 'LEFT' ? 'left' : d === 'RIGHT' ? 'right' : prev.facing
    if (c.kind === 'open') {
      if (!doors.has(tk) || prev.opened.includes(tk)) {
        frames.push({ ...base, facing, event: 'open_nothing', message: `第 ${n} 步：那邊沒有門` })
        break
      }
      frames.push({ ...base, facing, opened: [...prev.opened, tk], event: 'open', message: `第 ${n} 步：門打開了` })
      continue
    }
    const out = target[0] < 0 || target[1] < 0 || target[0] >= grid.width || target[1] >= grid.height
    if (out) {
      frames.push({ ...base, facing, event: 'out_of_bounds', bumpDir: d, message: `第 ${n} 步：再往${DIR_ZH[d]}走就掉出地圖了` })
      break
    }
    if (walls.has(tk)) {
      frames.push({ ...base, facing, event: 'wall', bumpDir: d, message: `第 ${n} 步：往${DIR_ZH[d]}撞到岩石了（${target[0]}, ${target[1]}）` })
      break
    }
    if (doors.has(tk) && !prev.opened.includes(tk)) {
      frames.push({ ...base, facing, event: 'door', bumpDir: d, message: `第 ${n} 步：門還關著，先 OPEN ${d}` })
      break
    }
    frames.push({ ...base, pos: target, facing, event: 'move', message: `第 ${n} 步：往${DIR_ZH[d]}走一格` })
  }
  return frames
}

export function isCleared(grid: GridDef, f: Frame): boolean {
  return key(f.pos) === key(grid.goal) && f.picked.length === grid.gems.length
}
