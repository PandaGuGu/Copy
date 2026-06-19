<template>
  <div
    class="report-wrap-module elevator-module"
    :style="{ top: elTop + 'px' }"
  >
    <div class="nav-bg">
      <div class="tips-img"></div>
    </div>
    <div class="nav-list">
      <div class="item online-stats" v-if="online">
        <span class="stat-label">在线</span>
        <span class="stat-num">{{ online.web_online }}</span>
      </div>
      <div class="item online-stats" v-if="online">
        <span class="stat-label">投稿</span>
        <span class="stat-num">{{ online.all_count }}</span>
      </div>
      <div class="item customize"><i class="icon"></i>排序</div>
    </div>
    <div class="s-line"></div>
    <div class="back-top icon" @click="goTop"></div>
    <div class="app-download">
      <a href="//app.bilibili.com/?channel=home_recommend" target="_blank">
        <div
          id="elevator-mobile-app"
          class="app-icon"
          style="background-position-x: 0px;"
        ></div>
        <div class="app-tips-icon" style="opacity: 1; display: none;"></div>
      </a>
    </div>
  </div>
</template>

<script>
import { mapGetters, mapMutations } from "vuex";
export default {
  created() {
    let vm = this;
    window.onscroll = function() {
      var scrollTop =
        document.documentElement.scrollTop || document.body.scrollTop;
      vm.scrollTop = scrollTop;
      vm.setScrollTop(scrollTop);
    };
  },
  components: {},
  props: {},
  computed: {
    ...mapGetters(["module", "online"]),
    activeTab() {
      //遍历返回符合条件的值
      let one = this.module.map((v, index) => {
        return this.scrollTop + 100 > v.offsetTop ? index : null;
      });
      //filter去掉null
      let two = one.filter(item => item);
      return two.length > 0 ? two.length : 0;
    },
    elTop() {
      return this.scrollTop > 60 ? 88 : 232;
    }
  },
  data() {
    return {
      scrollTop: 0
    };
  },
  methods: {
    ...mapMutations({
      setScrollTop: "SET_SCROLL_TOP"
    }),
    goTop() {
      window.scrollTo({ top: 0, left: 0, behavior: "smooth" });
    },
    goPosition(index) {
      document.documentElement.scrollTop = this.module[index].offsetTop - 30;
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss">
@import "../../../style/mixin";

.elevator-module {
  position: fixed;
  z-index: 299;
  left: 50%;
  top: 232px;
  margin-left: 590px;
  transition: top 0.3s;
  .nav-bg {
    opacity: 0;
    top: -15px;
    right: 0;
    height: 100%;
    padding-bottom: 20px;
    width: 60px;
    position: absolute;
    background: hsla(0, 0%, 100%, 0.8);
    border-radius: 4px;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.68, -0.55, 0.27, 1.55);
    .tips-img {
      position: absolute;
      width: 117px;
      height: 333px;
      background: url(//s1.hdslb.com/bfs/static/jinkela/home/asserts/tab2233.png);
      left: 12px;
      top: 14px;
    }
  }
  .nav-list {
    position: relative;
    background-color: #f6f9fa;
    border: 1px solid #e5e9ef;
    overflow: hidden;
    border-radius: 4px;
    .item {
      width: 48px;
      height: 32px;
      line-height: 32px;
      text-align: center;
      transition: background-color 0.3s, color 0.3s;
      cursor: pointer;
      -ms-user-select: none;
      user-select: none;
      &.on,
      &:hover {
        background-color: #00a1d6;
        color: #fff;
      }
    }
    .online-stats {
      cursor: default;
      height: 28px;
      line-height: 28px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      border-bottom: 1px solid #e5e9ef;
      width: 48px;
      &.on,
      &:hover {
        background-color: transparent;
        color: inherit;
      }
      .stat-label {
        font-size: 10px;
        color: #99a2aa;
        line-height: 1;
      }
      .stat-num {
        font-size: 12px;
        color: #222;
        line-height: 1;
        margin-top: 2px;
      }
    }
    .customize {
      height: 38px;
      line-height: 20px;
      padding: 8px 0;
      border-top: 1px solid #e5e9ef;
      .icon {
        display: block;
        margin: 0 auto 4px;
        background-position: -663px -151px;
        height: 18px;
        width: 18px;
      }
    }
  }
  .s-line {
    position: relative;
    border-left: 1px solid #ddd;
    border-right: 1px solid #ddd;
    height: 9px;
    width: 30px;
    margin: 0 auto;
  }
  .back-top {
    position: relative;
    display: block;
    cursor: pointer;
    height: 48px;
    background-position: -648px -72px;
    background-color: #f6f9fa;
    border: 1px solid #e5e9ef;
    overflow: hidden;
    border-radius: 4px;
  }
  .app-download {
    position: relative;
    width: 50px;
    height: 70px;
    .app-icon {
      position: absolute;
      left: -15px;
      width: 80px;
      height: 80px;
      background-image: url(//s1.hdslb.com/bfs/static/jinkela/home/asserts/app-download.png);
    }
    .app-tips-icon {
      display: none;
      position: absolute;
      left: -110px;
      top: -20px;
      width: 106px;
      height: 44px;
      background-image: url(//s1.hdslb.com/bfs/static/jinkela/home/asserts/app-download-tips.png);
    }
  }
}
</style>
