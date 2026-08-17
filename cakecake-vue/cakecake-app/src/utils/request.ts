/**
 * 移动端 axios 适配层
 *
 * 规则：
 * 1. 后端响应统一 {code, msg, data} 信封（R-API-1）
 * 2. JWT 双 Token：从 Pinia userStore 拿（40100 自动刷新）
 * 3. Loading/Error 状态由各页面用 try/catch + UI 控制（R-FE-6）
 * 4. BaseURL 从环境变量注入（R-FE-7）
 */
import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosResponse } from 'axios'
import type { ApiEnvelope } from './types'

const baseURL = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8080'

export const http: AxiosInstance = axios.create({
  baseURL,
  timeout: 15000,
  headers: { 'Content-Type': 'application/json' }
})

// Token 注入
http.interceptors.request.use((config) => {
  const token = uni.getStorageSync('access_token') as string | undefined
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// 响应处理：解信封 + 错误码映射 + 401 刷新
let isRefreshing = false
let pendingQueue: Array<(token?: string) => void> = []

http.interceptors.response.use(
  async (response: AxiosResponse<ApiEnvelope>) => {
    const { code, msg, data } = response.data

    // 业务成功
    if (code === 0) return data as any

    // 401 - 未登录 / Token 过期：尝试刷新
    if (code === 40100) {
      const newToken = await tryRefreshToken()
      if (newToken) {
        // 重新发起原请求
        response.config.headers.Authorization = `Bearer ${newToken}`
        return http.request(response.config).then((r) => (r.data as ApiEnvelope).data) as any
      } else {
        // 刷新失败 → 跳登录
        uni.removeStorageSync('access_token')
        uni.removeStorageSync('refresh_token')
        uni.showToast({ title: '请重新登录', icon: 'none' })
        setTimeout(() => uni.reLaunch({ url: '/pages/login/index' }), 800)
        return Promise.reject(new ApiError(code, msg))
      }
    }

    // 其他业务错误：Toast + reject
    uni.showToast({ title: msg || '请求失败', icon: 'none' })
    return Promise.reject(new ApiError(code, msg))
  },
  (error) => {
    // 网络层错误
    const msg = error?.response?.status === 401 ? '请重新登录' : '网络异常，请稍后重试'
    uni.showToast({ title: msg, icon: 'none' })
    return Promise.reject(error)
  }
)

async function tryRefreshToken(): Promise<string | null> {
  if (isRefreshing) {
    return new Promise((resolve) => pendingQueue.push(resolve))
  }
  isRefreshing = true
  const refresh = uni.getStorageSync('refresh_token') as string | undefined
  if (!refresh) {
    isRefreshing = false
    return null
  }
  try {
    const { data } = await axios.post<ApiEnvelope<{ access_token: string }>>(
      `${baseURL}/api/v1/auth/refresh`,
      { refresh_token: refresh }
    )
    const newToken = data.data.access_token
    uni.setStorageSync('access_token', newToken)
    pendingQueue.forEach((cb) => cb(newToken))
    pendingQueue = []
    return newToken
  } catch {
    pendingQueue.forEach((cb) => cb(undefined))
    pendingQueue = []
    return null
  } finally {
    isRefreshing = false
  }
}

/** 业务错误对象（含 code + msg） */
export class ApiError extends Error {
  code: number
  constructor(code: number, msg: string) {
    super(msg)
    this.code = code
    this.name = 'ApiError'
  }
}

/** 封装：把 http 返回值摊平成 Promise<T>（信封 data） */
export function request<T>(config: AxiosRequestConfig): Promise<T> {
  return http.request<unknown, T>(config)
}