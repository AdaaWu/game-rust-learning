<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import type { RunStatus } from '../api/types'
import { usePlayback } from '../composables/usePlayback'
import { deriveTimeline } from '../domain/timeline/derive'
import type { LaneState, TimelineLane, TimelineSegment } from '../domain/timeline/derive'
import { MAX_TIMELINE_EVENTS, parseTimelineEvents } from '../domain/timeline/parse'
import type { TimelineEvent } from '../domain/timeline/parse'
import { usePlaybackStore } from '../stores/playback'
import PlaybackBar from './PlaybackBar.vue'

// ux.md §11 並行時間軸
const props = defineProps<{
  // null 表示尚未執行，或 execution = null
  stdout: string | null
  status: RunStatus | null
  truncated: boolean
  // Run／Submit 請求中：回到第 0 步並半透明
  pending: boolean
}>()

const emit = defineEmits<{
  'line-change': [line: number | null]
}>()

const AUTOPLAY: RunStatus[] = ['ok', 'runtime_panic', 'timeout', 'memory_limit', 'output_limit']
const NARROW_WIDTH = 360
const LABEL_W = 120
const LABEL_W_NARROW = 64
const COL_MIN = 14
const COL_MAX = 32
const TICK_H = 20
const LAYER_H = 18
const ROW_TOP = 2
const MARK_BAND = 9
const TAIL_PAD = 36
const CIRCLED = '①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳'

const playbackStore = usePlaybackStore()

// 畫面依據的 Run 結果；請求中沿用上次 Run，總步數不歸零
const shown = shallowRef({ stdout: props.stdout, status: props.status, truncated: props.truncated })
const parsed = computed(() =>
  shown.value.stdout === null ? null : parseTimelineEvents(shown.value.stdout, shown.value.truncated),
)
const events = computed<TimelineEvent[]>(() => parsed.value?.events ?? [])
const total = computed(() => events.value.length)
const runStatus = computed<RunStatus>(() => shown.value.status ?? 'ok')

const playback = usePlayback({
  total: () => total.value,
  end: () => total.value,
  rewindTo: 0,
  stepMs: () => playbackStore.stepMs,
})
const { step, playing } = playback

watch(
  () => [props.pending, props.stdout, props.status, props.truncated] as const,
  ([pending, stdout, status, truncated]) => {
    if (pending) {
      playback.rewind()
      return
    }
    // 與請求前畫面上的 Run 相同視為沒有新結果（Submit、請求失敗）
    const prev = shown.value
    shown.value = { stdout, status, truncated }
    const runChanged = stdout !== prev.stdout || status !== prev.status || truncated !== prev.truncated
    if (!runChanged) {
      // 請求結束但沒有新的 Run 結果：還原到上次 Run 的終點
      playback.seek(total.value)
      return
    }
    if (stdout !== null && status !== null && AUTOPLAY.includes(status)) playback.playFromStart()
    else playback.rewind()
  },
)

const shownStep = computed(() => (props.pending ? 0 : Math.min(step.value, total.value)))
const state = computed(() => deriveTimeline(events.value, shownStep.value, runStatus.value))
// 播完全部事件時的畫面：用來固定疊層與泳道高度
const finalState = computed(() => deriveTimeline(events.value, total.value, runStatus.value))
const hasEvents = computed(() => total.value > 0)
const atEnd = computed(() => hasEvents.value && !props.pending && !playing.value && shownStep.value === total.value)

// 中央提示：尚未執行、沒編譯成功、零事件
const emptyText = computed(() => {
  if (shown.value.stdout === null) {
    if (shown.value.status === null) return '按『執行』，看看每條執行緒怎麼跑'
    return shown.value.status === 'memory_limit' ? '編譯時用太多記憶體' : '程式還沒編譯成功'
  }
  return total.value === 0 ? 'zero' : ''
})

// 圖例：鎖與 channel 依第一次出現的順序
const locks = computed(() => {
  const list: string[] = []
  for (const e of events.value) if (e.kind.startsWith('lock_') && !list.includes(e.target)) list.push(e.target)
  return list
})
const channels = computed(() => {
  const list: string[] = []
  for (const e of events.value) if ((e.kind === 'send' || e.kind.startsWith('recv')) && !list.includes(e.target)) list.push(e.target)
  return list
})
const lockCode = (lock: string) => {
  const i = locks.value.indexOf(lock)
  return i < CIRCLED.length ? CIRCLED[i] : `(${i + 1})`
}
const lockColor = (lock: string) => `var(--tl-lock-${(locks.value.indexOf(lock) % 4) + 1})`

// 版面寬度：ResizeObserver 回報容器寬度
const rootEl = ref<HTMLElement | null>(null)
const scrollEl = ref<HTMLElement | null>(null)
const width = ref(0)
let observer: ResizeObserver | null = null
onMounted(() => {
  // 帶著 Run 結果掛載（LevelView 換 key）時直接重播
  if (!props.pending && props.stdout !== null && props.status !== null && AUTOPLAY.includes(props.status)) {
    playback.playFromStart()
  }
  if (typeof ResizeObserver === 'undefined' || !rootEl.value) return
  observer = new ResizeObserver((entries) => {
    const entry = entries[0]
    if (entry) width.value = entry.contentRect.width
  })
  observer.observe(rootEl.value)
})
onBeforeUnmount(() => {
  observer?.disconnect()
  playback.dispose()
})

const narrow = computed(() => width.value > 0 && width.value < NARROW_WIDTH)
const labelW = computed(() => (narrow.value ? LABEL_W_NARROW : LABEL_W))
const colW = computed(() => {
  const avail = (width.value > 0 ? width.value : 390) - labelW.value - TAIL_PAD
  return Math.min(COL_MAX, Math.max(COL_MIN, avail / Math.max(total.value, 1)))
})
const svgW = computed(() => Math.max(total.value, 1) * colW.value + TAIL_PAD)
const colX = (s: number) => (s - 0.5) * colW.value

// 疊層：依 from 排序，取第一個沒有重疊的層
const segKey = (g: TimelineSegment) => `${g.lane}|${g.kind}|${g.from}`
const layout = computed(() => {
  const layerOf = new Map<string, number>()
  const layersOf = new Map<string, number>()
  const busyUntil = new Map<string, number[]>()
  for (const g of finalState.value.segments) {
    const ends = busyUntil.get(g.lane) ?? []
    const end = g.to ?? Infinity
    let layer = ends.findIndex((e) => e <= g.from)
    if (layer === -1) layer = ends.length
    ends[layer] = end
    busyUntil.set(g.lane, ends)
    layerOf.set(segKey(g), layer)
    layersOf.set(g.lane, Math.max(layersOf.get(g.lane) ?? 1, layer + 1))
  }
  const rows = new Map<string, { top: number; height: number }>()
  let top = 0
  for (const lane of finalState.value.lanes) {
    const height = ROW_TOP + (layersOf.get(lane.name) ?? 1) * LAYER_H + MARK_BAND
    rows.set(lane.name, { top, height })
    top += height
  }
  return { layerOf, rows, height: Math.max(top, 48) }
})
const rowOf = (lane: string) => layout.value.rows.get(lane) ?? { top: 0, height: 0 }
const layerY = (lane: string, layer: number) => rowOf(lane).top + ROW_TOP + layer * LAYER_H + LAYER_H / 2
const markY = (lane: string) => rowOf(lane).top + rowOf(lane).height - MARK_BAND / 2

const ticks = computed(() => {
  const list: number[] = []
  for (let s = 1; s <= total.value; s += s === 1 ? 4 : 5) list.push(s)
  return list
})

const segmentViews = computed(() =>
  state.value.segments.map((g) => {
    const layer = layout.value.layerOf.get(segKey(g)) ?? 0
    const x1 = colX(g.from)
    const x2 = Math.max(g.to !== null ? colX(g.to) : colX(shownStep.value) + colW.value / 2, x1 + colW.value / 2)
    const y = layerY(g.lane, layer)
    let head = '✉⏳'
    if (g.kind === 'hold') head = lockCode(g.target)
    else if (g.kind === 'wait_lock') head = `⏳${lockCode(g.target)}`
    return { g, layer, x1, x2, y, head, showHead: g.kind !== 'hold' || x2 - x1 >= colW.value * 2 }
  }),
)

const messageViews = computed(() =>
  state.value.messages.map((m) => {
    const x1 = colX(m.from.step)
    const y1 = layerY(m.from.lane, 0)
    if (m.to === null) return { m, pending: true, path: `M ${x1} ${y1} L ${x1 + colW.value * 1.2} ${y1 - 8}`, tailX: x1 + colW.value * 1.2 + 2, tailY: y1 - 6 }
    return { m, pending: false, path: `M ${x1} ${y1} L ${colX(m.to.step)} ${layerY(m.to.lane, 0)}`, tailX: 0, tailY: 0 }
  }),
)

// 死鎖箭頭：等待區段尾端指向持有者的 hold 區段
const deadlockArrows = computed(() =>
  (state.value.deadlock ?? []).flatMap((d) => {
    const wait = segmentViews.value.find((v) => v.g.lane === d.lane && v.g.kind === 'wait_lock' && v.g.to === null)
    const hold = segmentViews.value.find((v) => v.g.lane === d.heldBy && v.g.kind === 'hold' && v.g.target === d.waits && v.g.to === null)
    if (!wait || !hold) return []
    const x1 = wait.x2
    const y1 = wait.y
    const x2 = hold.x2 - 4
    const y2 = hold.y
    return [{ key: d.lane, path: `M ${x1} ${y1} C ${x1 + 16} ${y1}, ${x2 + 16} ${y2}, ${x2} ${y2}`, labelX: Math.max(x1, x2) + 10, labelY: (y1 + y2) / 2 }]
  }),
)

const MARK_ICON: Partial<Record<TimelineEvent['kind'], string>> = { start: '●', end: '■', recv_closed: '⊘', note: '🚩' }
const eventViews = computed(() =>
  events.value.map((e, i) => ({ e, step: i + 1, icon: MARK_ICON[e.kind] ?? '', x: colX(i + 1), y: markY(e.thread) })),
)

// 每條泳道到目前這一步的最後一個事件
const lastEventOf = computed(() => {
  const map = new Map<string, TimelineEvent>()
  for (const e of events.value.slice(0, shownStep.value)) map.set(e.thread, e)
  return map
})
const namedLanes = computed(() => new Set(events.value.slice(0, shownStep.value).filter((e) => e.kind === 'start').map((e) => e.thread)))

const STATE_ICON: Record<LaneState, string> = { not_started: '', running: '▶', waiting_lock: '⏳', waiting_msg: '✉', ended: '■', deadlocked: '✗' }
function stateText(lane: TimelineLane): string {
  const last = lastEventOf.value.get(lane.name)
  switch (lane.state) {
    case 'deadlocked':
      return `死鎖，等${state.value.deadlock?.find((d) => d.lane === lane.name)?.waits ?? ''}`
    case 'waiting_lock':
      return `等${last?.target ?? ''}`
    case 'waiting_msg':
      return `等${last?.target ?? ''}的訊息`
    case 'ended':
      return '已結束'
    case 'not_started':
      return '還沒開始'
    default:
      return '執行中'
  }
}

// 具名泳道沒做完：ok 與 runtime_panic 播完時尾端畫「…」
const unfinished = computed(() => {
  if (!atEnd.value || (runStatus.value !== 'ok' && runStatus.value !== 'runtime_panic')) return []
  return state.value.lanes.filter((l) => namedLanes.value.has(l.name) && l.state !== 'ended').map((l) => l.name)
})

// timeout 播完時，等訊息的泳道若其他具名泳道都已結束就補一句
function laneNote(lane: TimelineLane): string {
  if (!atEnd.value || runStatus.value !== 'timeout' || lane.state !== 'waiting_msg') return ''
  const others = state.value.lanes.filter((l) => l.name !== lane.name && namedLanes.value.has(l.name))
  return others.length > 0 && others.every((l) => l.state === 'ended') ? '其他執行緒都已結束' : ''
}

const conclusion = computed(() => {
  if (!atEnd.value) return ''
  const parts = [state.value.conclusion]
  if ((parsed.value?.malformed ?? 0) > 0) parts.push(`有 ${parsed.value!.malformed} 行時間軸事件格式不對，已略過`)
  if (parsed.value?.capped) parts.push(`事件超過 ${MAX_TIMELINE_EVENTS.toLocaleString('en-US')} 個，後面的不顯示了`)
  return parts.filter((p) => p !== '').join('；')
})

const svgLabel = computed(
  () => `${state.value.lanes.length} 條執行緒、${locks.value.length} 把鎖、${channels.value.length} 個 channel，第 ${shownStep.value} / ${total.value} 步`,
)

// 編輯器高亮：目前事件的行；line = null 沿用前一個有行號的事件；死鎖播完時改為循環第一條泳道的等待行
const currentLine = computed<number | null>(() => {
  const first = state.value.deadlock?.[0]
  if (atEnd.value && first) return state.value.lanes.find((l) => l.name === first.lane)?.line ?? null
  for (let i = shownStep.value - 1; i >= 0; i--) {
    const line = events.value[i].line
    if (line !== null) return line
  }
  return null
})
watch(currentLine, (line) => emit('line-change', line))

function onLaneClick(lane: TimelineLane) {
  if (lane.line !== null) emit('line-change', lane.line)
}

function onEventKeydown(event: KeyboardEvent, s: number) {
  if (event.key !== 'Enter') return
  event.preventDefault()
  playback.seek(s)
}

function onKeydown(event: KeyboardEvent) {
  if (event.target !== event.currentTarget || !hasEvents.value) return
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

// 播放時把目前步捲進可見範圍
const reducedMotion = () => typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
watch(shownStep, async (s) => {
  await nextTick()
  const el = scrollEl.value
  if (!el || s < 1) return
  const x = labelW.value + colX(s)
  const visible = el.clientWidth
  if (visible <= 0 || (x >= el.scrollLeft + labelW.value && x <= el.scrollLeft + visible - colW.value)) return
  const left = Math.max(0, x - labelW.value - (visible - labelW.value) / 2)
  if (typeof el.scrollTo === 'function') el.scrollTo({ left, behavior: reducedMotion() ? 'auto' : 'smooth' })
  else el.scrollLeft = left
})
</script>

<template>
  <section
    ref="rootEl"
    class="timeline-view"
    :class="{ 'is-pending': pending, 'is-narrow': narrow }"
    :style="{ '--tl-label-w': `${labelW}px` }"
    data-testid="timeline-view"
    tabindex="0"
    aria-label="時間軸"
    @keydown="onKeydown"
  >
    <div class="tl-legend" data-testid="tl-legend">
      <div class="tl-legend-row">
        <span v-for="lock in locks" :key="`l-${lock}`" class="tl-legend-item" data-testid="tl-legend-lock">
          <span class="tl-swatch" :style="{ background: lockColor(lock) }" aria-hidden="true" />{{ lockCode(lock) }} {{ lock }}
        </span>
        <span v-for="ch in channels" :key="`c-${ch}`" class="tl-legend-item" data-testid="tl-legend-channel">✉ {{ ch }}</span>
      </div>
      <div class="tl-legend-row is-muted">
        <span>▬ 持有</span>
        <span>┄ 等待</span>
        <span>➝ 訊息</span>
        <span v-if="!narrow" class="tl-order-note" data-testid="tl-order-note">每次執行的先後順序可能不同</span>
      </div>
    </div>

    <div class="tl-stage">
      <div ref="scrollEl" class="tl-scroll">
        <div class="tl-grid" :style="{ gridTemplateColumns: `${labelW}px ${svgW}px` }">
          <div class="tl-corner" />
          <svg class="tl-ticks" :width="svgW" :height="TICK_H" aria-hidden="true">
            <text v-for="t in ticks" :key="t" class="tl-tick" data-testid="tl-tick" :x="colX(t)" :y="TICK_H - 6" text-anchor="middle">{{ t }}</text>
          </svg>

          <div class="tl-labels">
            <div
              v-for="(lane, i) in state.lanes"
              :key="lane.name"
              class="tl-label-row"
              :class="{ 'is-alt': i % 2 === 1 }"
              :style="{ height: `${rowOf(lane.name).height}px` }"
            >
              <button
                type="button"
                class="tl-lane-label"
                :class="[`is-${lane.state.replace('_', '-')}`, { 'is-deadlocked': lane.state === 'deadlocked' }]"
                data-testid="tl-lane-label"
                :data-lane="lane.name"
                :data-state="lane.state"
                :title="lane.name"
                :aria-label="`${lane.name}，${stateText(lane)}`"
                @click="onLaneClick(lane)"
              >
                <span class="tl-lane-icon" aria-hidden="true">{{ STATE_ICON[lane.state] }}</span>
                <span class="tl-lane-name">{{ lane.name }}</span>
                <span v-if="lane.state === 'deadlocked'" class="tl-lane-dead">死鎖</span>
              </button>
              <span v-if="laneNote(lane)" class="tl-lane-note" data-testid="tl-lane-note" :data-lane="lane.name">{{ laneNote(lane) }}</span>
            </div>
          </div>

          <svg class="tl-svg" :width="svgW" :height="layout.height" role="img" :aria-label="svgLabel">
            <defs>
              <marker id="tl-arrow-info" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--info)" />
              </marker>
              <marker id="tl-arrow-danger" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--danger)" />
              </marker>
            </defs>

            <!-- 泳道底色 -->
            <g v-for="(lane, i) in state.lanes" :key="`bg-${lane.name}`">
              <rect :x="0" :y="rowOf(lane.name).top" :width="svgW" :height="rowOf(lane.name).height" :fill="i % 2 === 1 ? 'var(--surface-muted)' : 'var(--surface)'" />
              <line :x1="0" :x2="svgW" :y1="rowOf(lane.name).top + rowOf(lane.name).height" :y2="rowOf(lane.name).top + rowOf(lane.name).height" stroke="var(--border)" />
            </g>

            <!-- 區段 -->
            <g
              v-for="v in segmentViews"
              :key="segKey(v.g)"
              class="tl-segment"
              :class="[`is-${v.g.kind.replace('_', '-')}`, { 'is-deadlocked': v.g.deadlocked }]"
              data-testid="tl-segment"
              :data-lane="v.g.lane"
              :data-kind="v.g.kind"
              :data-target="v.g.target"
              :data-deadlocked="String(v.g.deadlocked)"
              :data-layer="v.layer"
            >
              <title>{{ v.g.kind === 'hold' ? `${lockCode(v.g.target)} ${v.g.target}：持有` : v.g.kind === 'wait_lock' ? `${lockCode(v.g.target)} ${v.g.target}：等待` : `${v.g.target}：等訊息` }}</title>
              <rect
                v-if="v.g.deadlocked"
                class="tl-dead-frame"
                :x="v.x1 - 3"
                :y="v.y - 9"
                :width="v.x2 - v.x1 + 6"
                :height="18"
                rx="6"
              />
              <rect
                v-if="v.g.kind === 'hold'"
                :x="v.x1"
                :y="v.y - 6"
                :width="v.x2 - v.x1"
                height="12"
                rx="4"
                :fill="lockColor(v.g.target)"
                stroke="var(--ferris-ink)"
                stroke-width="2"
              />
              <rect
                v-else-if="v.g.kind === 'wait_lock'"
                :x="v.x1"
                :y="v.y - 6"
                :width="v.x2 - v.x1"
                height="12"
                rx="4"
                fill="none"
                :stroke="lockColor(v.g.target)"
                stroke-width="2"
                stroke-dasharray="4 3"
              />
              <rect
                v-else
                :x="v.x1"
                :y="v.y - 6"
                :width="v.x2 - v.x1"
                height="12"
                rx="4"
                fill="none"
                stroke="var(--info)"
                stroke-width="2"
                stroke-dasharray="1 3"
                stroke-linecap="round"
              />
              <g v-if="v.showHead" class="tl-seg-head">
                <rect :x="v.x1 - 2" :y="v.y - 7" :width="v.head.length * 10 + 4" height="14" rx="7" />
                <text :x="v.x1 + 2" :y="v.y + 4">{{ v.head }}</text>
              </g>
              <text v-if="v.g.deadlocked" class="tl-dead-mark" :x="v.x2 + 4" :y="v.y + 4">✗</text>
            </g>

            <!-- 訊息箭頭 -->
            <g
              v-for="(v, i) in messageViews"
              :key="`m-${i}`"
              class="tl-message"
              :class="{ 'is-pending': v.pending }"
              data-testid="tl-message"
              :data-pending="String(v.pending)"
            >
              <path :d="v.path" :marker-end="v.pending ? undefined : 'url(#tl-arrow-info)'" />
              <text v-if="v.pending" :x="v.tailX" :y="v.tailY">✉</text>
            </g>

            <!-- 事件標記 -->
            <g
              v-for="v in eventViews"
              :key="`e-${v.step}`"
              class="tl-event"
              :class="{ 'is-current': v.step === shownStep }"
              :opacity="v.step > shownStep ? 0.4 : undefined"
              data-testid="tl-event"
              :data-step="v.step"
              :data-kind="v.e.kind"
              role="button"
              :aria-label="`第 ${v.step} 步`"
              :tabindex="v.step <= shownStep ? 0 : undefined"
              @click="playback.seek(v.step)"
              @keydown="onEventKeydown($event, v.step)"
            >
              <title>{{ v.e.kind === 'note' ? `${v.e.thread}：${v.e.target}` : `第 ${v.step} 步` }}</title>
              <text v-if="v.icon" :x="v.x" :y="v.y + 4" text-anchor="middle">{{ v.icon }}</text>
              <circle v-else :cx="v.x" :cy="v.y" r="2.5" />
            </g>

            <!-- 沒做完的泳道尾端 -->
            <text
              v-for="name in unfinished"
              :key="`u-${name}`"
              class="tl-unfinished"
              data-testid="tl-lane-unfinished"
              :data-lane="name"
              :x="colX(shownStep) + colW / 2 + 4"
              :y="layerY(name, 0) + 4"
            >…</text>

            <!-- 死鎖箭頭（最上層） -->
            <g v-for="a in deadlockArrows" :key="`d-${a.key}`" class="tl-deadlock-arrow" data-testid="tl-deadlock-arrow">
              <path :d="a.path" marker-end="url(#tl-arrow-danger)" />
              <text :x="a.labelX" :y="a.labelY + 4">✗</text>
            </g>
          </svg>
        </div>
        <p v-if="emptyText === 'zero'" class="tl-empty" data-testid="tl-empty">
          這次執行沒有時間軸事件：要透過 <code>timeline::</code> 小幫手上鎖、送訊息，時間軸才看得到（按『重設』可取回小幫手）
        </p>
        <p v-else-if="emptyText" class="tl-empty" data-testid="tl-empty">{{ emptyText }}</p>
      </div>
    </div>

    <div class="tl-message-row" aria-live="polite">
      <p class="tl-sentence" data-testid="tl-sentence">{{ state.sentence }}</p>
      <p class="tl-conclusion" :class="{ 'is-danger': atEnd && !!state.deadlock }" data-testid="tl-conclusion">{{ conclusion }}</p>
    </div>
    <p v-if="narrow" class="tl-order-note is-below" data-testid="tl-order-note">每次執行的先後順序可能不同</p>

    <PlaybackBar
      :step="shownStep"
      :total="total"
      :playing="playing"
      :min="0"
      :disabled="!hasEvents"
      @rewind="playback.rewind()"
      @toggle="playback.toggle()"
      @step-forward="playback.stepForward()"
      @seek="playback.seek($event)"
    />
  </section>
</template>

<style scoped>
.timeline-view {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  min-width: 0;
  contain: inline-size;
  overflow: hidden;
  outline: none;
  transition: opacity 200ms;
}
.timeline-view:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: -2px;
}
.timeline-view.is-pending .tl-stage,
.timeline-view.is-pending .tl-message-row {
  opacity: 0.45;
}
.tl-legend {
  display: grid;
  gap: 2px;
  padding: 8px 12px 6px;
  border-bottom: var(--border-w) solid var(--border);
  font-size: 13px;
}
.tl-legend-row {
  display: flex;
  flex-wrap: nowrap;
  align-items: center;
  gap: 4px 12px;
  white-space: nowrap;
  overflow: hidden;
}
.timeline-view.is-narrow .tl-legend-row {
  flex-wrap: wrap;
  white-space: normal;
}
.tl-legend-row.is-muted {
  color: var(--text-muted);
  font-size: 12px;
}
.tl-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.tl-swatch {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 2px solid var(--ferris-ink);
}
.tl-order-note {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
}
.tl-order-note.is-below {
  margin: 0;
  padding: 0 12px 6px;
}
.tl-stage {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  transition: opacity 200ms;
}
.tl-scroll {
  flex: 1;
  min-width: 0;
  overflow: auto;
  overscroll-behavior: contain;
}
.tl-grid {
  display: grid;
  width: max-content;
}
.tl-corner,
.tl-ticks {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface);
  border-bottom: var(--border-w) solid var(--border);
}
.tl-corner {
  left: 0;
  z-index: 2;
}
.tl-tick {
  font-size: 10px;
  fill: var(--text-muted);
}
.tl-labels {
  position: sticky;
  left: 0;
  z-index: 1;
  background: var(--surface);
  border-right: var(--border-w) solid var(--border);
}
.tl-label-row {
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 0 6px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}
.tl-label-row.is-alt {
  background: var(--surface-muted);
}
.tl-lane-label {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  min-width: 0;
  padding: 2px 4px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}
.tl-lane-label:hover {
  border-color: var(--border-strong);
}
.tl-lane-label:focus-visible {
  outline: 2px solid var(--focus-ring);
}
.tl-lane-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tl-lane-icon {
  flex: none;
  width: 1em;
  text-align: center;
}
.tl-lane-label.is-not-started {
  opacity: 0.4;
}
.tl-lane-label.is-ended {
  color: var(--text-muted);
}
.tl-lane-label.is-deadlocked {
  color: var(--danger);
  font-weight: 700;
}
.tl-lane-dead {
  flex: none;
  font-size: 11px;
}
.tl-lane-note {
  font-size: 10px;
  line-height: 1.2;
  color: var(--text-muted);
}
.timeline-view.is-narrow .tl-lane-dead,
.timeline-view.is-narrow .tl-lane-note {
  display: none;
}
.tl-svg {
  display: block;
}
.tl-seg-head rect {
  fill: var(--surface);
  stroke: var(--ferris-ink);
  stroke-width: 1;
}
.tl-seg-head text {
  font-size: 10px;
  fill: var(--text);
}
.tl-dead-frame {
  fill: none;
  stroke: var(--danger);
  stroke-width: 3;
}
.tl-dead-mark,
.tl-deadlock-arrow text {
  font-size: 12px;
  font-weight: 700;
  fill: var(--danger);
}
.tl-message path {
  fill: none;
  stroke: var(--info);
  stroke-width: 2;
}
.tl-message.is-pending path {
  stroke-dasharray: 4 3;
}
.tl-message text {
  font-size: 11px;
  fill: var(--info);
}
.tl-deadlock-arrow path {
  fill: none;
  stroke: var(--danger);
  stroke-width: 2;
  stroke-dasharray: 5 3;
}
.tl-event {
  cursor: pointer;
  outline: none;
}
.tl-event text {
  font-size: 9px;
  fill: var(--text);
}
.tl-event circle {
  fill: var(--text-muted);
}
.tl-event.is-current circle {
  fill: var(--brand);
}
.tl-event:focus-visible text,
.tl-event:focus-visible circle {
  outline: 2px solid var(--focus-ring);
  stroke: var(--focus-ring);
  stroke-width: 2;
}
.tl-unfinished {
  font-size: 14px;
  font-weight: 700;
  fill: var(--text-muted);
}
.tl-empty {
  position: absolute;
  inset: 40% 12px auto;
  margin: 0;
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
  pointer-events: none;
}
.tl-empty code {
  font-family: var(--font-code);
  font-size: 0.92em;
  padding: 0 4px;
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
}
.tl-message-row {
  padding: 6px 12px;
  border-top: var(--border-w) solid var(--border);
  font-size: 13px;
  min-height: 52px;
  transition: opacity 200ms;
}
.tl-message-row p {
  margin: 0;
  min-height: 1.4em;
}
.tl-conclusion {
  font-weight: 700;
}
.tl-conclusion.is-danger {
  color: var(--danger);
}
</style>
