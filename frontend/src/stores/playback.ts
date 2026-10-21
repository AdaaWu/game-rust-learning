import { defineStore } from 'pinia'

export type PlaybackSpeed = 0.5 | 1 | 2 | 4

export const PLAYBACK_SPEEDS: PlaybackSpeed[] = [0.5, 1, 2, 4]

// 1x 每步 500ms
export const BASE_STEP_MS = 500

// 播放速度只存在記憶體，不寫 localStorage
export const usePlaybackStore = defineStore('playback', {
  state: () => ({
    speed: 1 as PlaybackSpeed,
  }),
  getters: {
    stepMs: (state): number => BASE_STEP_MS / state.speed,
  },
  actions: {
    setSpeed(speed: PlaybackSpeed) {
      this.speed = speed
    },
  },
})
