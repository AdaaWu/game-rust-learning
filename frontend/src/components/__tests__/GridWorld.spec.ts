import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import GridWorld from '../GridWorld.vue'
import type { GridDef, RunStatus } from '../../api/types'
import { clearRoute, sampleGrid } from '../../test/fixtures'

type Props = { grid: GridDef; stdout: string | null; status: RunStatus | null; truncated: boolean; pending: boolean }

function mountGrid(props: Partial<Props> = {}) {
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(GridWorld, {
    props: { grid: sampleGrid, stdout: null, status: null, truncated: false, pending: false, ...props },
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

function ferrisAt(w: VueWrapper) {
  const f = w.get('[data-testid="ferris"]')
  return [Number(f.attributes('data-x')), Number(f.attributes('data-y'))]
}

const stepLabel = (w: VueWrapper) => w.get('[data-testid="step-label"]').text()
const message = (w: VueWrapper) => w.get('[data-testid="grid-message"]').text()
const badge = (w: VueWrapper) => w.find('[data-testid="ferris-badge"]')

describe('F4 GridWorld', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
    document.body.innerHTML = ''
  })

  describe('初始地圖', () => {
    it('尚未執行時 Ferris 在起點，畫出牆、寶石、門、終點', () => {
      const w = mountGrid()
      expect(ferrisAt(w)).toEqual([0, 0])
      expect(w.findAll('[data-testid="wall"]')).toHaveLength(6)
      expect(w.findAll('[data-testid="gem"]')).toHaveLength(2)
      expect(w.findAll('[data-testid="door"]')).toHaveLength(1)
      expect(w.get('[data-testid="door"]').attributes('data-open')).toBe('false')
      expect(w.findAll('[data-testid="goal"]')).toHaveLength(1)
    })

    it('右上角顯示「💎 0 / 2」', () => {
      expect(mountGrid().get('[data-testid="gem-counter"]').text()).toBe('💎 0 / 2')
    })

    it('用 SVG 繪製', () => {
      expect(mountGrid().find('svg').exists()).toBe(true)
    })

    it('正式環境地圖設定有誤時顯示「此關卡地圖設定有誤」，不 console.error', () => {
      vi.stubEnv('DEV', false)
      vi.stubEnv('PROD', true)
      const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
      try {
        const w = mountGrid({ grid: { ...sampleGrid, goal: [0, 0] } })
        expect(w.text()).toContain('此關卡地圖設定有誤')
        expect(spy).not.toHaveBeenCalled()
      } finally {
        vi.unstubAllEnvs()
      }
    })

    it('地圖設定有誤時顯示錯誤框並 console.error（開發模式）', () => {
      const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const w = mountGrid({ grid: { ...sampleGrid, goal: [0, 0] } })
      expect(w.find('[data-testid="grid-config-error"]').exists()).toBe(true)
      expect(spy).toHaveBeenCalled()
    })
  })

  describe('自動播放與播放控制', () => {
    it('Run ok 後自動從第 1 步播放，每 500ms 前進', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      expect(stepLabel(w)).toBe('第 1 / 20 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
      await vi.advanceTimersByTimeAsync(500)
      expect(stepLabel(w)).toBe('第 2 / 20 步')
    })

    it.each(['runtime_panic', 'timeout', 'memory_limit', 'output_limit'] as RunStatus[])('%s 且有 stdout 時也自動播放', async (status) => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\n', status)
      expect(stepLabel(w)).toBe('第 1 / 2 步')
    })

    it('2x 時每步 250ms', async () => {
      const w = mountGrid()
      await w.get('select[aria-label="速度"]').setValue('2')
      await runWith(w, clearRoute)
      await vi.advanceTimersByTimeAsync(250)
      expect(stepLabel(w)).toBe('第 2 / 20 步')
    })

    it('暫停後單步前進一步', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await w.get('button[aria-label="暫停"]').trigger('click')
      await w.get('button[aria-label="單步前進"]').trigger('click')
      expect(stepLabel(w)).toBe('第 2 / 20 步')
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('從頭回到第 0 步初始地圖並暫停', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await w.get('input[aria-label="進度"]').setValue('10')
      await w.get('button[aria-label="從頭"]').trigger('click')
      expect(stepLabel(w)).toBe('第 0 / 20 步')
      expect(ferrisAt(w)).toEqual([0, 0])
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('播完後按鈕變回播放，再按從第 1 步重播', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await playToEnd(w)
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
      await w.get('button[aria-label="播放"]').trigger('click')
      expect(stepLabel(w)).toBe('第 1 / 20 步')
    })

    it('點進度條跳到第 11 步：直接算出狀態', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await w.get('input[aria-label="進度"]').setValue('11')
      expect(ferrisAt(w)).toEqual([2, 0])
      expect(w.get('[data-testid="gem-counter"]').text()).toBe('💎 1 / 2')
      expect(w.findAll('[data-testid="gem"]').map((g) => [g.attributes('data-x'), g.attributes('data-y')])).toEqual([['4', '4']])
    })

    it('跳到第 16 步門為開啟', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await w.get('input[aria-label="進度"]').setValue('16')
      expect(w.get('[data-testid="door"]').attributes('data-open')).toBe('true')
    })

    it.each([
      [' ', 'play'],
      ['ArrowRight', 'step'],
      ['Home', 'rewind'],
    ])('可視化區焦點時按 %j', async (key, action) => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await w.get('button[aria-label="暫停"]').trigger('click')
      await w.get('input[aria-label="進度"]').setValue('5')
      await w.get('[data-testid="grid-world"]').trigger('keydown', { key })
      if (action === 'play') expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
      if (action === 'step') expect(stepLabel(w)).toBe('第 6 / 20 步')
      if (action === 'rewind') expect(stepLabel(w)).toBe('第 0 / 20 步')
    })

    it('可視化區可取得焦點', () => {
      expect(mountGrid().get('[data-testid="grid-world"]').attributes('tabindex')).toBe('0')
    })
  })

  describe('結束狀態與錯誤', () => {
    it('撿完且到終點：Ferris 開心、旗子揮動，沒有錯誤訊息', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await playToEnd(w)
      expect(ferrisAt(w)).toEqual([5, 4])
      expect(w.get('[data-testid="gem-counter"]').text()).toBe('💎 2 / 2')
      expect(w.get('[data-testid="ferris"]').attributes('data-mood')).toBe('happy')
      expect(w.get('[data-testid="goal"]').attributes('data-waving')).toBe('true')
    })

    it('到終點但寶石沒撿完：沒有開心回饋並顯示剩幾顆', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nOPEN RIGHT\nMOVE RIGHT\nMOVE RIGHT\nPICK\nMOVE RIGHT\n')
      await playToEnd(w)
      expect(ferrisAt(w)).toEqual([5, 4])
      expect(w.get('[data-testid="ferris"]').attributes('data-mood')).not.toBe('happy')
      expect(message(w)).toBe('到終點了，但還有 1 顆寶石沒撿')
      expect(badge(w).text()).toBe('…')
    })

    it('指令播完但沒到終點', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\n')
      await playToEnd(w)
      expect(ferrisAt(w)).toEqual([0, 2])
      expect(message(w)).toBe('指令跑完了，但 Ferris 還沒到終點')
      expect(badge(w).text()).toBe('…')
    })

    it.each([
      ['MOVE RIGHT', '💥', '第 1 步：往右撞到牆了（1, 0）', 'dizzy'],
      ['MOVE UP', '💥', '再走就掉出地圖了', 'dizzy'],
      ['PICK', '❓', '這格沒有寶石', 'confused'],
      ['OPEN DOWN', '❓', '那邊沒有門', 'confused'],
    ])('第一步 %s 出錯：停在第 1 步、總步數 2', async (first, icon, msg, mood) => {
      const w = mountGrid()
      await runWith(w, `${first}\nMOVE DOWN\n`)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 1 / 2 步')
      expect(ferrisAt(w)).toEqual([0, 0])
      expect(badge(w).text()).toBe(icon)
      expect(message(w)).toBe(msg)
      expect(w.get('[data-testid="ferris"]').attributes('data-mood')).toBe(mood)
    })

    it('撞牆時牆格標示被撞到', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE RIGHT\n')
      await playToEnd(w)
      const hit = w.findAll('[data-testid="wall"]').filter((c) => c.attributes('data-hit') === 'true')
      expect(hit.map((c) => [c.attributes('data-x'), c.attributes('data-y')])).toEqual([['1', '0']])
    })

    it('撞到關閉的門：停在第 7 步並提示先開門', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE DOWN\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\nMOVE RIGHT\n')
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 7 / 8 步')
      expect(ferrisAt(w)).toEqual([2, 4])
      expect(w.get('[data-testid="door"]').text()).toContain('🔒')
      expect(message(w)).toContain('門還關著，先 OPEN RIGHT')
    })

    it('無效指令：停在該步、❓、顯示原始行', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE FORWARD\nMOVE DOWN\n')
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 2 / 3 步')
      expect(ferrisAt(w)).toEqual([0, 1])
      expect(badge(w).text()).toBe('❓')
      expect(message(w)).toBe('第 2 步：看不懂 `MOVE FORWARD`，方向只能是 UP/DOWN/LEFT/RIGHT')
    })

    it('非指令行不影響 Ferris、沒有無效指令提示', async () => {
      const w = mountGrid()
      await runWith(w, 'debug: x = 1\nmove up\n\n  MOVE   DOWN\n箱內 2 樣\nMOVE DOWN\n')
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 2 / 2 步')
      expect(ferrisAt(w)).toEqual([0, 2])
      expect(badge(w).text()).not.toBe('❓')
    })

    it('1001 步：總步數 1000、停在第 1000 步、提示超過上限', async () => {
      const w = mountGrid()
      const stdout = Array.from({ length: 1001 }, (_, i) => (i % 2 === 0 ? 'MOVE DOWN' : 'MOVE UP')).join('\n')
      await runWith(w, stdout)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 1000 / 1000 步')
      expect(message(w)).toBe('指令超過 1,000 步，後面的不播放了')
      // 逐步播放 1000 步，高負載下會超過預設 5 秒
    }, 30_000)

    it.each([
      ['timeout', '時間到'],
      ['memory_limit', '記憶體不足'],
    ] as [RunStatus, string][])('%s：播放到最後一個收到的指令並顯示 %s', async (status, msg) => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\n', status)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 2 / 2 步')
      expect(ferrisAt(w)).toEqual([0, 2])
      expect(message(w)).toBe(msg)
    })

    it('output_limit 截斷：丟棄最後一行、顯示輸出過長已截斷', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\nMOVE RI', 'output_limit', true)
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 2 / 2 步')
      expect(message(w)).toBe('輸出過長已截斷')
      expect(badge(w).exists() && badge(w).text() === '❓').toBe(false)
    })

    it('runtime_panic：停在最後一個指令並標示程式在這裡結束', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\n', 'runtime_panic')
      await playToEnd(w)
      expect(stepLabel(w)).toBe('第 1 / 1 步')
      expect(ferrisAt(w)).toEqual([0, 1])
      expect(w.text()).toContain('程式在這裡結束')
    })

    it('compile_error：維持初始地圖並顯示程式還沒編譯成功', async () => {
      const w = mountGrid()
      await runWith(w, null, 'compile_error')
      expect(ferrisAt(w)).toEqual([0, 0])
      expect(w.text()).toContain('程式還沒編譯成功')
    })

    it('編譯階段 memory_limit（stdout 為 null）同 compile_error', async () => {
      const w = mountGrid()
      await runWith(w, null, 'memory_limit')
      expect(w.text()).toContain('程式還沒編譯成功')
    })

    it('訊息列放進 aria-live 區', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE RIGHT\n')
      await playToEnd(w)
      expect(w.get('[aria-live="polite"]').text()).toContain('第 1 步：往右撞到牆了（1, 0）')
    })
  })

  describe('請求中', () => {
    it('從未 Run 就 Submit：pending 結束後維持初始地圖', async () => {
      const w = mountGrid()
      await w.setProps({ pending: true })
      await w.setProps({ pending: false })
      expect(stepLabel(w)).toBe('第 0 / 0 步')
      expect(ferrisAt(w)).toEqual([0, 0])
      expect(w.get('[data-testid="grid-world"]').classes()).not.toContain('is-pending')
    })

    it('pending 時停止播放、回到第 0 步並半透明，總步數保留', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await playToEnd(w)
      await w.setProps({ pending: true })
      expect(stepLabel(w)).toBe('第 0 / 20 步')
      expect(w.get('[data-testid="grid-world"]').classes()).toContain('is-pending')
      expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    })

    it('Run 請求中 stdout 先清為 null：pending 期間仍保留上次總步數，結果回來後重播', async () => {
      const w = mountGrid()
      await runWith(w, clearRoute)
      await playToEnd(w)
      await w.setProps({ pending: true, stdout: null, status: null })
      expect(stepLabel(w)).toBe('第 0 / 20 步')
      expect(ferrisAt(w)).toEqual([0, 0])
      await w.setProps({ pending: false, stdout: clearRoute, status: 'ok' })
      expect(stepLabel(w)).toBe('第 1 / 20 步')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
    })

    it('pending 結束本身不觸發重播（重播只由 Run 結果到達觸發），還原上次 Run 的終點', async () => {
      const w = mountGrid()
      await runWith(w, 'MOVE DOWN\nMOVE DOWN\n')
      await playToEnd(w)
      await w.setProps({ pending: true })
      await w.setProps({ pending: false })
      expect(w.get('[data-testid="grid-world"]').classes()).not.toContain('is-pending')
      expect(w.find('button[aria-label="暫停"]').exists()).toBe(false)
      expect(stepLabel(w)).toBe('第 2 / 2 步')
      expect(ferrisAt(w)).toEqual([0, 2])
    })
  })
})
