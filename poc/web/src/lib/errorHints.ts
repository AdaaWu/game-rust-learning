// 錯誤碼 → 人話；{0} 代入 message 中第一個反引號名稱
const HINTS: Record<string, string> = {
  E0384: '`{0}` 沒有宣告成 mut，所以不能再改它的值。要改就寫成 `let mut {0}`。',
  E0596: '這裡想修改 `{0}`（例如 push），但它不是 mut。在宣告的地方加上 `mut` 就能改。',
  E0382: '`{0}` 已經被移走（move）了，這裡不能再用它。',
  E0499: '`{0}` 同時被借了兩次 &mut，Rust 只允許一個可變借用。',
  E0502: '`{0}` 正在被 & 借用時，又想用 &mut 修改它。',
  E0308: '型別對不上：編譯器預期的型別和你給的不同。',
  E0425: '找不到 `{0}` 這個名稱，檢查拼字或是否有宣告。',
  E0599: '`{0}` 這個方法不存在於這個型別上。',
}

export function humanize(code: string | null, message: string): string | null {
  if (!code || !HINTS[code]) return null
  const name = /`([^`]+)`/.exec(message)?.[1] ?? '這個變數'
  return HINTS[code].replaceAll('{0}', name)
}
