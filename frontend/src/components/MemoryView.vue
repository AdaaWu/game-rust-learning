<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { MemorySnapshot } from '../api/types'
import { usePlayback } from '../composables/usePlayback'
import { renderInline } from '../domain/markdown'
import { deriveSnapshotView, findStepForLine, stepCaption } from '../domain/memory'
import type { PointerView } from '../domain/memory'
import { usePlaybackStore } from '../stores/playback'
import PlaybackBar from './PlaybackBar.vue'

const props = defineProps<{
  snapshots: MemorySnapshot[]
  // 程式碼與 starter 不同
  codeModified: boolean
}>()

const emit = defineEmits<{
  // 編輯器要高亮的行；null 表示移除高亮
  'line-change': [line: number | null]
}>()

const playbackStore = usePlaybackStore()

// 步數 1 起算；第 n 步對應 snapshots[n - 1]
const playback = usePlayback({
  total: () => props.snapshots.length,
  end: () => props.snapshots.length,
  rewindTo: 1,
  stepMs: () => playbackStore.stepMs,
})
const { step, playing } = playback

const index = computed(() => Math.min(Math.max(step.value, 1), props.snapshots.length) - 1)
const view = computed(() => deriveSnapshotView(props.snapshots, index.value))
const caption = computed(() => stepCaption(props.snapshots, index.value))
const noteHtml = computed(() => renderInline(view.value.note))

const reducedMotion =
  typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

const POINTER_LABEL: Record<PointerView['kind'], string> = { own: 'own', shared: '& 👁', mut: '&mut ✎' }
const BORROW_NOTE = { mut: '被獨佔借用中，暫時不能用', shared: '被共享借用中，只能讀' } as const

const pointerLabel = (p: PointerView) => POINTER_LABEL[p.kind] + (p.danger ? ' ✗' : '')

watch(
  () => props.snapshots[index.value]?.line,
  (line) => emit('line-change', line ?? null),
)

onMounted(() => {
  emit('line-change', props.snapshots[index.value]?.line ?? null)
})

onBeforeUnmount(() => {
  playback.dispose()
  emit('line-change', null)
})

// 反向同步：點編輯器 gutter 時跳到對應步驟
function goToLine(line: number) {
  const found = findStepForLine(props.snapshots, line)
  if (found !== null) playback.seek(found + 1)
}

defineExpose({ goToLine })

function onKeydown(event: KeyboardEvent) {
  if (event.key === ' ') {
    event.preventDefault()
    playback.toggle()
  } else if (event.key === 'ArrowRight') {
    event.preventDefault()
    playback.stepForward()
  } else if (event.key === 'Home') {
    event.preventDefault()
    playback.rewind()
  }
}

// 指標箭頭：依畫面上變數框與 heap 區塊的位置計算
const board = ref<HTMLElement | null>(null)
interface ArrowGeometry {
  d: string
  labelX: number
  labelY: number
}
const geometry = ref(new Map<string, ArrowGeometry>())
const pointerKey = (p: { from: string; to: string }) => `${p.from}->${p.to}`
const NO_GEOMETRY: ArrowGeometry = { d: '', labelX: 0, labelY: 0 }
// 錯誤箭頭排在最後，畫在最上層
const arrows = computed(() =>
  [...view.value.pointers]
    .sort((x, y) => Number(x.danger) - Number(y.danger))
    .map((pointer) => ({ pointer, ...(geometry.value.get(pointerKey(pointer)) ?? NO_GEOMETRY) })),
)
const boardSize = ref({ width: 0, height: 0 })

function anchorRect(id: string): DOMRect | null {
  const el = board.value?.querySelector<HTMLElement>(`[data-anchor="${CSS.escape(id)}"]`)
  return el ? el.getBoundingClientRect() : null
}

function layoutArrows() {
  const root = board.value
  if (!root) return
  const base = root.getBoundingClientRect()
  boardSize.value = { width: base.width, height: base.height }
  const next = new Map<string, ArrowGeometry>()
  for (const pointer of view.value.pointers) {
    const from = anchorRect(pointer.from)
    const to = anchorRect(pointer.to)
    if (!from || !to) continue
    const x1 = from.right - base.left - 10
    const y1 = from.top + from.height / 2 - base.top
    let d: string
    let labelX: number
    let labelY: number
    const toIsHeap = !!root.querySelector(`[data-testid="mem-heap"][data-id="${CSS.escape(pointer.to)}"]`)
    const stacked = to.top >= from.bottom && toIsHeap && Math.abs(to.left - from.left) < 40
    if (toIsHeap && !stacked) {
      const x2 = to.left - base.left - 2
      const y2 = to.top + to.height / 2 - base.top
      const mid = (x1 + x2) / 2
      d = `M ${x1} ${y1} C ${mid} ${y1}, ${mid} ${y2}, ${x2} ${y2}`
      labelX = mid
      labelY = (y1 + y2) / 2 - 6
    } else if (toIsHeap) {
      const x2 = to.left - base.left + 24
      const y2 = to.top - base.top - 2
      d = `M ${x1} ${y1} C ${x1 + 30} ${y1}, ${x2} ${y2 - 30}, ${x2} ${y2}`
      labelX = (x1 + x2) / 2 + 20
      labelY = (y1 + y2) / 2
    } else {
      const x2 = to.right - base.left - 2
      const y2 = to.top + to.height / 2 - base.top
      const bulge = Math.max(x1, x2) + 46
      d = `M ${x1} ${y1} C ${bulge} ${y1}, ${bulge} ${y2}, ${x2} ${y2}`
      labelX = bulge - 10
      labelY = (y1 + y2) / 2
    }
    next.set(pointerKey(pointer), { d, labelX, labelY })
  }
  geometry.value = next
}

let observer: ResizeObserver | null = null
onMounted(() => {
  layoutArrows()
  if (typeof ResizeObserver === 'function' && board.value) {
    observer = new ResizeObserver(() => layoutArrows())
    observer.observe(board.value)
  }
})
onBeforeUnmount(() => observer?.disconnect())
watch(view, () => nextTick(layoutArrows))
</script>

<template>
  <section class="memory-view" data-testid="memory-view" tabindex="0" aria-label="記憶體" @keydown="onKeydown">
    <div ref="board" class="mem-board">
      <div class="mem-col mem-stack">
        <h3 class="mem-heading">Stack</h3>
        <div
          v-for="frame in view.frames"
          :key="frame.id"
          class="mem-frame"
          :class="{ 'is-current': frame.current, 'is-changed': frame.changed }"
          data-testid="mem-frame"
          :data-id="frame.id"
        >
          <div class="mem-frame-name">{{ frame.fn }}</div>
          <div
            v-for="v in frame.vars"
            :key="v.id"
            class="mem-var"
            :class="[
              `is-${v.state}`,
              v.borrow ? `borrow-${v.borrow}` : '',
              { 'is-error': !!v.error, 'is-changed': v.changed },
            ]"
            data-testid="mem-var"
            :data-id="v.id"
            :data-anchor="v.id"
          >
            <span class="mem-var-name">{{ v.name }}</span>
            <span class="mem-var-type">: {{ v.type }}</span>
            <span v-if="v.value !== null" class="mem-var-value">= {{ v.value }}</span>
            <span class="mem-badges">
              <span v-if="v.state === 'moved'" class="mem-badge">已移動</span>
              <span v-if="v.state === 'dropped'" class="mem-badge">已釋放</span>
              <span v-if="v.borrow === 'shared'" class="mem-badge is-shared">&amp;</span>
              <span v-if="v.borrow === 'mut'" class="mem-badge is-mut">🔒 借出中</span>
              <span v-if="v.borrow" class="mem-borrow-note">{{ BORROW_NOTE[v.borrow] }}</span>
              <span v-if="v.error" class="mem-badge is-error" data-testid="mem-error-badge">✗ {{ v.error.badge }}</span>
              <span v-if="v.changed && reducedMotion" class="mem-new-dot" data-testid="mem-new-dot">新</span>
            </span>
          </div>
        </div>
      </div>
      <div class="mem-col mem-heap">
        <h3 class="mem-heading">Heap</h3>
        <div
          v-for="h in view.heap"
          :key="h.id"
          class="mem-block"
          :class="[`is-${h.state}`, { 'is-error': !!h.error, 'is-changed': h.changed }]"
          data-testid="mem-heap"
          :data-id="h.id"
          :data-anchor="h.id"
        >
          <span class="mem-var-type">{{ h.type }}</span>
          <span class="mem-block-value">{{ h.value }}</span>
          <span class="mem-badges">
            <span v-if="h.state === 'freed'" class="mem-badge">已釋放</span>
            <span v-if="h.error" class="mem-badge is-error" data-testid="mem-error-badge">✗ {{ h.error.badge }}</span>
            <span v-if="h.changed && reducedMotion" class="mem-new-dot" data-testid="mem-new-dot">新</span>
          </span>
        </div>
      </div>
      <svg class="mem-arrows" :width="boardSize.width" :height="boardSize.height" aria-hidden="true">
        <defs>
          <marker id="mem-head-own" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0 0 L10 5 L0 10 Z" fill="var(--mem-own)" />
          </marker>
          <marker id="mem-head-shared" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
            <path d="M1 1 L9 5 L1 9" fill="none" stroke="var(--borrow-shared)" stroke-width="1.6" />
          </marker>
          <marker id="mem-head-mut" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M0 0 L10 5 L0 10 Z" fill="var(--borrow-mut)" />
          </marker>
          <!-- 轉紅的箭頭沿用原種類的箭頭形狀 -->
          <marker id="mem-head-own-danger" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0 0 L10 5 L0 10 Z" fill="var(--danger)" />
          </marker>
          <marker id="mem-head-shared-danger" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
            <path d="M1 1 L9 5 L1 9" fill="none" stroke="var(--danger)" stroke-width="1.6" />
          </marker>
          <marker id="mem-head-mut-danger" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M0 0 L10 5 L0 10 Z" fill="var(--danger)" />
          </marker>
        </defs>
        <g
          v-for="a in arrows"
          :key="pointerKey(a.pointer)"
          class="mem-pointer"
          :class="[`kind-${a.pointer.kind}`, { 'is-danger': a.pointer.danger }]"
          data-testid="mem-pointer"
          :data-from="a.pointer.from"
          :data-to="a.pointer.to"
          :data-kind="a.pointer.kind"
        >
          <path :d="a.d" fill="none" :marker-end="`url(#mem-head-${a.pointer.kind}${a.pointer.danger ? '-danger' : ''})`" />
          <text :x="a.labelX" :y="a.labelY" text-anchor="middle" data-testid="mem-pointer-label">{{ pointerLabel(a.pointer) }}</text>
        </g>
      </svg>
    </div>
    <p v-if="codeModified" class="mem-modified">程式碼已修改，行號高亮僅供參考</p>
    <div class="mem-caption-box" aria-live="polite">
      <div class="mem-caption" data-testid="mem-caption">{{ caption }}</div>
      <!-- note 經 DOMPurify 過濾 -->
      <div class="mem-note" data-testid="mem-note" v-html="noteHtml" />
    </div>
    <PlaybackBar
      :step="index + 1"
      :total="snapshots.length"
      :playing="playing"
      :min="1"
      @rewind="playback.rewind()"
      @toggle="playback.toggle()"
      @step-forward="playback.stepForward()"
      @seek="playback.seek($event)"
    />
  </section>
</template>

<style scoped>
.memory-view {
  container-type: inline-size;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  outline: none;
}
.memory-view:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: -2px;
}
.mem-board {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: auto;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 0.8fr);
  gap: 56px;
  padding: 12px 16px;
}
.mem-col {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}
.mem-heading {
  margin: 0;
  font-family: var(--font-heading);
  font-size: 14px;
  color: var(--heading);
}
.mem-frame {
  border: var(--border-w) solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  padding: 6px 8px 8px;
  display: grid;
  gap: 6px;
}
.mem-frame.is-current {
  border-width: var(--border-w-strong);
  border-color: var(--heading);
}
.mem-frame-name {
  font-family: var(--font-code);
  font-size: 12px;
  color: var(--text-muted);
}
.mem-var,
.mem-block {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 6px;
  padding: 6px 8px;
  border: var(--border-w) solid var(--border);
  border-radius: 10px;
  background: var(--surface-muted);
  font-family: var(--font-code);
  font-size: 13px;
}
.mem-var-name {
  font-weight: 700;
}
.mem-var-type {
  color: var(--text-muted);
}
.mem-block-value {
  width: 100%;
}
.mem-badges {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-left: auto;
}
.mem-badge {
  font-family: var(--font-ui);
  font-size: 11px;
  padding: 1px 6px;
  border-radius: var(--radius-pill);
  background: var(--surface);
  border: 1px solid var(--border-strong);
  color: var(--text-muted);
}
.mem-badge.is-shared {
  color: var(--borrow-shared);
  border-color: var(--borrow-shared);
  /* 固定在變數框右上角，窄欄換行時也不移位 */
  position: absolute;
  top: -8px;
  right: -6px;
}
.mem-badge.is-mut {
  color: var(--borrow-mut);
  border-color: var(--borrow-mut);
}
.mem-badge.is-error {
  color: var(--brand-contrast);
  background: var(--danger);
  border-color: var(--danger);
  font-weight: 700;
}
.mem-var.is-moved {
  filter: grayscale(1);
  color: var(--mem-moved);
  background-image: repeating-linear-gradient(135deg, transparent 0 6px, color-mix(in srgb, var(--mem-moved) 35%, transparent) 6px 8px);
}
.mem-var.is-moved .mem-var-name {
  text-decoration: line-through;
}
.mem-var.is-dropped,
.mem-block.is-freed {
  border-style: dashed;
  opacity: 0.4;
}
.mem-var.borrow-shared {
  border-color: var(--borrow-shared);
}
.mem-var.borrow-mut {
  border-color: var(--borrow-mut);
  border-width: var(--border-w-strong);
}
.mem-var.is-error,
.mem-block.is-error {
  border: var(--border-w-strong) solid var(--danger);
  opacity: 1;
  /* 錯誤標記畫在指標箭頭之上 */
  z-index: 1;
}
.mem-frame.is-changed,
.mem-var.is-changed,
.mem-block.is-changed {
  animation: mem-flash 300ms ease-out;
}
.mem-borrow-note {
  width: 100%;
  font-family: var(--font-ui);
  font-size: 12px;
}
.mem-var.borrow-shared .mem-borrow-note {
  color: var(--borrow-shared);
}
.mem-var.borrow-mut .mem-borrow-note {
  color: var(--borrow-mut);
}
.mem-new-dot {
  font-family: var(--font-ui);
  font-size: 11px;
  padding: 0 6px;
  border-radius: var(--radius-pill);
  background: var(--accent);
  color: var(--brand-contrast);
}
.mem-arrows {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: visible;
}
.mem-pointer path {
  stroke-width: 2;
}
.mem-pointer text {
  font-family: var(--font-code);
  font-size: 11px;
  paint-order: stroke;
  stroke: var(--surface);
  stroke-width: 3px;
}
.mem-pointer.kind-own path {
  stroke: var(--mem-own);
}
.mem-pointer.kind-own text {
  fill: var(--mem-own);
}
.mem-pointer.kind-shared path {
  stroke: var(--borrow-shared);
  stroke-width: 1.5;
}
.mem-pointer.kind-shared text {
  fill: var(--borrow-shared);
}
.mem-pointer.kind-mut path {
  stroke: var(--borrow-mut);
  stroke-width: 3;
  stroke-dasharray: 6 4;
}
.mem-pointer.kind-mut text {
  fill: var(--borrow-mut);
}
.mem-pointer.is-danger path {
  stroke: var(--danger);
}
.mem-pointer.is-danger text {
  fill: var(--danger);
}
.mem-modified {
  margin: 0;
  padding: 4px 12px;
  font-size: 13px;
  color: var(--warning);
  background: color-mix(in srgb, var(--warning) 12%, transparent);
}
.mem-caption-box {
  padding: 8px 12px;
  border-top: var(--border-w) solid var(--border);
  font-size: 14px;
}
.mem-caption {
  font-weight: 700;
  color: var(--heading);
}
.mem-note {
  margin-top: 2px;
}
@keyframes mem-flash {
  0% {
    box-shadow: 0 0 0 4px var(--accent);
  }
}
/* 可視化區寬度不足 360px 時上下排列 */
@container (max-width: 359px) {
  .mem-board {
    grid-template-columns: 1fr;
    gap: 36px;
  }
}
@media (prefers-reduced-motion: reduce) {
  .mem-frame.is-changed,
  .mem-var.is-changed,
  .mem-block.is-changed {
    animation: none;
  }
}
</style>
