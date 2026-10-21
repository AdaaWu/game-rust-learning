import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { PLAYBACK_SPEEDS, usePlaybackStore } from '../playback'

describe('F4 usePlaybackStore 播放速度', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('速度選項為 0.5x、1x、2x、4x', () => {
    expect(PLAYBACK_SPEEDS).toEqual([0.5, 1, 2, 4])
  })

  it('預設 1x，每步 500ms', () => {
    const store = usePlaybackStore()
    expect(store.speed).toBe(1)
    expect(store.stepMs).toBe(500)
  })

  it.each([
    [0.5, 1000],
    [2, 250],
    [4, 125],
  ] as const)('%sx 時每步 %i ms', (speed, ms) => {
    const store = usePlaybackStore()
    store.setSpeed(speed)
    expect(store.speed).toBe(speed)
    expect(store.stepMs).toBe(ms)
  })

  it('速度不寫入 localStorage', () => {
    usePlaybackStore().setSpeed(2)
    expect(localStorage.length).toBe(0)
  })
})
