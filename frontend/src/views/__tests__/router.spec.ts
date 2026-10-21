import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory } from 'vue-router'
import { chapters, levels } from '../../test/fixtures'

const api = vi.hoisted(() => ({
  listLevels: vi.fn(),
  getLevel: vi.fn(),
  run: vi.fn(),
  submit: vi.fn(),
  health: vi.fn(),
}))
vi.mock('../../api', () => ({ api }))
// 路由測試不渲染頁面，避免載入 Monaco
vi.mock('../LevelView.vue', () => ({ default: { render: () => null } }))
vi.mock('../LevelMapView.vue', () => ({ default: { render: () => null } }))

import { createAppRouter } from '../../router'
import { useToastStore } from '../../stores/toast'
import { PROGRESS_KEY } from '../../domain/progress'

describe('F7 路由', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
    api.listLevels.mockReset()
    api.listLevels.mockResolvedValue({ ok: true, data: { chapters, levels } })
  })

  it('/ 為章節地圖', async () => {
    const router = createAppRouter(createMemoryHistory())
    await router.push('/')
    expect(router.currentRoute.value.name).toBe('map')
  })

  it('/level/:levelId 為關卡頁', async () => {
    const router = createAppRouter(createMemoryHistory())
    await router.push('/level/01-hello-ferris')
    expect(router.currentRoute.value.name).toBe('level')
    expect(router.currentRoute.value.params.levelId).toBe('01-hello-ferris')
  })

  it('直接輸入鎖定關卡網址：導回地圖並 toast「這關還沒解鎖」', async () => {
    const router = createAppRouter(createMemoryHistory())
    await router.push('/level/02-mut-matters')
    expect(router.currentRoute.value.path).toBe('/')
    expect(useToastStore().messages).toContain('這關還沒解鎖')
  })

  it('已解鎖的關卡可直接進入', async () => {
    localStorage.setItem(
      PROGRESS_KEY,
      JSON.stringify({ version: 1, levels: { '01-hello-ferris': { stars: { pass: true, noHint: false, firstTry: false }, current: { hintsRevealed: 0, failedSubmits: 0 } } } }),
    )
    const router = createAppRouter(createMemoryHistory())
    await router.push('/level/02-mut-matters')
    expect(router.currentRoute.value.path).toBe('/level/02-mut-matters')
  })

  it('不在列表中的關卡放行，由關卡頁顯示找不到這關', async () => {
    const router = createAppRouter(createMemoryHistory())
    await router.push('/level/99-unknown')
    expect(router.currentRoute.value.path).toBe('/level/99-unknown')
  })

  it('關卡列表已快取時不重複請求', async () => {
    const router = createAppRouter(createMemoryHistory())
    await router.push('/level/01-hello-ferris')
    await router.push('/')
    await router.push('/level/01-hello-ferris')
    expect(api.listLevels).toHaveBeenCalledTimes(1)
  })
})
