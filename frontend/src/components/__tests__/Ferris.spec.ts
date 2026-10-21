import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import Ferris from '../Ferris.vue'
import type { FerrisMood } from '../Ferris.vue'

const MOODS: FerrisMood[] = ['normal', 'happy', 'dizzy', 'confused']

function mountFerris(mood: FerrisMood) {
  return mount(Ferris, { props: { mood } })
}

describe('Ferris 眼睛', () => {
  it.each(MOODS)('%s 表情有左右兩顆白色眼白', (mood) => {
    const whites = mountFerris(mood).findAll('.eye-white')
    expect(whites).toHaveLength(2)
    for (const w of whites) {
      expect(w.attributes('fill')).toBe('#fff')
      expect(w.attributes('stroke')).toBe('var(--ferris-ink)')
    }
  })

  it.each(MOODS)('%s 表情的眼珠或眼形畫在眼白之上', (mood) => {
    const svg = mountFerris(mood).element as SVGSVGElement
    const nodes = Array.from(svg.querySelectorAll<SVGElement>('.eye-white, .eye-mark'))
    const lastWhite = nodes.map((n) => n.classList.contains('eye-white')).lastIndexOf(true)
    const marks = nodes.slice(lastWhite + 1)
    expect(marks.length).toBeGreaterThan(0)
    expect(marks.every((n) => n.classList.contains('eye-mark'))).toBe(true)
  })

  it('開心表情的眼形是上彎弧線', () => {
    const marks = mountFerris('happy').findAll('.eye-mark')
    expect(marks.length).toBeGreaterThan(0)
    for (const m of marks) {
      expect(m.element.tagName).toBe('path')
      expect(m.attributes('d')).toMatch(/Q/)
      expect(m.attributes('fill')).toBe('none')
    }
  })

  it('疑惑表情右眼比左眼大', () => {
    const whites = mountFerris('confused').findAll('.eye-white')
    expect(whites).toHaveLength(2)
    const [left, right] = whites
    expect(Number(right!.attributes('r'))).toBeGreaterThan(Number(left!.attributes('r')))
  })
})
