import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import ConsolePanel from '../ConsolePanel.vue'
import type { ConsoleResult } from '../../domain/console'
import type { RunResponse, SubmitResponse, TestResult } from '../../api/types'
import { diag, runCompileError, runOk, submitResult } from '../../test/fixtures'

type Props = {
  result: ConsoleResult | null
  loading: boolean
  slow: boolean
  stale: boolean
  staleRequest: boolean
  markGridCommands: boolean
}

function mountConsole(props: Partial<Props> = {}) {
  return mount(ConsolePanel, {
    props: { result: null, loading: false, slow: false, stale: false, staleRequest: false, markGridCommands: false, ...props },
  })
}

const run = (data: RunResponse): ConsoleResult => ({ action: 'run', data })
const submit = (data: SubmitResponse): ConsoleResult => ({ action: 'submit', data })

function selectedTab(w: VueWrapper) {
  return w.get('[role="tab"][aria-selected="true"]').attributes('data-tab')
}

function tab(w: VueWrapper, name: string) {
  return w.get(`[role="tab"][data-tab="${name}"]`)
}

const cards = (w: VueWrapper) => w.findAll('[data-testid="diag-card"]')

function terminated(status: RunResponse['status'], stdout: string, extra: Partial<RunResponse> = {}): RunResponse {
  const base = runOk(stdout)
  return {
    ...base,
    status,
    execution: { ...base.execution!, exit_code: null, timed_out: status === 'timeout' },
    ...extra,
  }
}

const failedTests: TestResult[] = [
  { name: 'three_crabs', status: 'failed', message: 'assertion `left == right` failed: 少了螃蟹\n  left: 2\n right: 3' },
  { name: 'hundred_crabs', status: 'failed', message: 'assertion `left == right` failed: 有寶石在途中消失了\n  left: 0\n right: 5050' },
  { name: 'no_crab', status: 'passed', message: null },
]

describe('F6/F7 ConsolePanel', () => {
  describe('分頁', () => {
    it('三個分頁：輸出、編譯錯誤 (n)、測試結果', () => {
      const w = mountConsole({ result: run(runCompileError([diag(), diag({ line: 2 })])) })
      expect(w.findAll('[role="tab"]').map((t) => t.attributes('data-tab'))).toEqual(['output', 'diagnostics', 'tests'])
      expect(tab(w, 'diagnostics').text()).toContain('編譯錯誤 (2)')
    })

    it('點分頁切換', async () => {
      const w = mountConsole({ result: run(runOk('hi\n')) })
      await tab(w, 'tests').trigger('click')
      expect(selectedTab(w)).toBe('tests')
    })
  })

  describe('ok', () => {
    it('Run 成功：切到輸出、綠色列「✓ 執行完成」、顯示 stdout', () => {
      const w = mountConsole({ result: run(runOk('MOVE DOWN\nMOVE UP\n')) })
      expect(selectedTab(w)).toBe('output')
      expect(w.get('[data-testid="run-success"]').text()).toContain('✓ 執行完成')
      expect(w.get('[data-testid="stdout"]').text()).toContain('MOVE DOWN')
    })

    it('Run 成功但有 warning：warning 卡片在輸出分頁頂部且可收合', () => {
      const warn = diag({ code: 'unused_variables', level: 'warning', message: 'unused variable: `x`', explanation_url: null })
      const data = runOk('MOVE DOWN\n', { compile: { success: true, timed_out: false, diagnostics: [warn] } })
      const w = mountConsole({ result: run(data) })
      expect(selectedTab(w)).toBe('output')
      const box = w.get('[data-testid="warning-cards"]')
      expect(box.element.tagName).toBe('DETAILS')
      expect(box.text()).toContain('⚠')
    })

    it('Submit 通過：切到測試結果、全部打勾', () => {
      const tests: TestResult[] = [
        { name: 'a', status: 'passed', message: null },
        { name: 'b', status: 'passed', message: null },
      ]
      const w = mountConsole({ result: submit(submitResult('ok', tests)) })
      expect(selectedTab(w)).toBe('tests')
      const rows = w.findAll('[data-testid="test-row"]')
      expect(rows.map((r) => r.text())).toEqual([expect.stringContaining('✓'), expect.stringContaining('✓')])
    })
  })

  describe('grid 指令行標示（ux §4.5）', () => {
    const lines = (w: VueWrapper) => w.findAll('[data-testid="stdout-line"]')

    it('stdout 逐行顯示', () => {
      const w = mountConsole({ result: run(runOk('MOVE DOWN\ndebug: x\n')), markGridCommands: true })
      expect(lines(w).map((l) => l.text())).toEqual(['MOVE DOWN', 'debug: x'])
    })

    it('grid 關卡：無效指令那行加紅色左邊框與 ✗', () => {
      const w = mountConsole({ result: run(runOk('MOVE DOWN\nMOVE FORWARD\nMOVE DOWN\n')), markGridCommands: true })
      const marked = lines(w).filter((l) => l.classes().includes('is-invalid'))
      expect(marked.map((l) => l.text())).toEqual([expect.stringContaining('MOVE FORWARD')])
      expect(marked[0].text()).toContain('✗')
    })

    it('合法指令與一般輸出不標示', () => {
      const w = mountConsole({ result: run(runOk('MOVE DOWN\ndebug: x\nmove up\n')), markGridCommands: true })
      expect(w.findAll('.is-invalid')).toHaveLength(0)
    })

    it('非 grid 關卡不標示', () => {
      const w = mountConsole({ result: run(runOk('JUMP\n')), markGridCommands: false })
      expect(w.findAll('.is-invalid')).toHaveLength(0)
    })
  })

  describe('aria-live 摘要', () => {
    const live = (w: VueWrapper) => w.get('[aria-live="polite"]').text()

    it.each([
      ['Run ok', run(runOk('SECRET_STDOUT\n'))],
      ['Submit ok', submit(submitResult('ok'))],
      ['test_failed', submit(submitResult('test_failed', failedTests))],
      ['timeout', run(terminated('timeout', 'SECRET_STDOUT\n'))],
      ['memory_limit', run(terminated('memory_limit', 'SECRET_STDOUT\n'))],
      ['output_limit', run(terminated('output_limit', 'SECRET_STDOUT\n'))],
      ['server_busy', { action: 'run', failure: { kind: 'server_busy', retryAfterSeconds: 2 } } as ConsoleResult],
      ['network_error', { action: 'run', failure: { kind: 'network_error', status: null, code: null, message: null } } as ConsoleResult],
    ])('%s：朗讀一句摘要，不朗讀 stdout 全部內容', (_name, result) => {
      const w = mountConsole({ result })
      expect(live(w).trim()).not.toBe('')
      expect(live(w)).not.toContain('SECRET_STDOUT')
    })

    it('test_failed 摘要含通過數', () => {
      const w = mountConsole({ result: submit(submitResult('test_failed', failedTests)) })
      expect(live(w)).toContain('1')
      expect(live(w)).toContain('3')
    })
  })

  describe('compile_error', () => {
    it('Run 編譯失敗：切到編譯錯誤分頁，每筆一張卡片', () => {
      const w = mountConsole({ result: run(runCompileError([diag(), diag({ code: 'E0384', line: 2, message: 'cannot assign twice to immutable variable `x`' })])) })
      expect(selectedTab(w)).toBe('diagnostics')
      expect(cards(w)).toHaveLength(2)
    })

    it('Submit 編譯失敗：切到編譯錯誤分頁，測試結果照常列出未執行但不切過去', () => {
      const tests: TestResult[] = [{ name: 'a', status: 'not_run', message: null }]
      const data = submitResult('compile_error', tests, { compile: { success: false, timed_out: false, diagnostics: [diag()] } })
      const w = mountConsole({ result: submit(data) })
      expect(selectedTab(w)).toBe('diagnostics')
      expect(w.findAll('[data-testid="test-row"]')).toHaveLength(1)
      expect(w.get('[data-testid="test-row"]').text()).toContain('未執行')
    })

    it('卡片第一行：✗、位置、錯誤碼連結', () => {
      const w = mountConsole({ result: run(runCompileError()) })
      const card = cards(w)[0]
      expect(card.text()).toContain('✗')
      expect(card.text()).toContain('第 10 行第 17 欄')
      const codeLink = card.get('a[data-testid="diag-code"]')
      expect(codeLink.text()).toBe('E0382')
      expect(codeLink.attributes('href')).toBe('https://doc.rust-lang.org/error_codes/E0382.html')
    })

    it('卡片人話層用 errorHints', () => {
      const w = mountConsole({ result: run(runCompileError()) })
      expect(cards(w)[0].get('[data-testid="diag-hint"]').text()).toContain('已經交給別人了，這裡不能再用它。')
    })

    it('人話中的名稱以行內 <code> 呈現，不顯示反引號', () => {
      const w = mountConsole({ result: run(runCompileError()) })
      const hint = cards(w)[0].get('[data-testid="diag-hint"]')
      expect(hint.get('code').text()).toBe('bag')
      expect(hint.text()).not.toContain('`')
    })

    it('抓不到名稱時「這個值」以純文字呈現，不包 <code>', () => {
      const w = mountConsole({ result: run(runCompileError([diag({ message: 'use of moved value' })])) })
      const hint = cards(w)[0].get('[data-testid="diag-hint"]')
      expect(hint.text()).toContain('這個值已經交給別人了')
      expect(hint.findAll('code').map((c) => c.text())).not.toContain('這個值')
    })

    it('E0384 抓不到名稱時「let mut 這個值」整段純文字', () => {
      const w = mountConsole({ result: run(runCompileError([diag({ code: 'E0384', message: 'cannot assign twice' })])) })
      const hint = cards(w)[0].get('[data-testid="diag-hint"]')
      expect(hint.text()).toContain('let mut 這個值')
      expect(hint.findAll('code').map((c) => c.text())).toEqual(['mut'])
    })

    it('rustc 原文預設收合，展開後等寬顯示 rendered', () => {
      const w = mountConsole({ result: run(runCompileError([diag({ rendered: 'error[E0382]: value borrowed here after move' })])) })
      const raw = cards(w)[0].get('details[data-testid="diag-raw"]')
      expect((raw.element as HTMLDetailsElement).open).toBe(false)
      expect(raw.get('pre').text()).toContain('value borrowed here after move')
    })

    it('rendered = null 時原文區顯示 message', () => {
      const d = diag({ line: null, column: null, end_line: null, end_column: null, rendered: null, code: 'E0425', message: 'cannot find function `delta` in this scope' })
      const w = mountConsole({ result: run(runCompileError([d])) })
      expect(cards(w)[0].get('[data-testid="diag-raw"] pre').text()).toBe('cannot find function `delta` in this scope')
      expect(cards(w)[0].text()).toContain('隱藏測試')
    })

    it('官方說明在新分頁開啟；explanation_url 為空時不顯示', () => {
      const w = mountConsole({
        result: run(runCompileError([diag(), diag({ code: null, message: 'expected `;`, found `println`', explanation_url: null, line: 12 })])),
      })
      const [first, second] = cards(w)
      const doc = first.get('a[data-testid="diag-doc"]')
      expect(doc.text()).toContain('E0382 官方說明')
      expect(doc.attributes('target')).toBe('_blank')
      expect(second.find('a[data-testid="diag-doc"]').exists()).toBe(false)
      expect(second.get('[data-testid="diag-hint"]').text()).toBe('expected `;`, found `println`')
    })

    it('卡片排序：error 在前、依行號、無位置排最後、warning 最後', () => {
      const w = mountConsole({
        result: run(
          runCompileError([
            diag({ code: 'E0425', line: null, column: null, end_line: null, end_column: null, rendered: null }),
            diag({ code: null, level: 'warning', line: 2 }),
            diag({ code: 'E0384', line: 7 }),
            diag({ code: 'E0382', line: 3 }),
          ]),
        ),
      })
      expect(cards(w).map((c) => c.attributes('data-code'))).toEqual(['E0382', 'E0384', 'E0425', ''])
    })

    it('超過 5 筆只展開前 5 筆，其餘收進「還有 n 筆」', () => {
      const many = Array.from({ length: 7 }, (_, i) => diag({ line: i + 1 }))
      const w = mountConsole({ result: run(runCompileError(many)) })
      expect(tab(w, 'diagnostics').text()).toContain('編譯錯誤 (7)')
      expect(w.findAll('[data-testid="diag-card"]:not([data-collapsed="true"])')).toHaveLength(5)
      expect(w.text()).toContain('還有 2 筆')
    })

    it('note 與 help 不另開卡片', () => {
      const w = mountConsole({ result: run(runCompileError([diag(), diag({ level: 'note', code: null }), diag({ level: 'help', code: null })])) })
      expect(cards(w)).toHaveLength(1)
    })

    it('點有位置的卡片發出 jump(line, column)', async () => {
      const w = mountConsole({ result: run(runCompileError()) })
      await cards(w)[0].trigger('click')
      expect(w.emitted('jump')).toEqual([[10, 17]])
    })

    it('點 line = null 的卡片不動作', async () => {
      const w = mountConsole({ result: run(runCompileError([diag({ line: null, column: null, end_line: null, end_column: null })])) })
      await cards(w)[0].trigger('click')
      expect(w.emitted('jump')).toBeUndefined()
    })

    it('aria-live 朗讀摘要「編譯失敗，2 個錯誤」', () => {
      const w = mountConsole({ result: run(runCompileError([diag(), diag({ line: 2 }), diag({ level: 'warning', line: 3 })])) })
      expect(w.get('[aria-live="polite"]').text()).toContain('編譯失敗，2 個錯誤')
    })
  })

  describe('test_failed', () => {
    it('切到測試結果，標題「通過 x / 共 n」，依 tests 順序列出', () => {
      const w = mountConsole({ result: submit(submitResult('test_failed', failedTests)) })
      expect(selectedTab(w)).toBe('tests')
      expect(tab(w, 'tests').text()).toContain('通過 1 / 共 3')
      const rows = w.findAll('[data-testid="test-row"]')
      expect(rows.map((r) => r.attributes('data-name'))).toEqual(['three_crabs', 'hundred_crabs', 'no_crab'])
      expect(rows[0].text()).toContain('✗')
      expect(rows[0].text()).toContain('失敗')
      expect(rows[2].text()).toContain('✓')
      expect(rows[2].text()).toContain('通過')
    })

    it('只有第一個失敗測試預設展開', () => {
      const w = mountConsole({ result: submit(submitResult('test_failed', failedTests)) })
      const rows = w.findAll('[data-testid="test-row"]')
      expect(rows.map((r) => r.attributes('data-expanded'))).toEqual(['true', 'false', 'false'])
    })

    it('展開失敗列：自訂訊息當標題，實際與預期並排', async () => {
      const w = mountConsole({ result: submit(submitResult('test_failed', failedTests)) })
      const row = w.get('[data-testid="test-row"][data-name="hundred_crabs"]')
      await row.get('[data-testid="test-toggle"]').trigger('click')
      const expanded = w.get('[data-testid="test-row"][data-name="hundred_crabs"]')
      expect(expanded.get('[data-testid="assert-title"]').text()).toBe('有寶石在途中消失了')
      expect(expanded.get('[data-testid="assert-actual"]').text()).toContain('實際（left）')
      expect(expanded.get('[data-testid="assert-actual"]').text()).toContain('0')
      expect(expanded.get('[data-testid="assert-expected"]').text()).toContain('預期（right）')
      expect(expanded.get('[data-testid="assert-expected"]').text()).toContain('5050')
    })

    it('拆不出來時直接顯示 message 原文', () => {
      const tests: TestResult[] = [{ name: 'some_test', status: 'failed', message: 'RefCell already borrowed' }]
      const w = mountConsole({ result: submit(submitResult('test_failed', tests)) })
      const row = w.get('[data-testid="test-row"]')
      expect(row.text()).toContain('RefCell already borrowed')
      expect(row.find('[data-testid="assert-actual"]').exists()).toBe(false)
    })

    it('未執行：— 圖示、灰色、不可展開', async () => {
      const tests: TestResult[] = [
        { name: 'a', status: 'failed', message: 'x' },
        { name: 'gull_shares_the_chest', status: 'not_run', message: null },
      ]
      const w = mountConsole({ result: submit(submitResult('test_failed', tests)) })
      const row = w.get('[data-testid="test-row"][data-name="gull_shares_the_chest"]')
      expect(row.text()).toContain('—')
      expect(row.text()).toContain('未執行')
      expect(row.classes()).toContain('is-not-run')
      expect(row.find('[data-testid="test-toggle"]').exists()).toBe(false)
    })
  })

  describe('runtime_panic、timeout、memory_limit、output_limit', () => {
    it('panic：先顯示 stdout，最後一張紅色卡片含 stderr', () => {
      const data: RunResponse = {
        ...runOk('MOVE DOWN\n'),
        status: 'runtime_panic',
        execution: { ...runOk('').execution!, exit_code: 101, stdout: 'MOVE DOWN\n', stderr: 'RefCell already borrowed\n' },
      }
      const w = mountConsole({ result: run(data) })
      expect(selectedTab(w)).toBe('output')
      const card = w.get('[data-testid="status-card"]')
      expect(card.text()).toContain('程式在執行途中當掉了（panic）')
      expect(card.text()).toContain('RefCell already borrowed')
      expect(card.classes()).toContain('is-danger')
      expect(w.get('[data-testid="stdout"]').text()).toContain('MOVE DOWN')
    })

    it('執行超時：橘色卡片與常見原因，上方顯示部分 stdout', () => {
      const w = mountConsole({ result: run(terminated('timeout', 'MOVE DOWN\nMOVE DOWN\nMOVE DOWN\n')) })
      const card = w.get('[data-testid="status-card"]')
      expect(card.classes()).toContain('is-warning')
      expect(card.text()).toContain('程式跑太久被停下來了（超過 5 秒）')
      expect(card.text()).toContain('可能有無窮迴圈或死鎖，檢查 `loop`、`while` 的結束條件與鎖的順序')
      expect(w.get('[data-testid="stdout"]').text()).toContain('MOVE DOWN')
    })

    it('編譯超時：「編譯太久被停下來了（超過 15 秒）」', () => {
      const data: RunResponse = { status: 'timeout', compile: { success: false, timed_out: true, diagnostics: [] }, execution: null }
      const w = mountConsole({ result: run(data) })
      expect(w.get('[data-testid="status-card"]').text()).toContain('編譯太久被停下來了（超過 15 秒）')
    })

    it('Submit 超時：已有結果的測試照常顯示，其餘未執行', async () => {
      const tests: TestResult[] = [
        { name: 'deposit_new_item', status: 'passed', message: null },
        { name: 'duplicate_is_skipped', status: 'passed', message: null },
        { name: 'gull_shares_the_chest', status: 'not_run', message: null },
      ]
      const w = mountConsole({ result: submit(submitResult('timeout', tests)) })
      await tab(w, 'tests').trigger('click')
      const rows = w.findAll('[data-testid="test-row"]')
      expect(rows.map((r) => r.text())).toEqual([
        expect.stringContaining('通過'),
        expect.stringContaining('通過'),
        expect.stringContaining('未執行'),
      ])
    })

    it('執行階段記憶體超限：卡片與常見原因', () => {
      const w = mountConsole({ result: run(terminated('memory_limit', 'MOVE DOWN\n')) })
      const card = w.get('[data-testid="status-card"]')
      expect(card.text()).toContain('程式用太多記憶體被停下來了（超過 256 MB）')
      expect(card.text()).toContain('可能有無限遞迴，或一次配置了超大的 `Vec`')
    })

    it('編譯階段記憶體超限：不顯示常見原因', () => {
      const data: RunResponse = { status: 'memory_limit', compile: { success: false, timed_out: false, diagnostics: [] }, execution: null }
      const w = mountConsole({ result: run(data) })
      const card = w.get('[data-testid="status-card"]')
      expect(card.text()).toContain('編譯時用太多記憶體被停下來了（超過 256 MB）')
      expect(card.text()).not.toContain('可能有無限遞迴')
    })

    it('輸出超限：卡片、常見原因、stdout 結尾標（已截斷）', () => {
      const data = terminated('output_limit', 'MOVE DOWN\nMOVE DOWN\nMOVE DO')
      data.execution!.stdout_truncated = true
      const w = mountConsole({ result: run(data) })
      const card = w.get('[data-testid="status-card"]')
      expect(card.text()).toContain('輸出太多被停下來了（超過 64 KiB）')
      expect(card.text()).toContain('可能在無窮迴圈裡 `println!`')
      expect(w.get('[data-testid="stdout"]').text().trim().endsWith('（已截斷）')).toBe(true)
    })
  })

  describe('請求失敗（灰色卡片）', () => {
    it.each([
      [{ kind: 'server_busy', retryAfterSeconds: 2 }, '伺服器現在有點忙，等一下再試'],
      [{ kind: 'judge_unavailable' }, '判題服務還沒準備好，關卡內容仍可瀏覽'],
      [{ kind: 'payload_too_large' }, '程式碼太長了（上限 64 KiB），刪掉一些再試'],
      [{ kind: 'network_error', status: null, code: null, message: null }, '連不上伺服器，檢查網路後再試'],
    ] as const)('%o → %s', (failure, text) => {
      const w = mountConsole({ result: { action: 'run', failure } })
      const card = w.get('[data-testid="status-card"]')
      expect(card.text()).toContain(text)
      expect(card.classes()).toContain('is-muted')
    })
  })

  describe('請求中與程式碼修改', () => {
    it('請求中清空舊內容並顯示骨架列', () => {
      const w = mountConsole({ result: run(runOk('old\n')), loading: true })
      expect(w.find('[data-testid="console-skeleton"]').exists()).toBe(true)
      expect(w.text()).not.toContain('old')
    })

    it('超過 3 秒補一行「第一次編譯比較久，請稍候」', () => {
      const w = mountConsole({ loading: true, slow: true })
      expect(w.text()).toContain('第一次編譯比較久，請稍候')
    })

    it('結果後修改程式碼：內容保留並淡化顯示「程式已修改」', () => {
      const w = mountConsole({ result: run(runCompileError()), stale: true })
      expect(cards(w)).toHaveLength(1)
      expect(w.get('[data-testid="console-body"]').classes()).toContain('is-stale')
      expect(w.text()).toContain('程式已修改')
    })

    it('請求期間改過程式碼：頂部顯示「這是修改前那份程式碼的結果」', () => {
      const w = mountConsole({ result: run(runOk('x\n')), staleRequest: true })
      expect(w.text()).toContain('這是修改前那份程式碼的結果')
    })
  })
})
