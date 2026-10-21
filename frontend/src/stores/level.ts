import { defineStore } from 'pinia'
import type { ChapterSummary, LevelDetail, LevelSummary } from '../api/types'
import type { ApiResult, LevelFailure, RequestFailure } from '../api/client'
import { api } from '../api'

export const useLevelStore = defineStore('level', {
  state: () => ({
    chapters: [] as ChapterSummary[],
    summaries: [] as LevelSummary[],
    details: new Map<string, LevelDetail>(),
    listLoaded: false,
  }),
  actions: {
    // 已快取則不再請求
    async fetchList(): Promise<ApiResult<LevelSummary[], RequestFailure>> {
      if (this.listLoaded) return { ok: true, data: this.summaries }
      const result = await api.listLevels()
      if (!result.ok) return result
      this.chapters = result.data.chapters
      this.summaries = result.data.levels
      this.listLoaded = true
      return { ok: true, data: this.summaries }
    },
    async fetchDetail(id: string): Promise<ApiResult<LevelDetail, LevelFailure>> {
      const cached = this.details.get(id)
      if (cached) return { ok: true, data: cached }
      const result = await api.getLevel(id)
      if (result.ok) this.details.set(id, result.data)
      return result
    },
  },
})
