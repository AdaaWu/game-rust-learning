<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import type { GridDef } from '../lib/api'
import { buildFrames, initialFrame, isCleared, key, parseCommands, type Frame } from '../lib/grid'
import Ferris, { type Mood } from './Ferris.vue'

const props = defineProps<{ grid: GridDef; stdout: string | null; look: 'A' | 'B' | 'C'; overlay: string | null; playToken: number }>()
const emit = defineEmits<{ (e: 'line', line: number | null): void; (e: 'finished', f: Frame, cleared: boolean): void }>()

const C = 64
const frames = ref<Frame[]>([initialFrame(props.grid)])
const step = ref(0)
const playing = ref(false)
const speed = ref(1)
let timer: number | undefined

const frame = computed(() => frames.value[step.value])
const total = computed(() => frames.value.length - 1)
const atEnd = computed(() => step.value >= total.value)
const halted = computed(() => {
  const ev = frame.value.event
  return ev !== null && ev !== 'move' && ev !== 'pick' && ev !== 'open'
})
const cleared = computed(() => atEnd.value && total.value > 0 && isCleared(props.grid, frame.value))

const mood = computed<Mood>(() => {
  const ev = frame.value.event
  if (ev === 'wall' || ev === 'out_of_bounds' || ev === 'door') return 'dizzy'
  if (ev === 'invalid' || ev === 'pick_empty' || ev === 'open_nothing') return 'confused'
  if (cleared.value || ev === 'pick') return 'happy'
  if (atEnd.value && total.value > 0) return 'sweat'
  return 'normal'
})
const badge = computed(() => {
  const ev = frame.value.event
  if (ev === 'wall' || ev === 'out_of_bounds' || ev === 'door') return '💥'
  if (ev === 'invalid' || ev === 'pick_empty' || ev === 'open_nothing') return '❓'
  if (cleared.value) return '★'
  if (atEnd.value && total.value > 0) return '…'
  return ''
})
const message = computed(() => {
  if (props.overlay) return props.overlay
  if (cleared.value) return '抵達終點、寶石全拿！按「送出」驗收'
  if (atEnd.value && total.value > 0 && !halted.value) return '指令跑完了，但 Ferris 還沒到終點'
  return frame.value.message
})

const ferrisXY = computed(() => ({ x: frame.value.pos[0] * C, y: frame.value.pos[1] * C }))
const isWall = (x: number, y: number) => props.grid.walls.some((w) => w[0] === x && w[1] === y)
const cells = computed(() => {
  const out: { x: number; y: number; alt: boolean }[] = []
  for (let y = 0; y < props.grid.height; y++) for (let x = 0; x < props.grid.width; x++) out.push({ x, y, alt: (x + y) % 2 === 1 })
  return out
})
const gemsLeft = computed(() => frame.value.picked.length)

function stop() {
  playing.value = false
  window.clearTimeout(timer)
}
function tick() {
  if (atEnd.value) {
    stop()
    emit('finished', frame.value, cleared.value)
    return
  }
  step.value++
  emit('line', frame.value.line)
  timer = window.setTimeout(tick, 500 / speed.value)
}
function play() {
  if (atEnd.value) step.value = 0
  playing.value = true
  timer = window.setTimeout(tick, 250 / speed.value)
}
function toggle() {
  playing.value ? stop() : play()
}
function stepOnce() {
  stop()
  if (!atEnd.value) {
    step.value++
    emit('line', frame.value.line)
    if (atEnd.value) emit('finished', frame.value, cleared.value)
  }
}
function rewind() {
  stop()
  step.value = 0
  emit('line', null)
}
function seek(e: Event) {
  stop()
  step.value = Number((e.target as HTMLInputElement).value)
  emit('line', frame.value.line)
}

watch(
  () => [props.stdout, props.playToken],
  () => {
    stop()
    frames.value = props.stdout === null ? [initialFrame(props.grid)] : buildFrames(props.grid, parseCommands(props.stdout))
    step.value = 0
    if (props.stdout !== null && total.value > 0) play()
  },
)
function onKey(e: KeyboardEvent) {
  if (e.key === ' ') { e.preventDefault(); toggle() }
  else if (e.key === 'ArrowRight') { e.preventDefault(); stepOnce() }
  else if (e.key === 'Home') { e.preventDefault(); rewind() }
}
onBeforeUnmount(stop)
</script>

<template>
  <section class="grid-world" :class="`look-${look}`" tabindex="0" aria-label="Ferris 地圖" @keydown="onKey">
    <div class="gw-head">
      <span class="gw-title">{{ look === 'C' ? '// world.grid' : look === 'A' ? 'WORLD 1-2' : '沙灘小徑' }}</span>
      <span class="gem-count" :key="gemsLeft">💎 {{ gemsLeft }} / {{ grid.gems.length }}</span>
    </div>
    <div class="gw-stage" :class="{ dim: overlay }">
      <svg :viewBox="`-8 -8 ${grid.width * C + 16} ${grid.height * C + 16}`" class="gw-svg" :shape-rendering="look === 'A' ? 'crispEdges' : 'auto'">
        <defs>
          <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2.5" result="b" />
            <feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge>
          </filter>
          <pattern id="hatch" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
            <line x1="0" y1="0" x2="0" y2="8" stroke="var(--grid-wall)" stroke-width="2" />
          </pattern>
        </defs>

        <!-- 外框 -->
        <rect v-if="look === 'B'" x="-6" y="-6" :width="grid.width * C + 12" :height="grid.height * C + 12" rx="26" fill="var(--grid-edge)" />
        <rect v-else-if="look === 'A'" x="-6" y="-6" :width="grid.width * C + 12" :height="grid.height * C + 12" fill="var(--grid-edge)" />
        <rect v-else x="-4" y="-4" :width="grid.width * C + 8" :height="grid.height * C + 8" fill="none" stroke="var(--accent)" stroke-width="1.5" rx="4" filter="url(#glow)" />

        <!-- 地板 -->
        <g v-for="c in cells" :key="`f${c.x}-${c.y}`" :transform="`translate(${c.x * C} ${c.y * C})`">
          <template v-if="look === 'A'">
            <rect :width="C" :height="C" :fill="c.alt ? 'var(--grid-floor-2)' : 'var(--grid-floor)'" />
            <rect v-if="!isWall(c.x, c.y)" x="12" y="40" width="4" height="8" fill="var(--grid-floor-deco)" />
            <rect v-if="!isWall(c.x, c.y)" x="16" y="44" width="4" height="4" fill="var(--grid-floor-deco)" />
            <rect v-if="!isWall(c.x, c.y) && c.alt" x="44" y="16" width="4" height="4" fill="var(--grid-floor-deco)" />
          </template>
          <template v-else-if="look === 'B'">
            <rect x="3" y="3" :width="C - 6" :height="C - 6" rx="14" :fill="c.alt ? 'var(--grid-floor-2)' : 'var(--grid-floor)'" />
            <circle cx="18" cy="44" r="2" fill="var(--grid-floor-deco)" />
            <circle cx="46" cy="20" r="1.6" fill="var(--grid-floor-deco)" />
          </template>
          <template v-else>
            <rect :width="C" :height="C" fill="var(--grid-floor)" stroke="var(--grid-line)" stroke-width="1" />
            <circle :cx="C / 2" :cy="C / 2" r="1.2" fill="var(--grid-line)" />
          </template>
        </g>

        <!-- 牆 -->
        <g v-for="w in grid.walls" :key="`w${w[0]}-${w[1]}`" :transform="`translate(${w[0] * C} ${w[1] * C})`" class="wall"
           :class="{ flash: frame.event === 'wall' && frame.bumpDir && key([frame.pos[0] + (frame.bumpDir === 'RIGHT' ? 1 : frame.bumpDir === 'LEFT' ? -1 : 0), frame.pos[1] + (frame.bumpDir === 'DOWN' ? 1 : frame.bumpDir === 'UP' ? -1 : 0)]) === key(w) }">
          <template v-if="look === 'A'">
            <rect :width="C" :height="C" fill="var(--grid-wall)" />
            <rect x="0" y="0" width="30" height="14" fill="var(--grid-wall-2)" />
            <rect x="34" y="0" width="30" height="14" fill="var(--grid-wall-2)" />
            <rect x="0" y="18" width="14" height="14" fill="var(--grid-wall-2)" />
            <rect x="18" y="18" width="28" height="14" fill="var(--grid-wall-2)" />
            <rect x="50" y="18" width="14" height="14" fill="var(--grid-wall-2)" />
            <rect x="0" y="36" width="30" height="14" fill="var(--grid-wall-2)" />
            <rect x="34" y="36" width="30" height="14" fill="var(--grid-wall-2)" />
            <rect x="0" y="54" width="64" height="10" fill="var(--grid-wall-2)" />
          </template>
          <template v-else-if="look === 'B'">
            <path d="M10 50 C4 40 10 22 24 18 C30 8 46 10 50 22 C60 26 60 46 52 52 C40 58 20 58 10 50 Z" fill="var(--grid-wall)" stroke="var(--ferris-ink)" stroke-width="2" />
            <path d="M22 26 C26 22 32 22 34 24" stroke="var(--grid-wall-2)" stroke-width="4" stroke-linecap="round" fill="none" />
            <circle cx="44" cy="40" r="3" fill="var(--grid-wall-2)" />
          </template>
          <template v-else>
            <rect x="4" y="4" :width="C - 8" :height="C - 8" fill="url(#hatch)" stroke="var(--grid-wall)" stroke-width="1.5" rx="2" />
          </template>
        </g>

        <!-- 終點 -->
        <g :transform="`translate(${grid.goal[0] * C} ${grid.goal[1] * C})`" class="goal" :class="{ wave: cleared }">
          <template v-if="look === 'A'">
            <rect x="18" y="8" width="4" height="48" fill="var(--ferris-ink)" />
            <rect x="22" y="8" width="24" height="16" fill="var(--grid-goal)" />
            <rect x="30" y="12" width="8" height="8" fill="var(--grid-floor)" />
            <rect x="12" y="52" width="16" height="4" fill="var(--ferris-ink)" />
          </template>
          <template v-else-if="look === 'B'">
            <ellipse cx="32" cy="54" rx="14" ry="4" fill="var(--grid-floor-deco)" />
            <path d="M22 54 L22 10" stroke="var(--ferris-ink)" stroke-width="3" stroke-linecap="round" />
            <path class="flag" d="M23 11 C32 6 38 16 48 11 L48 29 C38 34 32 24 23 29 Z" fill="var(--grid-goal)" stroke="var(--ferris-ink)" stroke-width="2" stroke-linejoin="round" />
          </template>
          <template v-else>
            <rect x="10" y="10" width="44" height="44" fill="none" stroke="var(--grid-goal)" stroke-width="2" stroke-dasharray="6 4" filter="url(#glow)" />
            <text x="32" y="38" text-anchor="middle" font-family="JetBrains Mono, monospace" font-size="13" font-weight="700" fill="var(--grid-goal)">EXIT</text>
          </template>
        </g>

        <!-- 寶石 -->
        <g v-for="g in grid.gems" :key="`g${g[0]}-${g[1]}`" :transform="`translate(${g[0] * C} ${g[1] * C})`">
          <g class="gem" :class="{ gone: frame.picked.includes(key(g)) }">
            <template v-if="look === 'A'">
              <rect x="28" y="16" width="8" height="4" fill="var(--grid-gem)" />
              <rect x="24" y="20" width="16" height="4" fill="var(--grid-gem)" />
              <rect x="20" y="24" width="24" height="8" fill="var(--grid-gem)" />
              <rect x="24" y="32" width="16" height="4" fill="var(--grid-gem)" />
              <rect x="28" y="36" width="8" height="8" fill="var(--grid-gem)" />
              <rect x="24" y="24" width="4" height="4" fill="#fff" />
            </template>
            <template v-else-if="look === 'B'">
              <path d="M20 26 L26 18 L38 18 L44 26 L32 46 Z" fill="var(--grid-gem)" stroke="var(--ferris-ink)" stroke-width="2" stroke-linejoin="round" />
              <path d="M20 26 L44 26 M26 18 L32 26 L38 18 M32 26 L32 46" stroke="var(--ferris-ink)" stroke-width="1" opacity=".45" fill="none" />
              <path d="M48 12 L50 16 L54 18 L50 20 L48 24 L46 20 L42 18 L46 16 Z" fill="#FFE07A" />
            </template>
            <template v-else>
              <path d="M32 14 L48 32 L32 50 L16 32 Z" fill="none" stroke="var(--grid-gem)" stroke-width="2.5" filter="url(#glow)" />
              <path d="M32 22 L40 32 L32 42 L24 32 Z" fill="var(--grid-gem)" opacity=".35" />
            </template>
          </g>
        </g>

        <!-- Ferris -->
        <g class="ferris-pos" :style="{ transform: `translate(${ferrisXY.x}px, ${ferrisXY.y}px)` }">
          <g :class="['ferris-fx', frame.bumpDir ? `bump-${frame.bumpDir}` : '', cleared ? 'jump' : '']" :key="`${step}-${frame.event}`">
            <g :transform="frame.facing === 'left' ? `translate(${C} 0) scale(-1 1)` : ''">
              <g transform="translate(2 12)"><Ferris :look="look" :mood="mood" :size="60" /></g>
            </g>
          </g>
          <g v-if="badge" class="badge">
            <rect x="34" y="-14" width="26" height="22" :rx="look === 'A' ? 0 : look === 'B' ? 11 : 3" fill="var(--surface)" stroke="var(--border-strong)" stroke-width="2" />
            <text x="47" y="2" text-anchor="middle" font-size="14">{{ badge }}</text>
          </g>
        </g>
      </svg>
      <div v-if="overlay" class="gw-overlay">{{ overlay }}</div>
    </div>

    <div class="gw-msg" :class="{ bad: halted, good: cleared }" aria-live="polite">{{ message }}</div>

    <div class="playback">
      <button class="pb" aria-label="從頭" title="從頭（Home）" @click="rewind">⏮</button>
      <button class="pb main" :aria-label="playing ? '暫停' : '播放'" title="播放／暫停（Space）" :disabled="total === 0" @click="toggle">{{ playing ? '⏸' : '▶' }}</button>
      <button class="pb" aria-label="單步前進" title="單步（→）" :disabled="atEnd" @click="stepOnce">⏭</button>
      <input class="seek" type="range" min="0" :max="total" :value="step" aria-label="播放進度" @input="seek" />
      <span class="pb-count">{{ step }} / {{ total }}</span>
      <select v-model.number="speed" class="pb-speed" aria-label="播放速度">
        <option :value="0.5">0.5x</option>
        <option :value="1">1x</option>
        <option :value="2">2x</option>
        <option :value="4">4x</option>
      </select>
    </div>
  </section>
</template>

<style scoped>
.grid-world { display: flex; flex-direction: column; gap: 8px; min-height: 0; outline: none; }
.grid-world:focus-visible { box-shadow: 0 0 0 2px var(--focus); }
.gw-head { display: flex; justify-content: space-between; align-items: center; font-size: 13px; color: var(--muted); }
.gw-title { font-family: var(--font-title); font-size: var(--title-size-sm); color: var(--text); }
.gem-count { font-family: var(--font-code); padding: 2px 8px; border-radius: var(--radius-sm); background: var(--surface-2); border: var(--border-w) solid var(--border); animation: pop .3s; }
.gw-stage { position: relative; flex: 1; min-height: 0; display: flex; align-items: center; justify-content: center; }
.gw-stage.dim .gw-svg { opacity: .35; filter: grayscale(.6); }
.gw-svg { width: 100%; max-height: 100%; overflow: visible; transition: opacity .2s; }
.gw-overlay { position: absolute; padding: 8px 14px; background: var(--surface); border: var(--border-w) solid var(--border-strong); border-radius: var(--radius-sm); font-size: 14px; box-shadow: var(--shadow); }
.ferris-pos { transition: transform var(--move-dur) var(--move-ease); }
.ferris-fx { transform-box: view-box; }
.bump-RIGHT { animation: bumpR .5s; } .bump-LEFT { animation: bumpL .5s; }
.bump-UP { animation: bumpU .5s; } .bump-DOWN { animation: bumpD .5s; }
@keyframes bumpR { 30% { transform: translateX(19px); } 55% { transform: translateX(-4px); } 70% { transform: translateX(4px); } 85% { transform: translateX(-3px); } }
@keyframes bumpL { 30% { transform: translateX(-19px); } 55% { transform: translateX(4px); } 70% { transform: translateX(-4px); } 85% { transform: translateX(3px); } }
@keyframes bumpU { 30% { transform: translateY(-19px); } 60% { transform: translateX(-4px); } 80% { transform: translateX(4px); } }
@keyframes bumpD { 30% { transform: translateY(19px); } 60% { transform: translateX(-4px); } 80% { transform: translateX(4px); } }
.jump { animation: jump .7s 2; }
@keyframes jump { 40% { transform: translateY(-18px); } }
.wall.flash { animation: flash .5s 2; }
@keyframes flash { 50% { opacity: .3; } }
.gem { transform-box: fill-box; transform-origin: center; transition: transform .25s, opacity .25s; }
.gem.gone { transform: scale(1.6); opacity: 0; }
.goal.wave .flag { animation: wave .6s infinite alternate; transform-box: fill-box; transform-origin: left center; }
@keyframes wave { to { transform: skewY(-8deg); } }
@keyframes pop { 50% { transform: scale(1.15); } }
.gw-msg { font-size: 13px; padding: 6px 10px; border-radius: var(--radius-sm); background: var(--surface-2); border-left: 4px solid var(--border-strong); min-height: 1.6em; }
.gw-msg.bad { border-left-color: var(--danger); color: var(--danger); }
.gw-msg.good { border-left-color: var(--success); color: var(--success); }
.playback { display: flex; align-items: center; gap: 6px; }
.pb { width: 32px; height: 30px; padding: 0; display: grid; place-items: center; }
.seek { flex: 1; accent-color: var(--brand); min-width: 40px; }
.pb-count { font-family: var(--font-code); font-size: 12px; color: var(--muted); min-width: 44px; text-align: right; }
.pb-speed { font-family: var(--font-code); font-size: 12px; background: var(--surface); color: var(--text); border: var(--border-w) solid var(--border); border-radius: var(--radius-sm); padding: 3px; }
@media (prefers-reduced-motion: reduce) { .ferris-pos { transition: none; } .bump-RIGHT, .bump-LEFT, .bump-UP, .bump-DOWN, .jump { animation: none; } }
</style>
