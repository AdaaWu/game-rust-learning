import { describe, expect, it } from 'vitest'
import type { MemorySnapshot } from '../../api/types'
import { borrowStateOf, deriveSnapshotView, findStepForLine, stepCaption } from '../memory'
import { snapshots } from '../../test/fixtures'

function snap(overrides: Partial<MemorySnapshot>): MemorySnapshot {
  return { line: 1, note: '', frames: [], heap: [], pointers: [], error: null, ...overrides }
}

const mainFrame = (vars: MemorySnapshot['frames'][number]['vars']) => [{ id: 'main', fn: 'main', vars }]
const v = (id: string, state: 'live' | 'moved' | 'dropped' = 'live') => ({ id, name: id.split('.')[1], type: 'String', value: null, state })

describe('F5 borrowStateOf 借用狀態推導', () => {
  it('沒有指標指向：null', () => {
    expect(borrowStateOf('main.a', [])).toBeNull()
  })

  it('有 shared 指向：shared', () => {
    expect(borrowStateOf('main.a', [{ from: 'main.r', to: 'main.a', kind: 'shared' }])).toBe('shared')
  })

  it('有 mut 指向：mut', () => {
    expect(borrowStateOf('main.a', [{ from: 'main.r', to: 'main.a', kind: 'mut' }])).toBe('mut')
  })

  it('同時有 mut 與 shared：mut 優先', () => {
    expect(
      borrowStateOf('main.a', [
        { from: 'main.r', to: 'main.a', kind: 'shared' },
        { from: 'main.w', to: 'main.a', kind: 'mut' },
      ]),
    ).toBe('mut')
  })

  it('own 指標不算借用', () => {
    expect(borrowStateOf('h1', [{ from: 'main.a', to: 'h1', kind: 'own' }])).toBeNull()
  })

  it('從自己出發的指標不算被借用', () => {
    expect(borrowStateOf('main.r', [{ from: 'main.r', to: 'main.a', kind: 'mut' }])).toBeNull()
  })
})

describe('F5 deriveSnapshotView', () => {
  it('保留 line、note 與 frames 順序，最後一個 frame 為目前執行中', () => {
    const view = deriveSnapshotView(snapshots, 1)
    expect(view.line).toBe(3)
    expect(view.note).toBe(snapshots[1].note)
    expect(view.frames.map((f) => [f.id, f.current])).toEqual([
      ['report', false],
      ['count_items', true],
    ])
  })

  it('變數帶上借用狀態', () => {
    const s = [snap({ frames: mainFrame([v('main.a'), v('main.r')]), pointers: [{ from: 'main.r', to: 'main.a', kind: 'shared' }] })]
    const view = deriveSnapshotView(s, 0)
    expect(view.frames[0].vars.map((x) => x.borrow)).toEqual(['shared', null])
  })

  it('第 1 步沒有「與上一步相比」的變化標記', () => {
    const view = deriveSnapshotView(snapshots, 0)
    expect(view.frames[0].vars[0].changed).toBe(false)
    expect(view.heap[0].changed).toBe(false)
  })

  it('新增或狀態改變的項目標為 changed，沒變的不標', () => {
    const view = deriveSnapshotView(snapshots, 1)
    const report = view.frames[0].vars[0]
    const counted = view.frames[1].vars[0]
    expect(report.changed).toBe(true) // live → moved
    expect(counted.changed).toBe(true) // 新增
    expect(view.heap[0].changed).toBe(false)
  })

  it('新增的 frame 標為 changed，既有 frame 不標', () => {
    expect(deriveSnapshotView(snapshots, 0).frames.map((f) => f.changed)).toEqual([false])
    expect(deriveSnapshotView(snapshots, 1).frames.map((f) => [f.id, f.changed])).toEqual([
      ['report', false],
      ['count_items', true],
    ])
  })

  it('dropped 變數在該步顯示，下一步起移除', () => {
    const s = [
      snap({ frames: mainFrame([v('main.a')]) }),
      snap({ frames: mainFrame([v('main.a', 'dropped')]) }),
      snap({ frames: mainFrame([v('main.a', 'dropped')]) }),
    ]
    expect(deriveSnapshotView(s, 1).frames[0].vars.map((x) => x.id)).toEqual(['main.a'])
    expect(deriveSnapshotView(s, 2).frames[0].vars).toEqual([])
  })

  it('freed heap 區塊在該步顯示，下一步起移除', () => {
    const block = { id: 'h1', type: 'String', value: '"ruby"' }
    const s = [
      snap({ heap: [{ ...block, state: 'live' }] }),
      snap({ heap: [{ ...block, state: 'freed' }] }),
      snap({ heap: [{ ...block, state: 'freed' }] }),
    ]
    expect(deriveSnapshotView(s, 1).heap.map((h) => h.id)).toEqual(['h1'])
    expect(deriveSnapshotView(s, 2).heap).toEqual([])
  })

  it('第 1 步就是 dropped 時照常顯示', () => {
    const s = [snap({ frames: mainFrame([v('main.a', 'dropped')]) })]
    expect(deriveSnapshotView(s, 0).frames[0].vars).toHaveLength(1)
  })
})

describe('F5a 錯誤標記推導', () => {
  it('error.target 為變數：該變數標上錯誤碼徽章', () => {
    const view = deriveSnapshotView(snapshots, 2)
    const bag = view.frames[0].vars.find((x) => x.id === 'report.bag')
    expect(bag?.error).toEqual({ badge: 'E0382' })
    expect(view.frames[0].vars.find((x) => x.id === 'report.r')?.error).toBeNull()
  })

  it('錯誤標記與 moved 狀態疊加，不取代原本狀態', () => {
    const bag = deriveSnapshotView(snapshots, 2).frames[0].vars.find((x) => x.id === 'report.bag')
    expect(bag?.state).toBe('moved')
    expect(bag?.error).not.toBeNull()
  })

  it('code 為 null 時徽章為「執行期錯誤」', () => {
    const s = [snap({ frames: mainFrame([v('main.a')]), error: { code: null, target: 'main.a' } })]
    expect(deriveSnapshotView(s, 0).frames[0].vars[0].error).toEqual({ badge: '執行期錯誤' })
  })

  it('error.target 為 heap 區塊：該區塊標上錯誤', () => {
    const s = [
      snap({
        frames: mainFrame([v('main.a')]),
        heap: [{ id: 'h1', type: 'String', value: '"x"', state: 'freed' }],
        pointers: [{ from: 'main.a', to: 'h1', kind: 'own' }],
        error: { code: 'E0505', target: 'h1' },
      }),
    ]
    const view = deriveSnapshotView(s, 0)
    expect(view.heap[0].error).toEqual({ badge: 'E0505' })
    expect(view.frames[0].vars[0].error).toBeNull()
    expect(view.pointers[0].danger).toBe(false)
  })

  it('從出事變數出發的指標轉紅，種類不變', () => {
    const s = [
      snap({
        frames: mainFrame([v('main.a'), v('main.b')]),
        heap: [{ id: 'h1', type: 'String', value: '"x"', state: 'live' }],
        pointers: [
          { from: 'main.a', to: 'h1', kind: 'own' },
          { from: 'main.b', to: 'main.a', kind: 'mut' },
        ],
        error: { code: 'E0499', target: 'main.a' },
      }),
    ]
    const view = deriveSnapshotView(s, 0)
    expect(view.pointers).toEqual([
      { from: 'main.a', to: 'h1', kind: 'own', danger: true },
      { from: 'main.b', to: 'main.a', kind: 'mut', danger: false },
    ])
  })

  it('非錯誤步驟沒有任何錯誤標記', () => {
    const view = deriveSnapshotView(snapshots, 0)
    expect(view.frames.flatMap((f) => f.vars).length).toBeGreaterThan(0)
    expect(view.heap.length).toBeGreaterThan(0)
    expect(view.pointers.length).toBeGreaterThan(0)
    expect(view.frames.flatMap((f) => f.vars).every((x) => x.error === null)).toBe(true)
    expect(view.heap.every((h) => h.error === null)).toBe(true)
    expect(view.pointers.every((p) => !p.danger)).toBe(true)
  })
})

describe('F5 說明列與行號同步', () => {
  it('說明列「第 n 步 · 第 L 行」', () => {
    expect(stepCaption(snapshots, 0)).toBe('第 1 步 · 第 8 行')
    expect(stepCaption(snapshots, 2)).toBe('第 3 步 · 第 10 行')
  })

  it('反向同步：找第一個 line 相同的步驟', () => {
    const s = [snap({ line: 3 }), snap({ line: 5 }), snap({ line: 3 })]
    expect(findStepForLine(s, 3)).toBe(0)
    expect(findStepForLine(s, 5)).toBe(1)
  })

  it('沒有對應步驟回 null', () => {
    expect(findStepForLine(snapshots, 99)).toBeNull()
  })
})
