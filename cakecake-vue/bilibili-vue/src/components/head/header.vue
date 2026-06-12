<template>
  <div class="app-header">
    <nav-menu
      :leftNav="leftNav"
      :headBanner="headBanner"
      :menuShow="showHomeChrome"
    ></nav-menu>
    <template v-if="showHomeChrome">
      <div
        v-if="showGlobalHeadBanner"
        class="head-banner"
        :style="{ 'background-image': 'url(' + headBanner.pic + ')' }"
      >
        <div class="bili-wrapper head-content">
          <a
            class="head-logo"
            :style="{ background: 'url(' + headBanner.litpic + ')' }"
          ></a>
        </div>
        <a href="" target="_blank" class="banner-link"></a>
      </div>
    </template>
  </div>
</template>

<script>
import NavMenu from "../../components/navMenu/navMenu";
import { mapState, mapActions } from "vuex";
import {
  shouldShowMinibiliCompactHeader,
  shouldShowHomeHeaderChrome
} from "@/utils/minibiliRoutes";

export default {
  created() {
    this.setHeadBanner({
      pf: 0,
      id: 142
    });
    this.setMenuIcon();
  },
  components: {
    NavMenu
  },
  computed: {
    ...mapState("header", [
      "leftNav",
      "headBanner"
    ]),
    /** 个人中心 / 消息 / 个人空间等：仅顶栏 nav-menu，样式同消息中心 */
    isCompactHeaderRoute() {
      return shouldShowMinibiliCompactHeader(this.$route);
    },
    /** 首页头图与分区导航；搜索页与个人中心等为 false */
    showHomeChrome() {
      return shouldShowHomeHeaderChrome(this.$route);
    },
    showGlobalHeadBanner() {
      return !this.isCompactHeaderRoute;
    }
  },
  methods: {
    ...mapActions("header", [
      "setHeadBanner",
      "setMenuIcon"
    ])
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style></style>
