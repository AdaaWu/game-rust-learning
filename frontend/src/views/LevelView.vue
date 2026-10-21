<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { Diagnostic, LevelDetail, LevelKind } from '../api/types'
import CodeEditor from '../components/CodeEditor.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import ConsolePanel from '../components/ConsolePanel.vue'
import Ferris from '../components/Ferris.vue'
import GridWorld from '../components/GridWorld.vue'
import LevelCompleteDialog from '../components/LevelCompleteDialog.vue'
import MemoryView from '../components/MemoryView.vue'
import ShortcutHelp from '../components/ShortcutHelp.vue'
import { isMacPlatform, matchShortcut } from '../domain/editor/shortcuts'
import { levelNumber } from '../domain/levelNumber'
import { renderMarkdown } from '../domain/markdown'
import { nextLevelId, nextTheme } from '../domain/progress'
import { resolveTheme } from '../domain/theme'
import { useAttemptStore } from '../stores/attempt'
import { needsResetConfirm, useDraftStore } from '../stores/draft'
import { useLayoutStore } from '../stores/layout'
import type { MobileTab } from '../stores/layout'
import { useLevelStore } from '../stores/level'
import { useProgressStore } from '../stores/progress'

const CLEAR_ANIMATION_MS = 1500

const KIND_BANNER: Record<LevelKind, string> = {
  build: '任務：完成下面的程式',
  'fix-compile': '這段程式編不過，先按執行看錯誤訊息，再修好它',
  debug: '程式能跑，但結果不對，找出 bug',
}

const route = useRoute()
const router = useRouter()
const levelStore = useLevelStore()
const progressStore = useProgressStore()
const attempt = useAttemptStore()
const draft = useDraftStore()
const layout = useLayoutStore()

const levelId = ref('')
const level = ref<LevelDetail | null>(null)
const notFound = ref(false)
const code = ref('')
// 結果回來後程式碼被改過
const edited = ref(false)
const highlightLine = ref<number | null>(null)
const drawerOpen = ref(false)
const confirmKind = ref<'hint' | 'reset' | null>(null)
const showShortcuts = ref(false)
const celebrating = ref(false)
let celebrateTimer: ReturnType<typeof setTimeout> | null = null

const pageRoot = ref<HTMLElement | null>(null)
const memoryView = ref<InstanceType<typeof MemoryView> | null>(null)
const editor = ref<InstanceType<typeof CodeEditor> | null>(null)

const prefersReducedMotion = () =>
  typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

// 系統深淺色變化時同步編輯器主題
const darkQuery = typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null
const systemDark = ref(darkQuery?.matches ?? false)
const onSystemTheme = (e: MediaQueryListEvent) => (systemDark.value = e.matches)
const editorTheme = computed(() => resolveTheme(progressStore.prefs.theme, systemDark.value))

async function loadLevel(id: string) {
  levelId.value = id
  level.value = null
  notFound.value = false
  edited.value = false
  highlightLine.value = null
  confirmKind.value = null
  celebrating.value = false
  attempt.enterLevel(id)
  layout.enterLevelMobile(id)
  drawerOpen.value = layout.claimDrawerAutoOpen(id)
  await levelStore.fetchList()
  const result = await levelStore.fetchDetail(id)
  if (levelId.value !== id) return
  if (!result.ok) {
    notFound.value = true
    return
  }
  level.value = result.data
  code.value = draft.load(id, result.data.starter)
}

watch(
  () => String(route.params.levelId ?? ''),
  (id) => {
    if (id) loadLevel(id)
  },
  { immediate: true },
)

const summaries = computed(() => levelStore.summaries)
const chapterTitle = computed(() => levelStore.chapters.find((c) => c.chapter === level.value?.chapter)?.title ?? '')
const number = computed(() => levelNumber(levelId.value, summaries.value))
const titleText = computed(() =>
  level.value ? `第 ${level.value.chapter} 章 ${chapterTitle.value} ▸ ${number.value} ${level.value.title}` : '',
)
const levelProgress = computed(() => progressStore.levelProgress(levelId.value))
const storyHtml = computed(() => renderMarkdown(level.value?.story ?? ''))
const conceptHtml = computed(() => renderMarkdown(level.value?.concept ?? ''))

// 提示
const hints = computed(() => level.value?.hints ?? [])
const hintsRevealed = computed(() => Math.min(levelProgress.value.current.hintsRevealed, hints.value.length))
function onHintClick() {
  if (hintsRevealed.value === 0) confirmKind.value = 'hint'
  else progressStore.revealHint(levelId.value)
}

// 程式碼與草稿
function onCodeUpdate(value: string) {
  if (value === code.value) return
  code.value = value
  draft.scheduleSave(levelId.value, value)
  attempt.noteCodeChange()
  if (attempt.phase === 'idle' && attempt.result) edited.value = true
}

function doReset() {
  if (!level.value) return
  if (code.value !== level.value.starter && attempt.result) edited.value = true
  code.value = level.value.starter
  draft.clear(levelId.value)
}

function onReset() {
  if (!level.value) return
  if (needsResetConfirm(code.value, level.value.starter)) confirmKind.value = 'reset'
  else doReset()
}

function onConfirm() {
  const kind = confirmKind.value
  confirmKind.value = null
  if (kind === 'hint') progressStore.revealHint(levelId.value)
  if (kind === 'reset') doReset()
}

// Run／Submit
async function onRun() {
  if (!attempt.canRequest) return
  edited.value = false
  layout.notifyRequestStarted()
  await attempt.run(code.value)
}

async function onSubmit() {
  if (!attempt.canRequest) return
  edited.value = false
  layout.notifyRequestStarted()
  await attempt.submit(code.value)
}

watch(
  () => attempt.result,
  (result) => {
    if (result) layout.notifyResult()
  },
)

const resultData = computed(() => (attempt.result && 'data' in attempt.result ? attempt.result.data : null))
const editorDiagnostics = computed<Diagnostic[]>(() => (edited.value ? [] : (resultData.value?.compile.diagnostics ?? [])))
const panicStderr = computed(() =>
  !edited.value && resultData.value?.status === 'runtime_panic' ? (resultData.value.execution?.stderr ?? null) : null,
)
const runOk = computed(() => attempt.result?.action === 'run' && resultData.value?.status === 'ok')

const loading = computed(() => attempt.phase !== 'idle')
const runLabel = computed(() => {
  if (attempt.phase === 'running') return '判題中…'
  if (attempt.busySeconds > 0) return `執行（${attempt.busySeconds} 秒後可再試）`
  return '執行'
})
const submitLabel = computed(() => {
  if (attempt.phase === 'submitting') return '判題中…'
  if (attempt.busySeconds > 0) return `送出（${attempt.busySeconds} 秒後可再試）`
  return '送出'
})

// GridWorld 只吃 Run 結果；Run 請求中先清空，結果回來一定重播
const gridStdout = computed(() => (attempt.phase === 'running' ? null : (attempt.lastRun?.execution?.stdout ?? null)))
const gridStatus = computed(() => (attempt.phase === 'running' ? null : (attempt.lastRun?.status ?? null)))
const gridTruncated = computed(() => attempt.lastRun?.execution?.stdout_truncated ?? false)

const codeModified = computed(() => !!level.value && code.value !== level.value.starter)

function onGutterClick(line: number) {
  memoryView.value?.goToLine(line)
}

function onJump(line: number, column: number) {
  editor.value?.revealPosition(line, column)
}

// 通關動畫與結算
watch(
  () => attempt.clearedStars,
  (stars) => {
    if (!stars) return
    if (prefersReducedMotion()) return
    celebrating.value = true
    celebrateTimer = setTimeout(finishCelebration, CLEAR_ANIMATION_MS)
  },
)

function finishCelebration() {
  if (celebrateTimer !== null) clearTimeout(celebrateTimer)
  celebrateTimer = null
  celebrating.value = false
}

const showComplete = computed(() => !!attempt.clearedStars && !celebrating.value)

function onNext() {
  const next = nextLevelId(levelId.value, summaries.value, progressStore.progress)
  attempt.dismissClear()
  if (next) router.push({ name: 'level', params: { levelId: next } })
  else router.push({ name: 'map' })
}

function onMap() {
  attempt.dismissClear()
  router.push({ name: 'map' })
}

const CONFETTI = Array.from({ length: 24 }, (_, i) => ({
  left: `${(i * 37) % 100}%`,
  delay: `${(i % 6) * 120}ms`,
  color: ['var(--brand)', 'var(--star-on)', 'var(--grid-gem)'][i % 3],
}))

// 頂部列
const THEME_ICON = { system: '🌓', light: '☀', dark: '☾' }
const THEME_TEXT = { system: '跟隨系統', light: '淺色', dark: '深色' }
function cycleTheme() {
  progressStore.setTheme(nextTheme(progressStore.prefs.theme))
}

function setMobileTab(tab: MobileTab) {
  layout.setMobileTab(tab)
}

// 全域快捷鍵：焦點在編輯器外時
const isMac = isMacPlatform(typeof navigator === 'undefined' ? '' : navigator.platform)
function onGlobalKeydown(event: KeyboardEvent) {
  // 頁面已不在畫面上時不處理
  if (!pageRoot.value?.isConnected) return
  if (event.key === 'Escape') {
    if (celebrating.value) finishCelebration()
    else if (drawerOpen.value) drawerOpen.value = false
    return
  }
  const target = event.target as HTMLElement | null
  if (target?.closest?.('.monaco-editor')) return
  if (confirmKind.value || showShortcuts.value || attempt.clearedStars) return
  const action = matchShortcut(event, isMac)
  if (!action) return
  event.preventDefault()
  if (action === 'run') onRun()
  else onSubmit()
}

onMounted(() => {
  attempt.checkHealth()
  document.addEventListener('keydown', onGlobalKeydown)
  darkQuery?.addEventListener?.('change', onSystemTheme)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onGlobalKeydown)
  darkQuery?.removeEventListener?.('change', onSystemTheme)
  finishCelebration()
})
</script>

<template>
  <div v-if="notFound" class="not-found" data-testid="not-found">
    <Ferris mood="confused" :size="96" />
    <h1>找不到這關</h1>
    <RouterLink to="/" class="btn is-primary">回地圖</RouterLink>
  </div>

  <div v-else ref="pageRoot" class="level-page" :class="[`mtab-${layout.mobileTab}`, { 'is-drawer-open': drawerOpen }]">
    <header class="topbar">
      <RouterLink to="/" class="topbar-back" data-testid="back-to-map">← 地圖</RouterLink>
      <button type="button" class="topbar-drawer" :aria-expanded="drawerOpen" @click="drawerOpen = !drawerOpen">☰ 故事</button>
      <h1 class="level-title" data-testid="level-title">{{ titleText }}</h1>
      <span class="level-stars" data-testid="level-stars" aria-label="本關星等">
        <span
          v-for="key in ['pass', 'noHint', 'firstTry'] as const"
          :key="key"
          class="star-icon"
          :class="{ 'is-on': levelProgress.stars[key] }"
          data-testid="star-icon"
        >{{ levelProgress.stars[key] ? '★' : '☆' }}</span>
      </span>
      <button
        type="button"
        class="topbar-btn"
        data-testid="theme-toggle"
        :aria-label="`深淺色：${THEME_TEXT[progressStore.prefs.theme]}`"
        :title="`深淺色：${THEME_TEXT[progressStore.prefs.theme]}`"
        @click="cycleTheme"
      >
        {{ THEME_ICON[progressStore.prefs.theme] }}
      </button>
      <button type="button" class="topbar-btn" data-testid="shortcut-help-button" aria-label="快捷鍵說明" @click="showShortcuts = true">⌨ 快捷鍵</button>
    </header>

    <p v-if="runOk" class="run-ok-hint" data-testid="run-ok-hint">看起來可以了，按『送出』驗收</p>

    <nav class="mobile-tabs" role="tablist" aria-label="關卡分頁">
      <button type="button" role="tab" :aria-selected="layout.mobileTab === 'story'" @click="setMobileTab('story')">故事</button>
      <button type="button" role="tab" :aria-selected="layout.mobileTab === 'code'" @click="setMobileTab('code')">程式</button>
      <button type="button" role="tab" :aria-selected="layout.mobileTab === 'result'" @click="setMobileTab('result')">
        結果<span v-if="layout.resultDot" class="tab-dot" aria-label="有新結果"> •</span>
      </button>
    </nav>

    <p v-if="!level" class="level-loading">載入中…</p>

    <div v-else class="level-body">
      <div class="drawer-backdrop" @click="drawerOpen = false" />
      <aside class="col-info panel" data-testid="col-info">
        <div class="info-scroll">
          <div class="ferris-bubble-row">
            <Ferris class="bubble-avatar" :size="48" />
            <!-- story 經 DOMPurify 過濾 -->
            <div class="ferris-bubble" data-testid="ferris-bubble" v-html="storyHtml" />
          </div>
          <p class="kind-banner" data-testid="kind-banner">{{ KIND_BANNER[level.kind] }}</p>
          <!-- concept 經 DOMPurify 過濾 -->
          <div class="concept markdown" data-testid="concept" v-html="conceptHtml" />
        </div>
        <section v-if="hints.length > 0" class="hint-section" data-testid="hint-section">
          <h2 class="hint-heading">提示</h2>
          <ol v-if="hintsRevealed > 0" class="hint-list">
            <li v-for="(hint, i) in hints.slice(0, hintsRevealed)" :key="i" class="hint-item" data-testid="hint-item">{{ hint }}</li>
          </ol>
          <button
            v-if="hintsRevealed < hints.length"
            type="button"
            class="btn is-secondary hint-button"
            data-testid="hint-button"
            @click="onHintClick"
          >需要提示嗎？（{{ hintsRevealed }} / {{ hints.length }}）</button>
        </section>
      </aside>

      <section class="col-editor panel" data-testid="col-editor">
        <div class="editor-filename">🦀 main.rs</div>
        <div class="editor-host">
          <CodeEditor
            ref="editor"
            :model-value="code"
            :diagnostics="editorDiagnostics"
            :panic-stderr="panicStderr"
            :highlight-line="highlightLine"
            :theme="editorTheme"
            @update:model-value="onCodeUpdate"
            @run="onRun"
            @submit="onSubmit"
            @gutter-click="onGutterClick"
          />
        </div>
      </section>

      <div class="action-bar">
        <button type="button" class="btn is-danger" data-testid="btn-reset" :disabled="loading" @click="onReset">重設</button>
        <span v-if="attempt.judgeDegraded" class="degraded-note" data-testid="degraded-note">判題服務還沒準備好，關卡內容仍可瀏覽</span>
        <span class="action-spacer" />
        <button
          type="button"
          class="btn is-secondary btn-run"
          :class="{ 'is-loading': attempt.phase === 'running' }"
          data-testid="btn-run"
          :disabled="!attempt.canRequest"
          @click="onRun"
        >{{ runLabel }}</button>
        <button
          type="button"
          class="btn is-primary btn-submit"
          :class="{ 'is-loading': attempt.phase === 'submitting' }"
          data-testid="btn-submit"
          :disabled="!attempt.canRequest"
          @click="onSubmit"
        >{{ submitLabel }}</button>
      </div>

      <section class="col-result" data-testid="col-result">
        <div v-if="level.visual !== 'none'" class="visual-panel panel" data-testid="visual-panel">
          <GridWorld
            v-if="level.visual === 'grid' && level.grid"
            :grid="level.grid"
            :stdout="gridStdout"
            :status="gridStatus"
            :truncated="gridTruncated"
            :pending="loading"
          />
          <MemoryView
            v-else-if="level.visual === 'memory' && level.memory_snapshots && level.memory_snapshots.length > 0"
            ref="memoryView"
            :snapshots="level.memory_snapshots"
            :code-modified="codeModified"
            @line-change="highlightLine = $event"
          />
        </div>
        <div class="console-panel-wrap panel">
          <ConsolePanel
            :result="attempt.result"
            :loading="loading"
            :slow="attempt.slow"
            :stale="edited"
            :stale-request="attempt.codeChangedDuringRequest"
            :mark-grid-commands="level.visual === 'grid'"
            @jump="onJump"
          />
        </div>
      </section>
    </div>

    <ConfirmDialog
      v-if="confirmKind === 'hint'"
      message="看提示會拿不到『沒看提示』這顆星，確定要看嗎？"
      @confirm="onConfirm"
      @cancel="confirmKind = null"
    />
    <ConfirmDialog
      v-if="confirmKind === 'reset'"
      message="重設會把程式碼改回起始版本，目前的修改會消失，確定嗎？"
      @confirm="onConfirm"
      @cancel="confirmKind = null"
    />
    <ShortcutHelp v-if="showShortcuts" @close="showShortcuts = false" />

    <div v-if="celebrating" class="clear-animation" data-testid="clear-animation" @click="finishCelebration">
      <span v-for="(c, i) in CONFETTI" :key="i" class="confetti" :style="{ left: c.left, animationDelay: c.delay, background: c.color }" />
      <div class="clear-ferris">
        <Ferris mood="happy" :size="160" />
      </div>
    </div>

    <LevelCompleteDialog
      v-if="showComplete && attempt.clearedStars && level"
      :level-number="number"
      :title="level.title"
      :earned="attempt.clearedStars"
      @next="onNext"
      @replay="attempt.dismissClear()"
      @map="onMap"
      @close="attempt.dismissClear()"
    />
  </div>
</template>

<style scoped>
.level-page {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  padding: 10px 12px 12px;
  gap: 8px;
  box-sizing: border-box;
}
.topbar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 12px;
  height: 56px;
  box-sizing: border-box;
  padding: 0 16px;
  background: var(--surface);
  border: var(--border-w) solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}
.topbar-back {
  color: var(--heading);
  text-decoration: none;
  font-weight: 700;
  white-space: nowrap;
}
.topbar-drawer {
  display: none;
}
.level-title {
  flex: 1;
  min-width: 0;
  margin: 0;
  font-family: var(--font-heading);
  font-size: var(--title-size-sm);
  color: var(--heading);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.level-stars {
  display: flex;
  gap: 2px;
  font-size: 20px;
}
.star-icon {
  color: var(--star-off);
}
.star-icon.is-on {
  color: var(--star-on);
  text-shadow: var(--star-glow);
}
.topbar-btn,
.topbar-drawer {
  border: var(--border-w) solid var(--border);
  background: var(--surface-muted);
  color: var(--text);
  border-radius: var(--radius-pill);
  padding: 4px 12px;
  font: inherit;
  cursor: pointer;
  white-space: nowrap;
}
.run-ok-hint {
  flex: none;
  margin: 0;
  padding: 6px 16px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--success) 14%, transparent);
  color: var(--success);
  font-weight: 700;
  text-align: center;
}
.mobile-tabs {
  display: none;
}
.level-loading {
  text-align: center;
  color: var(--text-muted);
}
.level-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 28fr 40fr 32fr;
  grid-template-rows: minmax(0, 1fr) auto;
  grid-template-areas:
    'info editor result'
    'info actions result';
  gap: 0 12px;
}
.panel {
  background: var(--surface);
  border: var(--border-w) solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  overflow: hidden;
}
.col-info {
  grid-area: info;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.info-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 16px;
}
.ferris-bubble-row {
  display: flex;
  align-items: flex-end;
  gap: 10px;
}
.bubble-avatar {
  flex: none;
}
.ferris-bubble {
  position: relative;
  flex: 1;
  padding: 10px 14px;
  background: var(--surface);
  border: var(--border-w) solid var(--border);
  border-radius: var(--radius);
  line-height: 1.7;
}
.ferris-bubble::before {
  content: '';
  position: absolute;
  left: -9px;
  bottom: 12px;
  border: 8px solid transparent;
  border-right-color: var(--border);
  border-left: none;
}
.ferris-bubble :deep(p) {
  margin: 0;
}
.kind-banner {
  margin: 14px 0;
  padding: 8px 12px;
  border-left: 5px solid var(--brand);
  border-radius: var(--radius-sm);
  background: var(--brand-soft);
  font-weight: 700;
}
.markdown :deep(h1),
.markdown :deep(h2),
.markdown :deep(h3) {
  font-family: var(--font-heading);
  color: var(--heading);
}
.markdown :deep(pre) {
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
  overflow: auto;
  font-family: var(--font-code);
}
.markdown :deep(code) {
  font-family: var(--font-code);
}
.markdown :deep(:not(pre) > code) {
  padding: 0 4px;
  border-radius: 6px;
  background: var(--surface-muted);
}
.hint-section {
  flex: none;
  position: sticky;
  bottom: 0;
  padding: 12px 16px;
  border-top: var(--border-w) solid var(--border);
  background: var(--surface-muted);
  max-height: 40%;
  overflow: auto;
}
.hint-heading {
  margin: 0 0 6px;
  font-family: var(--font-heading);
  font-size: 15px;
  color: var(--heading);
}
.hint-list {
  margin: 0 0 8px;
  padding-left: 20px;
  display: grid;
  gap: 4px;
}
.col-editor {
  grid-area: editor;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}
.editor-filename {
  flex: none;
  padding: 6px 14px;
  background: var(--surface-muted);
  border-bottom: var(--border-w) solid var(--border);
  font-family: var(--font-code);
  font-size: 13px;
}
.editor-host {
  flex: 1;
  min-height: 0;
}
.action-bar {
  grid-area: actions;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--surface);
  border: var(--border-w) solid var(--border);
  border-top: none;
  border-radius: 0 0 var(--radius) var(--radius);
  box-shadow: var(--shadow);
}
.action-spacer {
  flex: 1;
}
.degraded-note {
  font-size: 13px;
  color: var(--neutral-busy);
}
.btn-run::before {
  content: '▶ ';
}
.btn-submit::before {
  content: '✓ ';
}
.btn-run.is-loading::before,
.btn-submit.is-loading::before {
  content: '';
  display: inline-block;
  width: 12px;
  height: 12px;
  margin-right: 6px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  vertical-align: -1px;
  animation: spin 800ms linear infinite;
}
.is-danger::before {
  content: '↺ ';
}
.col-result {
  grid-area: result;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}
.visual-panel {
  flex: 55 1 0;
  min-height: 0;
}
.console-panel-wrap {
  flex: 45 1 0;
  min-height: 0;
}
.col-result > .console-panel-wrap:only-child {
  flex: 1;
}
.drawer-backdrop {
  display: none;
}
.not-found {
  display: grid;
  place-items: center;
  gap: 12px;
  padding: 80px 16px;
  text-align: center;
}
.not-found h1 {
  margin: 0;
  font-family: var(--font-heading);
  color: var(--heading);
}
.clear-animation {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg) 60%, transparent);
  cursor: pointer;
}
.clear-ferris {
  animation: dance 500ms ease-in-out 3 alternate;
}
.confetti {
  position: absolute;
  top: -12px;
  width: 10px;
  height: 10px;
  border-radius: var(--confetti-radius);
  animation: fall 1400ms ease-in forwards;
}
@keyframes dance {
  0% {
    transform: translateY(0) rotate(-6deg);
  }
  100% {
    transform: translateY(-24px) rotate(6deg);
  }
}
@keyframes fall {
  to {
    transform: translateY(105vh) rotate(360deg);
  }
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 平板：左欄收成抽屜，中右 55 / 45 */
@media (min-width: 768px) and (max-width: 1199px) {
  .topbar-drawer {
    display: inline-block;
  }
  .level-body {
    position: relative;
    grid-template-columns: 55fr 45fr;
    grid-template-areas:
      'editor result'
      'actions result';
  }
  .col-info {
    position: fixed;
    top: 76px;
    bottom: 12px;
    left: 12px;
    width: 360px;
    z-index: 20;
    transform: translateX(calc(-100% - 24px));
    transition: transform 250ms ease-out;
  }
  .is-drawer-open .col-info {
    transform: none;
  }
  .is-drawer-open .drawer-backdrop {
    display: block;
    position: fixed;
    inset: 0;
    z-index: 19;
    background: color-mix(in srgb, var(--text) 20%, transparent);
  }
}

/* 手機：三個分頁，操作列固定底部 */
@media (max-width: 767px) {
  .level-page {
    padding: 8px 8px 72px;
  }
  .topbar-back {
    font-size: 0;
  }
  .topbar-back::before {
    content: '←';
    font-size: 18px;
  }
  .topbar .topbar-btn[data-testid='shortcut-help-button'] {
    display: none;
  }
  .mobile-tabs {
    display: flex;
    gap: 6px;
  }
  .mobile-tabs button {
    flex: 1;
    min-height: 44px;
    border: var(--border-w) solid var(--border);
    border-radius: var(--radius-pill);
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  .mobile-tabs button[aria-selected='true'] {
    background: var(--brand);
    border-color: var(--brand);
    color: var(--brand-contrast);
  }
  .tab-dot {
    color: var(--danger);
  }
  .level-body {
    display: block;
  }
  .col-info,
  .col-editor,
  .col-result {
    display: none;
    height: 100%;
  }
  .mtab-story .col-info,
  .mtab-code .col-editor,
  .mtab-result .col-result {
    display: flex;
  }
  .col-editor {
    border-radius: var(--radius);
  }
  .action-bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 15;
    height: 56px;
    box-sizing: border-box;
    border-radius: var(--radius) var(--radius) 0 0;
    border-top: var(--border-w) solid var(--border);
  }
  .action-bar .btn {
    min-height: 44px;
    min-width: 44px;
  }
  .degraded-note {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .clear-ferris,
  .confetti,
  .btn-run.is-loading::before,
  .btn-submit.is-loading::before {
    animation: none;
  }
  .col-info {
    transition: none;
  }
}
</style>
