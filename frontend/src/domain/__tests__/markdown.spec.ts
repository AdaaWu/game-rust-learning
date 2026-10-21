import { describe, expect, it } from 'vitest'
import { renderInline, renderMarkdown } from '../markdown'

describe('F7 Markdown 渲染與 XSS 過濾', () => {
  it('標題與程式碼區塊', () => {
    const html = renderMarkdown('## 觀念\n\n```rust\nfn main() {}\n```\n')
    expect(html).toContain('<h2>觀念</h2>')
    expect(html).toContain('<code')
  })

  it('移除 script', () => {
    expect(renderMarkdown('hi <script>alert(1)</script>')).not.toContain('<script')
  })

  it('移除事件屬性', () => {
    expect(renderMarkdown('<img src=x onerror="alert(1)">')).not.toContain('onerror')
  })

  it('javascript: 連結被移除', () => {
    expect(renderMarkdown('[x](javascript:alert(1))')).not.toContain('javascript:')
  })

  it('行內 markdown 不包 <p>', () => {
    const html = renderInline('**粗體** 與 `code`')
    expect(html).toBe('<strong>粗體</strong> 與 <code>code</code>')
  })

  it('行內 markdown 同樣過濾', () => {
    expect(renderInline('<img src=x onerror=alert(1)>')).not.toContain('onerror')
  })
})
