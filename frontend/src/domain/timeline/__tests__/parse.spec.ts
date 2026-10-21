import { describe, expect, it } from 'vitest'
import { parseTimelineEvents } from '../parse'
import type { TimelineEvent } from '../parse'
import { parseGridCommands } from '../../grid/commands'
import { tl31Starter, tl33Solution, tl33Starter } from '../../../test/timelineFixtures'

const ev = (kind: TimelineEvent['kind'], line: number | null, thread: string, target = ''): TimelineEvent => ({ kind, line, thread, target })

describe('S2-3 parseTimelineEvents（ux §11.4）', () => {
  describe('合法事件', () => {
    it('十種 kind 各一筆，依行序加入', () => {
      const stdout = [
        '@tl|start|36|分身 0|',
        '@tl|lock_wait|41|分身 0|窄橋 0',
        '@tl|lock_got|41|分身 0|窄橋 0',
        '@tl|lock_free||分身 0|窄橋 0',
        '@tl|send|46|分身 0|回報',
        '@tl|end||分身 0|',
        '@tl|recv_wait|61|main|回報',
        '@tl|recv|61|main|回報',
        '@tl|recv_closed|61|main|回報',
        '@tl|note|7|螃蟹 0|讀到 0',
      ].join('\n') + '\n'
      expect(parseTimelineEvents(stdout, false)).toEqual({
        events: [
          ev('start', 36, '分身 0'),
          ev('lock_wait', 41, '分身 0', '窄橋 0'),
          ev('lock_got', 41, '分身 0', '窄橋 0'),
          ev('lock_free', null, '分身 0', '窄橋 0'),
          ev('send', 46, '分身 0', '回報'),
          ev('end', null, '分身 0'),
          ev('recv_wait', 61, 'main', '回報'),
          ev('recv', 61, 'main', '回報'),
          ev('recv_closed', 61, 'main', '回報'),
          ev('note', 7, '螃蟹 0', '讀到 0'),
        ],
        malformed: 0,
        capped: false,
      })
    })

    it('line 空字串為 null', () => {
      expect(parseTimelineEvents('@tl|end||分身 2|\n', false).events).toEqual([ev('end', null, '分身 2')])
    })

    it('33 solution 實際輸出：29 個事件，一般輸出行不進事件', () => {
      const parsed = parseTimelineEvents(tl33Solution, false)
      expect(parsed.events).toHaveLength(29)
      expect(parsed.malformed).toBe(0)
      expect(parsed.capped).toBe(false)
      expect(parsed.events[0]).toEqual(ev('start', 36, '分身 2'))
      expect(parsed.events[28]).toEqual(ev('recv_closed', 63, 'main', '回報'))
      expect(parsed.events.some((e) => e.target.includes('MOVE') || e.thread.includes('指揮官'))).toBe(false)
    })

    it('33 starter 實際輸出（逾時）：14 個事件，最後兩個是互等的 lock_wait', () => {
      const parsed = parseTimelineEvents(tl33Starter, false)
      expect(parsed.events).toHaveLength(14)
      expect(parsed.events.slice(-2)).toEqual([ev('lock_wait', 39, '分身 1', '窄橋 0'), ev('lock_wait', 39, '分身 0', '窄橋 1')])
    })

    it('31 starter 實際輸出：21 個事件，note 文字完整保留', () => {
      const parsed = parseTimelineEvents(tl31Starter, false)
      expect(parsed.events).toHaveLength(21)
      expect(parsed.events.filter((e) => e.kind === 'note').map((e) => e.target)).toEqual(['讀到 0', '讀到 0', '讀到 0'])
    })

    it('thread 去除頭尾空白後存入', () => {
      expect(parseTimelineEvents('@tl|lock_got|5|  分身 0 \t|窄橋 0\n', false).events).toEqual([ev('lock_got', 5, '分身 0', '窄橋 0')])
    })

    it('第 4 段取剩下全部（含 |）', () => {
      expect(parseTimelineEvents('@tl|note|7|螃蟹 0|讀到 0|再讀一次\n', false).events).toEqual([ev('note', 7, '螃蟹 0', '讀到 0|再讀一次')])
    })

    it('start、end 的 target 一律當空字串', () => {
      expect(parseTimelineEvents('@tl|start|3|甲|多餘\n@tl|end||甲|多餘\n', false).events).toEqual([ev('start', 3, '甲'), ev('end', null, '甲')])
    })

    it('CRLF 行尾去掉 \\r', () => {
      expect(parseTimelineEvents('@tl|lock_wait|9|寄居蟹|貝殼\r\n@tl|end||寄居蟹|\r\n', false).events).toEqual([
        ev('lock_wait', 9, '寄居蟹', '貝殼'),
        ev('end', null, '寄居蟹'),
      ])
    })

    it('沒有結尾換行的最後一行照常解析', () => {
      expect(parseTimelineEvents('@tl|start|1|甲|', false).events).toEqual([ev('start', 1, '甲')])
    })
  })

  describe('非事件行', () => {
    it('空字串：沒有事件也沒有格式錯誤', () => {
      expect(parseTimelineEvents('', false)).toEqual({ events: [], malformed: 0, capped: false })
    })

    it('一般輸出不進事件、不算格式錯誤', () => {
      expect(parseTimelineEvents('總共 0 顆寶石\nMOVE DOWN\ntl|start|1|甲|\n@tl|end||甲|\n', false)).toEqual({
        events: [ev('end', null, '甲')],
        malformed: 0,
        capped: false,
      })
    })

    it('行首有空白的 @tl| 不算事件，也不算格式錯誤', () => {
      expect(parseTimelineEvents('  @tl|start|1|甲|\n\t@tl|end||甲|\n@tl|end||乙|\n', false)).toEqual({
        events: [ev('end', null, '乙')],
        malformed: 0,
        capped: false,
      })
    })

    it('同一份 stdout 給 parseGridCommands 不產生任何指令嘗試', () => {
      expect(parseGridCommands(tl33Starter, false)).toEqual([])
      expect(parseGridCommands(tl31Starter, false)).toEqual([])
    })
  })

  describe('格式錯誤只累加 malformed，不中斷解析', () => {
    it.each([
      ['未知 kind', '@tl|jump|1|甲|x'],
      ['段數不足（3 段）', '@tl|start|1|甲'],
      ['只有前綴', '@tl|'],
      ['line 為 0', '@tl|start|0|甲|'],
      ['line 前導 0', '@tl|start|07|甲|'],
      ['line 為負數', '@tl|start|-1|甲|'],
      ['line 非數字', '@tl|start|abc|甲|'],
      ['line 含空白', '@tl|start| 3|甲|'],
      ['thread 為空', '@tl|start|1||'],
      ['thread 只有空白', '@tl|start|1|   |'],
      ['lock_wait 缺 target', '@tl|lock_wait|1|甲|'],
      ['lock_got 缺 target', '@tl|lock_got|1|甲|'],
      ['lock_free 缺 target', '@tl|lock_free||甲|'],
      ['send 缺 target', '@tl|send|1|甲|'],
      ['recv_wait 缺 target', '@tl|recv_wait|1|甲|'],
      ['recv 缺 target', '@tl|recv|1|甲|'],
      ['recv_closed 缺 target', '@tl|recv_closed|1|甲|'],
      ['note 缺 target', '@tl|note|1|甲|'],
      ['kind 大寫', '@tl|START|1|甲|'],
    ])('%s', (_name, bad) => {
      const parsed = parseTimelineEvents(`@tl|start|1|甲|\n${bad}\n@tl|end||甲|\n`, false)
      expect(parsed.malformed).toBe(1)
      expect(parsed.events).toEqual([ev('start', 1, '甲'), ev('end', null, '甲')])
    })

    it('多行格式錯誤逐行累加', () => {
      expect(parseTimelineEvents('@tl|jump|1|甲|x\n@tl|start|0|甲|\n@tl|\n', false).malformed).toBe(3)
    })
  })

  describe('截斷與上限', () => {
    it('truncated 時丟棄最後一行（不完整的事件行）', () => {
      const parsed = parseTimelineEvents('@tl|start|1|甲|\n@tl|lock_wait|2|甲|窄', true)
      expect(parsed.events).toEqual([ev('start', 1, '甲')])
      expect(parsed.malformed).toBe(0)
    })

    it('truncated 且以換行結尾：結尾空字串不算一行，丟掉的是最後一個事件行（與 grid 相同）', () => {
      expect(parseTimelineEvents('@tl|start|1|甲|\n@tl|end||甲|\n', true).events).toEqual([ev('start', 1, '甲')])
    })

    it('未截斷時保留最後一行', () => {
      expect(parseTimelineEvents('@tl|start|1|甲|\n@tl|end||甲|', false).events).toHaveLength(2)
    })

    it('剛好 1,000 個事件：全部保留、capped = false', () => {
      const stdout = Array.from({ length: 1000 }, (_, i) => `@tl|note|${i + 1}|甲|第 ${i + 1} 面旗`).join('\n')
      const parsed = parseTimelineEvents(stdout, false)
      expect(parsed.events).toHaveLength(1000)
      expect(parsed.capped).toBe(false)
    })

    it('第 1,001 個起不再加入、capped = true', () => {
      const stdout = Array.from({ length: 1001 }, (_, i) => `@tl|note|${i + 1}|甲|第 ${i + 1} 面旗`).join('\n')
      const parsed = parseTimelineEvents(stdout, false)
      expect(parsed.events).toHaveLength(1000)
      expect(parsed.events[999].target).toBe('第 1000 面旗')
      expect(parsed.capped).toBe(true)
    })

    it('格式錯誤的行不佔上限名額', () => {
      const good = Array.from({ length: 1000 }, (_, i) => `@tl|note|${i + 1}|甲|旗`)
      const parsed = parseTimelineEvents(['@tl|jump|1|甲|x', ...good].join('\n'), false)
      expect(parsed.events).toHaveLength(1000)
      expect(parsed.capped).toBe(false)
      expect(parsed.malformed).toBe(1)
    })
  })
})
