<script setup lang="ts">
import { ref } from 'vue'
import type { StarSet } from '../domain/progress'
import { useModalFocus } from '../composables/useModalFocus'

defineProps<{
  // 「章-章內序」
  levelNumber: string
  title: string
  // 本次遊玩拿到的星
  earned: StarSet
}>()

const emit = defineEmits<{
  next: []
  replay: []
  map: []
  close: []
}>()

const STAR_ROWS: { key: keyof StarSet; label: string; tip: string }[] = [
  { key: 'pass', label: '通過', tip: '' },
  { key: 'noHint', label: '沒看提示', tip: '不看提示再挑戰一次就能拿到' },
  { key: 'firstTry', label: '一次就過', tip: '第一次送出就通過就能拿到' },
]

const root = ref<HTMLElement | null>(null)
const { trapTab } = useModalFocus(root)

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  trapTab(event)
}
</script>

<template>
  <div class="modal-backdrop">
    <div
      ref="root"
      class="modal complete-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="complete-title"
      tabindex="-1"
      @keydown="onKeydown"
    >
      <h2 id="complete-title" class="complete-title">{{ levelNumber }} {{ title }}</h2>
      <ul class="star-list">
        <li
          v-for="(row, i) in STAR_ROWS"
          :key="row.key"
          class="star-row"
          :class="{ 'is-earned': earned[row.key] }"
          :style="{ animationDelay: `${i * 250}ms` }"
          data-testid="star-row"
          :data-star="row.key"
          :data-earned="String(earned[row.key])"
        >
          <span class="star-icon" :class="{ 'is-on': earned[row.key] }" data-testid="star-icon" aria-hidden="true">{{ earned[row.key] ? '★' : '☆' }}</span>
          <span class="star-label">{{ row.label }}</span>
          <span class="star-status">{{ earned[row.key] ? '達成' : '未達成' }}</span>
          <span v-if="!earned[row.key] && row.tip" class="star-tip" data-testid="star-tip">{{ row.tip }}</span>
        </li>
      </ul>
      <div class="modal-actions">
        <button type="button" class="btn is-primary" @click="emit('next')">下一關</button>
        <button type="button" class="btn is-secondary" @click="emit('replay')">再玩一次</button>
        <button type="button" class="btn is-ghost" @click="emit('map')">回地圖</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.complete-dialog {
  max-width: 460px;
  text-align: center;
}
.complete-title {
  margin: 4px 0 16px;
  font-family: var(--font-heading);
  color: var(--heading);
}
.star-list {
  list-style: none;
  margin: 0 0 20px;
  padding: 0;
  display: grid;
  gap: 8px;
  text-align: left;
}
.star-row {
  display: grid;
  grid-template-columns: 36px 1fr auto;
  align-items: center;
  gap: 4px 10px;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
  animation: star-in 400ms var(--move-ease) both;
}
.star-icon {
  font-size: 28px;
  line-height: 1;
  color: var(--star-off);
}
.star-icon.is-on {
  color: var(--star-on);
  text-shadow: var(--star-glow);
}
.star-status {
  font-size: 13px;
  color: var(--text-muted);
}
.star-row.is-earned .star-status {
  color: var(--success);
}
.star-tip {
  grid-column: 2 / 4;
  font-size: 13px;
  color: var(--text-muted);
}
@keyframes star-in {
  from {
    opacity: 0;
    transform: scale(0.6);
  }
}
@media (prefers-reduced-motion: reduce) {
  .star-row {
    animation: none;
  }
}
</style>
