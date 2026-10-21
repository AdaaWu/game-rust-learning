import { describe, expect, it } from 'vitest'
import type { RunStatus } from '../../../api/types'
import { deriveTimeline } from '../derive'
import type { TimelineState } from '../derive'
import { parseTimelineEvents } from '../parse'
import {
  tl30Solution,
  tl31Solution,
  tl31Starter,
  tl33NoDrop,
  tl33Solution,
  tl33Starter,
  tlPanic,
  tlSelfLock,
  tlThreeCycle,
  tlUnjoined,
  tlWaitChain,
} from '../../../test/timelineFixtures'

const eventsOf = (stdout: string) => parseTimelineEvents(stdout, false).events
const at = (stdout: string, step: number, status: RunStatus = 'ok'): TimelineState => deriveTimeline(eventsOf(stdout), step, status)
const last = (stdout: string, status: RunStatus = 'ok') => at(stdout, eventsOf(stdout).length, status)
const laneNames = (s: TimelineState) => s.lanes.map((l) => l.name)
const stateOf = (s: TimelineState, lane: string) => s.lanes.find((l) => l.name === lane)?.state

describe('S2-3 deriveTimeline（ux §11.5、§11.6）', () => {
  describe('泳道', () => {
    it('main 置頂，其餘依第一個事件出現的先後（33 starter）', () => {
      expect(laneNames(at(tl33Starter, 0, 'timeout'))).toEqual(['main', '分身 2', '分身 1', '分身 0'])
    })

    it('main 不是第一個事件時仍置頂（31 starter：main 最後才上鎖）', () => {
      expect(laneNames(last(tl31Starter))).toEqual(['main', '螃蟹 0', '螃蟹 1', '螃蟹 2'])
    })

    it('沒有 main 時照首次出現排序（三條循環）', () => {
      expect(laneNames(at(tlThreeCycle, 0, 'timeout'))).toEqual(['螃蟹 0', '螃蟹 2', '螃蟹 1'])
    })

    it('第 0 步：泳道全部配置好、都是 not_started，沒有區段、箭頭、事件句', () => {
      const s = at(tl33Starter, 0, 'timeout')
      expect(s.lanes).toEqual([
        { name: 'main', state: 'not_started', line: null },
        { name: '分身 2', state: 'not_started', line: null },
        { name: '分身 1', state: 'not_started', line: null },
        { name: '分身 0', state: 'not_started', line: null },
      ])
      expect(s.segments).toEqual([])
      expect(s.messages).toEqual([])
      expect(s.marks).toEqual([])
      expect(s.deadlock).toBeNull()
      expect(s.sentence).toBe('')
    })

    it('not_started → running → ended（30 solution）', () => {
      expect(stateOf(at(tl30Solution, 0), '點名 0')).toBe('not_started')
      expect(stateOf(at(tl30Solution, 1), '點名 0')).toBe('running')
      expect(stateOf(at(tl30Solution, 2), '點名 0')).toBe('ended')
      expect(stateOf(at(tl30Solution, 2), '點名 1')).toBe('not_started')
    })

    it('waiting_lock／waiting_msg 依最後一個事件；其他為 running（33 starter 第 5 步）', () => {
      const s = at(tl33Starter, 5, 'timeout')
      expect(s.lanes.map((l) => [l.name, l.state])).toEqual([
        ['main', 'waiting_msg'],
        ['分身 2', 'running'],
        ['分身 1', 'waiting_lock'],
        ['分身 0', 'not_started'],
      ])
    })

    it('line：該泳道到目前為止最近一個有行號的事件，lock_free、end 不覆蓋', () => {
      const s = at(tl33Starter, 7, 'timeout')
      expect(s.lanes.map((l) => [l.name, l.line])).toEqual([
        ['main', 58],
        ['分身 2', 44],
        ['分身 1', 39],
        ['分身 0', null],
      ])
    })

    it('thread 頭尾空白修剪後歸同一泳道', () => {
      const s = deriveTimeline(parseTimelineEvents('@tl|start|1|甲|\n@tl|lock_wait|2| 甲 |鎖\n@tl|lock_got|2|甲  |鎖\n', false).events, 3, 'ok')
      expect(laneNames(s)).toEqual(['甲'])
      expect(s.segments).toEqual([
        { lane: '甲', kind: 'wait_lock', target: '鎖', from: 2, to: 3, deadlocked: false },
        { lane: '甲', kind: 'hold', target: '鎖', from: 3, to: null, deadlocked: false },
      ])
    })
  })

  describe('區段', () => {
    it('hold：lock_got 開始、同泳道 lock_free 結束（31 starter 螃蟹 0 拿到就放開）', () => {
      const s = last(tl31Starter)
      expect(s.segments.filter((g) => g.kind === 'hold' && g.lane === '螃蟹 0')).toEqual([
        { lane: '螃蟹 0', kind: 'hold', target: '計數器', from: 3, to: 4, deadlocked: false },
      ])
    })

    it('同泳道同時握兩把鎖：兩個 hold 區段重疊（33 solution 分身 0）', () => {
      const s = last(tl33Solution)
      expect(s.segments.filter((g) => g.kind === 'hold' && g.lane === '分身 0')).toEqual([
        { lane: '分身 0', kind: 'hold', target: '窄橋 0', from: 9, to: 17, deadlocked: false },
        { lane: '分身 0', kind: 'hold', target: '窄橋 1', from: 13, to: 18, deadlocked: false },
      ])
    })

    it('wait_lock：lock_wait 開始，到同泳道下一個事件結束（中間隔著別條泳道的事件）', () => {
      const s = last(tl33Solution)
      expect(s.segments.filter((g) => g.kind === 'wait_lock' && g.lane === '分身 1')).toEqual([
        { lane: '分身 1', kind: 'wait_lock', target: '窄橋 0', from: 11, to: 20, deadlocked: false },
        { lane: '分身 1', kind: 'wait_lock', target: '窄橋 1', from: 21, to: 22, deadlocked: false },
      ])
    })

    it('wait_msg：recv_wait 開始，到同泳道 recv 或 recv_closed 結束', () => {
      const s = last(tl33Solution)
      expect(s.segments.filter((g) => g.kind === 'wait_msg')).toEqual([
        { lane: 'main', kind: 'wait_msg', target: '回報', from: 2, to: 4, deadlocked: false },
        { lane: 'main', kind: 'wait_msg', target: '回報', from: 5, to: 15, deadlocked: false },
        { lane: 'main', kind: 'wait_msg', target: '回報', from: 16, to: 24, deadlocked: false },
        { lane: 'main', kind: 'wait_msg', target: '回報', from: 25, to: 29, deadlocked: false },
      ])
    })

    it('只看前 step 個事件：未結束的區段 to = null', () => {
      const open = at(tl33Solution, 11).segments.filter((g) => g.to === null)
      expect(open).toEqual(
        expect.arrayContaining([
          { lane: 'main', kind: 'wait_msg', target: '回報', from: 5, to: null, deadlocked: false },
          { lane: '分身 0', kind: 'hold', target: '窄橋 0', from: 9, to: null, deadlocked: false },
          { lane: '分身 1', kind: 'wait_lock', target: '窄橋 0', from: 11, to: null, deadlocked: false },
        ]),
      )
      expect(open).toHaveLength(3)
    })

    it('沒有對應 lock_got 的 lock_free 略過，不報錯', () => {
      const s = deriveTimeline(parseTimelineEvents('@tl|start|1|甲|\n@tl|lock_free||甲|鎖\n@tl|end||甲|\n', false).events, 3, 'ok')
      expect(s.segments).toEqual([])
      expect(stateOf(s, '甲')).toBe('ended')
    })
  })

  describe('訊息箭頭', () => {
    it('每個 channel 第 k 個 send 對第 k 個 recv（33 solution 三則回報）', () => {
      expect(last(tl33Solution).messages).toEqual([
        { channel: '回報', from: { lane: '分身 2', step: 3 }, to: { lane: 'main', step: 4 } },
        { channel: '回報', from: { lane: '分身 0', step: 14 }, to: { lane: 'main', step: 15 } },
        { channel: '回報', from: { lane: '分身 1', step: 23 }, to: { lane: 'main', step: 24 } },
      ])
    })

    it('還沒被收到的 send：to = null（未送達殘箭）', () => {
      expect(at(tl33Solution, 3).messages).toEqual([{ channel: '回報', from: { lane: '分身 2', step: 3 }, to: null }])
    })

    it('不同 channel 各自配對', () => {
      const stdout = ['@tl|send|1|甲|東', '@tl|send|2|甲|西', '@tl|recv|3|乙|西', '@tl|recv|4|乙|東'].map((l) => `${l}\n`).join('')
      expect(last(stdout).messages).toEqual([
        { channel: '東', from: { lane: '甲', step: 1 }, to: { lane: '乙', step: 4 } },
        { channel: '西', from: { lane: '甲', step: 2 }, to: { lane: '乙', step: 3 } },
      ])
    })

    it('配對結果 recv 在 send 之前（假事件）：不畫箭頭', () => {
      const s = last('@tl|recv|1|乙|東\n@tl|send|2|甲|東\n')
      expect(laneNames(s)).toEqual(['乙', '甲'])
      expect(s.messages).toEqual([])
    })

    it('recv_closed：收方泳道該步一個 recv_closed 標記', () => {
      expect(last(tl33Solution).marks.filter((m) => m.kind === 'recv_closed')).toEqual([{ lane: 'main', step: 29, kind: 'recv_closed', text: '回報' }])
    })
  })

  describe('標記', () => {
    it('start、end、note 依事件順序；note 帶文字（31 starter 小旗子在放開之後）', () => {
      expect(at(tl31Starter, 7).marks).toEqual([
        { lane: '螃蟹 0', step: 1, kind: 'start', text: '' },
        { lane: '螃蟹 1', step: 5, kind: 'start', text: '' },
        { lane: '螃蟹 0', step: 6, kind: 'note', text: '讀到 0' },
        { lane: '螃蟹 0', step: 7, kind: 'end', text: '' },
      ])
    })
  })

  describe('死鎖判定', () => {
    it('兩條循環：循環出現前一步不算死鎖（33 starter 第 13 步）', () => {
      const s = at(tl33Starter, 13, 'timeout')
      expect(s.deadlock).toBeNull()
      expect(stateOf(s, '分身 1')).toBe('waiting_lock')
    })

    it('兩條循環：第 14 步成立，從循環中泳道排序最前者（分身 1）開始', () => {
      const s = at(tl33Starter, 14, 'timeout')
      expect(s.deadlock).toEqual([
        { lane: '分身 1', waits: '窄橋 0', heldBy: '分身 0' },
        { lane: '分身 0', waits: '窄橋 1', heldBy: '分身 1' },
      ])
      expect(s.lanes.map((l) => [l.name, l.state])).toEqual([
        ['main', 'waiting_msg'],
        ['分身 2', 'ended'],
        ['分身 1', 'deadlocked'],
        ['分身 0', 'deadlocked'],
      ])
    })

    it('循環上的 wait_lock 與被等待那把鎖的 hold 標 deadlocked，其餘不標', () => {
      const s = at(tl33Starter, 14, 'timeout')
      expect(s.segments.filter((g) => g.deadlocked)).toEqual(
        expect.arrayContaining([
          { lane: '分身 1', kind: 'hold', target: '窄橋 1', from: 6, to: null, deadlocked: true },
          { lane: '分身 0', kind: 'hold', target: '窄橋 0', from: 11, to: null, deadlocked: true },
          { lane: '分身 1', kind: 'wait_lock', target: '窄橋 0', from: 13, to: null, deadlocked: true },
          { lane: '分身 0', kind: 'wait_lock', target: '窄橋 1', from: 14, to: null, deadlocked: true },
        ]),
      )
      expect(s.segments.filter((g) => g.deadlocked)).toHaveLength(4)
      expect(s.segments).toHaveLength(8)
      // 已結束的舊等待區段不標
      expect(s.segments.find((g) => g.lane === '分身 1' && g.kind === 'wait_lock' && g.target === '窄橋 1')).toEqual({
        lane: '分身 1', kind: 'wait_lock', target: '窄橋 1', from: 5, to: 6, deadlocked: false,
      })
    })

    it('三條循環：最晚進入等待的螃蟹 0 排序最前，deadlock 從牠開始沿等待關係排列', () => {
      expect(at(tlThreeCycle, 11, 'timeout').deadlock).toBeNull()
      expect(at(tlThreeCycle, 12, 'timeout').deadlock).toEqual([
        { lane: '螃蟹 0', waits: '礁石 1', heldBy: '螃蟹 1' },
        { lane: '螃蟹 1', waits: '礁石 2', heldBy: '螃蟹 2' },
        { lane: '螃蟹 2', waits: '礁石 0', heldBy: '螃蟹 0' },
      ])
    })

    it('自己等自己（重複鎖同一把 Mutex）：循環長度 1', () => {
      const s = last(tlSelfLock, 'timeout')
      expect(s.deadlock).toEqual([{ lane: '寄居蟹', waits: '貝殼', heldBy: '寄居蟹' }])
      expect(stateOf(s, '寄居蟹')).toBe('deadlocked')
      expect(s.segments).toEqual(
        expect.arrayContaining([
          { lane: '寄居蟹', kind: 'hold', target: '貝殼', from: 3, to: null, deadlocked: true },
          { lane: '寄居蟹', kind: 'wait_lock', target: '貝殼', from: 4, to: null, deadlocked: true },
        ]),
      )
    })

    it('等待鏈不成環（甲等乙的鎖、乙在等訊息）不算死鎖', () => {
      const stdout = ['@tl|lock_wait|1|乙|窄橋', '@tl|lock_got|1|乙|窄橋', '@tl|recv_wait|2|乙|回報', '@tl|lock_wait|3|甲|窄橋'].map((l) => `${l}\n`).join('')
      const s = last(stdout, 'timeout')
      expect(s.deadlock).toBeNull()
      expect(stateOf(s, '甲')).toBe('waiting_lock')
      expect(stateOf(s, '乙')).toBe('waiting_msg')
      expect(s.segments.some((g) => g.deadlocked)).toBe(false)
    })

    it('等待鏈不成環（實際輸出）：分身 B 等分身 A 的窄橋 0、分身 A 在等訊息', () => {
      const s = last(tlWaitChain, 'timeout')
      expect(s.deadlock).toBeNull()
      expect(stateOf(s, '分身 A')).toBe('waiting_msg')
      expect(stateOf(s, '分身 B')).toBe('waiting_lock')
    })

    it('33 solution：分身 1 等分身 0 的窄橋 0 但不成環，全程沒有死鎖', () => {
      for (let step = 0; step <= 29; step++) expect(at(tl33Solution, step).deadlock).toBeNull()
      expect(stateOf(at(tl33Solution, 12), '分身 1')).toBe('waiting_lock')
    })

    it('循環出現的那一步起維持標紅（之後還有其他泳道的事件）', () => {
      const stdout = tlThreeCycle + '@tl|start|20|旁觀蟹|\n@tl|end||旁觀蟹|\n'
      const s = at(stdout, 14, 'timeout')
      expect(s.deadlock?.map((d) => d.lane)).toEqual(['螃蟹 0', '螃蟹 1', '螃蟹 2'])
      expect(stateOf(s, '旁觀蟹')).toBe('ended')
    })

    it('鎖有多個未結束 hold 時，持有者取最近開始的', () => {
      const stdout = ['@tl|lock_got|1|甲|鎖', '@tl|lock_got|2|乙|鎖', '@tl|lock_wait|3|丙|鎖'].map((l) => `${l}\n`).join('')
      expect(last(stdout, 'timeout').sentence).toBe('第 3 步 · 第 3 行：丙 想拿 鎖（被 乙 佔著）')
    })

    it('判定不看 status：ok 也照實標出死鎖', () => {
      expect(at(tl33Starter, 14, 'ok').deadlock).toHaveLength(2)
    })
  })

  describe('事件句', () => {
    it.each([
      [1, '第 1 步 · 第 58 行：main 等 回報 的訊息'],
      [2, '第 2 步 · 第 36 行：分身 2 開始'],
      [3, '第 3 步 · 第 44 行：分身 2 往 回報 送出一則訊息'],
      [5, '第 5 步 · 第 39 行：分身 1 想拿 窄橋 1'],
      [6, '第 6 步 · 第 39 行：分身 1 拿到 窄橋 1'],
      [7, '第 7 步：分身 2 結束'],
      [9, '第 9 步 · 第 58 行：main 從 回報 收到一則訊息（分身 2 送的）'],
      [13, '第 13 步 · 第 39 行：分身 1 想拿 窄橋 0（被 分身 0 佔著）'],
      [14, '第 14 步 · 第 39 行：分身 0 想拿 窄橋 1（被 分身 1 佔著）'],
    ])('33 starter 第 %i 步', (step, sentence) => {
      expect(at(tl33Starter, step, 'timeout').sentence).toBe(sentence)
    })

    it('lock_free 省略行號', () => {
      expect(at(tl33Solution, 17).sentence).toBe('第 17 步：分身 0 放開 窄橋 0')
    })

    it('recv_closed', () => {
      expect(at(tl33Solution, 29).sentence).toBe('第 29 步 · 第 63 行：回報 已關閉，main 不會再收到訊息')
    })

    it('note', () => {
      expect(at(tl31Starter, 6).sentence).toBe('第 6 步 · 第 7 行：螃蟹 0：讀到 0')
    })

    it('想拿自己佔著的鎖：補「（被自己佔著）」', () => {
      expect(last(tlSelfLock, 'timeout').sentence).toBe('第 4 步 · 第 10 行：寄居蟹 想拿 貝殼（被自己佔著）')
    })
  })

  describe('結論', () => {
    it('ok 全部具名泳道都結束：main 沒有 start／end 不影響（31 starter）', () => {
      expect(last(tl31Starter).conclusion).toBe('全部執行緒都結束了')
    })

    it('ok 全部結束（33 solution、30 solution、31 solution）', () => {
      expect(last(tl33Solution).conclusion).toBe('全部執行緒都結束了')
      expect(last(tl30Solution).conclusion).toBe('全部執行緒都結束了')
      expect(last(tl31Solution).conclusion).toBe('全部執行緒都結束了')
    })

    it('ok 沒有任何具名泳道：同「全部執行緒都結束了」', () => {
      expect(last('@tl|lock_wait|3|main|鎖\n@tl|lock_got|3|main|鎖\n@tl|lock_free||main|鎖\n').conclusion).toBe('全部執行緒都結束了')
    })

    it('ok 有具名泳道沒做完（main 沒 join 就結束）', () => {
      expect(last(tlUnjoined).conclusion).toBe('程式結束時，還有 2 條執行緒沒做完（慢吞蟹 0、慢吞蟹 1）')
    })

    it('ok 但有死鎖循環：改用死鎖結論', () => {
      expect(at(tl33Starter, 14, 'ok').conclusion).toBe('✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）')
    })

    it('timeout 兩條循環：從循環第一條泳道依序列出', () => {
      expect(last(tl33Starter, 'timeout').conclusion).toBe('✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）')
    })

    it('timeout 三條循環', () => {
      expect(last(tlThreeCycle, 'timeout').conclusion).toBe(
        '✗ 死鎖：螃蟹 0 等 礁石 1（被 螃蟹 1 佔著）；螃蟹 1 等 礁石 2（被 螃蟹 2 佔著）；螃蟹 2 等 礁石 0（被 螃蟹 0 佔著）',
      )
    })

    it('timeout 自己等自己：死鎖結論', () => {
      expect(last(tlSelfLock, 'timeout').conclusion).toBe('✗ 死鎖：寄居蟹 等 貝殼（被自己佔著）')
    })

    it('timeout 只有 main 在等訊息（33 沒 drop(tx)）：「時間到：main 等訊息」', () => {
      const s = last(tl33NoDrop, 'timeout')
      expect(s.deadlock).toBeNull()
      expect(stateOf(s, 'main')).toBe('waiting_msg')
      expect(s.conclusion).toBe('時間到：main 等訊息')
    })

    it('timeout 只有具名泳道在等（等待鏈不成環）：只計具名泳道', () => {
      const s = last(tlWaitChain, 'timeout')
      expect(s.deadlock).toBeNull()
      expect(s.conclusion).toBe('時間到：還有 2 條執行緒在等')
    })

    it('timeout 具名泳道與 main 同時在等：兩句以「；」相接', () => {
      expect(at(tl33Solution, 11, 'timeout').conclusion).toBe('時間到：還有 1 條執行緒在等；main 等訊息')
    })

    it('timeout 沒有任何泳道在等：時間到', () => {
      expect(last(tl30Solution, 'timeout').conclusion).toBe('時間到')
    })

    it('runtime_panic：程式在這裡結束（panic）', () => {
      expect(last(tlPanic, 'runtime_panic').conclusion).toBe('程式在這裡結束（panic）')
    })

    it('memory_limit：記憶體不足', () => {
      expect(last(tl31Starter, 'memory_limit').conclusion).toBe('記憶體不足')
    })

    it('output_limit：輸出過長已截斷', () => {
      expect(last(tl31Starter, 'output_limit').conclusion).toBe('輸出過長已截斷')
    })
  })
})
