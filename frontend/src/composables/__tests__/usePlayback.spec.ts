import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePlayback } from '../usePlayback'

function make(total = 5, end = total, rewindTo = 0) {
  return usePlayback({ total: () => total, end: () => end, rewindTo, stepMs: () => 500 })
}

describe('F4 usePlayback 播放控制', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('初始停在「從頭」位置、暫停', () => {
    const p = make()
    expect(p.step.value).toBe(0)
    expect(p.playing.value).toBe(false)
  })

  it('MemoryView 的從頭位置是第 1 步', () => {
    const p = make(3, 3, 1)
    expect(p.step.value).toBe(1)
  })

  it('playFromStart 立即顯示第 1 步並開始播放，每 500ms 前進一步', () => {
    const p = make()
    p.playFromStart()
    expect(p.step.value).toBe(1)
    expect(p.playing.value).toBe(true)
    vi.advanceTimersByTime(499)
    expect(p.step.value).toBe(1)
    vi.advanceTimersByTime(1)
    expect(p.step.value).toBe(2)
  })

  it('播到最後一步自動暫停', () => {
    const p = make(3)
    p.playFromStart()
    vi.advanceTimersByTime(5000)
    expect(p.step.value).toBe(3)
    expect(p.playing.value).toBe(false)
  })

  it('播到最後一步後再按播放從第 1 步重播', () => {
    const p = make(3)
    p.playFromStart()
    vi.advanceTimersByTime(5000)
    p.play()
    expect(p.step.value).toBe(1)
    expect(p.playing.value).toBe(true)
  })

  it('遇到錯誤停止時只播到 end，不播到 total', () => {
    const p = make(2, 1)
    p.playFromStart()
    vi.advanceTimersByTime(5000)
    expect(p.step.value).toBe(1)
    expect(p.playing.value).toBe(false)
  })

  it('toggle 切換播放與暫停', () => {
    const p = make()
    p.seek(2)
    p.toggle()
    expect(p.playing.value).toBe(true)
    p.toggle()
    expect(p.playing.value).toBe(false)
  })

  it('單步前進一步並暫停', () => {
    const p = make()
    p.stepForward()
    expect(p.step.value).toBe(1)
    expect(p.playing.value).toBe(false)
  })

  it('播放中按單步會先暫停', () => {
    const p = make()
    p.playFromStart()
    p.stepForward()
    expect(p.playing.value).toBe(false)
    expect(p.step.value).toBe(2)
    vi.advanceTimersByTime(2000)
    expect(p.step.value).toBe(2)
  })

  it('在最後一步按單步不會超過', () => {
    const p = make(2)
    p.seek(2)
    p.stepForward()
    expect(p.step.value).toBe(2)
  })

  it('從頭：回到 rewindTo 並暫停', () => {
    const p = make()
    p.playFromStart()
    vi.advanceTimersByTime(1000)
    p.rewind()
    expect(p.step.value).toBe(0)
    expect(p.playing.value).toBe(false)
  })

  it('seek 直接跳到指定步', () => {
    const p = make(20)
    p.seek(11)
    expect(p.step.value).toBe(11)
  })

  it('seek 超過 end 時夾到 end，低於 rewindTo 時夾到 rewindTo', () => {
    const p = make(10, 4, 1)
    p.seek(9)
    expect(p.step.value).toBe(4)
    p.seek(-3)
    expect(p.step.value).toBe(1)
  })

  it('總步數為 0 時播放不動作', () => {
    const p = make(0)
    p.play()
    expect(p.step.value).toBe(0)
    expect(p.playing.value).toBe(false)
  })

  it('dispose 後計時器停止', () => {
    const p = make()
    p.playFromStart()
    p.dispose()
    vi.advanceTimersByTime(2000)
    expect(p.step.value).toBe(1)
  })
})
