import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TimelineView from '../TimelineView.vue'
import type { RunStatus } from '../../api/types'
import {
  tl31Starter,
  tl33NoDrop,
  tl33Solution,
  tl33Starter,
  tlPanic,
  tlUnjoined,
} from '../../test/timelineFixtures'

type Props = { stdout: string | null; status: RunStatus | null; truncated: boolean; pending: boolean }

function mountTimeline(props: Partial<Props> = {}) {
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(TimelineView, {
    props: { stdout: null, status: null, truncated: false, pending: false, ...props },
    global: { plugins: [pinia] },
    attachTo: document.body,
  })
}

async function runWith(w: VueWrapper, stdout: string | null, status: RunStatus = 'ok', truncated = false) {
  await w.setProps({ stdout, status, truncated })
}

async function playToEnd(w: VueWrapper) {
  await vi.advanceTimersByTimeAsync(600_000)
  await w.vm.$nextTick()
}

// 元素不存在時以斷言失敗呈現
function must(w: VueWrapper, selector: string) {
  const el = w.find(selector)
  expect(el.exists(), `找不到 ${selector}`).toBe(true)
  return el
}

async function seek(w: VueWrapper, step: number) {
  await must(w, 'input[aria-label="進度"]').setValue(String(step))
}

const root = (w: VueWrapper) => must(w, '[data-testid="timeline-view"]')
const textOf = (w: VueWrapper, id: string) => {
  const el = w.find(`[data-testid="${id}"]`)
  return el.exists() ? el.text() : null
}
const stepLabel = (w: VueWrapper) => textOf(w, 'step-label')
const labels = (w: VueWrapper) => w.findAll('[data-testid="tl-lane-label"]')
const label = (w: VueWrapper, lane: string) => must(w, `[data-testid="tl-lane-label"][data-lane="${lane}"]`)
const segments = (w: VueWrapper) => w.findAll('[data-testid="tl-segment"]')
const lastLine = (w: VueWrapper) => {
  const list = w.emitted<[number | null]>('line-change') ?? []
  return list.length > 0 ? list[list.length - 1][0] : undefined
}
const isDisabledEl = (w: VueWrapper, selector: string) => {
  const el = w.find(selector)
  return el.exists() && el.attributes('disabled') !== undefined
}
const playbackDisabled = (w: VueWrapper) =>
  ['從頭', '播放', '單步前進'].every((name) => isDisabledEl(w, `button[aria-label="${name}"]`)) && isDisabledEl(w, 'input[aria-label="進度"]')

// 33 starter 改成分身 1 卡在第 45 行（其餘取自實際輸出），用來區分「最後一個事件的行」與「循環第一條泳道等待的行」
const tl33StarterSplitLines = tl33Starter.replace('@tl|lock_wait|39|分身 1|窄橋 0', '@tl|lock_wait|45|分身 1|窄橋 0')

describe('S2-3 TimelineView（ux §11）', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
    document.body.innerHTML = ''
  })

  describe('尚未執行', () => {
    it('只顯示圖例與空白軸，中央一行提示按執行', () => {
      const w = mountTimeline()
      expect(w.find('[data-testid="tl-legend"]').exists()).toBe(true)
      expect(textOf(w, 'tl-empty')).toBe('按『執行』，看看每條執行緒怎麼跑')
      expect(labels(w)).toHaveLength(0)
    })

    it('播放控制列 disabled', () => {
      expect(playbackDisabled(mountTimeline())).toBe(true)
    })

    it('圖例下方常駐「每次執行的先後順序可能不同」', () => {
      expect(textOf(mountTimeline(), 'tl-order-note')).toBe('每次執行的先後順序可能不同')
    })

    it('可視化區可取得焦點，用 SVG 繪製', () => {
      const w = mountTimeline()
      expect(root(w).attributes('tabindex')).toBe('0')
      expect(w.find('svg[role="img"]').exists()).toBe(true)
    })
  })

  describe('自動播放與播放控制', () => {
    it('Run ok 後自動從第 1 步播放，每 500ms 前進；N 為合法事件數', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      expect(stepLabel(w)).toBe('第 1 / 29 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
      await vi.advanceTimersByTimeAsync(500)
      expect(stepLabel(w)).toBe('第 2 / 29 步')
    })

    it.each(['runtime_panic', 'timeout', 'memory_limit', 'output_limit'] as RunStatus[])('%s 且有 stdout 時也自動播放', async (status) => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, status)
      expect(stepLabel(w)).toBe('第 1 / 14 步')
    })

    it('泳道一開始全部配置好，還沒輪到的泳道為 not_started', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      expect(labels(w).map((l) => l.attributes('data-lane'))).toEqual(['main', '分身 2', '分身 0', '分身 1'])
      expect(label(w, '分身 0').attributes('data-state')).toBe('not_started')
      expect(label(w, '分身 2').attributes('data-state')).toBe('running')
    })

    it('從頭回到第 0 步：泳道仍在、沒有任何區段', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      await must(w, 'button[aria-label="從頭"]').trigger('click')
      expect(stepLabel(w)).toBe('第 0 / 29 步')
      expect(labels(w)).toHaveLength(4)
      expect(segments(w)).toHaveLength(0)
    })

    it.each([
      [' ', 'play'],
      ['ArrowRight', 'step'],
      ['Home', 'rewind'],
    ])('可視化區焦點時按 %j', async (key, action) => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 5)
      await root(w).trigger('keydown', { key })
      if (action === 'play') expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
      if (action === 'step') expect(stepLabel(w)).toBe('第 6 / 29 步')
      if (action === 'rewind') expect(stepLabel(w)).toBe('第 0 / 29 步')
    })

    it('點事件標記：跳到該步並暫停', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      await must(w, '[data-testid="tl-event"][data-step="7"]').trigger('click')
      expect(stepLabel(w)).toBe('第 7 / 29 步')
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('播放中點尚未播放的事件標記：跳到該步並暫停；未播放的標記淡色顯示、不隱藏', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      const future = must(w, '[data-testid="tl-event"][data-step="13"]')
      expect(future.attributes('opacity')).toBe('0.4')
      expect(future.attributes('visibility')).toBeUndefined()
      expect(must(w, '[data-testid="tl-event"][data-step="1"]').attributes('opacity')).toBeUndefined()
      await future.trigger('click')
      expect(stepLabel(w)).toBe('第 13 / 29 步')
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('每個事件一個標記；只有已播放的事件可用 Tab 走訪', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await seek(w, 5)
      expect(w.findAll('[data-testid="tl-event"]')).toHaveLength(29)
      const focusable = w.findAll('[data-testid="tl-event"][tabindex="0"]').map((e) => Number(e.attributes('data-step')))
      expect(focusable).toEqual([1, 2, 3, 4, 5])
    })

    it('事件標記按 Enter 跳到該步', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await seek(w, 10)
      await must(w, '[data-testid="tl-event"][data-step="3"]').trigger('keydown', { key: 'Enter' })
      expect(stepLabel(w)).toBe('第 3 / 29 步')
    })
  })

  describe('訊息列與結果狀態', () => {
    it('訊息列第一行是目前這一步的事件句，放在 aria-live 區', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 9)
      expect(textOf(w, 'tl-sentence')).toBe('第 9 步 · 第 58 行：main 從 回報 收到一則訊息（分身 2 送的）')
      expect(must(w, '[aria-live="polite"]').text()).toContain('第 9 步 · 第 58 行：main 從 回報 收到一則訊息（分身 2 送的）')
    })

    it('還沒播到最後一步時不顯示結論', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 4)
      expect(textOf(w, 'tl-sentence')).toBe('第 4 步 · 第 63 行：main 從 回報 收到一則訊息（分身 2 送的）')
      expect(textOf(w, 'tl-conclusion') ?? '').toBe('')
    })

    it('ok 全部結束：「全部執行緒都結束了」，aria-live 朗讀結論', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 29 / 29 步')
      expect(textOf(w, 'tl-conclusion')).toBe('全部執行緒都結束了')
      expect(must(w, '[aria-live="polite"]').text()).toContain('全部執行緒都結束了')
    })

    it('ok 有具名泳道沒做完：結論列出名稱，只有那些泳道尾端畫「…」', async () => {
      const w = mountTimeline()
      await runWith(w, tlUnjoined)
      await playToEnd(w)
      expect(textOf(w, 'tl-conclusion')).toBe('程式結束時，還有 2 條執行緒沒做完（慢吞蟹 0、慢吞蟹 1）')
      const tails = w.findAll('[data-testid="tl-lane-unfinished"]')
      expect(tails.map((t) => t.attributes('data-lane'))).toEqual(['慢吞蟹 0', '慢吞蟹 1'])
      expect(tails.every((t) => t.text().includes('…'))).toBe(true)
    })

    it('runtime_panic：結論「程式在這裡結束（panic）」，main 沒有 start 不畫「…」', async () => {
      const w = mountTimeline()
      await runWith(w, tlPanic, 'runtime_panic')
      await playToEnd(w)
      expect(textOf(w, 'tl-conclusion')).toBe('程式在這裡結束（panic）')
      expect(w.findAll('[data-testid="tl-lane-unfinished"]').map((t) => t.attributes('data-lane'))).toEqual(['守衛蟹'])
    })

    it('timeout 只有 main 等訊息且其他具名泳道都結束：「時間到：main 等訊息」，main 標籤旁補「其他執行緒都已結束」', async () => {
      const w = mountTimeline()
      await runWith(w, tl33NoDrop, 'timeout')
      await playToEnd(w)
      expect(textOf(w, 'tl-conclusion')).toBe('時間到：main 等訊息')
      const notes = w.findAll('[data-testid="tl-lane-note"]')
      expect(notes.map((n) => [n.attributes('data-lane'), n.text()])).toEqual([['main', '其他執行緒都已結束']])
    })

    it('compile_error：不畫泳道，中央一行「程式還沒編譯成功」，播放控制列 disabled', async () => {
      const w = mountTimeline()
      await runWith(w, null, 'compile_error')
      expect(textOf(w, 'tl-empty')).toBe('程式還沒編譯成功')
      expect(labels(w)).toHaveLength(0)
      expect(playbackDisabled(w)).toBe(true)
    })

    it('編譯階段 memory_limit（stdout 為 null）：「編譯時用太多記憶體」', async () => {
      const w = mountTimeline()
      await runWith(w, null, 'memory_limit')
      expect(textOf(w, 'tl-empty')).toBe('編譯時用太多記憶體')
      expect(labels(w)).toHaveLength(0)
    })

    it('執行階段 memory_limit：自動播放到最後，結論「記憶體不足」', async () => {
      const w = mountTimeline()
      await runWith(w, tl31Starter, 'memory_limit')
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 21 / 21 步')
      expect(textOf(w, 'tl-conclusion')).toBe('記憶體不足')
    })

    it('output_limit：丟棄最後一行後播放，結論「輸出過長已截斷」', async () => {
      const w = mountTimeline()
      await runWith(w, tl31Starter + '@tl|lock_wa', 'output_limit', true)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 21 / 21 步')
      expect(textOf(w, 'tl-conclusion')).toBe('輸出過長已截斷')
    })

    it('有 execution 但 0 個合法事件：零事件提示（按重設取回），播放控制列 disabled', async () => {
      const w = mountTimeline()
      await runWith(w, '總共 0 顆寶石\n')
      const empty = textOf(w, 'tl-empty') ?? ''
      expect(empty).toContain('這次執行沒有時間軸事件')
      expect(empty).toContain('timeline::')
      expect(empty).toContain('按『重設』可取回小幫手')
      expect(playbackDisabled(w)).toBe(true)
    })

    it('malformed > 0：結論後補「有 n 行時間軸事件格式不對，已略過」', async () => {
      const w = mountTimeline()
      await runWith(w, tl31Starter + '@tl|jump|1|main|x\n@tl|start|0|main|\n')
      await playToEnd(w)
      const conclusion = textOf(w, 'tl-conclusion') ?? ''
      expect(conclusion).toContain('全部執行緒都結束了')
      expect(conclusion).toContain('有 2 行時間軸事件格式不對，已略過')
    })

    it('capped：停在第 1,000 步，結論後補「事件超過 1,000 個，後面的不顯示了」', async () => {
      const w = mountTimeline()
      const stdout = Array.from({ length: 1001 }, (_, i) => `@tl|note|${i + 1}|main|第 ${i + 1} 面旗`).join('\n') + '\n'
      await runWith(w, stdout)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 1000 / 1000 步')
      expect(textOf(w, 'tl-conclusion')).toContain('事件超過 1,000 個，後面的不顯示了')
    })
  })

  describe('圖形', () => {
    it('圖例第二行：▬ 持有、┄ 等待、➝ 訊息', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      const legend = textOf(w, 'tl-legend') ?? ''
      expect(legend).toContain('▬ 持有')
      expect(legend).toContain('┄ 等待')
      expect(legend).toContain('➝ 訊息')
    })

    it('刻度每 5 步一個：14 個事件標 1、5、10', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      expect(w.findAll('[data-testid="tl-tick"]').map((t) => t.text())).toEqual(['1', '5', '10'])
    })

    it('圖例：鎖依第一次出現編號 ①②，channel 標 ✉', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      const locks = w.findAll('[data-testid="tl-legend-lock"]').map((l) => l.text().replace(/\s+/g, ''))
      expect(locks).toEqual(['①窄橋1', '②窄橋0'])
      const channels = w.findAll('[data-testid="tl-legend-channel"]').map((l) => l.text().replace(/\s+/g, ''))
      expect(channels).toEqual(['✉回報'])
    })

    it('SVG aria-label 摘要：n 條執行緒、m 把鎖、k 個 channel，第 s / N 步', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      expect(must(w, 'svg[role="img"]').attributes('aria-label')).toBe('4 條執行緒、2 把鎖、1 個 channel，第 14 / 14 步')
    })

    it('持有區段填該鎖的顏色，第 5 把鎖起循環使用 --tl-lock-1', async () => {
      const w = mountTimeline()
      const stdout = [1, 2, 3, 4, 5].map((n) => `@tl|lock_got|${n}|甲|鎖 ${n}\n`).join('')
      await runWith(w, stdout)
      await playToEnd(w)
      const hold = (target: string) => must(w, `[data-testid="tl-segment"][data-kind="hold"][data-target="${target}"]`).html()
      expect(hold('鎖 1')).toContain('var(--tl-lock-1)')
      expect(hold('鎖 4')).toContain('var(--tl-lock-4)')
      expect(hold('鎖 5')).toContain('var(--tl-lock-1)')
    })

    it('同泳道同時握兩把鎖：區段往下疊一層', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 14)
      const holds = w.findAll('[data-testid="tl-segment"][data-lane="分身 0"][data-kind="hold"]')
      expect(holds.map((h) => h.attributes('data-target'))).toEqual(['窄橋 0', '窄橋 1'])
      expect(new Set(holds.map((h) => h.attributes('data-layer'))).size).toBe(2)
    })

    it('等鎖、等訊息區段各自標出（33 starter 第 13 步）', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 13)
      const kinds = segments(w).map((s) => `${s.attributes('data-lane')}:${s.attributes('data-kind')}:${s.attributes('data-target')}`)
      expect(kinds).toEqual(expect.arrayContaining(['分身 1:wait_lock:窄橋 0', 'main:wait_msg:回報', '分身 0:hold:窄橋 0']))
      expect(segments(w)).toHaveLength(7)
    })

    it('訊息箭頭：33 solution 播完 3 條，都已送達', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      const arrows = w.findAll('[data-testid="tl-message"]')
      expect(arrows).toHaveLength(3)
      expect(arrows.every((a) => a.attributes('data-pending') === 'false')).toBe(true)
    })

    it('還沒被收到的訊息畫殘箭，尾端 ✉', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 3)
      const arrows = w.findAll('[data-testid="tl-message"]')
      expect(arrows).toHaveLength(1)
      expect(arrows[0].attributes('data-pending')).toBe('true')
      expect(arrows[0].text()).toContain('✉')
    })

    it('事件標記圖示：開始 ●、結束 ■、channel 關閉 ⊘、標註 🚩（文字放 tooltip）', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      const ev = (step: number) => must(w, `[data-testid="tl-event"][data-step="${step}"]`)
      expect(ev(1).attributes('data-kind')).toBe('start')
      expect(ev(1).text()).toContain('●')
      expect(ev(6).text()).toContain('■')
      expect(ev(29).text()).toContain('⊘')

      await runWith(w, tl31Starter)
      await playToEnd(w)
      const note = must(w, '[data-testid="tl-event"][data-step="6"]')
      expect(note.text()).toContain('🚩')
      expect(note.html()).toContain('讀到 0')
    })
  })

  describe('死鎖（不只靠顏色）', () => {
    it('第 13 步還沒有死鎖標記', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 13)
      expect(w.findAll('[data-testid="tl-lane-label"].is-deadlocked')).toHaveLength(0)
      expect(w.findAll('[data-testid="tl-deadlock-arrow"]')).toHaveLength(0)
    })

    it('循環上的泳道標籤：✗＋「死鎖」字樣＋danger 類別，aria-label 含名稱、狀態、等哪把鎖', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      for (const [lane, waits] of [
        ['分身 1', '窄橋 0'],
        ['分身 0', '窄橋 1'],
      ]) {
        const l = label(w, lane)
        expect(l.element.tagName).toBe('BUTTON')
        expect(l.attributes('data-state')).toBe('deadlocked')
        expect(l.text()).toContain('✗')
        expect(l.text()).toContain('死鎖')
        expect(l.classes()).toContain('is-deadlocked')
        expect(l.attributes('aria-label')).toBe(`${lane}，死鎖，等${waits}`)
      }
      expect(label(w, 'main').classes()).not.toContain('is-deadlocked')
      expect(label(w, '分身 2').attributes('data-state')).toBe('ended')
    })

    it('deadlocked 區段：外框類別＋條尾 ✗，共 4 段；另畫 2 條死鎖箭頭', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      const bad = w.findAll('[data-testid="tl-segment"][data-deadlocked="true"]')
      expect(bad).toHaveLength(4)
      expect(must(w, '[data-testid="tl-segment"][data-lane="分身 1"][data-kind="wait_lock"][data-target="窄橋 1"]').attributes('data-deadlocked')).toBe('false')
      expect(bad.every((s) => s.classes().includes('is-deadlocked') && s.text().includes('✗'))).toBe(true)
      expect(w.findAll('[data-testid="tl-deadlock-arrow"]')).toHaveLength(2)
    })

    it('結論：從循環第一條泳道開始列出', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      expect(textOf(w, 'tl-conclusion')).toBe('✗ 死鎖：分身 1 等 窄橋 0（被 分身 0 佔著）；分身 0 等 窄橋 1（被 分身 1 佔著）')
    })
  })

  describe('line-change（ux §11.8）', () => {
    it('播放到哪一步就發該事件的行號', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      expect(lastLine(w)).toBe(58)
      await vi.advanceTimersByTimeAsync(500)
      expect(lastLine(w)).toBe(36)
      await vi.advanceTimersByTimeAsync(500)
      expect(lastLine(w)).toBe(44)
    })

    it('line = null（lock_free、end）時沿用前一個有行號事件的行', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await must(w, 'button[aria-label="暫停"]').trigger('click')
      await seek(w, 19)
      expect(lastLine(w)).toBe(63)
      await seek(w, 14)
      expect(lastLine(w)).toBe(47)
      await seek(w, 6)
      expect(lastLine(w)).toBe(63)
    })

    it('死鎖的最後一步：高亮循環第一條泳道等待的那一行', async () => {
      const w = mountTimeline()
      await runWith(w, tl33StarterSplitLines, 'timeout')
      await playToEnd(w)
      expect(lastLine(w)).toBe(45)
    })

    it('點泳道標籤：發該泳道到目前這一步最近一個有行號的事件，不改變步數', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      await label(w, '分身 2').trigger('click')
      expect(lastLine(w)).toBe(44)
      await label(w, 'main').trigger('click')
      expect(lastLine(w)).toBe(58)
      expect(stepLabel(w)).toBe('第 14 / 14 步')
    })
  })

  describe('請求中', () => {
    it('pending 時停止播放、回到第 0 步並半透明，總步數保留', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      await w.setProps({ pending: true })
      expect(stepLabel(w)).toBe('第 0 / 29 步')
      expect(root(w).classes()).toContain('is-pending')
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('Submit 回來或請求失敗（stdout 不變）：還原上次 Run 播放結束的那一步，不重播', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      await w.setProps({ pending: true })
      await w.setProps({ pending: false })
      expect(root(w).classes()).not.toContain('is-pending')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(false)
      expect(stepLabel(w)).toBe('第 14 / 14 步')
      expect(textOf(w, 'tl-conclusion')).toContain('✗ 死鎖')
    })

    it('Run 請求中 stdout 先清為 null：pending 期間保留總步數，新結果回來後重播', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Solution)
      await playToEnd(w)
      await w.setProps({ pending: true, stdout: null, status: null })
      expect(stepLabel(w)).toBe('第 0 / 29 步')
      await w.setProps({ pending: false, stdout: tl33Starter, status: 'timeout' })
      expect(stepLabel(w)).toBe('第 1 / 14 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
    })

    it('Run 請求失敗（stdout 先清為 null 再回到原值）：還原上次 Run 播放結束的那一步，不重播', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      await w.setProps({ pending: true, stdout: null, status: null })
      await w.setProps({ pending: false, stdout: tl33Starter, status: 'timeout' })
      expect(stepLabel(w)).toBe('第 14 / 14 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(false)
      expect(textOf(w, 'tl-conclusion')).toContain('✗ 死鎖')
    })

    it('從未 Run 就 Submit：pending 結束後回到空白狀態', async () => {
      const w = mountTimeline()
      await w.setProps({ pending: true })
      await w.setProps({ pending: false })
      expect(textOf(w, 'tl-empty')).toBe('按『執行』，看看每條執行緒怎麼跑')
      expect(root(w).classes()).not.toContain('is-pending')
    })
  })

  describe('窄寬度（容器 < 360px）', () => {
    let resize: ((entries: { contentRect: { width: number } }[]) => void) | null = null

    beforeEach(() => {
      resize = null
      vi.stubGlobal(
        'ResizeObserver',
        class {
          constructor(cb: (entries: { contentRect: { width: number } }[]) => void) {
            resize = cb
          }
          observe() {}
          unobserve() {}
          disconnect() {}
        },
      )
    })

    async function resizeTo(w: VueWrapper, width: number) {
      resize?.([{ contentRect: { width } }])
      await w.vm.$nextTick()
    }

    it('寬 320px：進入窄版，泳道完整名稱放 title 與 aria-label', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      await resizeTo(w, 320)
      expect(root(w).classes()).toContain('is-narrow')
      expect(label(w, '分身 0').attributes('title')).toBe('分身 0')
      expect(label(w, '分身 0').attributes('aria-label')).toContain('分身 0')
    })

    it('窄版：「先後可能不同」小字移到訊息列下方', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await playToEnd(w)
      await resizeTo(w, 320)
      const note = w.find('[data-testid="tl-order-note"]')
      const conclusion = w.find('[data-testid="tl-conclusion"]')
      expect(note.exists() && conclusion.exists()).toBe(true)
      expect(conclusion.element.compareDocumentPosition(note.element) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
      expect(w.find('[data-testid="tl-legend"] [data-testid="tl-order-note"]').exists()).toBe(false)
    })

    it('寬 400px：不是窄版', async () => {
      const w = mountTimeline()
      await runWith(w, tl33Starter, 'timeout')
      await resizeTo(w, 400)
      expect(root(w).classes()).not.toContain('is-narrow')
    })
  })
})
