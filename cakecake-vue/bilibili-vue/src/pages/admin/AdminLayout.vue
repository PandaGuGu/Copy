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
        <div v-for="g in groups" :key="g.key" class="adm-side__sec">
          <div class="adm-side__sec-title">{{ g.title }}</div>
          <a
            v-for="it in g.items" :key="it.name"
            href="javascript:;"
            class="adm-side__item"
            :class="{ 'adm-side__item--on': $route.name === it.name }"
            @click="navigate(it.name)"
          >{{ it.label }}</a>
        </div>
      </aside>
      <main class="adm-main">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script>
import { adminMe } from "@/api/admin";
import { clearAdminTokens } from "@/utils/adminAuth";

const GROUPS = [
  { key:"data",  title:"数据", items:[
    { name:"adminDashboard", label:"数据概览" },
    { name:"adminBIReport", label:"数据报表" }
  ]},
  { key:"ops",   title:"运营", items:[
    { name:"adminBanners", label:"首页轮播" },
    { name:"adminHotSearch", label:"热搜运营" },
    { name:"adminSpecialManage", label:"专题活动" },
    { name:"adminDynamicManage", label:"动态管理" },
    { name:"adminSubtitleManage", label:"字幕管理" }
  ]},
  { key:"audit", title:"审核", items:[
    { name:"adminVideoReview", label:"视频审核" },
    { name:"adminArticleReview", label:"专栏审核" },
    { name:"adminComments", label:"评论管理" },
    { name:"adminReports", label:"举报处理" },
    { name:"adminCopyrightManage", label:"版权管理" },
    { name:"adminRiskManage", label:"风控管理" }
  ]},
  { key:"user",  title:"用户", items:[
    { name:"adminUsers", label:"用户管理" },
    { name:"adminCSManage", label:"客服后台" },
    { name:"adminTicketManage", label:"工单管理" }
  ]},
  { key:"ai",    title:"AI", items:[
    { name:"adminAgent", label:"AI 角色" }
  ]},
  { key:"sys",   title:"系统", items:[
    { name:"adminRBACManage", label:"权限审计" },
    { name:"adminSettings", label:"系统设置" },
    { name:"adminConfigManage", label:"配置发布" },
    { name:"adminOpsMonitor", label:"运维监控" }
  ]}
];

export default {
  name: 'AdminLayout',
  data() {
    return { me: null, groups: GROUPS };
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

.adm-layout { min-height:100vh; background:#f4f5f7; }
.adm-header {
  height:50px; background:$white; border-bottom:1px solid #e3e5e7;
  display:flex; align-items:center; justify-content:space-between; padding:0 24px;
}
.adm-header__brand { display:flex; align-items:center; gap:10px;
  img { height:24px; }
  strong { @include sc(15px,$blue); }
}
.adm-header__right { display:flex; align-items:center; gap:16px; @include sc(13px,#61666d); }
.adm-header__link { color:$blue; &:hover { color:#00b5e5; } }

.adm-body {
  display:flex; max-width:1200px; margin:0 auto; padding:20px 16px 40px; gap:16px;
}

/* 侧栏 */
.adm-side {
  width:190px; flex-shrink:0; background:$white; border-radius:8px;
  border:1px solid #e3e5e7; padding:8px 0; height:fit-content;
}
.adm-side__sec { }
.adm-side__sec-title {
  padding:14px 16px 6px;
  @include sc(12px,#9499a0); font-weight:600; letter-spacing:.5px;
}
.adm-side__item {
  display:block; padding:9px 16px 9px 28px; @include sc(13px,#61666d);
  transition:color .2s,background .2s;
  &:hover { color:$blue; background:#f6f7f8; }
}
.adm-side__item--on {
  color:$blue; font-weight:600; background:#e3f3ff; border-right:3px solid $blue;
}

.adm-main { flex:1; min-width:0; }
</style>
