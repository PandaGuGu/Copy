import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const globalLoading = ref(false)
  const networkOnline = ref(true)

  /** 自定义 tabBar 当前选中项：0=首页 1=关注 2=会员购 3=我的（由各 tab 页 onShow 设置） */
  const currentTab = ref(0)

  function showLoading(title = '加载中...') {
    globalLoading.value = true
    uni.showLoading({ title, mask: true })
  }
  function hideLoading() {
    globalLoading.value = false
    uni.hideLoading()
  }

  return { globalLoading, networkOnline, currentTab, showLoading, hideLoading }
})