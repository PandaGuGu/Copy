/**
 * 移动端 axios 适配层
 *
 * 规则：
 * 1. 后端响应统一 {code, msg, data} 信封（R-API-1）
 * 2. JWT 双 Token：从 Pinia userStore 拿（40100 自动刷新）
 * 3. Loading/Error 状态由各页面用 try/catch + UI 控制（R-FE-6）
 * 4. BaseURL 从环境变量注入（R-FE-7）
 */
import axios, {
  type AxiosAdapter,
  type AxiosInstance,
  type AxiosRequestConfig,
  type AxiosResponse,
  type InternalAxiosRequestConfig
} from 'axios'
import type { ApiEnvelope } from './types'
import { apiCache } from './cache'

// baseURL 策略（条件编译，App/H5 分离）：
// - App 原生端：无 vite 代理，直连后端 → VITE_API_BASE_URL_APP 或局域网默认（打包前按实际 IP 改）
// - H5/小程序 dev：VITE_API_BASE_URL 为空 → 相对路径 /api，由 vite dev 代理服务端转发（真机扫码同源）
// - H5/小程序 生产：VITE_API_BASE_URL 直连后端
let appBaseURL = ''
// #ifdef APP-PLUS
appBaseURL = import.meta.env.VITE_API_BASE_URL_APP || 'http://192.168.1.100:8080'
// #endif
// #ifndef APP-PLUS
appBaseURL = import.meta.env.VITE_API_BASE_URL || ''
// #endif
const baseURL = appBaseURL

/** 扩展请求配置：支持本地缓存（离线回退） */
export interface RequestOptions extends AxiosRequestConfig {
  /** 开启响应缓存（仅 GET，按 URL+params 为键） */
  cacheable?: boolean
  /** 缓存有效期（秒），默认 300 */
  cacheTTL?: number
  /** 强制刷新：跳过缓存直接请求并覆盖缓存 */
  forceRefresh?: boolean
  /** 401 时不强制跳登录（用于可降级的可选接口，如搜索历史；未登录走本地兜底） */
  noAuthRedirect?: boolean
}

/**
 * App 端 axios 适配器：改用 uni.request（原生网络栈）。
 * 背景：App 端默认走 WebView XHR，混合内容策略会拦截 http 明文（与 app 级
 * usesCleartextTraffic 无关），而 uni.request 走原生网络（cleartext 由 manifest 控制）。
 */
function uniRequestAdapter(config: InternalAxiosRequestConfig): Promise<AxiosResponse> {
  return new Promise((resolve, reject) => {
    // 自定义适配器不会像内置适配器那样自动拼 config.params，需手动序列化进 URL
    let url = (config.baseURL || '') + (config.url || '')
    const params = (config.params || {}) as Record<string, any>
    const qs = Object.keys(params)
      .filter((k) => params[k] !== undefined && params[k] !== null && params[k] !== '')
      .map((k) => `${k}=${encodeURIComponent(String(params[k]))}`)
      .join('&')
    if (qs) url += (url.includes('?') ? '&' : '?') + qs
    const method = (config.method || 'get').toUpperCase()
    // axios 的 headers 可能是 AxiosHeaders 实例，统一摊平成普通对象
    const headers: Record<string, string> = {}
    const h = config.headers as any
    if (h) {
      if (typeof h.each === 'function') {
        h.each((v: any, k: string) => {
          if (v != null) headers[k] = String(v)
        })
      } else {
        Object.keys(h).forEach((k) => {
          if (h[k] != null) headers[k] = String(h[k])
        })
      }
    }
    uni.request({
      url,
      method: method as any,
      data: config.data,
      header: headers,
      timeout: config.timeout,
      success: (res) => {
        resolve({
          data: res.data,
          status: res.statusCode,
          statusText: String(res.statusCode),
          headers: (res.header || {}) as any,
          config,
          request: res
        } as AxiosResponse)
      },
      fail: (err) => {
        const error = new Error(err.errMsg || 'network error') as any
        error.isAxiosError = true
        error.config = config
        error.code = err.errMsg
        error.request = err
        reject(error)
      }
    })
  })
}

let appAdapter: AxiosAdapter | undefined
// #ifdef APP-PLUS
appAdapter = uniRequestAdapter
// #endif

export const http: AxiosInstance = axios.create({
  baseURL,
  timeout: 15000,
  headers: { 'Content-Type': 'application/json' },
  adapter: appAdapter
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
        // noAuthRedirect：可选接口（如搜索历史）401 时静默失败，不打断浏览
        if ((response.config as RequestOptions).noAuthRedirect) {
          return Promise.reject(new ApiError(code, msg))
        }
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
    // 网络层错误：若该请求开启了缓存且命中，静默走离线回退（不弹 Toast）
    const cfg = error?.config as (RequestOptions & { _ckCacheKey?: string }) | undefined
    if (error?.isAxiosError && !error?.response && cfg?._ckCacheKey) {
      const hit = apiCache.get(cfg._ckCacheKey)
      if (hit) return hit.data
    }
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

/** 封装：把 http 返回值摊平成 Promise<T>（信封 data）；支持 cacheable 缓存 + 离线回退 */
export function request<T>(config: RequestOptions): Promise<T> {
  const { cacheable, cacheTTL, forceRefresh, ...rest } = config
  const method = (rest.method || 'GET').toUpperCase()

  // 构造缓存键：GET + url + 排序后的 params
  let cacheKey = ''
  if (cacheable && method === 'GET') {
    cacheKey = `${baseURL}${rest.url || ''}${qs(rest.params)}`
    if (!forceRefresh) {
      const hit = apiCache.get<T>(cacheKey)
      if (hit) return Promise.resolve(hit.data)
    }
    // 标记供响应/错误拦截器识别
    ;(rest as RequestOptions & { _ckCacheKey?: string })._ckCacheKey = cacheKey
  }

  return http
    .request(rest as AxiosRequestConfig)
    .then((data) => {
      if (cacheable && method === 'GET' && cacheKey) {
        apiCache.set(cacheKey, data, cacheTTL)
      }
      return data as T
    })
    .catch((err) => {
      // 离线回退兜底（错误拦截器已优先处理，这里再保一层）
      if (cacheable && method === 'GET' && cacheKey && err?.isAxiosError && !err?.response) {
        const hit = apiCache.get<T>(cacheKey)
        if (hit) return hit.data
      }
      throw err
    })
}

/** 将 params 序列化为查询串（参与缓存键） */
function qs(params?: Record<string, any>): string {
  if (!params) return ''
  const keys = Object.keys(params).sort()
  const parts = keys
    .filter((k) => params[k] !== undefined && params[k] !== null && params[k] !== '')
    .map((k) => `${k}=${encodeURIComponent(String(params[k]))}`)
  return parts.length ? `?${parts.join('&')}` : ''
}