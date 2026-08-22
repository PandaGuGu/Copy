import { defineConfig, loadEnv } from 'vite'
import uni from '@dcloudio/vite-plugin-uni'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  // 代理目标（服务端转发）：显式 VITE_PROXY_TARGET 优先，其次 VITE_API_BASE_URL，最后本地后端
  const apiBase = env.VITE_PROXY_TARGET || env.VITE_API_BASE_URL || 'http://127.0.0.1:18080'

  return {
    plugins: [uni()],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      }
    },
    server: {
      host: '0.0.0.0',
      port: 5173,
      proxy: {
        // 开发期跨域代理：H5 前端 → 本地/远端 Go 后端
        '/api': {
          target: apiBase,
          changeOrigin: true,
          rewrite: (p) => p.replace(/^\/api/, '/api')
        }
      }
    },
    build: {
      // H5 部署到 Netlify 路径保持 "/"
      outDir: 'dist/build/h5',
      sourcemap: false
    }
  }
})