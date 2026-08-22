import { request } from '@/utils/request'

/** 视频上传（multipart/form-data）：真实 POST /api/v1/videos */
export interface UploadResult {
  id: number
  title: string
  status: string  // processing / published / failed
}

export const uploadApi = {
  /**
   * 真实视频上传。用 uni.uploadFile 走 form-data；不能用普通 axios（JSON Content-Type）。
   * @param token 已登录用户 access token
   */
  uploadVideo(token: string, filePath: string, title: string, description = ''): Promise<UploadResult> {
    // App 端无 vite 代理，必须用局域网 IP（与 request.ts 同款条件编译）
    // #ifdef APP-PLUS
    const apiBase = import.meta.env.VITE_API_BASE_URL_APP || 'http://192.168.1.100:18080'
    // #endif
    // #ifndef APP-PLUS
    const apiBase = import.meta.env.VITE_API_BASE_URL || ''
    // #endif
    return new Promise((resolve, reject) => {
      uni.uploadFile({
        url: apiBase + '/api/v1/videos',
        filePath,
        name: 'file',
        formData: { title, description },
        header: { Authorization: `Bearer ${token}` },
        success: (res) => {
          try {
            const data = JSON.parse(res.data) as { code: number; data?: UploadResult; msg?: string }
            if (data.code === 0) resolve(data.data!)
            else reject(new Error(data.msg || `上传失败 ${data.code}`))
          } catch (e) {
            reject(new Error('响应解析失败'))
          }
        },
        fail: (err) => reject(new Error(err.errMsg || '网络异常'))
      })
    })
  }
}