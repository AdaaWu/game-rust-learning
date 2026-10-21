<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import Ferris from './Ferris.vue'

const props = defineProps<{ look: 'A' | 'B' | 'C'; stars: [boolean, boolean, boolean] }>()
const emit = defineEmits<{ (e: 'close'): void }>()
const LABELS = ['通過', '沒看提示', '一次就過']
const confetti = Array.from({ length: 36 }, (_, i) => ({
  left: (i * 37) % 100,
  delay: (i % 9) * 0.12,
  color: ['var(--brand)', 'var(--star-on)', 'var(--grid-gem)', 'var(--accent)'][i % 4],
  rot: (i * 47) % 360,
}))
const onKey = (e: KeyboardEvent) => e.key === 'Escape' && emit('close')
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
void props
</script>

<template>
  <div class="backdrop" @click.self="emit('close')">
    <div class="confetti" aria-hidden="true">
      <i v-for="(c, i) in confetti" :key="i" :style="{ left: `${c.left}%`, animationDelay: `${c.delay}s`, background: c.color, transform: `rotate(${c.rot}deg)` }" />
    </div>
    <div class="modal" role="dialog" aria-modal="true" aria-label="通關結算">
      <p class="eyebrow">{{ look === 'A' ? 'STAGE CLEAR!' : look === 'C' ? '$ cargo test → ok' : '通關啦！' }}</p>
      <h2>黏住的腳　完成</h2>
      <div class="hero"><Ferris :look="look" mood="happy" :size="120" /></div>
      <div class="stars">
        <div v-for="(on, i) in stars" :key="i" class="star" :class="{ on }" :style="{ animationDelay: `${0.3 + i * 0.25}s` }">
          <span class="s">★</span>
          <small>{{ LABELS[i] }}</small>
        </div>
      </div>
      <p class="line">Ferris：「原來加個 <code>mut</code> 腳就不黏了！」</p>
      <div class="actions">
        <button class="btn-ghost" @click="emit('close')">留在這關</button>
        <button class="btn-primary" @click="emit('close')">下一關 ▸</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop { position: fixed; inset: 0; background: rgba(10, 8, 6, .55); display: grid; place-items: center; z-index: 50; animation: fade .25s; overflow: hidden; }
@keyframes fade { from { opacity: 0; } }
.modal { position: relative; background: var(--surface); border: var(--border-w-strong) solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--shadow-lg); padding: 28px 36px; text-align: center; min-width: 380px; animation: rise .45s cubic-bezier(.2, 1.4, .4, 1); }
@keyframes rise { from { transform: translateY(30px) scale(.9); opacity: 0; } }
.eyebrow { font-family: var(--font-title); color: var(--brand); font-size: var(--title-size-sm); margin: 0; letter-spacing: .05em; }
h2 { font-family: var(--font-heading); margin: 8px 0; color: var(--heading); }
.hero { display: grid; place-items: center; height: 110px; animation: hop .6s 3; }
@keyframes hop { 50% { transform: translateY(-16px) rotate(-4deg); } }
.stars { display: flex; justify-content: center; gap: 22px; margin: 6px 0 10px; }
.star { display: flex; flex-direction: column; align-items: center; opacity: 0; animation: starIn .4s forwards; }
.star .s { font-size: 46px; line-height: 1; color: var(--star-off); }
.star.on .s { color: var(--star-on); text-shadow: var(--star-glow); }
.star small { font-size: 12px; color: var(--muted); }
@keyframes starIn { from { opacity: 0; transform: scale(.2) rotate(-40deg); } to { opacity: 1; transform: none; } }
.line { font-size: 14px; }
.line code { font-family: var(--font-code); background: var(--surface-2); padding: 1px 5px; border-radius: var(--radius-sm); }
.actions { display: flex; gap: 10px; justify-content: center; margin-top: 12px; }
.confetti i { position: absolute; top: -20px; width: 9px; height: 14px; border-radius: var(--confetti-radius); animation: fall 2.6s linear infinite; }
@keyframes fall { to { top: 105%; transform: rotate(720deg) translateX(30px); } }
@media (prefers-reduced-motion: reduce) { .confetti { display: none; } .hero, .modal, .star { animation: none; opacity: 1; } }
</style>
