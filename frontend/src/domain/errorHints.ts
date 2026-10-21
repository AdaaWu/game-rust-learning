import type { Diagnostic } from '../api/types'

const FALLBACK_NAME = '這個值'
// 中文字與全形標點
const CJK = '[\\p{Script=Han}\\u3000-\\u303f\\uff00-\\uffef]'

// ux.md §3.4 錯誤碼字典
export const ERROR_HINTS: Record<string, string> = {
  E0384: '`{name}` 宣告時沒有加 `mut`，之後不能再改它。想改的話，宣告時寫 `let mut {name}`。',
  E0596: '`{name}` 不是 `mut`，不能借出可修改的參考（例如呼叫 `push`）。宣告時改成 `let mut {name}`。',
  E0382: '`{name}` 已經交給別人了，這裡不能再用它。想一想：要先用完再交出去，還是交一份複製品（`clone`）？',
  E0502: '`{name}` 正被借去讀，同時又要被修改。先讓讀的借用結束（或複製一份出來），再修改。',
  E0499: '`{name}` 同時被借出兩個可修改的參考，一次只能有一個 `&mut`。',
  E0308: '型別對不上：這裡要的型別和你給的不一樣。看原文裡的 expected 與 found。',
  E0425: '找不到 `{name}`：可能拼錯字、還沒宣告，或被刪掉了。',
  E0599: '`{name}` 這個方法在這個型別上不存在。檢查拼字，或確認型別是不是你以為的那個。',
}

// message 第一組反引號內的文字；沒有時回「這個值」
export function extractName(message: string): string {
  const match = /`([^`]*)`/.exec(message)
  return match ? match[1] : FALLBACK_NAME
}

// 人話層文字；字典沒有時回 message 原文
export function humanize(diag: Diagnostic): string {
  const template = diag.code === null ? undefined : ERROR_HINTS[diag.code]
  if (template === undefined) return diag.message
  const found = /`([^`]*)`/.test(diag.message)
  if (found) return template.replaceAll('{name}', extractName(diag.message))
  // 抓不到名稱時含 {name} 的反引號片段改為純文字，「這個值」與相鄰中文之間不留空白
  return template
    .replace(/`([^`]*\{name\}[^`]*)`/g, '$1')
    .replaceAll('{name}', FALLBACK_NAME)
    .replace(new RegExp(`(${CJK}) +${FALLBACK_NAME}`, 'gu'), `$1${FALLBACK_NAME}`)
    .replace(new RegExp(`${FALLBACK_NAME} +(?=${CJK})`, 'gu'), FALLBACK_NAME)
}
