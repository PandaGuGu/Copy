import { request } from '@/utils/request'
import type { LoginResp, User } from '@/utils/types'

export const authApi = {
  login(username: string, password: string): Promise<LoginResp> {
    return request({ url: '/api/v1/auth/login', method: 'POST', data: { username, password } })
  },
  register(username: string, password: string, nickname: string): Promise<LoginResp> {
    return request({ url: '/api/v1/auth/register', method: 'POST', data: { username, password, nickname } })
  },
  me(): Promise<User> {
    return request({ url: '/api/v1/users/me', method: 'GET' })
  },
  logout(): Promise<void> {
    return request({ url: '/api/v1/auth/logout', method: 'POST' })
  }
}