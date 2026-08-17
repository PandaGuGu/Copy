import { request } from '@/utils/request'
import type { Banner } from '@/utils/types'

export const bannerApi = {
  /** 首页轮播图（真实响应：{code:0,data:{items:[...]}}；request 已自动解信封 → data.items） */
  async active(): Promise<Banner[]> {
    const data = await request<{ items?: Banner[]; items_legacy?: Banner[]; banners?: Banner[] }>({
      url: '/api/v1/home-banners',
      method: 'GET'
    })
    return data?.items || data?.banners || data?.items_legacy || []
  }
}