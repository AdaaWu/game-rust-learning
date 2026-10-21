import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import MemoryView from '../MemoryView.vue'
import type { MemorySnapshot } from '../../api/types'
import { snapshots } from '../../test/fixtures'

function mountMem(props: Partial<{ snapshots: MemorySnapshot[]; codeModified: boolean }> = {}) {
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(MemoryView, {
    props: { snapshots, codeModified: false, ...props },
    global: { plugins: [pinia] },
  })
}

const caption = (w: VueWrapper) => w.get('[data-testid="mem-caption"]').text()
const varEl = (w: VueWrapper, id: string) => w.get(`[data-testid="mem-var"][data-id="${id}"]`)
const pointerEl = (w: VueWrapper, from: string) => w.get(`[data-testid="mem-pointer"][data-from="${from}"]`)

async function seek(w: VueWrapper, step: number) {
  await w.get('input[aria-label="進度"]').setValue(String(step))
}

describe('F5 MemoryView', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('進關即顯示第 1 步並暫停，不自動播放', async () => {
    const w = mountMem()
    expect(caption(w)).toBe('第 1 步 · 第 8 行')
    expect(w.get('[data-testid="step-label"]').text()).toBe('第 1 / 3 步')
    expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    await vi.advanceTimersByTimeAsync(3000)
    expect(caption(w)).toBe('第 1 步 · 第 8 行')
  })

  it('單步前進到下一個快照', async () => {
    const w = mountMem()
    await w.get('button[aria-label="單步前進"]').trigger('click')
    expect(caption(w)).toBe('第 2 步 · 第 3 行')
  })

  it('從頭回到第 1 步', async () => {
    const w = mountMem()
    await seek(w, 3)
    await w.get('button[aria-label="從頭"]').trigger('click')
    expect(caption(w)).toBe('第 1 步 · 第 8 行')
  })

  it('播放到最後一步後按鈕變回播放', async () => {
    const w = mountMem()
    await w.get('button[aria-label="播放"]').trigger('click')
    await vi.advanceTimersByTimeAsync(10_000)
    expect(w.get('[data-testid="step-label"]').text()).toBe('第 3 / 3 步')
    expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
  })

  it('點進度條跳到指定步', async () => {
    const w = mountMem()
    await seek(w, 3)
    expect(caption(w)).toBe('第 3 步 · 第 10 行')
  })

  it('說明列渲染行內 markdown 並過濾 HTML', async () => {
    const xss: MemorySnapshot[] = [{ ...snapshots[0], note: '**粗體** <img src=x onerror="alert(1)">' }]
    const w = mountMem({ snapshots: xss })
    const note = w.get('[data-testid="mem-note"]')
    expect(note.find('strong').text()).toBe('粗體')
    expect(note.html()).not.toContain('onerror')
  })

  it('目前執行中的 frame（最後一個）加粗外框', async () => {
    const w = mountMem()
    await seek(w, 2)
    const frames = w.findAll('[data-testid="mem-frame"]')
    expect(frames.map((f) => f.classes().includes('is-current'))).toEqual([false, true])
  })

  it('已移動變數：網底、刪除線、「已移動」徽章', async () => {
    const w = mountMem()
    await seek(w, 2)
    const bag = varEl(w, 'report.bag')
    expect(bag.classes()).toContain('is-moved')
    expect(bag.text()).toContain('已移動')
  })

  it('已釋放的 heap 區塊顯示「已釋放」', async () => {
    const w = mountMem()
    await seek(w, 3)
    const h = w.get('[data-testid="mem-heap"][data-id="h1"]')
    expect(h.classes()).toContain('is-freed')
    expect(h.text()).toContain('已釋放')
  })

  it('own 指標標籤為 own', () => {
    const w = mountMem()
    const p = pointerEl(w, 'report.bag')
    expect(p.attributes('data-kind')).toBe('own')
    expect(p.get('[data-testid="mem-pointer-label"]').text()).toBe('own')
  })

  it('& 指標標籤含 & 與 👁，被指向變數有 & 徽章', async () => {
    const shared: MemorySnapshot[] = [
      {
        ...snapshots[0],
        frames: [{ id: 'main', fn: 'main', vars: [
          { id: 'main.a', name: 'a', type: 'String', value: null, state: 'live' },
          { id: 'main.r', name: 'r', type: '&String', value: null, state: 'live' },
        ] }],
        heap: [],
        pointers: [{ from: 'main.r', to: 'main.a', kind: 'shared' }],
      },
    ]
    const w = mountMem({ snapshots: shared })
    const label = pointerEl(w, 'main.r').get('[data-testid="mem-pointer-label"]').text()
    expect(label).toContain('&')
    expect(label).toContain('👁')
    expect(varEl(w, 'main.a').classes()).toContain('borrow-shared')
  })

  it('&mut 指標標籤含 &mut 與 ✎，被指向變數顯示 🔒 借出中', () => {
    const mut: MemorySnapshot[] = [
      {
        ...snapshots[0],
        frames: [{ id: 'main', fn: 'main', vars: [
          { id: 'main.a', name: 'a', type: 'String', value: null, state: 'live' },
          { id: 'main.w', name: 'w', type: '&mut String', value: null, state: 'live' },
        ] }],
        heap: [],
        pointers: [{ from: 'main.w', to: 'main.a', kind: 'mut' }],
      },
    ]
    const w = mountMem({ snapshots: mut })
    const label = pointerEl(w, 'main.w').get('[data-testid="mem-pointer-label"]').text()
    expect(label).toContain('&mut')
    expect(label).toContain('✎')
    const a = varEl(w, 'main.a')
    expect(a.classes()).toContain('borrow-mut')
    expect(a.text()).toContain('🔒')
    expect(a.text()).toContain('借出中')
  })

  it('被借用的變數以文字標示借用狀態', () => {
    const borrowed: MemorySnapshot[] = [
      {
        ...snapshots[0],
        frames: [{ id: 'main', fn: 'main', vars: [
          { id: 'main.a', name: 'a', type: 'String', value: null, state: 'live' },
          { id: 'main.b', name: 'b', type: 'String', value: null, state: 'live' },
          { id: 'main.r', name: 'r', type: '&String', value: null, state: 'live' },
          { id: 'main.w', name: 'w', type: '&mut String', value: null, state: 'live' },
        ] }],
        heap: [],
        pointers: [
          { from: 'main.r', to: 'main.a', kind: 'shared' },
          { from: 'main.w', to: 'main.b', kind: 'mut' },
        ],
      },
    ]
    const w = mountMem({ snapshots: borrowed })
    expect(varEl(w, 'main.a').text()).toContain('被共享借用中，只能讀')
    expect(varEl(w, 'main.b').text()).toContain('被獨佔借用中，暫時不能用')
    expect(varEl(w, 'main.r').text()).not.toContain('借用中')
  })

  it('新增的 frame 外框標示變化', async () => {
    const w = mountMem()
    await seek(w, 2)
    expect(w.get('[data-testid="mem-frame"][data-id="count_items"]').classes()).toContain('is-changed')
    expect(w.get('[data-testid="mem-frame"][data-id="report"]').classes()).not.toContain('is-changed')
  })

  it('新增或狀態改變的項目標示變化', async () => {
    const w = mountMem()
    await seek(w, 2)
    expect(varEl(w, 'report.bag').classes()).toContain('is-changed')
    expect(w.get('[data-testid="mem-heap"][data-id="h1"]').classes()).not.toContain('is-changed')
  })

  it('減少動態效果時變化項目改為常駐小圓點「新」', async () => {
    window.matchMedia = vi.fn((query: string) => ({
      matches: query.includes('reduce'),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })) as unknown as typeof window.matchMedia
    const w = mountMem()
    await seek(w, 2)
    expect(varEl(w, 'report.bag').get('[data-testid="mem-new-dot"]').text()).toBe('新')
    expect(w.get('[data-testid="mem-heap"][data-id="h1"]').find('[data-testid="mem-new-dot"]').exists()).toBe(false)
  })

  it('說明列與 note 放進 aria-live 區', () => {
    const w = mountMem()
    expect(w.get('[aria-live="polite"]').text()).toContain('bag 擁有 heap 上的三個字串')
  })

  describe('行號同步', () => {
    it('步數改變時發出對應行號', async () => {
      const w = mountMem()
      await w.get('button[aria-label="單步前進"]').trigger('click')
      expect(w.emitted('line-change')?.at(-1)).toEqual([3])
    })

    it('掛載時發出第 1 步的行號', () => {
      const w = mountMem()
      expect(w.emitted('line-change')?.at(-1)).toEqual([8])
    })

    it('自動播放結束時保留最後一步的行高亮', async () => {
      const w = mountMem()
      await w.get('button[aria-label="播放"]').trigger('click')
      await vi.advanceTimersByTimeAsync(10_000)
      expect(w.emitted('line-change')?.at(-1)).toEqual([10])
      expect(w.emitted('line-change')?.some((e) => e[0] === null)).toBe(false)
    })

    it('卸載（離開關卡）時移除高亮', () => {
      const w = mountMem()
      w.unmount()
      expect(w.emitted('line-change')?.at(-1)).toEqual([null])
    })

    it('goToLine 跳到第一個 line 相同的步驟', async () => {
      const w = mountMem()
      w.vm.goToLine(10)
      await w.vm.$nextTick()
      expect(caption(w)).toBe('第 3 步 · 第 10 行')
    })

    it('goToLine 沒有對應步驟時不動作', async () => {
      const w = mountMem()
      await seek(w, 2)
      w.vm.goToLine(99)
      await w.vm.$nextTick()
      expect(caption(w)).toBe('第 2 步 · 第 3 行')
    })

    it('程式碼已修改時顯示行號僅供參考提示', async () => {
      const w = mountMem()
      expect(w.text()).not.toContain('程式碼已修改，行號高亮僅供參考')
      await w.setProps({ codeModified: true })
      expect(w.text()).toContain('程式碼已修改，行號高亮僅供參考')
    })
  })
})

describe('F5a MemoryView 錯誤標記', () => {
  it('錯誤那一步：target 變數紅框、✗、錯誤碼徽章', async () => {
    const w = mountMem()
    await seek(w, 3)
    const bag = varEl(w, 'report.bag')
    expect(bag.classes()).toContain('is-error')
    expect(bag.get('[data-testid="mem-error-badge"]').text()).toContain('E0382')
    expect(bag.text()).toContain('✗')
  })

  it('錯誤標記與已移動狀態疊加', async () => {
    const w = mountMem()
    await seek(w, 3)
    const bag = varEl(w, 'report.bag')
    expect(bag.classes()).toEqual(expect.arrayContaining(['is-error', 'is-moved']))
    expect(bag.text()).toContain('已移動')
  })

  it('code 為 null 時徽章顯示「執行期錯誤」', () => {
    const s: MemorySnapshot[] = [{ ...snapshots[0], error: { code: null, target: 'report.bag' } }]
    const w = mountMem({ snapshots: s })
    expect(varEl(w, 'report.bag').get('[data-testid="mem-error-badge"]').text()).toContain('執行期錯誤')
  })

  it('target 為 heap 區塊時該區塊紅框', () => {
    const s: MemorySnapshot[] = [{ ...snapshots[0], error: { code: 'E0505', target: 'h1' } }]
    const w = mountMem({ snapshots: s })
    expect(w.get('[data-testid="mem-heap"][data-id="h1"]').classes()).toContain('is-error')
  })

  it('從出事變數出發的箭頭轉紅，標籤不變並加 ✗', () => {
    const s: MemorySnapshot[] = [{ ...snapshots[0], error: { code: 'E0382', target: 'report.bag' } }]
    const w = mountMem({ snapshots: s })
    const p = pointerEl(w, 'report.bag')
    expect(p.classes()).toContain('is-danger')
    expect(p.attributes('data-kind')).toBe('own')
    expect(p.get('[data-testid="mem-pointer-label"]').text()).toBe('own ✗')
  })

  it('轉紅的箭頭沿用原種類的箭頭形狀：& 仍為空心箭頭', () => {
    const s: MemorySnapshot[] = [
      {
        ...snapshots[0],
        frames: [{ id: 'main', fn: 'main', vars: [
          { id: 'main.a', name: 'a', type: 'String', value: null, state: 'live' },
          { id: 'main.r', name: 'r', type: '&String', value: null, state: 'live' },
        ] }],
        heap: [],
        pointers: [{ from: 'main.r', to: 'main.a', kind: 'shared' }],
        error: { code: 'E0502', target: 'main.r' },
      },
    ]
    const w = mountMem({ snapshots: s })
    const marker = pointerEl(w, 'main.r').get('path').attributes('marker-end')
    const id = /#([^)]+)/.exec(marker ?? '')?.[1] ?? ''
    const head = w.get(`marker[id="${id}"] path`)
    expect(head.attributes('fill')).toBe('none')
    expect(head.attributes('stroke')).toBe('var(--danger)')
  })

  it('非錯誤步驟沒有錯誤標記', () => {
    const w = mountMem()
    expect(w.findAll('[data-testid="mem-var"]').length).toBeGreaterThan(0)
    expect(w.findAll('[data-testid="mem-pointer"]').length).toBeGreaterThan(0)
    expect(w.findAll('.is-error')).toHaveLength(0)
    expect(w.findAll('.is-danger')).toHaveLength(0)
    expect(w.find('[data-testid="mem-error-badge"]').exists()).toBe(false)
  })
})
