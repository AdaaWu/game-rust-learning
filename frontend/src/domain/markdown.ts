import DOMPurify from 'dompurify'
import MarkdownIt from 'markdown-it'

const md = new MarkdownIt({ html: true, linkify: false })
// 連結一律交給 DOMPurify 過濾危險協定
md.validateLink = () => true

// Markdown 渲染，經 DOMPurify 過濾
export function renderMarkdown(source: string): string {
  return DOMPurify.sanitize(md.render(source))
}

// 行內 markdown（MemoryView note）
export function renderInline(source: string): string {
  return DOMPurify.sanitize(md.renderInline(source))
}
