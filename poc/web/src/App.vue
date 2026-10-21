<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ApiError, fetchLevel, runCode, submitCode, type Diagnostic, type LevelDetail, type RunResponse, type SubmitResponse } from './lib/api'
import type { Frame } from './lib/grid'
import CodeEditor from './components/CodeEditor.vue'
import ConsolePanel, { type Tab } from './components/ConsolePanel.vue'
import GridWorld from './components/GridWorld.vue'
import StoryPanel from './components/StoryPanel.vue'
import ClearModal from './components/ClearModal.vue'
import type { Mood } from './components/Ferris.vue'

type Look = 'A' | 'B' | 'C'
type Mode = 'system' | 'light' | 'dark'
const STYLES: { id: Look; name: string; sub: string }[] = [
  { id: 'A', name: '像素冒險', sub: '8-bit RPG' },
  { id: 'B', name: '海洋繪本', sub: '手繪溫暖' },
  { id: 'C', name: '霓虹終端', sub: 'IDE 暗黑' },
]

const safeGet = (k: string) => { try { return localStorage.getItem(k) } catch { return null } }
const safeSet = (k: string, v: string) => { try { localStorage.setItem(k, v) } catch { /* 無痕模式 */ } }

const html = document.documentElement
const look = ref<Look>((html.dataset.style as Look) ?? 'B')
const q = new URLSearchParams(location.search)
const mode = ref<Mode>(((q.get('mode') ?? safeGet('rq.poc.mode')) as Mode) || 'system')
const media = matchMedia('(prefers-color-scheme: dark)')
const sysDark = ref(media.matches)
media.addEventListener('change', (e) => (sysDark.value = e.matches))
const dark = computed(() => mode.value === 'dark' || (mode.value === 'system' && sysDark.value))
watch([look, dark], () => {
  html.dataset.style = look.value
  html.dataset.theme = dark.value ? 'dark' : 'light'
}, { immediate: true })
function setLook(l: Look) { look.value = l; safeSet('rq.poc.style', l) }
function cycleMode() {
  mode.value = mode.value === 'system' ? 'light' : mode.value === 'light' ? 'dark' : 'system'
  safeSet('rq.poc.mode', mode.value)
}
const modeLabel = computed(() => (mode.value === 'system' ? '◐ 系統' : mode.value === 'light' ? '☀ 淺色' : '☾ 深色'))

const level = ref<LevelDetail | null>(null)
const loadError = ref<string | null>(null)
const code = ref('')
const result = ref<RunResponse | SubmitResponse | null>(null)
const action = ref<'run' | 'submit' | null>(null)
const loading = ref(false)
const slow = ref(false)
const netError = ref<string | null>(null)
const stale = ref(false)
const tab = ref<Tab>('output')
const diagnostics = ref<Diagnostic[]>([])
const gridStdout = ref<string | null>(null)
const playToken = ref(0)
const activeLine = ref<number | null>(null)
const revealed = ref(0)
const failedSubmits = ref(0)
const showClear = ref(false)
const stars = ref<[boolean, boolean, boolean]>([false, false, false])
const bestStars = ref(0)
const ferrisLine = ref<string | null>(null)
const mood = ref<Mood>('normal')
const editorRef = ref<InstanceType<typeof CodeEditor>>()

onMounted(async () => {
  try {
    level.value = await fetchLevel()
    code.value = level.value.starter
  } catch (e) {
    loadError.value = e instanceof ApiError ? e.message : '載入關卡失敗'
  }
})

watch(code, () => {
  if (result.value && !loading.value) stale.value = true
  diagnostics.value = []
})

const storyLines = computed(() => level.value?.story.split('\n').filter(Boolean) ?? [])
const gridOverlay = computed(() => {
  if (loading.value) return action.value === 'submit' ? '跑隱藏測試中…' : '編譯中…'
  if (result.value?.status === 'compile_error') return '程式還沒編譯成功'
  return null
})

function say(line: string, m: Mood) { ferrisLine.value = line; mood.value = m }

async function judge(kind: 'run' | 'submit') {
  if (loading.value || !level.value) return
  loading.value = true
  action.value = kind
  netError.value = null
  stale.value = false
  slow.value = false
  const slowTimer = window.setTimeout(() => (slow.value = true), 3000)
  if (kind === 'run') gridStdout.value = null
  activeLine.value = null
  try {
    const r = kind === 'run' ? await runCode(code.value) : await submitCode(code.value)
    result.value = r
    diagnostics.value = r.compile.diagnostics
    if (r.status === 'compile_error') {
      tab.value = 'errors'
      say('腳好像被黏住了…編譯器說有變數不能改，看看右下角的錯誤卡片？', 'sweat')
    } else if (kind === 'run') {
      tab.value = 'output'
      if (r.execution) { gridStdout.value = r.execution.stdout; playToken.value++ }
      if (r.status === 'ok') say('我動起來了！看我走～', 'happy')
      else if (r.status === 'timeout') say('跑太久了…是不是迴圈停不下來？', 'sweat')
      else say('程式中途當掉了，看看主控台的訊息', 'dizzy')
    } else {
      const s = r as SubmitResponse
      tab.value = 'tests'
      if (s.passed) {
        stars.value = [true, revealed.value === 0, failedSubmits.value === 0]
        bestStars.value = Math.max(bestStars.value, stars.value.filter(Boolean).length)
        say('成功了！我自由了！', 'happy')
        window.setTimeout(() => (showClear.value = true), 400)
      } else {
        failedSubmits.value++
        say('隱藏測試沒過…再檢查一下路線？', 'sweat')
      }
    }
    if (r.status === 'compile_error' && kind === 'submit') failedSubmits.value++
  } catch (e) {
    result.value = null
    netError.value = e instanceof ApiError ? e.message : '連不上伺服器，檢查網路後再試'
  } finally {
    window.clearTimeout(slowTimer)
    loading.value = false
  }
}

function onGridFinished(f: Frame, cleared: boolean) {
  if (f.event === 'wall' || f.event === 'out_of_bounds') say('哎唷！撞到岩石了，頭好暈…', 'dizzy')
  else if (f.event === 'invalid' || f.event === 'pick_empty') say('咦？這個指令我看不懂…', 'confused')
  else if (cleared) say('寶石到手、抵達終點！看起來可以了，按「送出」驗收', 'happy')
  else say('指令跑完了，但我還沒到終點…', 'sweat')
}

function reset() {
  if (!level.value) return
  if (code.value !== level.value.starter && !window.confirm('確定要把程式碼重設回起始狀態嗎？')) return
  code.value = level.value.starter
  result.value = null
  gridStdout.value = null
  ferrisLine.value = null
  mood.value = 'normal'
}
function goto(line: number, col: number) { editorRef.value?.reveal(line, col) }

function onKey(e: KeyboardEvent) {
  const mod = e.ctrlKey || e.metaKey
  if (mod && e.key === 'Enter') {
    e.preventDefault()
    judge(e.shiftKey ? 'submit' : 'run')
  }
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="app">
    <header class="topbar">
      <button class="btn-ghost back" disabled title="POC 只有一關">← 地圖</button>
      <div class="crumb">
        <span class="chapter">第 1 章 沙灘甦醒</span>
        <span class="sep">▸</span>
        <span class="lvl">1-2 {{ level?.title ?? '…' }}</span>
        <span class="best" :aria-label="`最佳 ${bestStars} 星`"><span v-for="i in 3" :key="i" :class="{ on: i <= bestStars }">★</span></span>
      </div>
      <div class="switcher" role="radiogroup" aria-label="視覺風格">
        <button
          v-for="s in STYLES"
          :key="s.id"
          role="radio"
          :aria-checked="look === s.id"
          :class="['sw', `sw-${s.id}`, { on: look === s.id }]"
          @click="setLook(s.id)"
        >
          <span class="sw-dot" />
          <span class="sw-name">{{ s.id }} {{ s.name }}</span>
        </button>
      </div>
      <button class="btn-ghost mode" :title="`深淺色：${modeLabel}`" @click="cycleMode">{{ modeLabel }}</button>
    </header>

    <p v-if="loadError" class="load-error">{{ loadError }}（server 有啟動嗎？）</p>

    <main v-if="level" class="cols">
      <section class="col col-left panel">
        <StoryPanel :look="look" :story-lines="storyLines" :ferris-line="ferrisLine" :mood="mood" :concept="level.concept" :hints="level.hints" :revealed="revealed" @reveal="revealed++" />
      </section>

      <section class="col col-mid panel">
        <div class="file-tab"><span class="file-name">main.rs</span><span class="kbd-hint">Ctrl+Enter 執行 · Ctrl+Shift+Enter 送出</span></div>
        <div class="editor-wrap">
          <CodeEditor ref="editorRef" v-model="code" :look="look" :dark="dark" :diagnostics="diagnostics" @run="judge('run')" @submit="judge('submit')" />
        </div>
        <div class="actionbar">
          <button class="btn-danger" :disabled="loading" @click="reset">↺ 重設</button>
          <span class="spacer" />
          <button class="btn-secondary" :disabled="loading" @click="judge('run')">
            <span v-if="loading && action === 'run'" class="spin" />{{ loading && action === 'run' ? '編譯中…' : '▶ 執行' }}
          </button>
          <button class="btn-primary" :disabled="loading" @click="judge('submit')">
            <span v-if="loading && action === 'submit'" class="spin" />{{ loading && action === 'submit' ? '判題中…' : '✓ 送出' }}
          </button>
        </div>
      </section>

      <section class="col col-right">
        <div class="panel viz">
          <GridWorld :grid="level.grid" :stdout="gridStdout" :look="look" :overlay="gridOverlay" :play-token="playToken" @line="activeLine = $event" @finished="onGridFinished" />
        </div>
        <div class="panel cons">
          <ConsolePanel v-model:tab="tab" :result="result" :action="action" :loading="loading" :slow="slow" :net-error="netError" :stale="stale" :active-line="activeLine" :look="look" @goto="goto" />
        </div>
      </section>
    </main>

    <ClearModal v-if="showClear" :look="look" :stars="stars" @close="showClear = false" />
  </div>
</template>
