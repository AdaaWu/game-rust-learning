import type { LevelSummary } from '../api/types'

// 關卡編號「章-章內序」；找不到回空字串
export function levelNumber(levelId: string, levels: LevelSummary[]): string {
  const level = levels.find((l) => l.id === levelId)
  if (!level) return ''
  const sameChapter = levels.filter((l) => l.chapter === level.chapter).sort((a, b) => a.order - b.order)
  return `${level.chapter}-${sameChapter.indexOf(level) + 1}`
}
