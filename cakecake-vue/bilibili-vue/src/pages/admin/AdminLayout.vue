<template>
  <div class="adm-layout">
    <header class="adm-header">
      <div class="adm-header__brand">
        <img src="@/assets/cakelogo.png" alt="" />
        <span>|</span>
        <strong>cakecake 运营中心</strong>
      </div>
      <div class="adm-header__right">
        <span v-if="me" class="adm-header__user">{{ me.display_name || me.username }}</span>
        <a href="javascript:;" class="adm-header__link" @click.prevent="logout">退出</a>
        <router-link to="/" class="adm-header__link">返回主站</router-link>
      </div>
    </header>
    <div class="adm-body">
      <aside class="adm-side">
        <a href="javascript:;" @click="navigate('adminDashboard')"       class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminDashboard' }" >数据概览</a>
        <a href="javascript:;" @click="navigate('adminBanners')"         class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminBanners' }" >首页轮播</a>
        <a href="javascript:;" @click="navigate('adminHotSearch')"       class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminHotSearch' }" >热搜运营</a>
        <a href="javascript:;" @click="navigate('adminUsers')"           class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminUsers' }" >用户管理</a>
        <a href="javascript:;" @click="navigate('adminVideoReview')"     class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminVideoReview' }" >视频审核</a>
        <a href="javascript:;" @click="navigate('adminArticleReview')"   class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminArticleReview' }" >专栏审核</a>
        <a href="javascript:;" @click="navigate('adminDynamicManage')"   class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminDynamicManage' }" >动态管理</a>
        <a href="javascript:;" @click="navigate('adminComments')"        class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminComments' }" >评论管理</a>
        <a href="javascript:;" @click="navigate('adminSettings')"        class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminSettings' }" >系统设置</a>
        <a href="javascript:;" @click="navigate('adminReports')"         class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminReports' }" >举报处理</a>
        <a href="javascript:;" @click="navigate('adminAgent')"           class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminAgent' }" >AI 角色</a>
        <a href="javascript:;" @click="navigate('adminTicketManage')"    class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminTicketManage' }" >工单管理</a>
        <a href="javascript:;" @click="navigate('adminRiskManage')"      class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminRiskManage' }" >风控管理</a>
        <a href="javascript:;" @click="navigate('adminCopyrightManage')" class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminCopyrightManage' }" >版权管理</a>
        <a href="javascript:;" @click="navigate('adminBIReport')"        class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminBIReport' }" >数据报表</a>
        <a href="javascript:;" @click="navigate('adminCSManage')"        class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminCSManage' }" >客服后台</a>
        <a href="javascript:;" @click="navigate('adminOpsMonitor')"      class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminOpsMonitor' }" >运维监控</a>
        <a href="javascript:;" @click="navigate('adminConfigManage')"    class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminConfigManage' }" >配置发布</a>
        <a href="javascript:;" @click="navigate('adminRBACManage')"      class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminRBACManage' }" >权限审计</a>
        <a href="javascript:;" @click="navigate('adminSubtitleManage')"  class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminSubtitleManage' }" >字幕管理</a>
        <a href="javascript:;" @click="navigate('adminSpecialManage')"   class="adm-side__item" :class="{ 'adm-side__item--on': $route.name === 'adminSpecialManage' }" >专题活动</a>
      </aside>
      <main class="adm-main">
        <router-view v-slot="{ Component, route }">
          <transition name="adm-page" mode="out-in">
            <component :is="Component" :key="route.path" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script>
import { adminMe } from "@/api/admin";
import { clearAdminTokens } from "@/utils/adminAuth";

export default {
  name: 'AdminLayout',
  data() {
    return {
      me: null
    };
  },
  created() {
    this.loadMe();
  },
  methods: {
    navigate(name) {
      if (this.$route.name === name) return;
      this.$router.push({ name });
    },
    async loadMe() {
      try {
        const body = await adminMe();
        this.me = body.data;
      } catch {
        this.$router.replace({ name: "adminLogin" });
      }
    },
    logout() {
      clearAdminTokens();
      this.$router.replace({ name: "adminLogin" });
    }
  }
};
</script>

<style lang="scss" scoped>
@import "@/style/mixin";

.adm-layout {
  min-height: 100vh;
  background: #f4f5f7;
}
.adm-header {
  height: 50px;
  background: $white;
  border-bottom: 1px solid #e3e5e7;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
}
.adm-header__brand {
  display: flex;
  align-items: center;
  gap: 10px;
  img {
    height: 24px;
  }
  strong {
    @include sc(15px, $blue);
  }
}
.adm-header__right {
  display: flex;
  align-items: center;
  gap: 16px;
  @include sc(13px, #61666d);
}
.adm-header__link {
  color: $blue;
  &:hover {
    color: #00b5e5;
  }
}
.adm-body {
  display: flex;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px 16px 40px;
  gap: 16px;
}
.adm-side {
  width: 160px;
  flex-shrink: 0;
  background: $white;
  border-radius: 8px;
  padding: 8px 0;
  border: 1px solid #e3e5e7;
  height: fit-content;
}
.adm-side__item {
  display: block;
  padding: 12px 20px;
  @include sc(14px, #61666d);
  &:hover {
    color: $blue;
    background: #f6f7f8;
  }
}
.adm-side__item--on {
  color: $blue;
  font-weight: 600;
  background: #e3f3ff;
  border-right: 3px solid $blue;
}
.adm-main {
  flex: 1;
  min-width: 0;
}

/* ── 页面切换过渡动画 ── */
.adm-page-enter-active,
.adm-page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.adm-page-enter-from {
  opacity: 0;
  transform: translateX(12px);
}
.adm-page-leave-to {
  opacity: 0;
  transform: translateX(-12px);
}

/* 侧栏 active 态过渡 */
.adm-side__item {
  transition: color 0.2s, background 0.2s, border-color 0.2s;
}
</style>
