<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { LevelKind, LevelSummary } from '../api/types'
import Ferris from '../components/Ferris.vue'
import { levelNumber } from '../domain/levelNumber'
import { blockingLevel, chapterStars, currentLevelId, nodeState } from '../domain/progress'
import { useLevelStore } from '../stores/level'
import { useProgressStore } from '../stores/progress'

const router = useRouter()
const levelStore = useLevelStore()
const progressStore = useProgressStore()

const loadFailed = ref(false)
const tip = ref<{ id: string; text: string } | null>(null)

const KIND_ICON: Record<LevelKind, string> = { build: '🔨', 'fix-compile': '🩹', debug: '🔍' }

const levels = computed(() => levelStore.summaries)
const progress = computed(() => progressStore.progress)
const currentId = computed(() => currentLevelId(levels.value, progress.value))

const sections = computed(() =>
  levelStore.chapters.map((c) => ({
    ...c,
    stars: chapterStars(c.chapter, levels.value, progress.value),
    levels: levels.value.filter((l) => l.chapter === c.chapter).sort((a, b) => a.order - b.order),
  })),
)

const stateOf = (l: LevelSummary) => nodeState(l.id, levels.value, progress.value)
const starsOf = (l: LevelSummary) => progress.value.levels[l.id]?.stars ?? { pass: false, noHint: false, firstTry: false }

function onNodeClick(level: LevelSummary) {
  if (stateOf(level) === 'locked') {
    const blocker = blockingLevel(level.id, levels.value, progress.value)
    tip.value = { id: level.id, text: blocker ? `先完成〈${blocker.title}〉` : '這關還沒解鎖' }
    return
  }
  tip.value = null
  router.push({ name: 'level', params: { levelId: level.id } })
}

const mapRoot = ref<HTMLElement | null>(null)

onMounted(async () => {
  const result = await levelStore.fetchList()
  loadFailed.value = !result.ok
  await nextTick()
  const reduce = typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
  mapRoot.value
    ?.querySelector('[data-current="true"]')
    ?.scrollIntoView({ block: 'center', behavior: reduce ? 'auto' : 'smooth' })
})
</script>

<template>
  <main ref="mapRoot" class="map-page">
    <header class="map-header">
      <h1 class="map-title">Rust Quest</h1>
    </header>

    <p v-if="loadFailed" class="map-error" role="alert">連不上伺服器，檢查網路後再試</p>

    <section v-for="(section, s) in sections" :key="section.chapter" class="chapter">
      <h2 class="chapter-title" data-testid="chapter-title">
        <span>第 {{ section.chapter }} 章 {{ section.title }}</span>
        <span class="chapter-stars" data-testid="chapter-stars">★ {{ section.stars.earned }} / {{ section.stars.max }}</span>
      </h2>
      <ol class="trail">
        <li
          v-for="(level, i) in section.levels"
          :key="level.id"
          class="trail-item"
          :class="{ 'is-low': i % 2 === 1 }"
        >
          <!-- 各章第一關也畫出從前一章接過來的路徑 -->
          <span
            v-if="i > 0 || s > 0"
            class="trail-path"
            :class="stateOf(level) === 'locked' ? 'is-dashed' : 'is-solid'"
            aria-hidden="true"
          />
          <button
            type="button"
            class="level-node"
            :class="`is-${stateOf(level)}`"
            data-testid="level-node"
            :data-id="level.id"
            :data-state="stateOf(level)"
            :data-current="String(level.id === currentId)"
            :aria-label="`${levelNumber(level.id, levels)} ${level.title}${stateOf(level) === 'locked' ? '（鎖定）' : ''}`"
            :aria-describedby="tip?.id === level.id ? `tip-${level.id}` : undefined"
            @click="onNodeClick(level)"
          >
            <span v-if="level.id === currentId" class="map-ferris" data-testid="map-ferris">
              <Ferris :size="44" />
            </span>
            <span class="node-disc">
              <span v-if="stateOf(level) === 'locked'" class="node-lock" aria-hidden="true">🔒</span>
              <span v-else class="node-play" aria-hidden="true">{{ stateOf(level) === 'passed' ? '✓' : '▶' }}</span>
            </span>
            <span v-if="stateOf(level) === 'passed'" class="node-stars">
              <span
                v-for="key in ['pass', 'noHint', 'firstTry'] as const"
                :key="key"
                class="star-icon"
                :class="{ 'is-on': starsOf(level)[key] }"
                data-testid="star-icon"
              >{{ starsOf(level)[key] ? '★' : '☆' }}</span>
            </span>
            <span class="node-label">
              <span class="node-number">{{ levelNumber(level.id, levels) }}</span>
              <span class="node-title"><span class="node-kind-icon" :title="level.kind">{{ KIND_ICON[level.kind] }}</span> {{ level.title }}</span>
            </span>
          </button>
          <span v-if="tip?.id === level.id" :id="`tip-${level.id}`" class="node-tip" role="tooltip">{{ tip.text }}</span>
        </li>
      </ol>
    </section>
  </main>
</template>

<style scoped>
.map-page {
  max-width: 1080px;
  margin: 0 auto;
  padding: 24px 16px 64px;
}
.map-header {
  text-align: center;
  margin-bottom: 16px;
}
.map-title {
  margin: 0;
  font-family: var(--font-title);
  font-size: 40px;
  color: var(--brand);
}
.map-error {
  text-align: center;
  color: var(--neutral-busy);
}
.chapter {
  margin: 24px 0;
  padding: 16px 20px 24px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  border: var(--border-w) solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}
.chapter-title {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 12px;
  margin: 0 0 12px;
  font-family: var(--font-heading);
  color: var(--heading);
  font-size: 22px;
}
.chapter-stars {
  font-size: 15px;
  color: var(--text-muted);
}
.trail {
  list-style: none;
  margin: 0;
  padding: 24px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px 0;
}
.trail-item {
  position: relative;
  display: flex;
  align-items: flex-start;
}
.trail-item.is-low {
  padding-top: 36px;
}
.trail-path {
  align-self: flex-start;
  width: 48px;
  margin-top: 44px;
  border-top: 4px solid var(--accent);
}
.trail-item.is-low .trail-path {
  margin-top: 8px;
  transform: rotate(28deg);
  transform-origin: left center;
}
.trail-item:not(.is-low) .trail-path {
  transform: rotate(-28deg);
  transform-origin: left center;
  margin-top: 64px;
}
.trail-path.is-dashed {
  border-top-style: dashed;
  border-top-color: var(--border-strong);
}
.level-node {
  position: relative;
  width: 132px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  border: none;
  background: none;
  color: var(--text);
  font: inherit;
  cursor: pointer;
  padding: 0;
}
.node-disc {
  width: 64px;
  height: 64px;
  display: grid;
  place-items: center;
  font-size: 26px;
  border-radius: var(--radius-avatar);
  border: var(--border-w-strong) solid var(--ferris-ink);
  background: var(--brand);
  box-shadow: 0 4px 0 var(--ferris-shade);
  transition: transform 150ms;
}
.level-node:hover .node-disc {
  transform: translateY(-2px);
}
.level-node.is-available .node-disc {
  animation: node-pulse 1.8s ease-in-out infinite;
}
.level-node.is-passed .node-disc {
  background: var(--grid-gem);
}
.level-node.is-locked {
  cursor: not-allowed;
}
.level-node.is-locked .node-disc {
  background: var(--star-off);
  border-color: var(--border-strong);
  box-shadow: none;
  filter: grayscale(1);
}
.node-stars {
  display: flex;
  gap: 2px;
  font-size: 16px;
}
.star-icon {
  color: var(--star-off);
}
.star-icon.is-on {
  color: var(--star-on);
  text-shadow: var(--star-glow);
}
.node-label {
  display: grid;
  justify-items: center;
  text-align: center;
  line-height: 1.3;
}
.node-play {
  color: var(--brand-contrast);
  font-size: 22px;
}
.node-number {
  font-family: var(--font-code);
  font-size: 12px;
  color: var(--text-muted);
}
.node-title {
  font-family: var(--font-heading);
  font-weight: 700;
}
.map-ferris {
  position: absolute;
  top: -34px;
  left: 50%;
  transform: translateX(-50%);
  animation: bob 1.6s ease-in-out infinite;
}
.node-tip {
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translate(-50%, -100%);
  z-index: 5;
  white-space: nowrap;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  background: var(--text);
  color: var(--bg);
  font-size: 13px;
  box-shadow: var(--shadow-sm);
}
@keyframes node-pulse {
  50% {
    box-shadow: 0 4px 0 var(--ferris-shade), 0 0 0 8px var(--brand-soft);
  }
}
@keyframes bob {
  50% {
    transform: translate(-50%, -4px);
  }
}
@media (prefers-reduced-motion: reduce) {
  .level-node.is-available .node-disc {
    animation: none;
    outline: 4px solid var(--brand-soft);
  }
  .map-ferris {
    animation: none;
  }
}
@media (max-width: 767px) {
  .trail-path {
    display: none;
  }
  .trail-item.is-low {
    padding-top: 0;
  }
  .level-node {
    width: 112px;
  }
}
</style>
