import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory } from 'vue-router'
import { chapters, levels } from '../../test/fixtures'
import { PROGRESS_KEY } from '../../domain/progress'

const api = vi.hoisted(() => ({
  listLevels: vi.fn(),
  getLevel: vi.fn(),
  run: vi.fn(),
  submit: vi.fn(),
  health: vi.fn(),
}))
vi.mock('../../api', () => ({ api }))
vi.mock('../LevelView.vue', () => ({ default: { render: () => null } }))

import LevelMapView from '../LevelMapView.vue'
import { createAppRouter } from '../../router'

function seedProgress(passed: Record<string, [boolean, boolean]>) {
  const lv: Record<string, unknown> = {}
  for (const [id, [noHint, firstTry]] of Object.entries(passed)) {
    lv[id] = { stars: { pass: true, noHint, firstTry }, current: { hintsRevealed: 0, failedSubmits: 0 } }
  }
  localStorage.setItem(PROGRESS_KEY, JSON.stringify({ version: 1, levels: lv }))
}

async function mountMap() {
  const pinia = createPinia()
  setActivePinia(pinia)
  const router = createAppRouter(createMemoryHistory())
  await router.push('/')
  const w = mount(LevelMapView, { global: { plugins: [pinia, router] }, attachTo: document.body })
  await flushPromises()
  return { w, router }
}

const node = (w: Awaited<ReturnType<typeof mountMap>>['w'], id: string) => w.get(`[data-testid="level-node"][data-id="${id}"]`)

describe('F7 LevelMapView 章節地圖', () => {
  beforeEach(() => {
    localStorage.clear()
    document.body.innerHTML = ''
    api.listLevels.mockReset()
    api.listLevels.mockResolvedValue({ ok: true, data: { chapters, levels } })
    Element.prototype.scrollIntoView = vi.fn()
  })

  it('依章節分組，標題「第 N 章 章節名稱」', async () => {
    const { w } = await mountMap()
    const titles = w.findAll('[data-testid="chapter-title"]').map((t) => t.text())
    expect(titles).toEqual([
      expect.stringContaining('第 1 章 沙灘甦醒'),
      expect.stringContaining('第 2 章 潮間帶'),
      expect.stringContaining('第 3 章 所有權之海'),
      expect.stringContaining('第 5 章 借用燈塔'),
    ])
  })

  it('章節標題旁顯示累計星數「★ 已得 / 關卡數 × 3」', async () => {
    seedProgress({ '01-hello-ferris': [true, true], '02-mut-matters': [false, false] })
    const { w } = await mountMap()
    const stars = w.findAll('[data-testid="chapter-stars"]').map((s) => s.text())
    expect(stars).toEqual(['★ 4 / 6', '★ 0 / 3', '★ 0 / 6', '★ 0 / 3'])
  })

  it('節點顯示關卡編號、名稱與 kind 圖示', async () => {
    const { w } = await mountMap()
    const n = node(w, '02-mut-matters')
    expect(n.text()).toContain('1-2')
    expect(n.text()).toContain('黏住的腳')
    expect(n.text()).toContain('🩹')
    expect(node(w, '01-hello-ferris').text()).toContain('🔨')
    expect(node(w, '19-lighthouse').text()).toContain('🔍')
    expect(node(w, '19-lighthouse').text()).toContain('5-1')
  })

  it('節點狀態：已通過、可挑戰、鎖定', async () => {
    seedProgress({ '01-hello-ferris': [false, false] })
    const { w } = await mountMap()
    expect(node(w, '01-hello-ferris').attributes('data-state')).toBe('passed')
    expect(node(w, '02-mut-matters').attributes('data-state')).toBe('available')
    expect(node(w, '05-delta').attributes('data-state')).toBe('locked')
  })

  it('已通過節點下方三格星（亮星實心、暗星空心）', async () => {
    seedProgress({ '01-hello-ferris': [true, false] })
    const { w } = await mountMap()
    const icons = node(w, '01-hello-ferris').findAll('[data-testid="star-icon"]').map((s) => s.text())
    expect(icons).toEqual(['★', '★', '☆'])
  })

  it('每個節點前都有路徑（含各章第一關，全部第一關除外），鎖定為虛線、其餘為實線', async () => {
    seedProgress({ '01-hello-ferris': [false, false] })
    const { w } = await mountMap()
    const pathOf = (id: string) => node(w, id).element.parentElement?.querySelector('.trail-path')
    expect(pathOf('01-hello-ferris')).toBeNull()
    expect(pathOf('02-mut-matters')?.classList.contains('is-solid')).toBe(true)
    expect(pathOf('05-delta')?.classList.contains('is-dashed')).toBe(true)
  })

  it('鎖定節點有鎖頭與 aria 標示', async () => {
    const { w } = await mountMap()
    const n = node(w, '05-delta')
    expect(n.text()).toContain('🔒')
  })

  it('Ferris 站在目前所在（可挑戰節點中最前的一個）並自動捲到該處', async () => {
    seedProgress({ '01-hello-ferris': [false, false] })
    const { w } = await mountMap()
    expect(w.findAll('[data-testid="map-ferris"]')).toHaveLength(1)
    expect(node(w, '02-mut-matters').find('[data-testid="map-ferris"]').exists()).toBe(true)
    expect(Element.prototype.scrollIntoView).toHaveBeenCalled()
  })

  it('全部通過時 Ferris 站在最後一關', async () => {
    seedProgress(Object.fromEntries(levels.map((l) => [l.id, [true, true]])) as Record<string, [boolean, boolean]>)
    const { w } = await mountMap()
    expect(node(w, '19-lighthouse').find('[data-testid="map-ferris"]').exists()).toBe(true)
  })

  it('點已解鎖節點進入關卡頁', async () => {
    const { w, router } = await mountMap()
    await node(w, '01-hello-ferris').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/level/01-hello-ferris')
  })

  it('點鎖定節點不跳頁，顯示「先完成〈前一關名稱〉」', async () => {
    const { w, router } = await mountMap()
    await node(w, '02-mut-matters').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/')
    expect(w.get('[role="tooltip"]').text()).toBe('先完成〈第一步〉')
  })

  it('鎖定的章節第 1 關提示前一章第一個未通過的關卡', async () => {
    seedProgress({ '01-hello-ferris': [false, false] })
    const { w } = await mountMap()
    await node(w, '05-delta').trigger('click')
    expect(w.get('[role="tooltip"]').text()).toBe('先完成〈黏住的腳〉')
  })
})
