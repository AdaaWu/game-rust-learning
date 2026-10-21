// 測試共用資料，依 api.md 範例與 ux.md §4.2 範例地圖
import type {
  ChapterSummary,
  Diagnostic,
  GridDef,
  LevelDetail,
  LevelSummary,
  MemorySnapshot,
  RunResponse,
  SubmitResponse,
  TestResult,
} from '../api/types'

export const chapters: ChapterSummary[] = [
  { chapter: 1, title: '沙灘甦醒' },
  { chapter: 2, title: '潮間帶' },
  { chapter: 3, title: '所有權之海' },
  { chapter: 5, title: '借用燈塔' },
]

// MVP 結構縮減版：第 4 章沒有關卡
export const levels: LevelSummary[] = [
  { id: '01-hello-ferris', chapter: 1, order: 1, title: '第一步', kind: 'build', visual: 'grid' },
  { id: '02-mut-matters', chapter: 1, order: 2, title: '黏住的腳', kind: 'fix-compile', visual: 'grid' },
  { id: '05-delta', chapter: 2, order: 5, title: '方向盤', kind: 'build', visual: 'grid' },
  { id: '09-move-the-gem', chapter: 3, order: 9, title: '交出去的背包', kind: 'fix-compile', visual: 'memory' },
  { id: '11-borrow', chapter: 3, order: 11, title: '借我看看', kind: 'fix-compile', visual: 'memory' },
  { id: '19-lighthouse', chapter: 5, order: 19, title: '燈塔', kind: 'debug', visual: 'none' },
]

// ux.md §4.2 範例地圖
export const sampleGrid: GridDef = {
  width: 6,
  height: 5,
  start: [0, 0],
  goal: [5, 4],
  walls: [[1, 0], [1, 1], [1, 2], [3, 2], [3, 3], [4, 3]],
  gems: [[2, 0], [4, 4]],
  doors: [[3, 4]],
}

// 撿完 2 顆寶石並停在終點的 20 步
export const clearRoute = [
  ...Array(4).fill('MOVE DOWN'),
  ...Array(2).fill('MOVE RIGHT'),
  ...Array(4).fill('MOVE UP'),
  'PICK',
  ...Array(4).fill('MOVE DOWN'),
  'OPEN RIGHT',
  'MOVE RIGHT',
  'MOVE RIGHT',
  'PICK',
  'MOVE RIGHT',
].join('\n') + '\n'

export function gridLevel(overrides: Partial<LevelDetail> = {}): LevelDetail {
  return {
    id: '01-hello-ferris',
    chapter: 1,
    order: 1,
    title: '第一步',
    kind: 'build',
    visual: 'grid',
    story: 'Ferris 在沙灘上醒來',
    concept: '## 函式與回傳值',
    hints: ['提示一', '提示二', '提示三'],
    starter: 'fn main() {}\n',
    grid: sampleGrid,
    memory_snapshots: null,
    ...overrides,
  }
}

export const snapshots: MemorySnapshot[] = [
  {
    line: 8,
    note: 'bag 擁有 heap 上的三個字串',
    frames: [
      {
        id: 'report',
        fn: 'report',
        vars: [{ id: 'report.bag', name: 'bag', type: 'Vec<String>', value: 'len=3 cap=3', state: 'live' }],
      },
    ],
    heap: [{ id: 'h1', type: '[String; 3]', value: '珍珠 | 貝殼 | 海星', state: 'live' }],
    pointers: [{ from: 'report.bag', to: 'h1', kind: 'own' }],
    error: null,
  },
  {
    line: 3,
    note: '擁有權移動到 **count_items** 的參數',
    frames: [
      {
        id: 'report',
        fn: 'report',
        vars: [{ id: 'report.bag', name: 'bag', type: 'Vec<String>', value: null, state: 'moved' }],
      },
      {
        id: 'count_items',
        fn: 'count_items',
        vars: [{ id: 'count_items.bag', name: 'bag', type: 'Vec<String>', value: 'len=3 cap=3', state: 'live' }],
      },
    ],
    heap: [{ id: 'h1', type: '[String; 3]', value: '珍珠 | 貝殼 | 海星', state: 'live' }],
    pointers: [{ from: 'count_items.bag', to: 'h1', kind: 'own' }],
    error: null,
  },
  {
    line: 10,
    note: 'bag 已經被移走，這裡借用失敗',
    frames: [
      {
        id: 'report',
        fn: 'report',
        vars: [
          { id: 'report.bag', name: 'bag', type: 'Vec<String>', value: null, state: 'moved' },
          { id: 'report.r', name: 'r', type: '&Vec<String>', value: null, state: 'live' },
        ],
      },
    ],
    heap: [{ id: 'h1', type: '[String; 3]', value: '珍珠 | 貝殼 | 海星', state: 'freed' }],
    pointers: [{ from: 'report.r', to: 'report.bag', kind: 'shared' }],
    error: { code: 'E0382', target: 'report.bag' },
  },
]

export function memoryLevel(overrides: Partial<LevelDetail> = {}): LevelDetail {
  return {
    id: '09-move-the-gem',
    chapter: 3,
    order: 9,
    title: '交出去的背包',
    kind: 'fix-compile',
    visual: 'memory',
    story: '把背包交給海鷗',
    concept: '## 所有權',
    hints: [],
    starter: 'fn main() {}\n',
    grid: null,
    memory_snapshots: snapshots,
    ...overrides,
  }
}

export function diag(overrides: Partial<Diagnostic> = {}): Diagnostic {
  return {
    code: 'E0382',
    message: 'borrow of moved value: `bag`',
    line: 10,
    column: 17,
    end_line: 10,
    end_column: 20,
    level: 'error',
    explanation_url: 'https://doc.rust-lang.org/error_codes/E0382.html',
    rendered: 'error[E0382]: borrow of moved value: `bag`\n',
    ...overrides,
  }
}

export function runOk(stdout: string, overrides: Partial<RunResponse> = {}): RunResponse {
  return {
    status: 'ok',
    compile: { success: true, timed_out: false, diagnostics: [] },
    execution: {
      exit_code: 0,
      stdout,
      stderr: '',
      timed_out: false,
      stdout_truncated: false,
      stderr_truncated: false,
    },
    ...overrides,
  }
}

export function runCompileError(diagnostics: Diagnostic[] = [diag()]): RunResponse {
  return {
    status: 'compile_error',
    compile: { success: false, timed_out: false, diagnostics },
    execution: null,
  }
}

export function submitResult(
  status: SubmitResponse['status'],
  tests: TestResult[] = [{ name: 'ferris_reaches_exit', status: 'passed', message: null }],
  overrides: Partial<SubmitResponse> = {},
): SubmitResponse {
  return {
    status,
    passed: status === 'ok',
    compile: { success: status !== 'compile_error', timed_out: false, diagnostics: [] },
    execution: null,
    tests,
    ...overrides,
  }
}
