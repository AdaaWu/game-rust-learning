import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { chapters, gridLevel, levels } from '../../test/fixtures'

const api = vi.hoisted(() => ({
  listLevels: vi.fn(),
  getLevel: vi.fn(),
  run: vi.fn(),
  submit: vi.fn(),
  health: vi.fn(),
}))
vi.mock('../../api', () => ({ api }))

import { useLevelStore } from '../level'

describe('F1 useLevelStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    api.listLevels.mockReset()
    api.getLevel.mockReset()
  })

  it('fetchList 存下章節與關卡', async () => {
    api.listLevels.mockResolvedValue({ ok: true, data: { chapters, levels } })
    const store = useLevelStore()
    await store.fetchList()
    expect(store.chapters).toEqual(chapters)
    expect(store.summaries).toEqual(levels)
  })

  it('fetchList 已快取時不再請求', async () => {
    api.listLevels.mockResolvedValue({ ok: true, data: { chapters, levels } })
    const store = useLevelStore()
    await store.fetchList()
    await store.fetchList()
    expect(api.listLevels).toHaveBeenCalledTimes(1)
  })

  it('fetchList 失敗時不快取，下次再請求', async () => {
    api.listLevels
      .mockResolvedValueOnce({ ok: false, failure: { kind: 'network_error', status: null, code: null, message: null } })
      .mockResolvedValueOnce({ ok: true, data: { chapters, levels } })
    const store = useLevelStore()
    const first = await store.fetchList()
    expect(first.ok).toBe(false)
    await store.fetchList()
    expect(api.listLevels).toHaveBeenCalledTimes(2)
    expect(store.summaries).toEqual(levels)
  })

  it('fetchDetail 依 id 快取', async () => {
    const level = gridLevel()
    api.getLevel.mockResolvedValue({ ok: true, data: level })
    const store = useLevelStore()
    const first = await store.fetchDetail(level.id)
    const second = await store.fetchDetail(level.id)
    expect(first).toEqual({ ok: true, data: level })
    expect(second).toEqual({ ok: true, data: level })
    expect(api.getLevel).toHaveBeenCalledTimes(1)
    expect(store.details.get(level.id)).toEqual(level)
  })

  it('fetchDetail 404 時回 level_not_found 且不快取', async () => {
    api.getLevel.mockResolvedValue({ ok: false, failure: { kind: 'level_not_found' } })
    const store = useLevelStore()
    const result = await store.fetchDetail('99-unknown')
    expect(result).toEqual({ ok: false, failure: { kind: 'level_not_found' } })
    expect(store.details.has('99-unknown')).toBe(false)
    expect(api.getLevel).toHaveBeenCalledTimes(1)
  })
})
