<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { Diagnostic, RunResponse, SubmitResponse } from '../lib/api'
import { humanize } from '../lib/errorHints'
import { parseCommands } from '../lib/grid'

export type Tab = 'output' | 'errors' | 'tests'
const props = defineProps<{
  tab: Tab
  result: RunResponse | SubmitResponse | null
  action: 'run' | 'submit' | null
  loading: boolean
  slow: boolean
  netError: string | null
  stale: boolean
  activeLine: number | null
  look: 'A' | 'B' | 'C'
}>()
const emit = defineEmits<{ (e: 'update:tab', t: Tab): void; (e: 'goto', line: number, col: number): void }>()

const diags = computed<Diagnostic[]>(() => props.result?.compile.diagnostics ?? [])
const errors = computed(() => diags.value.filter((d) => d.level === 'error'))
const warnings = computed(() => diags.value.filter((d) => d.level === 'warning'))
const tests = computed(() => (props.result && 'tests' in props.result ? props.result.tests : []))
const summary = computed(() => (props.result && 'summary' in props.result ? props.result.summary : null))
const exec = computed(() => props.result?.execution ?? null)
const lines = computed(() => {
  if (!exec.value || props.action !== 'run') return []
  const cmdLines = new Set(parseCommands(exec.value.stdout).map((c) => c.line))
  const invalid = new Set(parseCommands(exec.value.stdout).filter((c) => c.kind === 'invalid').map((c) => c.line))
  return exec.value.stdout.replace(/\n$/, '').split('\n').map((text, i) => ({ text, i, cmd: cmdLines.has(i), invalid: invalid.has(i) }))
})
const openRaw = ref<Record<number, boolean>>({})
const openTest = ref<Record<string, boolean>>({})
watch(
  () => props.result,
  () => {
    openRaw.value = {}
    const firstFail = tests.value.find((t) => t.status !== 'passed')
    openTest.value = firstFail ? { [firstFail.name]: true } : {}
  },
)
const outBox = ref<HTMLElement>()
watch(
  () => props.activeLine,
  async (l) => {
    await nextTick()
    if (l !== null) outBox.value?.querySelector(`[data-line="${l}"]`)?.scrollIntoView({ block: 'nearest' })
  },
)
const TEST_ZH: Record<string, string> = {
  ferris_picks_the_gem: 'Ferris 撿到寶石',
  ferris_reaches_the_flag: 'Ferris 抵達終點旗子',
}
const banner = computed(() => {
  const r = props.result
  if (!r) return null
  switch (r.status) {
    case 'ok':
      return props.action === 'submit'
        ? { kind: 'good', text: '✓ 全部測試通過！' }
        : { kind: 'good', text: `✓ 執行完成（編譯 ${(r.compile.duration_ms / 1000).toFixed(1)}s）` }
    case 'compile_error':
      return { kind: 'bad', text: `✗ 編譯失敗，${errors.value.length} 個錯誤` }
    case 'test_failed':
      return { kind: 'bad', text: `✗ 測試沒過：${summary.value?.passed ?? 0} / ${summary.value?.total ?? 0} 通過` }
    case 'runtime_panic':
      return { kind: 'bad', text: '✗ 程式在執行途中當掉了（panic）' }
    case 'timeout':
      return { kind: 'warn', text: '⏱ 程式跑太久被停下來了，檢查 loop 或 while 的結束條件' }
    case 'output_limit':
      return { kind: 'warn', text: '⚠ 輸出太多被截斷了，是不是在無窮迴圈裡 println!？' }
    default:
      return { kind: 'warn', text: r.status }
  }
})
// 反引號包住的片段用等寬字顯示
const split = (t: string) => t.split('`')
const prompt = computed(() => (props.look === 'C' ? '$' : props.look === 'A' ? '>' : '›'))
</script>

<template>
  <section class="console">
    <div class="tabs" role="tablist">
      <button role="tab" :aria-selected="tab === 'output'" :class="{ on: tab === 'output' }" @click="emit('update:tab', 'output')">輸出</button>
      <button role="tab" :aria-selected="tab === 'errors'" :class="{ on: tab === 'errors' }" @click="emit('update:tab', 'errors')">
        編譯錯誤 <span v-if="errors.length" class="count">{{ errors.length }}</span>
      </button>
      <button role="tab" :aria-selected="tab === 'tests'" :class="{ on: tab === 'tests' }" @click="emit('update:tab', 'tests')">
        測試結果 <span v-if="summary" class="count" :class="{ okc: summary.failed + summary.not_run === 0 && tests.length }">{{ summary.passed }}/{{ summary.total }}</span>
      </button>
    </div>

    <div class="body" aria-live="polite">
      <div v-if="loading" class="skeleton">
        <div class="sk" /><div class="sk short" /><div class="sk" />
        <p class="muted">{{ action === 'submit' ? '編譯並跑隱藏測試中…' : '編譯中…' }}<template v-if="slow">　第一次編譯比較久，請稍候</template></p>
      </div>
      <div v-else-if="netError" class="card busy">{{ netError }}</div>
      <template v-else-if="result">
        <div v-if="stale" class="stale">程式已修改，這是修改前那份程式碼的結果</div>
        <div v-if="banner" class="banner" :class="banner.kind">{{ banner.text }}</div>

        <!-- 輸出 -->
        <div v-if="tab === 'output'" ref="outBox" class="out">
          <div v-for="w in warnings" :key="`w${w.line}${w.message}`" class="card warn small">⚠ 第 {{ w.line }} 行：{{ w.message }}</div>
          <template v-if="exec">
            <div v-for="l in lines" :key="l.i" :data-line="l.i" class="line" :class="{ cmd: l.cmd, active: activeLine === l.i, invalid: l.invalid }">
              <span class="pr">{{ l.invalid ? '✗' : prompt }}</span>{{ l.text || ' ' }}
            </div>
            <pre v-if="action === 'submit'" class="raw">{{ exec.stdout }}</pre>
            <pre v-if="exec.stderr" class="stderr">{{ exec.stderr }}</pre>
          </template>
          <p v-else class="muted">程式還沒編譯成功，沒有輸出。看看「編譯錯誤」分頁。</p>
        </div>

        <!-- 編譯錯誤 -->
        <div v-else-if="tab === 'errors'" class="errs">
          <p v-if="!errors.length && !warnings.length" class="muted">沒有編譯錯誤 🎉</p>
          <article
            v-for="(d, i) in [...errors, ...warnings]"
            :key="i"
            class="card err-card"
            :class="d.level"
            tabindex="0"
            @click="d.line && emit('goto', d.line, d.column ?? 1)"
            @keydown.enter="d.line && emit('goto', d.line, d.column ?? 1)"
          >
            <header>
              <span class="icon">{{ d.level === 'error' ? '✗' : '⚠' }}</span>
              <span v-if="d.line">第 {{ d.line }} 行第 {{ d.column }} 欄</span>
              <a v-if="d.code && d.explanation_url" class="code" :href="d.explanation_url" target="_blank" rel="noopener" @click.stop>{{ d.code }}</a>
              <span v-else-if="d.code" class="code">{{ d.code }}</span>
            </header>
            <p class="human"><template v-for="(seg, k) in split(humanize(d.code, d.message) ?? d.message)" :key="k"><code v-if="k % 2" class="ic">{{ seg }}</code><template v-else>{{ seg }}</template></template></p>
            <button class="link" @click.stop="openRaw[i] = !openRaw[i]">{{ openRaw[i] ? '▾' : '▸' }} 顯示 rustc 原文</button>
            <pre v-if="openRaw[i]" class="raw">{{ d.rendered ?? d.message }}</pre>
            <a v-if="d.explanation_url" class="doc" :href="d.explanation_url" target="_blank" rel="noopener" @click.stop>📖 {{ d.code }} 官方說明 ↗</a>
          </article>
        </div>

        <!-- 測試結果 -->
        <div v-else class="tests">
          <p v-if="!tests.length" class="muted">{{ action === 'submit' ? '編譯失敗，測試沒有執行。' : '按「送出」才會跑隱藏測試。' }}</p>
          <div v-for="t in tests" :key="t.name" class="test" :class="t.status">
            <button class="test-row" @click="openTest[t.name] = !openTest[t.name]">
              <span class="ti">{{ t.status === 'passed' ? '✓' : t.status === 'failed' ? '✗' : '–' }}</span>
              <span>{{ TEST_ZH[t.name] ?? t.name }}</span>
              <code class="tn">{{ t.name }}</code>
            </button>
            <pre v-if="t.message && openTest[t.name]" class="raw">{{ t.message }}</pre>
          </div>
        </div>
      </template>
      <p v-else class="muted">按「執行」（Ctrl+Enter）看看 Ferris 會怎麼走。</p>
    </div>
  </section>
</template>

<style scoped>
.console { display: flex; flex-direction: column; min-height: 0; height: 100%; }
.tabs { display: flex; gap: 4px; border-bottom: var(--border-w) solid var(--border); }
.tabs button { background: none; border: none; box-shadow: none; padding: 8px 10px; color: var(--muted); border-bottom: 3px solid transparent; border-radius: 0; font-size: 13px; transform: none; }
.tabs button.on { color: var(--text); border-bottom-color: var(--brand); font-weight: 700; }
.count { display: inline-block; min-width: 18px; padding: 0 5px; border-radius: 9px; background: var(--danger); color: #fff; font-size: 11px; font-family: var(--font-code); }
.count.okc { background: var(--success); }
.body { flex: 1; overflow: auto; padding: 10px 2px; font-size: 13px; }
.muted { color: var(--muted); }
.banner { padding: 7px 10px; border-radius: var(--radius-sm); margin-bottom: 8px; font-weight: 700; }
.banner.good { background: color-mix(in srgb, var(--success) 15%, transparent); color: var(--success); }
.banner.bad { background: color-mix(in srgb, var(--danger) 13%, transparent); color: var(--danger); }
.banner.warn { background: color-mix(in srgb, var(--warning) 15%, transparent); color: var(--warning); }
.stale { font-size: 12px; color: var(--muted); margin-bottom: 6px; font-style: italic; }
.out { font-family: var(--font-code); font-size: var(--console-size); }
.line { padding: 1px 8px; border-left: 3px solid transparent; white-space: pre-wrap; }
.line.cmd { color: var(--text); }
.line:not(.cmd) { color: var(--muted); }
.line.active { background: var(--brand-soft); border-left-color: var(--brand); }
.line.invalid { border-left-color: var(--danger); color: var(--danger); }
.pr { color: var(--brand); margin-right: 8px; }
.raw, .stderr { font-family: var(--font-code); font-size: 12px; white-space: pre-wrap; background: var(--surface-2); padding: 8px; border-radius: var(--radius-sm); margin: 6px 0; overflow-x: auto; }
.stderr { color: var(--danger); }
.card { background: var(--surface); border: var(--border-w) solid var(--border); border-radius: var(--radius); padding: 10px 12px; margin-bottom: 8px; box-shadow: var(--shadow-sm); }
.card.small { padding: 6px 10px; font-size: 12px; }
.card.busy { color: var(--muted); }
.err-card { cursor: pointer; border-left: 5px solid var(--danger); }
.err-card.warning { border-left-color: var(--warning); }
.err-card:hover { border-color: var(--danger); }
.err-card header { display: flex; gap: 8px; align-items: center; font-family: var(--font-code); font-size: 12px; color: var(--muted); }
.err-card .icon { color: var(--danger); font-weight: 700; }
.err-card .code { margin-left: auto; color: var(--danger); font-weight: 700; text-decoration: none; padding: 1px 6px; border: 1px solid currentColor; border-radius: var(--radius-sm); }
.human { margin: 6px 0; font-size: 14px; line-height: 1.6; }
.ic { font-family: var(--font-code); background: var(--surface-2); padding: 0 5px; border-radius: var(--radius-sm); color: var(--heading); }
.link { background: none; border: none; box-shadow: none; color: var(--muted); padding: 0; font-size: 12px; transform: none; }
.doc { display: block; font-size: 12px; color: var(--info); margin-top: 4px; }
.test { border-bottom: 1px dashed var(--border); }
.test-row { display: flex; gap: 8px; width: 100%; text-align: left; background: none; border: none; box-shadow: none; padding: 8px 4px; color: var(--text); transform: none; }
.ti { width: 18px; font-weight: 700; }
.test.passed .ti { color: var(--success); } .test.failed .ti { color: var(--danger); } .test.not_run .ti { color: var(--muted); }
.tn { margin-left: auto; font-size: 11px; color: var(--muted); }
.skeleton .sk { height: 12px; background: var(--surface-2); border-radius: var(--radius-sm); margin: 8px 0; animation: pulse 1s infinite alternate; }
.skeleton .sk.short { width: 60%; }
@keyframes pulse { to { opacity: .4; } }
</style>
