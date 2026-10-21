import { describe, expect, it } from 'vitest'
import { parseGridCommands } from '../commands'

describe('F3 parseGridCommands 指令語法', () => {
  it('解析 MOVE 四個方向、PICK、OPEN', () => {
    const cmds = parseGridCommands('MOVE UP\nMOVE DOWN\nMOVE LEFT\nMOVE RIGHT\nPICK\nOPEN RIGHT\n', false)
    expect(cmds).toEqual([
      { kind: 'move', dir: 'UP', raw: 'MOVE UP', line: 0 },
      { kind: 'move', dir: 'DOWN', raw: 'MOVE DOWN', line: 1 },
      { kind: 'move', dir: 'LEFT', raw: 'MOVE LEFT', line: 2 },
      { kind: 'move', dir: 'RIGHT', raw: 'MOVE RIGHT', line: 3 },
      { kind: 'pick', raw: 'PICK', line: 4 },
      { kind: 'open', dir: 'RIGHT', raw: 'OPEN RIGHT', line: 5 },
    ])
  })

  it('空字串沒有指令', () => {
    expect(parseGridCommands('', false)).toEqual([])
  })

  it('去除行首尾空白、空行忽略、多個空白視為一個', () => {
    const cmds = parseGridCommands('\n   \n  MOVE   DOWN  \n', false)
    expect(cmds).toEqual([{ kind: 'move', dir: 'DOWN', raw: 'MOVE   DOWN', line: 2 }])
  })

  it('CRLF 換行也能解析', () => {
    expect(parseGridCommands('MOVE DOWN\r\nPICK\r\n', false).map((c) => c.kind)).toEqual(['move', 'pick'])
  })

  it('非指令嘗試的行（含小寫、符號、中文、數字）只當一般輸出', () => {
    const cmds = parseGridCommands('debug: x = 1\nmove up\n箱內 2 樣\nMOVE 2\nMove Up\nMOVE DOWN\n', false)
    expect(cmds).toEqual([{ kind: 'move', dir: 'DOWN', raw: 'MOVE DOWN', line: 5 }])
  })

  it('全大寫加底線的行視為指令嘗試', () => {
    expect(parseGridCommands('MOVE_UP\n', false)).toEqual([
      { kind: 'invalid', reason: 'unknown_verb', raw: 'MOVE_UP', line: 0 },
    ])
  })

  it.each([
    ['JUMP', 'unknown_verb'],
    ['SAY HELLO', 'unknown_verb'],
    ['PICK GEM', 'pick_args'],
    ['MOVE', 'bad_direction'],
    ['OPEN', 'bad_direction'],
    ['MOVE FORWARD', 'bad_direction'],
    ['MOVE UP UP', 'bad_direction'],
    ['OPEN LEFT RIGHT', 'bad_direction'],
  ] as const)('不合法的指令嘗試 %s 標為 invalid（%s）', (line, reason) => {
    expect(parseGridCommands(`${line}\n`, false)).toEqual([{ kind: 'invalid', reason, raw: line, line: 0 }])
  })

  it('無效指令也加入序列，不中斷後續解析', () => {
    const cmds = parseGridCommands('MOVE DOWN\nMOVE FORWARD\nMOVE DOWN\n', false)
    expect(cmds.map((c) => c.kind)).toEqual(['move', 'invalid', 'move'])
  })

  it('輸出被截斷時丟棄最後一行（不完整的指令）', () => {
    const cmds = parseGridCommands('MOVE DOWN\nMOVE DOWN\nMOVE RI', true)
    expect(cmds.map((c) => c.raw)).toEqual(['MOVE DOWN', 'MOVE DOWN'])
  })

  it('截斷時最後一行剛好完整也一樣丟棄', () => {
    const cmds = parseGridCommands('MOVE DOWN\nMOVE DOWN', true)
    expect(cmds.map((c) => c.raw)).toEqual(['MOVE DOWN'])
  })

  it('未截斷時最後一行沒有換行仍照常解析', () => {
    expect(parseGridCommands('MOVE DOWN\nPICK', false).map((c) => c.kind)).toEqual(['move', 'pick'])
  })

  it('解析不設步數上限，上限由模擬器處理', () => {
    const stdout = Array(1500).fill('MOVE DOWN').join('\n')
    expect(parseGridCommands(stdout, false)).toHaveLength(1500)
  })
})
