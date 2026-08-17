<template>
  <view class="login-page">
    <view class="logo-wrap">
      <view class="logo">🎂</view>
      <text class="app-name">CakeCake</text>
    </view>

    <view class="form">
      <view class="input-row">
        <text class="label">账号</text>
        <input v-model="username" placeholder="请输入用户名" />
      </view>
      <view class="input-row">
        <text class="label">密码</text>
        <input v-model="password" type="password" placeholder="请输入密码" />
      </view>
    </view>

    <view class="submit-btn" @tap="onSubmit">
      <text>登录</text>
    </view>

    <view class="switch-mode" @tap="goRegister">
      <text>没有账号？请联系管理员开通</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useUserStore } from '@/store/user'

const username = ref('')
const password = ref('')
const userStore = useUserStore()

function goRegister() {
  uni.showToast({ title: '后端暂未开放注册，请联系管理员', icon: 'none' })
}

async function onSubmit() {
  if (!username.value || !password.value) {
    uni.showToast({ title: '请填写完整', icon: 'none' })
    return
  }
  uni.showLoading({ title: '处理中…', mask: true })
  try {
    // 后端无注册接口（仅 login/refresh），统一走登录
    await userStore.login(username.value, password.value)
    uni.hideLoading()
    uni.showToast({ title: '登录成功', icon: 'success' })
    setTimeout(() => uni.reLaunch({ url: '/pages/mine/index' }), 800)
  } catch { /* 已 toast */ }
    uni.hideLoading()
}
</script>

<style lang="scss" scoped>
.login-page {
  min-height: 100vh;
  background: #FFF;
  padding: 80rpx 60rpx;
}

.logo-wrap {
  text-align: center;
  margin-bottom: 80rpx;
  .logo {
    font-size: 120rpx;
    margin-bottom: 16rpx;
  }
  .app-name {
    font-size: 40rpx;
    font-weight: 600;
    color: #FB7299;
  }
}

.form {
  display: flex;
  flex-direction: column;
  gap: 24rpx;
  .input-row {
    display: flex;
    align-items: center;
    border-bottom: 1rpx solid #EEE;
    padding: 24rpx 0;
    .label { width: 100rpx; font-size: 28rpx; color: #666; }
    input   { flex: 1; font-size: 30rpx; color: #181818; }
  }
}

.submit-btn {
  margin-top: 60rpx;
  background: linear-gradient(90deg, #FB7299, #FF9DB5);
  color: #FFF;
  text-align: center;
  padding: 28rpx 0;
  border-radius: 48rpx;
  font-size: 32rpx;
  font-weight: 500;
}

.switch-mode {
  text-align: center;
  margin-top: 40rpx;
  font-size: 26rpx;
  color: #FB7299;
}
</style>