import type { GridDef, RunStatus } from '../../api/types'
import type { Direction } from './commands'
import { samePoint } from './simulate'
import type { GridStop, SimResult } from './simulate'

const DIRECTION_TEXT: Record<Direction, string> = { UP: '上', DOWN: '下', LEFT: '左', RIGHT: '右' }

const TERMINATED_MESSAGE: Partial<Record<RunStatus, string>> = {
  timeout: '時間到',
  memory_limit: '記憶體不足',
  output_limit: '輸出過長已截斷',
  runtime_panic: '程式在這裡結束',
}

// ux.md §4.5 停止原因的訊息列文字
export function stopMessage(stop: GridStop): string {
  const command = stop.command
  switch (stop.reason) {
    case 'wall': {
      const dir = command?.kind === 'move' ? DIRECTION_TEXT[command.dir] : ''
      const [x, y] = stop.target ?? [0, 0]
      return `第 ${stop.step} 步：往${dir}撞到牆了（${x}, ${y}）`
    }
    case 'door':
      return `門還關著，先 OPEN ${command?.kind === 'move' ? command.dir : ''}`
    case 'out_of_bounds':
      return '再走就掉出地圖了'
    case 'pick_empty':
      return '這格沒有寶石'
    case 'open_nothing':
      return '那邊沒有門'
    case 'step_limit':
      return '指令超過 1,000 步，後面的不播放了'
    case 'invalid': {
      if (command?.kind !== 'invalid') return ''
      const prefix = `第 ${stop.step} 步：看不懂 \`${command.raw}\`，`
      if (command.reason === 'unknown_verb') return prefix + '指令只有 MOVE、PICK、OPEN'
      if (command.reason === 'pick_args') return prefix + 'PICK 後面不用接東西'
      return prefix + '方向只能是 UP/DOWN/LEFT/RIGHT'
    }
  }
}

// 指令播完後的訊息列文字；撿完且到終點回 null
export function finalMessage(result: SimResult, grid: GridDef, status: RunStatus): string | null {
  if (result.stop) return stopMessage(result.stop)
  const terminated = TERMINATED_MESSAGE[status]
  if (terminated) return terminated
  if (result.cleared) return null
  const last = result.states[result.states.length - 1]
  if (samePoint(last.pos, grid.goal)) return `到終點了，但還有 ${last.gemsLeft.length} 顆寶石沒撿`
  return '指令跑完了，但 Ferris 還沒到終點'
}

// Ferris 頭上的圖示：💥、❓、…；沒有時回 null
export function ferrisBadge(result: SimResult, status: RunStatus): string | null {
  if (result.stop) {
    const reason = result.stop.reason
    if (reason === 'wall' || reason === 'door' || reason === 'out_of_bounds') return '💥'
    if (reason === 'step_limit') return '…'
    return '❓'
  }
  if (TERMINATED_MESSAGE[status]) return null
  return result.cleared ? null : '…'
}
