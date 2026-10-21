import { describe, expect, it } from 'vitest'
import { ERROR_HINTS, extractName, humanize } from '../errorHints'
import { diag } from '../../test/fixtures'

describe('F6 errorHints 錯誤碼字典', () => {
  it('首批 8 個錯誤碼', () => {
    expect(Object.keys(ERROR_HINTS).sort()).toEqual(['E0308', 'E0382', 'E0384', 'E0425', 'E0499', 'E0502', 'E0596', 'E0599'])
  })

  it('extractName 取第一組反引號內文字', () => {
    expect(extractName('borrow of moved value: `bag`')).toBe('bag')
    expect(extractName('cannot borrow `a` as mutable because `b` is borrowed')).toBe('a')
  })

  it('extractName 沒有反引號時為「這個值」', () => {
    expect(extractName('mismatched types')).toBe('這個值')
  })

  it('E0382 代入名稱', () => {
    expect(humanize(diag())).toBe('`bag` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？')
  })

  it('E0384 模板中出現兩次 {name} 都代入', () => {
    const d = diag({ code: 'E0384', message: 'cannot assign twice to immutable variable `x`' })
    expect(humanize(d)).toBe('`x` 宣告時沒有加 `mut`，之後不能再改它。想改的話，宣告時寫 `let mut x`。')
  })

  it.each([
    ['E0596', 'cannot borrow `v` as mutable, as it is not declared as mutable', '`v` 不是 `mut`，不能借出可修改的參考（例如呼叫 `push`）。宣告時改成 `let mut v`。'],
    ['E0502', 'cannot borrow `v` as mutable because it is also borrowed as immutable', '`v` 正被借去讀，同時又要被修改。先讓讀的借用結束（或複製一份出來），再修改。'],
    ['E0499', 'cannot borrow `v` as mutable more than once at a time', '`v` 同時被借出兩個可修改的參考，一次只能有一個 `&mut`。'],
    ['E0308', 'mismatched types', '型別對不上：這裡要的型別和你給的不一樣。看原文裡的 expected 與 found。'],
    ['E0425', 'cannot find function `delta` in this scope', '找不到 `delta`：可能拼錯字、還沒宣告，或被刪掉了。'],
    ['E0599', 'no method named `pushh` found for struct `Vec`', '`pushh` 這個方法在這個型別上不存在。檢查拼字，或確認型別是不是你以為的那個。'],
  ])('%s 模板', (code, message, expected) => {
    expect(humanize(diag({ code, message }))).toBe(expected)
  })

  it('模板需要名稱但 message 沒有反引號時代入「這個值」，純文字後不留程式碼用的空白', () => {
    expect(humanize(diag({ code: 'E0382', message: 'use of moved value' }))).toBe(
      '這個值已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
    )
  })

  it('「這個值」以純文字呈現，不帶反引號', () => {
    const text = humanize(diag({ code: 'E0384', message: 'cannot assign twice' }))
    expect(text).not.toContain('`這個值`')
    expect(text).toBe('這個值宣告時沒有加 `mut`，之後不能再改它。想改的話，宣告時寫 let mut 這個值。')
  })

  it.each([
    ['E0384', '這個值宣告時沒有加 `mut`，之後不能再改它。想改的話，宣告時寫 let mut 這個值。'],
    ['E0596', '這個值不是 `mut`，不能借出可修改的參考（例如呼叫 `push`）。宣告時改成 let mut 這個值。'],
    ['E0382', '這個值已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？'],
    ['E0502', '這個值正被借去讀，同時又要被修改。先讓讀的借用結束（或複製一份出來），再修改。'],
    ['E0499', '這個值同時被借出兩個可修改的參考，一次只能有一個 `&mut`。'],
    ['E0308', '型別對不上：這裡要的型別和你給的不一樣。看原文裡的 expected 與 found。'],
    ['E0425', '找不到這個值：可能拼錯字、還沒宣告，或被刪掉了。'],
    ['E0599', '這個值這個方法在這個型別上不存在。檢查拼字，或確認型別是不是你以為的那個。'],
  ])('%s 抓不到名稱時「這個值」與相鄰中文之間不留空白', (code, expected) => {
    expect(humanize(diag({ code, message: 'no name here' }))).toBe(expected)
  })

  it('字典沒有的錯誤碼直接顯示 message 原文', () => {
    expect(humanize(diag({ code: 'E0277', message: 'the trait bound is not satisfied' }))).toBe('the trait bound is not satisfied')
  })

  it('code 為 null 直接顯示 message', () => {
    expect(humanize(diag({ code: null, message: 'expected `;`, found `println`' }))).toBe('expected `;`, found `println`')
  })

  it('lint 名直接顯示 message', () => {
    expect(humanize(diag({ code: 'unused_variables', message: 'unused variable: `x`', level: 'warning' }))).toBe('unused variable: `x`')
  })
})
