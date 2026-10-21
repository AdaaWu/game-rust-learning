<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Diagnostic, ExecutionResult, TestResult } from '../api/types'
import { defaultTab } from '../domain/console'
import type { ConsoleResult, ConsoleTab } from '../domain/console'
import { diagnosticCards, errorCount, locationLabel, VISIBLE_CARD_LIMIT } from '../domain/diagnostics'
import { ERROR_HINTS, humanize } from '../domain/errorHints'
import { parseGridCommands } from '../domain/grid/commands'
import { splitAssertion, testCounts } from '../domain/testResults'

const props = defineProps<{
  result: ConsoleResult | null
  loading: boolean
  // 請求超過 3 秒
  slow: boolean
  // 結果回來後程式碼被改過
  stale: boolean
  // 請求期間程式碼被改過
  staleRequest: boolean
  // grid 關卡：stdout 中的無效指令行加紅色左邊框與 ✗
  markGridCommands: boolean
  // timeline 關卡：輸出分頁隱藏 @tl| 開頭的行
  hideTimelineEvents?: boolean
}>()

const TL_PREFIX = '@tl|'

const emit = defineEmits<{
  jump: [line: number, column: number]
}>()

const TABS: { key: ConsoleTab; label: string }[] = [
  { key: 'output', label: '輸出' },
  { key: 'diagnostics', label: '編譯錯誤' },
  { key: 'tests', label: '測試結果' },
]

const data = computed(() => (props.result && 'data' in props.result ? props.result.data : null))
const failure = computed(() => (props.result && 'failure' in props.result ? props.result.failure : null))
const execution = computed<ExecutionResult | null>(() => data.value?.execution ?? null)
const diagnostics = computed<Diagnostic[]>(() => data.value?.compile.diagnostics ?? [])
const tests = computed<TestResult[]>(() => (data.value && 'tests' in data.value ? data.value.tests : []))
const counts = computed(() => testCounts(tests.value))
const isCompileError = computed(() => data.value?.status === 'compile_error')

const cards = computed(() => diagnosticCards(diagnostics.value))
const warningCards = computed(() => (isCompileError.value ? [] : cards.value.filter((d) => d.level === 'warning')))
const diagnosticTabCards = computed(() => (isCompileError.value ? cards.value : []))

const selected = ref<ConsoleTab>('output')
watch(
  () => props.result,
  (result) => {
    if (result && 'data' in result) selected.value = defaultTab(result.action, result.data.status)
    else selected.value = 'output'
  },
  { immediate: true },
)

// 測試列展開狀態：第一個失敗測試預設展開
const expanded = ref(new Set<string>())
watch(
  tests,
  (list) => {
    const first = list.find((t) => t.status === 'failed')
    expanded.value = new Set(first ? [first.name] : [])
  },
  { immediate: true },
)
function toggleTest(name: string) {
  const next = new Set(expanded.value)
  if (next.has(name)) next.delete(name)
  else next.add(name)
  expanded.value = next
}

const stdoutLines = computed(() => {
  const exec = execution.value
  if (!exec || exec.stdout === '') return []
  const lines = exec.stdout.replace(/\r?\n$/, '').split(/\r?\n/)
  const invalid = new Set(
    props.markGridCommands
      ? parseGridCommands(exec.stdout, exec.stdout_truncated)
          .filter((c) => c.kind === 'invalid')
          .map((c) => c.line)
      : [],
  )
  const all = lines.map((text, i) => ({ text, invalid: invalid.has(i) }))
  return props.hideTimelineEvents ? all.filter((l) => !l.text.startsWith(TL_PREFIX)) : all
})

// timeline 關卡隱藏的事件行數（含格式錯誤）
const hiddenTimelineCount = computed(() => {
  const exec = execution.value
  if (!props.hideTimelineEvents || !exec) return 0
  return exec.stdout.split(/\r?\n/).filter((l) => l.startsWith(TL_PREFIX)).length
})

// 測試程式中途當掉：沒有 failed、至少一個 not_run、stderr 非空
const crashStderr = computed(() => {
  const stderr = execution.value?.stderr ?? ''
  const list = tests.value
  if (stderr.trim() === '' || list.some((t) => t.status === 'failed') || !list.some((t) => t.status === 'not_run')) return null
  return stderr.trim()
})

interface StatusCard {
  tone: 'is-danger' | 'is-warning' | 'is-muted'
  title: string
  reason?: string
  detail?: string
}

const statusCard = computed<StatusCard | null>(() => {
  const f = failure.value
  if (f) {
    const titles = {
      server_busy: '伺服器現在有點忙，等一下再試',
      judge_unavailable: '判題服務還沒準備好，關卡內容仍可瀏覽',
      payload_too_large: '程式碼太長了（上限 64 KiB），刪掉一些再試',
      network_error: '連不上伺服器，檢查網路後再試',
    }
    return { tone: 'is-muted', title: titles[f.kind] }
  }
  const d = data.value
  if (!d) return null
  switch (d.status) {
    case 'runtime_panic':
      return { tone: 'is-danger', title: '程式在執行途中當掉了（panic）', detail: d.execution?.stderr ?? '' }
    case 'timeout':
      if (d.compile.timed_out) return { tone: 'is-warning', title: '編譯太久被停下來了（超過 15 秒）' }
      return {
        tone: 'is-warning',
        title: '程式跑太久被停下來了（超過 5 秒）',
        reason: '可能有無窮迴圈或死鎖，檢查 `loop`、`while` 的結束條件與鎖的順序',
      }
    case 'memory_limit':
      if (d.execution === null) return { tone: 'is-warning', title: '編譯時用太多記憶體被停下來了（超過 256 MB）' }
      return {
        tone: 'is-warning',
        title: '程式用太多記憶體被停下來了（超過 256 MB）',
        reason: '可能有無限遞迴，或一次配置了超大的 `Vec`',
      }
    case 'output_limit':
      return { tone: 'is-warning', title: '輸出太多被停下來了（超過 64 KiB）', reason: '可能在無窮迴圈裡 `println!`' }
    default:
      return null
  }
})

const summary = computed(() => {
  if (props.loading) return '判題中'
  const f = failure.value
  if (f) return statusCard.value?.title ?? ''
  const d = data.value
  if (!d) return ''
  const isSubmit = props.result?.action === 'submit'
  switch (d.status) {
    case 'ok':
      return isSubmit ? `全部測試通過，共 ${counts.value.total} 個` : '執行完成'
    case 'compile_error':
      return `編譯失敗，${errorCount(diagnostics.value)} 個錯誤`
    case 'test_failed':
      return `測試失敗，通過 ${counts.value.passed} / 共 ${counts.value.total}`
    case 'runtime_panic':
      return '程式在執行途中當掉了'
    case 'timeout':
      return '執行逾時'
    case 'memory_limit':
      return '記憶體超過上限'
    case 'output_limit':
      return '輸出超過上限'
  }
  return ''
})

function tabLabel(key: ConsoleTab): string {
  if (key === 'diagnostics') return diagnosticTabCards.value.length > 0 ? `編譯錯誤 (${diagnosticTabCards.value.length})` : '編譯錯誤'
  if (key === 'tests' && tests.value.length > 0) return `測試結果 通過 ${counts.value.passed} / 共 ${counts.value.total}`
  return TABS.find((t) => t.key === key)?.label ?? ''
}

// 模板人話：反引號片段渲染為 <code>；字典外的 message 原文照字面顯示
function hintSegments(d: Diagnostic): { text: string; code: boolean }[] {
  const text = humanize(d)
  if (d.code === null || ERROR_HINTS[d.code] === undefined) return [{ text, code: false }]
  return text
    .split('`')
    .map((part, i) => ({ text: part, code: i % 2 === 1 }))
    .filter((s) => s.text !== '')
}

function onCardClick(d: Diagnostic) {
  if (d.line === null) return
  emit('jump', d.line, d.column ?? 1)
}

const TEST_ICON = { passed: '✓', failed: '✗', not_run: '—' }
const TEST_TEXT = { passed: '通過', failed: '失敗', not_run: '未執行' }
</script>

<template>
  <section class="console" data-testid="console">
    <div class="console-tabs" role="tablist" aria-label="主控台">
      <button
        v-for="t in TABS"
        :key="t.key"
        type="button"
        role="tab"
        class="console-tab"
        :data-tab="t.key"
        :aria-selected="selected === t.key"
        @click="selected = t.key"
      >
        {{ tabLabel(t.key) }}
      </button>
    </div>

    <p class="sr-only" aria-live="polite">{{ summary }}</p>

    <div v-if="loading" class="console-body" data-testid="console-skeleton">
      <div class="skeleton-row" />
      <div class="skeleton-row is-short" />
      <div class="skeleton-row" />
      <p v-if="slow" class="console-note">第一次編譯比較久，請稍候</p>
    </div>

    <div v-else class="console-body" :class="{ 'is-stale': stale }" data-testid="console-body">
      <p v-if="staleRequest" class="console-note is-warning">這是修改前那份程式碼的結果</p>
      <p v-if="stale" class="console-note">程式已修改</p>

      <!-- 輸出 -->
      <div v-show="selected === 'output'" role="tabpanel" class="console-panel">
        <div v-if="data?.status === 'ok' && result?.action === 'run'" class="run-success" data-testid="run-success">✓ 執行完成</div>
        <details v-if="warningCards.length > 0" class="warning-cards" data-testid="warning-cards">
          <summary>⚠ 警告 {{ warningCards.length }} 筆</summary>
          <article v-for="(d, i) in warningCards" :key="i" class="diag-card is-warning">
            <header class="diag-head">
              <span>⚠ 警告</span>
              <span>{{ locationLabel(d) }}</span>
              <span v-if="d.code">{{ d.code }}</span>
            </header>
            <p class="diag-hint">
              <template v-for="(s, j) in hintSegments(d)" :key="j"><code v-if="s.code">{{ s.text }}</code><template v-else>{{ s.text }}</template></template>
            </p>
          </article>
        </details>
        <p v-if="hiddenTimelineCount > 0" class="console-note" data-testid="tl-hidden-note">已隱藏 {{ hiddenTimelineCount }} 行時間軸事件</p>
        <pre v-if="stdoutLines.length > 0" class="stdout" data-testid="stdout"><span
            v-for="(line, i) in stdoutLines"
            :key="i"
            class="stdout-line"
            :class="{ 'is-invalid': line.invalid }"
            data-testid="stdout-line"
          ><span v-if="line.invalid" class="stdout-mark" aria-label="無效指令">✗ </span>{{ line.text }}</span><span v-if="execution?.stdout_truncated" class="stdout-truncated">（已截斷）</span></pre>
        <article v-if="statusCard" class="status-card" :class="statusCard.tone" data-testid="status-card">
          <strong>{{ statusCard.title }}</strong>
          <p v-if="statusCard.reason">{{ statusCard.reason }}</p>
          <pre v-if="statusCard.detail" class="status-detail">{{ statusCard.detail }}</pre>
        </article>
        <p v-if="!result" class="console-empty">按「執行」看看結果</p>
      </div>

      <!-- 編譯錯誤 -->
      <div v-show="selected === 'diagnostics'" role="tabpanel" class="console-panel">
        <p v-if="diagnosticTabCards.length === 0" class="console-empty">沒有編譯錯誤</p>
        <template v-for="(d, i) in diagnosticTabCards" :key="i">
          <details v-if="i === VISIBLE_CARD_LIMIT" class="more-cards">
            <summary>還有 {{ diagnosticTabCards.length - VISIBLE_CARD_LIMIT }} 筆</summary>
            <article
              v-for="(m, k) in diagnosticTabCards.slice(VISIBLE_CARD_LIMIT)"
              :key="k"
              class="diag-card"
              :class="[m.level === 'error' ? 'is-error' : 'is-warning', { 'is-clickable': m.line !== null }]"
              data-testid="diag-card"
              data-collapsed="true"
              :data-code="m.code ?? ''"
              @click="onCardClick(m)"
            >
              <header class="diag-head">
                <span>{{ m.level === 'error' ? '✗ 錯誤' : '⚠ 警告' }}</span>
                <span>{{ locationLabel(m) }}</span>
                <a v-if="m.code && m.explanation_url" :href="m.explanation_url" target="_blank" rel="noopener" data-testid="diag-code" @click.stop>{{ m.code }}</a>
                <span v-else-if="m.code">{{ m.code }}</span>
              </header>
              <p class="diag-hint" data-testid="diag-hint">
                <template v-for="(s, j) in hintSegments(m)" :key="j"><code v-if="s.code">{{ s.text }}</code><template v-else>{{ s.text }}</template></template>
              </p>
              <details class="diag-raw" data-testid="diag-raw" @click.stop>
                <summary>顯示 rustc 原文</summary>
                <pre>{{ m.rendered ?? m.message }}</pre>
              </details>
              <a v-if="m.explanation_url" class="diag-doc" :href="m.explanation_url" target="_blank" rel="noopener" data-testid="diag-doc" @click.stop>📖 {{ m.code }} 官方說明 ↗</a>
            </article>
          </details>
          <article
            v-if="i < VISIBLE_CARD_LIMIT"
            class="diag-card"
            :class="[d.level === 'error' ? 'is-error' : 'is-warning', { 'is-clickable': d.line !== null }]"
            data-testid="diag-card"
            :data-code="d.code ?? ''"
            @click="onCardClick(d)"
          >
            <header class="diag-head">
              <span>{{ d.level === 'error' ? '✗ 錯誤' : '⚠ 警告' }}</span>
              <span>{{ locationLabel(d) }}</span>
              <a v-if="d.code && d.explanation_url" :href="d.explanation_url" target="_blank" rel="noopener" data-testid="diag-code" @click.stop>{{ d.code }}</a>
              <span v-else-if="d.code">{{ d.code }}</span>
            </header>
            <p class="diag-hint" data-testid="diag-hint">
              <template v-for="(s, j) in hintSegments(d)" :key="j"><code v-if="s.code">{{ s.text }}</code><template v-else>{{ s.text }}</template></template>
            </p>
            <details class="diag-raw" data-testid="diag-raw" @click.stop>
              <summary>顯示 rustc 原文</summary>
              <pre>{{ d.rendered ?? d.message }}</pre>
            </details>
            <a v-if="d.explanation_url" class="diag-doc" :href="d.explanation_url" target="_blank" rel="noopener" data-testid="diag-doc" @click.stop>📖 {{ d.code }} 官方說明 ↗</a>
          </article>
        </template>
      </div>

      <!-- 測試結果 -->
      <div v-show="selected === 'tests'" role="tabpanel" class="console-panel">
        <article v-if="crashStderr !== null" class="status-card is-danger" data-testid="test-crash-card">
          <strong>測試程式在執行途中整個當掉了，後面的測試沒有機會跑</strong>
          <pre class="status-detail">{{ crashStderr }}</pre>
        </article>
        <p v-if="tests.length === 0" class="console-empty">按「送出」跑測試</p>
        <ul v-else class="test-list">
          <li
            v-for="t in tests"
            :key="t.name"
            class="test-row"
            :class="`is-${t.status.replace('_', '-')}`"
            data-testid="test-row"
            :data-name="t.name"
            :data-expanded="String(expanded.has(t.name))"
          >
            <div class="test-head">
              <span class="test-icon" aria-hidden="true">{{ TEST_ICON[t.status] }}</span>
              <span class="test-status">{{ TEST_TEXT[t.status] }}</span>
              <code class="test-name">{{ t.name }}</code>
              <button
                v-if="t.status === 'failed'"
                type="button"
                class="test-toggle"
                data-testid="test-toggle"
                :aria-expanded="expanded.has(t.name)"
                @click="toggleTest(t.name)"
              >
                {{ expanded.has(t.name) ? '收合' : '展開' }}
              </button>
            </div>
            <div v-if="t.status === 'failed' && expanded.has(t.name) && t.message !== null" class="test-detail">
              <template v-if="splitAssertion(t.message)">
                <p v-if="splitAssertion(t.message)!.title" class="assert-title" data-testid="assert-title">{{ splitAssertion(t.message)!.title }}</p>
                <div class="assert-grid">
                  <div class="assert-box" data-testid="assert-actual">
                    <span class="assert-label">實際（left）</span>
                    <code class="is-diff">{{ splitAssertion(t.message)!.actual }}</code>
                  </div>
                  <div class="assert-box" data-testid="assert-expected">
                    <span class="assert-label">預期（right）</span>
                    <code>{{ splitAssertion(t.message)!.expected }}</code>
                  </div>
                </div>
              </template>
              <pre v-else class="test-message">{{ t.message }}</pre>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </section>
</template>

<style scoped>
.console {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}
.console-tabs {
  display: flex;
  gap: 4px;
  padding: 8px 8px 0;
  border-bottom: var(--border-w) solid var(--border);
  overflow-x: auto;
}
.console-tab {
  border: var(--border-w) solid transparent;
  border-bottom: none;
  border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  background: transparent;
  color: var(--text-muted);
  padding: 6px 12px;
  font: inherit;
  font-size: 13px;
  white-space: nowrap;
  cursor: pointer;
}
.console-tab[aria-selected='true'] {
  background: var(--surface-muted);
  border-color: var(--border);
  color: var(--text);
  font-weight: 700;
}
.console-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px 12px;
  font-size: var(--console-size);
  transition: opacity 200ms;
}
.console-body.is-stale > .console-panel {
  opacity: 0.55;
}
.console-note {
  margin: 0 0 8px;
  color: var(--text-muted);
}
.console-note.is-warning {
  color: var(--warning);
  font-weight: 700;
}
.console-empty {
  color: var(--text-muted);
  margin: 4px 0;
}
.skeleton-row {
  height: 12px;
  margin: 8px 0;
  border-radius: var(--radius-pill);
  background: var(--surface-muted);
  animation: pulse 1.2s ease-in-out infinite;
}
.skeleton-row.is-short {
  width: 60%;
}
.run-success {
  margin-bottom: 8px;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
  font-weight: 700;
}
.stdout {
  margin: 0 0 8px;
  font-family: var(--font-code);
  white-space: pre-wrap;
  word-break: break-word;
}
.stdout-line {
  display: block;
  padding-left: 8px;
  border-left: 3px solid transparent;
}
.stdout-line.is-invalid {
  border-left-color: var(--danger);
  color: var(--danger);
}
.stdout-truncated {
  display: block;
  color: var(--text-muted);
  padding-left: 8px;
}
.status-card,
.diag-card {
  margin: 0 0 8px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  border: var(--border-w) solid var(--border);
  background: var(--surface);
}
.status-card p {
  margin: 4px 0 0;
}
.status-card.is-danger {
  border-color: var(--danger);
  color: var(--danger);
}
.status-card.is-warning {
  border-color: var(--warning);
  color: var(--warning);
}
.status-card.is-muted {
  border-color: var(--neutral-busy);
  color: var(--neutral-busy);
}
.status-detail {
  margin: 6px 0 0;
  font-family: var(--font-code);
  white-space: pre-wrap;
  color: var(--text);
}
.diag-card.is-error {
  border-left: 4px solid var(--danger);
}
.diag-card.is-warning {
  border-left: 4px solid var(--warning);
}
.diag-card.is-clickable {
  cursor: pointer;
}
.diag-card.is-clickable:hover {
  background: var(--surface-muted);
}
.diag-head {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-weight: 700;
}
.diag-card.is-error .diag-head {
  color: var(--danger);
}
.diag-card.is-warning .diag-head {
  color: var(--warning);
}
.diag-head a {
  color: inherit;
}
.diag-hint {
  margin: 6px 0;
  line-height: 1.6;
}
.diag-hint code,
.test-name,
.assert-box code {
  font-family: var(--font-code);
  background: var(--surface-muted);
  padding: 0 4px;
  border-radius: 6px;
}
.diag-raw pre,
.test-message {
  margin: 6px 0 0;
  padding: 8px;
  font-family: var(--font-code);
  white-space: pre-wrap;
  background: var(--surface-muted);
  border-radius: var(--radius-sm);
}
.diag-raw summary,
.more-cards summary,
.warning-cards summary {
  cursor: pointer;
  color: var(--text-muted);
}
.diag-doc {
  display: inline-block;
  margin-top: 6px;
  color: var(--info);
}
.warning-cards {
  margin-bottom: 8px;
}
.test-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}
.test-row {
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  background: var(--surface-muted);
}
.test-head {
  display: flex;
  align-items: center;
  gap: 8px;
}
.test-row.is-passed .test-icon,
.test-row.is-passed .test-status {
  color: var(--success);
}
.test-row.is-failed .test-icon,
.test-row.is-failed .test-status {
  color: var(--danger);
}
.test-row.is-not-run {
  color: var(--text-muted);
}
.test-toggle {
  margin-left: auto;
  border: none;
  background: none;
  color: var(--info);
  cursor: pointer;
  font: inherit;
}
.assert-title {
  margin: 6px 0;
  font-weight: 700;
}
.assert-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.assert-box {
  display: grid;
  gap: 4px;
}
.assert-label {
  font-size: 12px;
  color: var(--text-muted);
}
.assert-box code.is-diff {
  background: color-mix(in srgb, var(--danger) 18%, transparent);
}
@keyframes pulse {
  50% {
    opacity: 0.4;
  }
}
@media (prefers-reduced-motion: reduce) {
  .skeleton-row {
    animation: none;
  }
}
</style>
