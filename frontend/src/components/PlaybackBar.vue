<script setup lang="ts">
import { PLAYBACK_SPEEDS, usePlaybackStore } from '../stores/playback'
import type { PlaybackSpeed } from '../stores/playback'

// GridWorld 與 MemoryView 共用的播放控制列
defineProps<{
  step: number
  total: number
  playing: boolean
  min: number
}>()

const emit = defineEmits<{
  rewind: []
  toggle: []
  'step-forward': []
  seek: [step: number]
}>()

const playback = usePlaybackStore()

function onSpeed(event: Event) {
  playback.setSpeed(Number((event.target as HTMLSelectElement).value) as PlaybackSpeed)
}

function onSeek(event: Event) {
  emit('seek', Number((event.target as HTMLInputElement).value))
}
</script>

<template>
  <div class="playback-bar">
    <div class="playback-buttons">
      <button type="button" class="pb-btn" aria-label="從頭" title="從頭" @click="emit('rewind')">⏮</button>
      <button
        type="button"
        class="pb-btn is-main"
        :aria-label="playing ? '暫停' : '播放'"
        :title="playing ? '暫停' : '播放'"
        @click="emit('toggle')"
      >
        {{ playing ? '⏸' : '▶' }}
      </button>
      <button type="button" class="pb-btn" aria-label="單步前進" title="單步前進" @click="emit('step-forward')">⏭</button>
      <label class="pb-speed">
        <span>速度</span>
        <select aria-label="速度" :value="String(playback.speed)" @change="onSpeed">
          <option v-for="s in PLAYBACK_SPEEDS" :key="s" :value="String(s)">{{ s }}x</option>
        </select>
      </label>
    </div>
    <div class="playback-progress">
      <input
        type="range"
        aria-label="進度"
        :min="min"
        :max="total"
        :value="step"
        :disabled="total <= min"
        @input="onSeek"
      />
      <span class="pb-label" data-testid="step-label">第 {{ step }} / {{ total }} 步</span>
    </div>
  </div>
</template>

<style scoped>
.playback-bar {
  display: grid;
  gap: 6px;
  padding: 8px 12px;
  border-top: var(--border-w) solid var(--border);
  background: var(--surface-muted);
}
.playback-buttons {
  display: flex;
  align-items: center;
  gap: 6px;
}
.pb-btn {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-pill);
  border: var(--border-w) solid var(--border-strong);
  background: var(--surface);
  color: var(--text);
  font-size: 15px;
  cursor: pointer;
}
.pb-btn.is-main {
  background: var(--brand);
  border-color: var(--brand);
  color: var(--brand-contrast);
}
.pb-btn:hover {
  transform: translateY(-1px);
}
.pb-speed {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-muted);
}
.pb-speed select {
  border-radius: var(--radius-pill);
  border: var(--border-w) solid var(--border);
  background: var(--surface);
  color: var(--text);
  padding: 2px 8px;
  font: inherit;
}
.playback-progress {
  display: flex;
  align-items: center;
  gap: 10px;
}
.playback-progress input {
  flex: 1;
  accent-color: var(--brand);
}
.pb-label {
  font-size: 13px;
  color: var(--text-muted);
  white-space: nowrap;
}
</style>
