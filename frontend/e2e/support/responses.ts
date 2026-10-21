type Json = any

// 由 status 組出完整的 Run／Submit 回應

export function diag(partial: Partial<Json>): Json {
  return {
    code: null,
    message: 'error',
    line: null,
    column: null,
    end_line: null,
    end_column: null,
    level: 'error',
    explanation_url: null,
    rendered: null,
    ...partial,
  }
}

export function e0384(line: number): Json {
  return diag({
    code: 'E0384',
    message: 'cannot assign twice to immutable variable `x`',
    line,
    column: 5,
    end_line: line,
    end_column: 10,
    explanation_url: 'https://doc.rust-lang.org/error_codes/E0384.html',
    rendered: `error[E0384]: cannot assign twice to immutable variable \`x\`\n --> /tmp/main.rs:${line}:5\n`,
  })
}

function execution(stdout: string, extra: Partial<Json> = {}): Json {
  return { exit_code: 0, stdout, stderr: '', timed_out: false, stdout_truncated: false, stderr_truncated: false, ...extra }
}

const COMPILED = { success: true, timed_out: false, diagnostics: [] as Json[] }

export function runResponse(status: string, stdout = '', truncated = false): Json {
  switch (status) {
    case 'ok':
      return { status, compile: COMPILED, execution: execution(stdout) }
    case 'compile_error':
      return { status, compile: { success: false, timed_out: false, diagnostics: [e0384(3)] }, execution: null }
    case 'runtime_panic':
      return {
        status,
        compile: COMPILED,
        execution: execution(stdout, {
          exit_code: 101,
          stderr: "\nthread 'main' (12) panicked at /tmp/main.rs:2:5:\nboom\n",
        }),
      }
    case 'timeout':
      return { status, compile: COMPILED, execution: execution(stdout, { exit_code: null, timed_out: true }) }
    case 'memory_limit':
      return { status, compile: COMPILED, execution: execution(stdout, { exit_code: 137 }) }
    case 'output_limit':
      return { status, compile: COMPILED, execution: execution(stdout, { exit_code: null, stdout_truncated: truncated }) }
    default:
      throw new Error(`未知的 Run status ${status}`)
  }
}

// execution = null 的 Run（編譯失敗、編譯階段 OOM）
export function runWithoutExecution(status: string): Json {
  const diagnostics = status === 'compile_error' ? [e0384(3)] : []
  return { status, compile: { success: false, timed_out: false, diagnostics }, execution: null }
}

const TEST_NAMES = ['reaches_goal', 'collects_gems', 'stops_on_goal']

export function submitResponse(status: string): Json {
  const tests = (s: string) => TEST_NAMES.map((name) => ({ name, status: s, message: null }))
  const base = { status, passed: status === 'ok', compile: COMPILED }
  switch (status) {
    case 'ok':
      return { ...base, execution: execution('test result: ok'), tests: tests('passed') }
    case 'compile_error':
      return {
        ...base,
        compile: { success: false, timed_out: false, diagnostics: [e0384(3)] },
        execution: null,
        tests: tests('not_run'),
      }
    case 'test_failed':
      return {
        ...base,
        execution: execution('test result: FAILED', { exit_code: 101 }),
        tests: [
          { name: TEST_NAMES[0], status: 'failed', message: 'assertion `left == right` failed\n  left: 1\n right: 2' },
          { name: TEST_NAMES[1], status: 'passed', message: null },
          { name: TEST_NAMES[2], status: 'passed', message: null },
        ],
      }
    case 'timeout':
      return { ...base, execution: execution('', { exit_code: null, timed_out: true }), tests: tests('not_run') }
    case 'memory_limit':
      return { ...base, execution: execution('', { exit_code: 137 }), tests: tests('not_run') }
    case 'output_limit':
      return { ...base, execution: execution('x', { exit_code: null, stdout_truncated: true }), tests: tests('not_run') }
    default:
      throw new Error(`未知的 Submit status ${status}`)
  }
}

// Gherkin 字串中的 \n、\" 轉義
export function unescape(text: string): string {
  return text.replace(/\\(n|"|\\)/g, (_, c) => (c === 'n' ? '\n' : c))
}
