import { describe, expect, it } from 'vitest'
import type { GridDef, RunStatus } from '../../../api/types'
import { parseGridCommands } from '../commands'
import { ferrisBadge, finalMessage, stopMessage } from '../messages'
import { simulate } from '../simulate'
import { clearRoute, sampleGrid } from '../../../test/fixtures'

function sim(stdout: string, grid: GridDef = sampleGrid) {
  return simulate(grid, parseGridCommands(stdout, false))
}

function stopOf(stdout: string, grid: GridDef = sampleGrid) {
  const r = sim(stdout, grid)
  if (!r.stop) throw new Error('預期要停止')
  return r.stop
}

const open3: GridDef = { width: 3, height: 3, start: [1, 1], goal: [0, 0], walls: [[1, 0], [1, 2], [0, 1], [2, 1]], gems: [], doors: [] }

describe('F3 stopMessage 錯誤訊息', () => {
  it('撞牆：第 n 步＋方向＋牆座標', () => {
    expect(stopMessage(stopOf('MOVE RIGHT\nMOVE DOWN\n'))).toBe('第 1 步：往右撞到牆了（1, 0）')
  })

  it.each([
    ['MOVE UP', '第 1 步：往上撞到牆了（1, 0）'],
    ['MOVE DOWN', '第 1 步：往下撞到牆了（1, 2）'],
    ['MOVE LEFT', '第 1 步：往左撞到牆了（0, 1）'],
    ['MOVE RIGHT', '第 1 步：往右撞到牆了（2, 1）'],
  ])('撞牆方向文字 %s', (cmd, msg) => {
    expect(stopMessage(stopOf(cmd + '\n', open3))).toBe(msg)
  })

  it('撞到關閉的門：提示先 OPEN 該方向', () => {
    const stop = stopOf('MOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\n')
    expect(stopMessage(stop)).toContain('門還關著，先 OPEN RIGHT')
  })

  it('往下撞到關閉的門：提示先 OPEN DOWN', () => {
    const g: GridDef = { width: 2, height: 3, start: [0, 0], goal: [1, 2], walls: [], gems: [], doors: [[0, 1]] }
    expect(stopMessage(stopOf('MOVE DOWN\n', g))).toContain('門還關著，先 OPEN DOWN')
  })

  it('走出地圖', () => {
    expect(stopMessage(stopOf('MOVE UP\n'))).toBe('再走就掉出地圖了')
  })

  it('PICK 空格', () => {
    expect(stopMessage(stopOf('PICK\n'))).toBe('這格沒有寶石')
  })

  it('OPEN 沒有門', () => {
    expect(stopMessage(stopOf('OPEN DOWN\n'))).toBe('那邊沒有門')
  })

  it('無效指令：動詞不明', () => {
    expect(stopMessage(stopOf('JUMP\n'))).toBe('第 1 步：看不懂 `JUMP`，指令只有 MOVE、PICK、OPEN')
  })

  it('無效指令：方向不對，顯示去除頭尾空白後的原始行', () => {
    expect(stopMessage(stopOf('MOVE DOWN\n  MOVE FORWARD  \n'))).toBe(
      '第 2 步：看不懂 `MOVE FORWARD`，方向只能是 UP/DOWN/LEFT/RIGHT',
    )
  })

  it('無效指令：MOVE 沒接方向', () => {
    expect(stopMessage(stopOf('MOVE\n'))).toBe('第 1 步：看不懂 `MOVE`，方向只能是 UP/DOWN/LEFT/RIGHT')
  })

  it('無效指令：接了兩個以上的字', () => {
    expect(stopMessage(stopOf('MOVE UP UP\n'))).toBe('第 1 步：看不懂 `MOVE UP UP`，方向只能是 UP/DOWN/LEFT/RIGHT')
  })

  it('無效指令：PICK 多接東西', () => {
    expect(stopMessage(stopOf('PICK GEM\n'))).toBe('第 1 步：看不懂 `PICK GEM`，PICK 後面不用接東西')
  })

  it('超過步數上限', () => {
    const stdout = Array(1001).fill('MOVE DOWN\nMOVE UP').join('\n')
    expect(stopMessage(stopOf(stdout))).toBe('指令超過 1,000 步，後面的不播放了')
  })
})

describe('F3 finalMessage 播完後的訊息', () => {
  it('撿完且到終點：沒有訊息', () => {
    expect(finalMessage(sim(clearRoute), sampleGrid, 'ok')).toBeNull()
  })

  it('指令跑完但沒到終點', () => {
    expect(finalMessage(sim('MOVE DOWN\nMOVE DOWN\n'), sampleGrid, 'ok')).toBe('指令跑完了，但 Ferris 還沒到終點')
  })

  it('到終點但寶石沒撿完：顯示剩幾顆', () => {
    const r = sim('MOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nOPEN RIGHT\nMOVE RIGHT\nMOVE RIGHT\nPICK\nMOVE RIGHT\n')
    expect(finalMessage(r, sampleGrid, 'ok')).toBe('到終點了，但還有 1 顆寶石沒撿')
  })

  it('沒到終點也沒撿完：用「還沒到終點」', () => {
    expect(finalMessage(sim('MOVE DOWN\n'), sampleGrid, 'ok')).toBe('指令跑完了，但 Ferris 還沒到終點')
  })

  it('有停止原因時以停止訊息為準', () => {
    expect(finalMessage(sim('MOVE RIGHT\n'), sampleGrid, 'ok')).toBe('第 1 步：往右撞到牆了（1, 0）')
  })

  it.each([
    ['timeout', '時間到'],
    ['memory_limit', '記憶體不足'],
    ['output_limit', '輸出過長已截斷'],
    ['runtime_panic', '程式在這裡結束'],
  ] as [RunStatus, string][])('程式被終止（%s）時顯示 %s', (status, msg) => {
    expect(finalMessage(sim('MOVE DOWN\nMOVE DOWN\n'), sampleGrid, status)).toBe(msg)
  })

  it('被終止但先撞牆時以撞牆為準', () => {
    expect(finalMessage(sim('MOVE RIGHT\n'), sampleGrid, 'timeout')).toBe('第 1 步：往右撞到牆了（1, 0）')
  })
})

describe('F3 ferrisBadge 頭上圖示', () => {
  it.each([
    ['MOVE RIGHT\n', '💥'],
    ['MOVE UP\n', '💥'],
    ['MOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\n', '💥'],
    ['JUMP\n', '❓'],
    ['PICK\n', '❓'],
    ['OPEN DOWN\n', '❓'],
    ['MOVE DOWN\n', '…'],
  ])('%j → %s', (stdout, badge) => {
    expect(ferrisBadge(sim(stdout), 'ok')).toBe(badge)
  })

  it('超過步數上限顯示 …', () => {
    const stdout = Array(1001).fill('MOVE DOWN\nMOVE UP').join('\n')
    expect(ferrisBadge(sim(stdout), 'ok')).toBe('…')
  })

  it('撿完且到終點沒有圖示', () => {
    expect(ferrisBadge(sim(clearRoute), 'ok')).toBeNull()
  })

  it('程式被終止時沒有圖示', () => {
    expect(ferrisBadge(sim('MOVE DOWN\n'), 'timeout')).toBeNull()
  })
})
