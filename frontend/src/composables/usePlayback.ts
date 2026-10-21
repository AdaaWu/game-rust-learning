import { ref } from 'vue'
import type { Ref } from 'vue'

export interface PlaybackOptions {
  // 進度條總步數
  total: () => number
  // 可播放到的最後一步（遇到錯誤停止時小於 total）
  end: () => number
  // 「從頭」回到的步數：GridWorld 為 0、MemoryView 為 1
  rewindTo: number
  // 每步毫秒數
  stepMs: () => number
}

export interface Playback {
  step: Ref<number>
  playing: Ref<boolean>
  play(): void
  pause(): void
  toggle(): void
  stepForward(): void
  rewind(): void
  seek(step: number): void
  // 從第 1 步開始自動播放
  playFromStart(): void
  dispose(): void
}

export function usePlayback(options: PlaybackOptions): Playback {
  const step = ref(options.rewindTo)
  const playing = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null

  const clearTimer = () => {
    if (timer !== null) clearTimeout(timer)
    timer = null
  }

  const pause = () => {
    clearTimer()
    playing.value = false
  }

  const schedule = () => {
    clearTimer()
    timer = setTimeout(tick, options.stepMs())
  }

  function tick() {
    timer = null
    if (step.value >= options.end()) {
      playing.value = false
      return
    }
    step.value += 1
    if (step.value >= options.end()) playing.value = false
    else schedule()
  }

  const startAt = (first: number) => {
    if (options.end() < 1) {
      pause()
      return
    }
    step.value = first
    playing.value = step.value < options.end()
    if (playing.value) schedule()
  }

  const play = () => {
    if (options.end() < 1) return
    if (step.value >= options.end()) {
      startAt(1)
      return
    }
    playing.value = true
    schedule()
  }

  return {
    step,
    playing,
    play,
    pause,
    toggle: () => (playing.value ? pause() : play()),
    stepForward: () => {
      pause()
      if (step.value < options.end()) step.value += 1
    },
    rewind: () => {
      pause()
      step.value = options.rewindTo
    },
    seek: (target: number) => {
      pause()
      step.value = Math.min(Math.max(target, options.rewindTo), Math.max(options.end(), options.rewindTo))
    },
    playFromStart: () => startAt(1),
    dispose: clearTimer,
  }
}
