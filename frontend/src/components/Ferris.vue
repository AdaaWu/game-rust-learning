<script setup lang="ts">
// 海洋繪本 Ferris（viewBox 64×48）
export type FerrisMood = 'normal' | 'happy' | 'dizzy' | 'confused'

withDefaults(defineProps<{ mood?: FerrisMood; size?: number }>(), {
  mood: 'normal',
  size: 48,
})
</script>

<template>
  <svg class="ferris" :width="size" :height="size * 0.75" viewBox="0 0 64 48" aria-hidden="true">
    <g stroke="var(--ferris-ink)" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M18 35 L12 42 M23 38 L20 44 M46 35 L52 42 M41 38 L44 44" fill="none" />
      <path class="claw claw-left" d="M15 25 C7 25 3 18 6 12 C8 8 12 8 14 11 L11 15 C12 18 14 20 18 21" fill="var(--ferris-body)" />
      <path class="claw claw-right" d="M49 25 C57 25 61 18 58 12 C56 8 52 8 50 11 L53 15 C52 18 50 20 46 21" fill="var(--ferris-body)" />
      <path
        d="M10 30 C10 22 15 17 20 17 C21 14 25 13 26 16 C28 12 36 12 38 16 C39 13 43 14 44 17 C49 17 54 22 54 30 C54 38 44 41 32 41 C20 41 10 38 10 30 Z"
        fill="var(--ferris-body)"
      />
      <path d="M22 37 C27 39 37 39 42 37" fill="none" stroke="var(--ferris-shade)" />
      <path d="M26 16 L25 10 M38 16 L39 10" fill="none" />
    </g>
    <circle class="eye-white" cx="25" cy="8" r="5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
    <circle class="eye-white" cx="39" cy="8" :r="mood === 'confused' ? 6.5 : 5" fill="#fff" stroke="var(--ferris-ink)" stroke-width="2" />
    <template v-if="mood === 'happy'">
      <path class="eye-mark" d="M22.5 9.5 Q25 6 27.5 9.5 M36.5 9.5 Q39 6 41.5 9.5" stroke="var(--ferris-eye)" stroke-width="2" fill="none" stroke-linecap="round" />
    </template>
    <template v-else-if="mood === 'dizzy'">
      <path class="eye-mark" d="M23 6 L27 10 M27 6 L23 10 M37 6 L41 10 M41 6 L37 10" stroke="var(--ferris-eye)" stroke-width="1.6" stroke-linecap="round" />
    </template>
    <template v-else>
      <circle class="eye-mark" cx="26" cy="9" r="2.3" fill="var(--ferris-eye)" />
      <circle class="eye-mark" cx="40" cy="9" r="2.3" fill="var(--ferris-eye)" />
      <circle class="eye-mark" cx="27" cy="8" r="0.8" fill="#fff" />
      <circle class="eye-mark" cx="41" cy="8" r="0.8" fill="#fff" />
    </template>
    <ellipse cx="20" cy="31" rx="3.5" ry="2" fill="var(--ferris-blush)" />
    <ellipse cx="44" cy="31" rx="3.5" ry="2" fill="var(--ferris-blush)" />
    <path v-if="mood === 'happy'" d="M27 29 Q32 36 37 29 Z" fill="var(--ferris-ink)" />
    <path v-else-if="mood === 'dizzy'" d="M27 32 Q29.5 30 32 32 Q34.5 34 37 32" stroke="var(--ferris-ink)" stroke-width="2" fill="none" stroke-linecap="round" />
    <path v-else-if="mood === 'confused'" d="M28 32 L36 30.5" stroke="var(--ferris-ink)" stroke-width="2" stroke-linecap="round" />
    <path v-else d="M28 30 Q32 34 36 30" stroke="var(--ferris-ink)" stroke-width="2" fill="none" stroke-linecap="round" />
  </svg>
</template>

<style scoped>
.ferris {
  display: block;
  overflow: visible;
}
</style>
