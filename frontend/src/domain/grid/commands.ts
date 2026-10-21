// ux.md §4.3 指令語法

export type Direction = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT'

export type InvalidReason = 'unknown_verb' | 'bad_direction' | 'pick_args'

// raw：去除頭尾空白後的原始行；line：在 stdout 中的行索引（0 起算）
export type GridCommand =
  | { kind: 'move'; dir: Direction; raw: string; line: number }
  | { kind: 'pick'; raw: string; line: number }
  | { kind: 'open'; dir: Direction; raw: string; line: number }
  | { kind: 'invalid'; reason: InvalidReason; raw: string; line: number }

const COMMAND_ATTEMPT = /^[A-Z_]+( +[A-Z_]+)*$/
const DIRECTIONS: string[] = ['UP', 'DOWN', 'LEFT', 'RIGHT']

function parseLine(raw: string, line: number): GridCommand {
  const [verb, ...args] = raw.split(/ +/)
  if (verb === 'PICK') return args.length === 0 ? { kind: 'pick', raw, line } : { kind: 'invalid', reason: 'pick_args', raw, line }
  if (verb !== 'MOVE' && verb !== 'OPEN') return { kind: 'invalid', reason: 'unknown_verb', raw, line }
  if (args.length !== 1 || !DIRECTIONS.includes(args[0])) return { kind: 'invalid', reason: 'bad_direction', raw, line }
  const dir = args[0] as Direction
  return verb === 'MOVE' ? { kind: 'move', dir, raw, line } : { kind: 'open', dir, raw, line }
}

// 逐行解析 stdout；truncated 時先丟棄最後一行
export function parseGridCommands(stdout: string, truncated: boolean): GridCommand[] {
  const lines = stdout.split(/\r?\n/)
  if (truncated) {
    // 結尾換行產生的空字串不算一行
    if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop()
    lines.pop()
  }
  const commands: GridCommand[] = []
  lines.forEach((text, index) => {
    const raw = text.trim()
    if (raw === '' || !COMMAND_ATTEMPT.test(raw)) return
    commands.push(parseLine(raw, index))
  })
  return commands
}
