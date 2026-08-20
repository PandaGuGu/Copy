<script setup lang="ts">
import { onLaunch, onShow, onHide } from '@dcloudio/uni-app'
import { useUserStore } from '@/store/user'

onLaunch(() => {
  console.log('[App] onLaunch')
  const userStore = useUserStore()
  userStore.restoreFromStorage()
  // #ifdef APP-PLUS
  // uni-app 的 App-vue 页面(WebView 渲染)不像 nvue 那样自动提供 --status-bar-height
  // CSS 变量,导致全局 .safe-area-top { padding-top: var(--status-bar-height, 0) } 在
  // 真机基座上 fallback 成 0,导航栏被状态栏穿透(参看 user phone-screen.png 现场)。
  // 这里读取状态栏高度并注入到 documentElement,使 .safe-area-top 真正把内容下推到
  // 状态栏底部。H5 端因条件编译被排除, --status-bar-height 仍为 0,不受影响。
  try {
    const info = uni.getSystemInfoSync()
    const sbh = (info && info.statusBarHeight) || 0
    const doc = (globalThis as any).document
    if (doc && doc.documentElement) {
      doc.documentElement.style.setProperty('--status-bar-height', sbh + 'px')
    }
  } catch (_) { /* noop */ }
  // #endif
})

onShow(() => {
  console.log('[App] onShow')
})

onHide(() => {
  console.log('[App] onHide')
})
</script>

<style lang="scss">
@import '@/styles/common.scss';

page {
  background-color: var(--bg-page);
  color: var(--text-primary);
  font-family: -apple-system, BlinkMacSystemFont, 'Helvetica Neue', Helvetica, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif;
  font-size: 14px;
  line-height: 1.5;
}

.flex-row { display: flex; flex-direction: row; }
.flex-col { display: flex; flex-direction: column; }
.flex-center { display: flex; align-items: center; justify-content: center; }
.flex-between { display: flex; justify-content: space-between; align-items: center; }
.flex-1 { flex: 1; }

.text-ellipsis-1 {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.text-ellipsis-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>