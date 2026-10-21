<script setup lang="ts">
import { ref } from 'vue'
import { useModalFocus } from '../composables/useModalFocus'

const emit = defineEmits<{
  close: []
}>()

const root = ref<HTMLElement | null>(null)
const { trapTab } = useModalFocus(root)

const ROWS: [string, string][] = [
  ['Ctrl+Enter', '執行（Run），macOS 用 Cmd'],
  ['Ctrl+Shift+Enter', '送出（Submit），macOS 用 Cmd'],
  ['Space', '可視化區：播放／暫停'],
  ['→', '可視化區：單步前進'],
  ['Home', '可視化區：從頭'],
  ['Esc', '關閉視窗、抽屜，跳過通關動畫'],
  ['Ctrl+M', '編輯器內切換「Tab 移動焦點」模式，鍵盤使用者用它離開編輯器'],
]

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
  <div class="modal-backdrop" @click.self="emit('close')">
    <div
      ref="root"
      class="modal shortcut-help"
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcut-title"
      data-testid="shortcut-help"
      tabindex="-1"
      @keydown="onKeydown"
    >
      <h2 id="shortcut-title" class="shortcut-title">⌨ 快捷鍵</h2>
      <dl class="shortcut-list">
        <template v-for="[key, desc] in ROWS" :key="key">
          <dt><kbd>{{ key }}</kbd></dt>
          <dd>{{ desc }}</dd>
        </template>
      </dl>
      <p class="shortcut-note">重設沒有快捷鍵，避免誤觸清掉程式碼。</p>
      <div class="modal-actions">
        <button type="button" class="btn is-secondary" @click="emit('close')">關閉</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shortcut-help {
  max-width: 520px;
}
.shortcut-title {
  margin: 0 0 12px;
  font-family: var(--font-heading);
  color: var(--heading);
}
.shortcut-list {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px 16px;
  margin: 0;
}
.shortcut-list dd {
  margin: 0;
}
kbd {
  font-family: var(--font-code);
  font-size: 13px;
  padding: 2px 8px;
  border-radius: 8px;
  border: var(--border-w) solid var(--border-strong);
  background: var(--surface-muted);
  white-space: nowrap;
}
.shortcut-note {
  color: var(--text-muted);
  font-size: 13px;
}
</style>
