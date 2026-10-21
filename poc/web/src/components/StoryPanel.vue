<script setup lang="ts">
import { computed, ref } from 'vue'
import MarkdownIt from 'markdown-it'
import Ferris, { type Mood } from './Ferris.vue'

const props = defineProps<{ look: 'A' | 'B' | 'C'; storyLines: string[]; ferrisLine: string | null; mood: Mood; concept: string; hints: string[]; revealed: number }>()
const emit = defineEmits<{ (e: 'reveal'): void }>()

// 關卡內容是自家撰寫，POC 關閉 html 即可
const md = new MarkdownIt({ html: false, linkify: true })
const conceptHtml = computed(() => md.render(props.concept))
const idx = ref(0)
const line = computed(() => props.ferrisLine ?? props.storyLines[idx.value] ?? '')
const hasNext = computed(() => props.ferrisLine === null && idx.value < props.storyLines.length - 1)
const confirming = ref(false)
function askHint() {
  if (props.revealed === 0 && !confirming.value) {
    confirming.value = true
    return
  }
  confirming.value = false
  emit('reveal')
}
</script>

<template>
  <aside class="story">
    <div class="dialog">
      <div class="avatar"><Ferris :look="look" :mood="mood" :size="56" /></div>
      <div class="bubble">
        <p :key="line">{{ line }}</p>
        <button v-if="hasNext" class="next" @click="idx++">下一句 ▸</button>
      </div>
    </div>

    <div class="mission"><span class="tag">修好編譯錯誤</span> 讓 Ferris 撿到寶石並走到終點旗子</div>

    <div class="concept md" v-html="conceptHtml" />

    <div class="hints">
      <div v-for="(h, i) in hints.slice(0, revealed)" :key="i" class="hint"><b>提示 {{ i + 1 }}</b>　{{ h }}</div>
      <div v-if="confirming" class="confirm">
        看提示會拿不到「沒看提示」這顆星，確定要看嗎？
        <div class="row"><button class="btn-primary sm" @click="askHint">確定看</button><button class="btn-ghost sm" @click="confirming = false">再想想</button></div>
      </div>
      <button v-else-if="revealed < hints.length" class="btn-secondary hint-btn" @click="askHint">
        {{ revealed === 0 ? '需要提示嗎？' : '看下一個提示' }}（{{ revealed }} / {{ hints.length }}）
      </button>
    </div>
  </aside>
</template>

<style scoped>
.story { display: flex; flex-direction: column; gap: 14px; height: 100%; overflow: auto; padding-right: 4px; }
.dialog { display: flex; gap: 10px; align-items: flex-start; }
.avatar { flex: none; width: 64px; height: 64px; display: grid; place-items: center; background: var(--surface-2); border: var(--border-w) solid var(--border-strong); border-radius: var(--radius-avatar); }
.bubble { position: relative; flex: 1; background: var(--surface); border: var(--border-w) solid var(--border-strong); border-radius: var(--radius); padding: 10px 12px; box-shadow: var(--shadow-sm); font-size: 14px; line-height: 1.6; }
.bubble::before { content: ''; position: absolute; left: -9px; top: 18px; width: 14px; height: 14px; background: var(--surface); border-left: var(--border-w) solid var(--border-strong); border-bottom: var(--border-w) solid var(--border-strong); transform: rotate(45deg); }
.bubble p { margin: 0; animation: type .35s; }
@keyframes type { from { opacity: 0; transform: translateY(3px); } }
.next { margin-top: 6px; font-size: 12px; padding: 3px 10px; }
.mission { padding: 8px 12px; border-left: 5px solid var(--brand); background: var(--brand-soft); border-radius: var(--radius-sm); font-size: 13px; }
.tag { font-family: var(--font-title); font-size: var(--title-size-xs); color: var(--brand); margin-right: 6px; }
.concept { font-size: 14px; line-height: 1.7; }
.concept :deep(h2) { font-family: var(--font-title); font-size: var(--title-size-sm); margin: 14px 0 6px; color: var(--heading); }
.concept :deep(code) { font-family: var(--font-code); background: var(--surface-2); padding: 1px 5px; border-radius: var(--radius-sm); font-size: .92em; }
.concept :deep(pre) { background: var(--surface-2); padding: 10px; border-radius: var(--radius-sm); border: var(--border-w) solid var(--border); overflow-x: auto; }
.concept :deep(pre code) { background: none; padding: 0; }
.concept :deep(ul) { padding-left: 20px; margin: 4px 0; }
.hints { position: sticky; bottom: 0; background: var(--surface); padding-top: 8px; display: flex; flex-direction: column; gap: 6px; margin-top: auto; }
.hint { background: var(--surface); border: var(--border-w) dashed var(--border-strong); border-radius: var(--radius-sm); padding: 8px 10px; font-size: 13px; }
.hint b { color: var(--brand); }
.confirm { background: var(--surface); border: var(--border-w) solid var(--warning); border-radius: var(--radius-sm); padding: 10px; font-size: 13px; }
.row { display: flex; gap: 8px; margin-top: 8px; }
.sm { font-size: 12px; padding: 4px 12px; }
.hint-btn { width: 100%; }
</style>
