import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import PlaybackBar from '../PlaybackBar.vue'
import { usePlaybackStore } from '../../stores/playback'

function mountBar(props: Partial<{ step: number; total: number; playing: boolean; min: number }> = {}) {
  const pinia = createPinia()
  setActivePinia(pinia)
  return mount(PlaybackBar, {
    props: { step: 3, total: 12, playing: false, min: 0, ...props },
    global: { plugins: [pinia] },
  })
}

describe('F4 PlaybackBar', () => {
  it('顯示「第 n / N 步」', () => {
    const w = mountBar()
    expect(w.get('[data-testid="step-label"]').text()).toBe('第 3 / 12 步')
  })

  it('三顆按鈕都有 aria-label', () => {
    const w = mountBar()
    expect(w.find('button[aria-label="從頭"]').exists()).toBe(true)
    expect(w.find('button[aria-label="播放"]').exists()).toBe(true)
    expect(w.find('button[aria-label="單步前進"]').exists()).toBe(true)
  })

  it('播放中按鈕變為「暫停」', () => {
    const w = mountBar({ playing: true })
    expect(w.find('button[aria-label="暫停"]').exists()).toBe(true)
    expect(w.find('button[aria-label="播放"]').exists()).toBe(false)
  })

  it('按鈕發出對應事件', async () => {
    const w = mountBar()
    await w.get('button[aria-label="從頭"]').trigger('click')
    await w.get('button[aria-label="播放"]').trigger('click')
    await w.get('button[aria-label="單步前進"]').trigger('click')
    expect(w.emitted('rewind')).toHaveLength(1)
    expect(w.emitted('toggle')).toHaveLength(1)
    expect(w.emitted('step-forward')).toHaveLength(1)
  })

  it('進度條範圍為 min 到 total，拖動發出 seek', async () => {
    const w = mountBar({ min: 1, total: 8 })
    const range = w.get('input[type="range"][aria-label="進度"]')
    expect(range.attributes('min')).toBe('1')
    expect(range.attributes('max')).toBe('8')
    await range.setValue('5')
    expect(w.emitted('seek')?.at(-1)).toEqual([5])
  })

  it('速度選單有 0.5x、1x、2x、4x，切換寫入 store', async () => {
    const w = mountBar()
    const select = w.get('select[aria-label="速度"]')
    expect(select.findAll('option').map((o) => o.text())).toEqual(['0.5x', '1x', '2x', '4x'])
    expect((select.element as HTMLSelectElement).value).toBe('1')
    await select.setValue('2')
    expect(usePlaybackStore().speed).toBe(2)
  })
})
