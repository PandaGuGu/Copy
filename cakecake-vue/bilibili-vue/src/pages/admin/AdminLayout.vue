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
        <div
          v-for="group in groups"
          :key="group.key"
          class="adm-side__group"
          :class="{ 'adm-side__group--open': isGroupOpen(group) }"
        >
          <div class="adm-side__group-hd" @click="toggleGroup(group.key)">
            <span class="adm-side__group-icon">{{ group.icon }}</span>
            <span class="adm-side__group-label">{{ group.title }}</span>
            <span class="adm-side__group-arrow">▾</span>
          </div>
          <transition name="adm-group-slide">
            <div v-if="isGroupOpen(group)" class="adm-side__group-bd">
              <a
                v-for="item in group.items"
                :key="item.name"
                href="javascript:;"
                class="adm-side__item"
                :class="{ 'adm-side__item--on': $route.name === item.name }"
                @click="navigate(item.name)"
              >{{ item.label }}</a>
            </div>
          </transition>
        </div>
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

const GROUPS = [
  {
    key: "data", icon: "📊", title: "数据",
    items: [
      { name: "adminDashboard", label: "数据概览" },
      { name: "adminBIReport", label: "数据报表" }
    ]
  },
  {
    key: "ops", icon: "📢", title: "运营",
    items: [
      { name: "adminBanners", label: "首页轮播" },
      { name: "adminHotSearch", label: "热搜运营" },
      { name: "adminSpecialManage", label: "专题活动" },
      { name: "adminDynamicManage", label: "动态管理" },
      { name: "adminSubtitleManage", label: "字幕管理" }
    ]
  },
  {
    key: "audit", icon: "🛡️", title: "审核",
    items: [
      { name: "adminVideoReview", label: "视频审核" },
      { name: "adminArticleReview", label: "专栏审核" },
      { name: "adminComments", label: "评论管理" },
      { name: "adminReports", label: "举报处理" },
      { name: "adminCopyrightManage", label: "版权管理" },
      { name: "adminRiskManage", label: "风控管理" }
    ]
  },
  {
    key: "user", icon: "👤", title: "用户",
    items: [
      { name: "adminUsers", label: "用户管理" },
      { name: "adminCSManage", label: "客服后台" },
      { name: "adminTicketManage", label: "工单管理" }
    ]
  },
  {
    key: "ai", icon: "🤖", title: "AI",
    items: [
      { name: "adminAgent", label: "AI 角色" }
    ]
  },
  {
    key: "sys", icon: "⚙️", title: "系统",
    items: [
      { name: "adminRBACManage", label: "权限审计" },
      { name: "adminSettings", label: "系统设置" },
      { name: "adminConfigManage", label: "配置发布" },
      { name: "adminOpsMonitor", label: "运维监控" }
    ]
  }
];

export default {
  name: 'AdminLayout',
  data() {
    return {
      me: null,
      groups: GROUPS,
      expanded: {}
    };
  },
  created() {
    this.loadMe();
    // 初始化：当前路由所在分組默认展开
    this.syncExpanded();
  },
  watch: {
    '$route.name'() {
      this.syncExpanded();
    }
  },
  methods: {
    navigate(name) {
      if (this.$route.name === name) return;
      this.$router.push({ name });
    },
    getGroupByRoute() {
      return this.groups.find(g => g.items.some(it => it.name === this.$route.name));
    },
    syncExpanded() {
      const g = this.getGroupByRoute();
      if (g) {
        this.$set(this.expanded, g.key, true);
      }
    },
    isGroupOpen(group) {
      return !!this.expanded[group.key];
    },
    toggleGroup(key) {
      this.$set(this.expanded, key, !this.expanded[key]);
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
  img { height: 24px; }
  strong { @include sc(15px, $blue); }
}
.adm-header__right {
  display: flex; align-items: center; gap: 16px;
  @include sc(13px, #61666d);
}
.adm-header__link { color: $blue; &:hover { color: #00b5e5; } }

.adm-body {
  display: flex;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px 16px 40px;
  gap: 16px;
}

/* ── 侧栏 ── */
.adm-side {
  width: 190px; flex-shrink: 0;
  background: $white;
  border-radius: 8px;
  border: 1px solid #e3e5e7;
  height: fit-content;
  overflow: hidden;
  padding-bottom: 6px;
}

/* 分组 */
.adm-side__group { }

.adm-side__group-hd {
  display: flex; align-items: center;
  padding: 14px 16px 10px;
  cursor: pointer; user-select: none;
  transition: background 0.15s;
  &:hover { background: #f6f7f8; }
}
.adm-side__group-icon {
  font-size: 14px; width: 22px; text-align: center;
}
.adm-side__group-label {
  flex: 1;
  @include sc(13px, #9499a0);
  font-weight: 600; letter-spacing: 0.5px;
}
.adm-side__group-arrow {
  @include sc(10px, #c0c4cc);
  transition: transform 0.25s ease;
}
.adm-side__group--open .adm-side__group-arrow {
  transform: rotate(-180deg);
  color: $blue;
}
.adm-side__group--open .adm-side__group-label {
  color: #61666d;
}

/* 子项 */
.adm-side__group-bd {
  overflow: hidden;
}
.adm-side__item {
  display: block;
  padding: 10px 16px 10px 40px;
  @include sc(13px, #61666d);
  transition: color 0.2s, background 0.2s;
  &:hover {
    color: $blue;
    background: #f6f7f8;
  }
}
.adm-side__item--on {
  color: $blue; font-weight: 600;
  background: #e3f3ff;
  border-right: 3px solid $blue;
}

/* 分组展开/折叠动画 */
.adm-group-slide-enter-active,
.adm-group-slide-leave-active {
  transition: all 0.25s ease;
  max-height: 500px;
}
.adm-group-slide-enter-from,
.adm-group-slide-leave-to {
  max-height: 0;
  opacity: 0;
}

.adm-main {
  flex: 1; min-width: 0;
}

/* ── 页面切换过渡 ── */
.adm-page-enter-active,
.adm-page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.adm-page-enter-from {
  opacity: 0; transform: translateX(12px);
}
.adm-page-leave-to {
  opacity: 0; transform: translateX(-12px);
}
</style>
