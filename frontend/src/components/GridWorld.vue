<script setup lang="ts">
import { computed, onBeforeUnmount, shallowRef, watch } from 'vue'
import type { GridDef, Point, RunStatus } from '../api/types'
import { usePlayback } from '../composables/usePlayback'
import { parseGridCommands } from '../domain/grid/commands'
import type { Direction } from '../domain/grid/commands'
import { ferrisBadge, finalMessage } from '../domain/grid/messages'
import { samePoint, simulate } from '../domain/grid/simulate'
import { validateGrid } from '../domain/grid/validate'
import { usePlaybackStore } from '../stores/playback'
import Ferris from './Ferris.vue'
import type { FerrisMood } from './Ferris.vue'
import PlaybackBar from './PlaybackBar.vue'

const props = defineProps<{
  grid: GridDef
  // null 表示尚未執行，顯示初始地圖
  stdout: string | null
  status: RunStatus | null
  truncated: boolean
  // Run／Submit 請求中：停止播放、回到第 0 步並半透明
  pending: boolean
}>()

const C = 64
const AUTOPLAY: RunStatus[] = ['ok', 'runtime_panic', 'timeout', 'memory_limit', 'output_limit']

const playbackStore = usePlaybackStore()

const issues = computed(() => validateGrid(props.grid))
const isDev = import.meta.env.DEV
watch(
  issues,
  (list) => {
    if (list.length > 0 && import.meta.env.DEV) console.error('[GridWorld] 地圖設定有誤', list)
  },
  { immediate: true },
)

// 畫面依據的 Run 結果；請求中沿用上次 Run，總步數不歸零
const shown = shallowRef({ stdout: props.stdout, status: props.status, truncated: props.truncated })
const commands = computed(() => (shown.value.stdout === null ? [] : parseGridCommands(shown.value.stdout, shown.value.truncated)))
const sim = computed(() => simulate(props.grid, commands.value))
const endStep = computed(() => sim.value.states.length - 1)

const playback = usePlayback({
  total: () => sim.value.totalSteps,
  end: () => endStep.value,
  rewindTo: 0,
  stepMs: () => playbackStore.stepMs,
})
const { step, playing } = playback

watch(
  () => [props.pending, props.stdout, props.status, props.truncated] as const,
  ([pending, stdout, status, truncated], [, oldStdout, oldStatus, oldTruncated]) => {
    if (pending) {
      playback.rewind()
      return
    }
    shown.value = { stdout, status, truncated }
    const runChanged = stdout !== oldStdout || status !== oldStatus || truncated !== oldTruncated
    if (!runChanged) {
      // 請求結束但沒有新的 Run 結果：還原到上次 Run 的終點
      playback.seek(endStep.value)
      return
    }
    if (stdout !== null && status !== null && AUTOPLAY.includes(status)) playback.playFromStart()
    else playback.rewind()
  },
)

onBeforeUnmount(() => playback.dispose())

const shownStep = computed(() => (props.pending ? 0 : Math.min(step.value, endStep.value)))
const state = computed(() => sim.value.states[shownStep.value])
const hasRun = computed(() => shown.value.stdout !== null)
const atEnd = computed(() => hasRun.value && !props.pending && !playing.value && shownStep.value === endStep.value)
const stop = computed(() => (atEnd.value ? sim.value.stop : null))
const runStatus = computed<RunStatus>(() => shown.value.status ?? 'ok')

const message = computed(() => {
  if (props.pending) return ''
  if (!hasRun.value) return shown.value.status !== null ? '程式還沒編譯成功' : ''
  if (!atEnd.value) return ''
  return finalMessage(sim.value, props.grid, runStatus.value) ?? ''
})
const badge = computed(() => (atEnd.value ? ferrisBadge(sim.value, runStatus.value) : null))
const cleared = computed(() => atEnd.value && sim.value.cleared)

const mood = computed<FerrisMood>(() => {
  if (cleared.value) return 'happy'
  const reason = stop.value?.reason
  if (reason === 'wall' || reason === 'door' || reason === 'out_of_bounds') return 'dizzy'
  if (reason === 'invalid' || reason === 'pick_empty' || reason === 'open_nothing') return 'confused'
  return 'normal'
})

// 撞擊方向（撞牆、撞門、走出地圖）
const bumpDir = computed<Direction | null>(() => {
  const s = stop.value
  if (!s || !(s.reason === 'wall' || s.reason === 'door' || s.reason === 'out_of_bounds')) return null
  return s.command?.kind === 'move' ? s.command.dir : null
})

const facingLeft = computed(() => {
  for (let i = shownStep.value - 1; i >= 0; i--) {
    const cmd = commands.value[i]
    if (cmd?.kind === 'move' && (cmd.dir === 'LEFT' || cmd.dir === 'RIGHT')) return cmd.dir === 'LEFT'
  }
  return false
})

const cells = computed(() => {
  const list: { x: number; y: number; alt: boolean }[] = []
  for (let y = 0; y < props.grid.height; y++) for (let x = 0; x < props.grid.width; x++) list.push({ x, y, alt: (x + y) % 2 === 1 })
  return list
})

const isHitWall = (p: Point) => stop.value?.reason === 'wall' && stop.value.target !== null && samePoint(stop.value.target, p)
const isOpen = (p: Point) => state.value.openDoors.some((d) => samePoint(d, p))
const pickedCount = computed(() => props.grid.gems.length - state.value.gemsLeft.length)

const edgeSide = computed(() => (stop.value?.reason === 'out_of_bounds' ? bumpDir.value : null))

const svgStyle = computed(() => ({
  maxWidth: `${props.grid.width * C}px`,
  minWidth: `${props.grid.width * 32}px`,
}))

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
</script>

<template>
  <section
    class="grid-world"
    :class="{ 'is-pending': pending }"
    data-testid="grid-world"
    tabindex="0"
    aria-label="地圖"
    @keydown="onKeydown"
  >
    <div v-if="issues.length > 0" class="grid-config-error" data-testid="grid-config-error" role="alert">
      <strong>此關卡地圖設定有誤</strong>
      <ul v-if="isDev">
        <li v-for="(issue, i) in issues" :key="i">{{ issue.kind }} {{ issue.point ? `(${issue.point[0]}, ${issue.point[1]})` : '' }}</li>
      </ul>
    </div>
    <template v-else>
      <div class="grid-stage">
        <span :key="pickedCount" class="gem-counter" data-testid="gem-counter">💎 {{ pickedCount }} / {{ grid.gems.length }}</span>
        <svg
          class="grid-svg"
          :viewBox="`-8 -8 ${grid.width * C + 16} ${grid.height * C + 16}`"
          :style="svgStyle"
          role="img"
          :aria-label="`${grid.width} × ${grid.height} 地圖`"
        >
          <rect class="grid-edge" x="-6" y="-6" :width="grid.width * C + 12" :height="grid.height * C + 12" rx="26" />
          <rect
            v-if="edgeSide"
            class="edge-flash"
            :class="`edge-${edgeSide.toLowerCase()}`"
            :x="edgeSide === 'RIGHT' ? grid.width * C : -6"
            :y="edgeSide === 'DOWN' ? grid.height * C : -6"
            :width="edgeSide === 'UP' || edgeSide === 'DOWN' ? grid.width * C + 12 : 6"
            :height="edgeSide === 'LEFT' || edgeSide === 'RIGHT' ? grid.height * C + 12 : 6"
            rx="3"
          />

          <!-- 地板 -->
          <g v-for="c in cells" :key="`f${c.x}-${c.y}`" :transform="`translate(${c.x * C} ${c.y * C})`">
            <rect x="3" y="3" :width="C - 6" :height="C - 6" rx="14" :fill="c.alt ? 'var(--grid-floor-2)' : 'var(--grid-floor)'" />
            <circle cx="17" cy="45" r="2" fill="var(--grid-floor-deco)" />
            <circle cx="47" cy="19" r="1.6" fill="var(--grid-floor-deco)" />
          </g>

          <!-- 牆 -->
          <g
            v-for="w in grid.walls"
            :key="`w${w[0]}-${w[1]}`"
            class="wall"
            :class="{ 'is-hit': isHitWall(w) }"
            data-testid="wall"
            :data-x="w[0]"
            :data-y="w[1]"
            :data-hit="String(isHitWall(w))"
            :transform="`translate(${w[0] * C} ${w[1] * C})`"
          >
            <path d="M11 49 C5 39 11 22 24 18 C31 9 46 11 50 22 C59 27 59 46 51 52 C40 57 21 57 11 49 Z" fill="var(--grid-wall)" stroke="var(--ferris-ink)" stroke-width="2" />
            <path d="M22 27 C26 22 31 22 34 24" stroke="var(--grid-wall-2)" stroke-width="4" stroke-linecap="round" fill="none" />
            <circle cx="44" cy="40" r="3" fill="var(--grid-wall-2)" />
            <rect v-if="isHitWall(w)" class="hit-frame" x="2" y="2" :width="C - 4" :height="C - 4" rx="14" />
          </g>

          <!-- 門 -->
          <g
            v-for="d in grid.doors"
            :key="`d${d[0]}-${d[1]}`"
            class="door"
            :class="{ 'is-open': isOpen(d) }"
            data-testid="door"
            :data-x="d[0]"
            :data-y="d[1]"
            :data-open="String(isOpen(d))"
            :transform="`translate(${d[0] * C} ${d[1] * C})`"
          >
            <template v-if="isOpen(d)">
              <rect x="14" y="8" width="36" height="50" rx="12" fill="none" stroke="var(--grid-door)" stroke-width="3" stroke-dasharray="5 4" opacity="0.6" />
            </template>
            <template v-else>
              <path d="M14 58 L14 20 C14 10 22 6 32 6 C42 6 50 10 50 20 L50 58 Z" fill="var(--grid-door)" stroke="var(--ferris-ink)" stroke-width="2" stroke-linejoin="round" />
              <path d="M24 16 L24 54 M32 10 L32 54 M40 16 L40 54" stroke="var(--ferris-ink)" stroke-width="1" opacity="0.35" />
              <text x="32" y="44" text-anchor="middle" font-size="18">🔒</text>
            </template>
          </g>

          <!-- 終點 -->
          <g
            class="goal"
            :class="{ 'is-waving': cleared }"
            data-testid="goal"
            :data-waving="String(cleared)"
            :transform="`translate(${grid.goal[0] * C} ${grid.goal[1] * C})`"
          >
            <ellipse cx="32" cy="54" rx="14" ry="4" fill="var(--grid-floor-deco)" />
            <path d="M22 54 L22 10" stroke="var(--ferris-ink)" stroke-width="3" stroke-linecap="round" />
            <path class="flag" d="M23 11 C32 6 38 16 48 11 L48 29 C38 34 32 24 23 29 Z" fill="var(--grid-goal)" stroke="var(--ferris-ink)" stroke-width="2" stroke-linejoin="round" />
          </g>

          <!-- 寶石 -->
          <TransitionGroup name="gem" tag="g">
            <g
              v-for="g in state.gemsLeft"
              :key="`g${g[0]}-${g[1]}`"
              data-testid="gem"
              :data-x="g[0]"
              :data-y="g[1]"
              :transform="`translate(${g[0] * C} ${g[1] * C})`"
            >
              <g class="gem">
                <path d="M20 26 L26 18 L38 18 L44 26 L32 46 Z" fill="var(--grid-gem)" stroke="var(--ferris-ink)" stroke-width="2" stroke-linejoin="round" />
                <path d="M20 26 L44 26 M26 18 L32 26 L38 18 M32 26 L32 46" stroke="var(--ferris-ink)" stroke-width="1" opacity="0.45" fill="none" />
                <path d="M48 12 L50 16 L54 18 L50 20 L48 24 L46 20 L42 18 L46 16 Z" fill="#ffe07a" />
              </g>
            </g>
          </TransitionGroup>

          <!-- Ferris -->
          <g
            class="ferris-pos"
            data-testid="ferris"
            :data-x="state.pos[0]"
            :data-y="state.pos[1]"
            :data-mood="mood"
            :style="{ transform: `translate(${state.pos[0] * C}px, ${state.pos[1] * C}px)` }"
          >
            <g :key="`${shownStep}-${atEnd}`" class="ferris-fx" :class="[bumpDir ? `bump-${bumpDir.toLowerCase()}` : '', { 'is-jumping': cleared }]">
              <g :transform="facingLeft ? `translate(${C} 0) scale(-1 1)` : undefined">
                <g transform="translate(2 12)">
                  <Ferris :mood="mood" :size="60" />
                </g>
              </g>
            </g>
            <!-- 錯誤標記在最上層 -->
            <g v-if="badge" class="ferris-badge" data-testid="ferris-badge">
              <rect x="34" y="-14" width="28" height="24" rx="12" fill="var(--surface)" stroke="var(--border-strong)" stroke-width="2" />
              <text x="48" y="3" text-anchor="middle" font-size="14">{{ badge }}</text>
            </g>
          </g>
        </svg>
      </div>
      <p class="grid-message" :class="{ 'is-bad': !!stop || (atEnd && !cleared) }" data-testid="grid-message" aria-live="polite">{{ message }}</p>
      <PlaybackBar
        :step="shownStep"
        :total="sim.totalSteps"
        :playing="playing"
        :min="0"
        @rewind="playback.rewind()"
        @toggle="playback.toggle()"
        @step-forward="playback.stepForward()"
        @seek="playback.seek($event)"
      />
    </template>
  </section>
</template>

<style scoped>
.grid-world {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  outline: none;
  transition: opacity 200ms;
}
.grid-world:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: -2px;
}
.grid-world.is-pending .grid-stage {
  opacity: 0.45;
}
.grid-stage {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 28px 12px 8px;
  overflow: auto;
}
.grid-svg {
  width: 100%;
  height: auto;
  max-height: 100%;
}
.gem-counter {
  position: absolute;
  top: 6px;
  right: 10px;
  font-family: var(--font-code);
  font-size: 13px;
  padding: 2px 10px;
  border-radius: var(--radius-pill);
  background: var(--surface-muted);
  border: var(--border-w) solid var(--border);
  animation: pop 300ms var(--move-ease);
}
.grid-edge {
  fill: var(--grid-edge);
}
.edge-flash {
  fill: var(--danger);
  animation: flash 300ms ease-in-out 2;
}
.wall .hit-frame {
  fill: none;
  stroke: var(--danger);
  stroke-width: 4;
  animation: flash 300ms ease-in-out 2;
}
.goal.is-waving .flag {
  transform-box: fill-box;
  transform-origin: left center;
  animation: wave 600ms ease-in-out infinite alternate;
}
.ferris-pos {
  transition: transform var(--move-dur) var(--move-ease);
}
.ferris-fx {
  transform-box: fill-box;
}
.ferris-fx.bump-right {
  animation: bump-x 520ms ease-out;
  --bump: 19px;
}
.ferris-fx.bump-left {
  animation: bump-x 520ms ease-out;
  --bump: -19px;
}
.ferris-fx.bump-down {
  animation: bump-y 520ms ease-out;
  --bump: 19px;
}
.ferris-fx.bump-up {
  animation: bump-y 520ms ease-out;
  --bump: -19px;
}
.ferris-fx.is-jumping {
  animation: jump 500ms var(--move-ease);
}
.gem-leave-active {
  transition: opacity 200ms, transform 200ms;
}
.gem-leave-to {
  opacity: 0;
}
.grid-message {
  margin: 0;
  min-height: 1.6em;
  padding: 6px 12px;
  font-size: 14px;
  color: var(--text);
}
.grid-message.is-bad {
  color: var(--danger);
}
.grid-config-error {
  margin: 12px;
  padding: 12px 16px;
  border: var(--border-w-strong) solid var(--danger);
  border-radius: var(--radius);
  color: var(--danger);
}
@keyframes pop {
  50% {
    transform: scale(1.18);
  }
}
@keyframes flash {
  50% {
    opacity: 0.15;
  }
}
@keyframes wave {
  to {
    transform: skewY(-8deg) scaleX(0.9);
  }
}
@keyframes bump-x {
  30% {
    transform: translateX(var(--bump));
  }
  55% {
    transform: translateX(0);
  }
  65% {
    transform: translateX(-4px);
  }
  75% {
    transform: translateX(4px);
  }
  85% {
    transform: translateX(-4px);
  }
  95% {
    transform: translateX(4px);
  }
}
@keyframes bump-y {
  30% {
    transform: translateY(var(--bump));
  }
  55% {
    transform: translateY(0);
  }
  65% {
    transform: translateX(-4px);
  }
  75% {
    transform: translateX(4px);
  }
  85% {
    transform: translateX(-4px);
  }
  95% {
    transform: translateX(4px);
  }
}
@keyframes jump {
  40% {
    transform: translateY(-14px);
  }
}
@media (prefers-reduced-motion: reduce) {
  .ferris-pos {
    transition: none;
  }
  .ferris-fx,
  .goal.is-waving .flag,
  .edge-flash,
  .wall .hit-frame,
  .gem-counter {
    animation: none !important;
  }
}
</style>
