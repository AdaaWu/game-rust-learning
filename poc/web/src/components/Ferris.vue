<script setup lang="ts">
import { computed } from 'vue'

export type Mood = 'normal' | 'happy' | 'dizzy' | 'confused' | 'sweat'
const props = withDefaults(defineProps<{ look: 'A' | 'B' | 'C'; mood?: Mood; size?: number }>(), {
  mood: 'normal',
  size: 64,
})

// 像素版 Ferris：左半邊 8 格，右半邊鏡像
const HALF = [
  '.dd.....',
  'dood..k.',
  '.dd...d.',
  '..d.oooo',
  '...doooo',
  '..dooooo',
  '.ddooooo',
  'd.dooooo',
  '..ddoooo',
  '.d.ddddd',
  'd..d....',
]
const pixels = computed(() => {
  const rows = HALF.map((h) => (h + [...h].reverse().join('')).split(''))
  const set = (r: number, c: number, v: string) => (rows[r][c] = v)
  if (props.mood === 'happy') {
    ;[5, 6, 9, 10].forEach((c) => set(6, c, 'k'))
    ;[7, 8].forEach((c) => set(7, c, 'k'))
  } else if (props.mood === 'dizzy') {
    set(1, 6, 'x'); set(1, 9, 'x')
    ;[6, 7, 8, 9].forEach((c) => set(7, c, 'k'))
  } else if (props.mood === 'confused') {
    set(0, 9, 'k')
    ;[7, 8, 9].forEach((c) => set(7, c, 'k'))
  } else if (props.mood === 'sweat') {
    set(3, 13, 'w'); set(4, 13, 'w')
    ;[6, 7, 8, 9].forEach((c) => set(7, c, 'k'))
  } else {
    ;[7, 8].forEach((c) => set(7, c, 'k'))
  }
  const out: { x: number; y: number; c: string }[] = []
  rows.forEach((row, y) => row.forEach((c, x) => c !== '.' && out.push({ x, y, c })))
  return out
})
const PIX_FILL: Record<string, string> = {
  o: 'var(--ferris-body)',
  d: 'var(--ferris-shade)',
  k: 'var(--ferris-ink)',
  x: '#E53935',
  w: '#6EC6FF',
}
</script>

<template>
  <!-- A：像素 -->
  <svg v-if="look === 'A'" :width="size" :height="size * 0.75" viewBox="0 0 16 12" shape-rendering="crispEdges" class="ferris ferris-a" aria-hidden="true">
    <rect v-for="p in pixels" :key="`${p.x}-${p.y}`" :x="p.x" :y="p.y" width="1" height="1" :fill="PIX_FILL[p.c]" />
  </svg>

  <!-- B：繪本 -->
  <svg v-else-if="look === 'B'" :width="size" :height="size * 0.75" viewBox="0 0 64 48" class="ferris ferris-b" aria-hidden="true">
    <g stroke="var(--ferris-ink)" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
      <!-- 腳 -->
      <path d="M17 36 L11 43 M22 39 L19 45 M47 36 L53 43 M42 39 L45 45" fill="none" />
      <!-- 鉗子 -->
      <path d="M14 26 C6 26 3 18 6 12 C8 9 12 9 13 12 L10 15 C11 18 13 20 17 22" fill="var(--ferris-body)" />
      <path d="M50 26 C58 26 61 18 58 12 C56 9 52 9 51 12 L54 15 C53 18 51 20 47 22" fill="var(--ferris-body)" />
      <!-- 身體（頂部波浪） -->
      <path
        d="M10 30 C10 22 16 17 20 17 C21 14 25 14 26 16 C28 13 36 13 38 16 C39 14 43 14 44 17 C48 17 54 22 54 30 C54 38 44 41 32 41 C20 41 10 38 10 30 Z"
        fill="var(--ferris-body)"
      />
      <!-- 眼柄 -->
      <path d="M26 17 L25 10 M38 17 L39 10" fill="none" />
    </g>
    <!-- 眼睛 -->
    <template v-if="mood === 'happy'">
      <path d="M21 9 Q25 4 29 9 M35 9 Q39 4 43 9" stroke="var(--ferris-ink)" stroke-width="2.4" fill="none" stroke-linecap="round" />
    </template>
    <template v-else-if="mood === 'dizzy'">
      <circle cx="25" cy="8" r="5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
      <circle cx="39" cy="8" r="5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
      <path d="M23 6 L27 10 M27 6 L23 10 M37 6 L41 10 M41 6 L37 10" stroke="var(--ferris-ink)" stroke-width="1.6" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle cx="25" cy="8" r="5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
      <circle cx="39" cy="8" :r="mood === 'confused' ? 6 : 5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
      <circle cx="26" cy="9" r="2.4" fill="var(--ferris-ink)" />
      <circle cx="40" cy="9" r="2.4" fill="var(--ferris-ink)" />
      <circle cx="27" cy="8" r="0.9" fill="#fff" />
      <circle cx="41" cy="8" r="0.9" fill="#fff" />
    </template>
    <!-- 腮紅 -->
    <ellipse cx="20" cy="31" rx="3.5" ry="2" fill="var(--ferris-blush)" />
    <ellipse cx="44" cy="31" rx="3.5" ry="2" fill="var(--ferris-blush)" />
    <!-- 嘴 -->
    <path v-if="mood === 'happy'" d="M27 29 Q32 36 37 29 Z" fill="var(--ferris-ink)" />
    <path v-else-if="mood === 'sweat' || mood === 'dizzy'" d="M28 32 Q30 30 32 32 Q34 34 36 32" stroke="var(--ferris-ink)" stroke-width="2" fill="none" stroke-linecap="round" />
    <path v-else-if="mood === 'confused'" d="M28 32 L36 31" stroke="var(--ferris-ink)" stroke-width="2" stroke-linecap="round" />
    <path v-else d="M28 30 Q32 34 36 30" stroke="var(--ferris-ink)" stroke-width="2" fill="none" stroke-linecap="round" />
    <path v-if="mood === 'sweat'" d="M50 6 Q53 11 50 13 Q47 11 50 6 Z" fill="#7FD3FF" stroke="var(--ferris-ink)" stroke-width="1" />
  </svg>

  <!-- C：霓虹線框 -->
  <svg v-else :width="size" :height="size * 0.75" viewBox="0 0 64 48" class="ferris ferris-c" aria-hidden="true">
    <g fill="none" stroke="var(--ferris-body)" stroke-width="2" stroke-linejoin="miter" stroke-linecap="square">
      <polygon points="12,30 18,20 26,18 38,18 46,20 52,30 46,38 18,38" fill="var(--ferris-fill)" />
      <polyline points="16,22 8,22 4,14 10,8 12,14" />
      <polyline points="48,22 56,22 60,14 54,8 52,14" />
      <path d="M18 38 L12 44 M24 38 L20 44 M46 38 L52 44 M40 38 L44 44" />
      <path d="M26 18 L26 12 M38 18 L38 12" />
    </g>
    <g fill="var(--ferris-eye)" stroke="none">
      <template v-if="mood === 'happy'">
        <path d="M22 11 L26 7 L30 11" stroke="var(--ferris-eye)" stroke-width="2" fill="none" />
        <path d="M34 11 L38 7 L42 11" stroke="var(--ferris-eye)" stroke-width="2" fill="none" />
      </template>
      <template v-else-if="mood === 'dizzy'">
        <path d="M23 6 L29 12 M29 6 L23 12 M35 6 L41 12 M41 6 L35 12" stroke="var(--danger)" stroke-width="2" />
      </template>
      <template v-else>
        <rect x="23" y="6" width="6" height="6" />
        <rect x="35" y="6" :width="mood === 'confused' ? 8 : 6" :height="mood === 'confused' ? 8 : 6" />
      </template>
    </g>
    <text x="32" y="32" text-anchor="middle" font-family="JetBrains Mono, monospace" font-size="9" font-weight="700" fill="var(--ferris-eye)">
      {{ mood === 'happy' ? '^_^' : mood === 'dizzy' ? 'x_x' : mood === 'confused' ? 'o_O' : mood === 'sweat' ? '-_-;' : '>_' }}
    </text>
  </svg>
</template>

<style scoped>
.ferris { display: block; overflow: visible; }
.ferris-c { filter: drop-shadow(0 0 3px var(--ferris-body)); }
</style>
