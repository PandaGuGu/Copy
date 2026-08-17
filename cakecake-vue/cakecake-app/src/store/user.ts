import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/api/auth'
import type { User } from '@/utils/types'

export const useUserStore = defineStore('user', () => {
  const token = ref<string>('')
  const refreshToken = ref<string>('')
  const user = ref<User | null>(null)

  const isLoggedIn = computed(() => !!token.value)

  /** 登录后：存 token，再拉取用户信息（真实登录响应只有 token） */
  async function login(username: string, password: string) {
    const resp = await authApi.login(username, password)
    token.value = resp.access_token
    refreshToken.value = resp.refresh_token
    uni.setStorageSync('access_token', resp.access_token)
    uni.setStorageSync('refresh_token', resp.refresh_token)
    await refreshMe()
  }

  function restoreFromStorage() {
    const t = uni.getStorageSync('access_token')
    const r = uni.getStorageSync('refresh_token')
    const u = uni.getStorageSync('user')
    if (t) token.value = t as string
    if (r) refreshToken.value = r as string
    if (u) user.value = u as User
  }

  async function refreshMe() {
    if (!token.value) return null
    try {
      const me = await authApi.me()
      user.value = me
      uni.setStorageSync('user', me)
      return me
    } catch {
      return null
    }
  }

  async function logout() {
    try { await authApi.logout() } catch { /* ignore */ }
    token.value = ''
    refreshToken.value = ''
    user.value = null
    uni.removeStorageSync('access_token')
    uni.removeStorageSync('refresh_token')
    uni.removeStorageSync('user')
  }

  return {
    token,
    refreshToken,
    user,
    isLoggedIn,
    login,
    restoreFromStorage,
    refreshMe,
    logout
  }
})