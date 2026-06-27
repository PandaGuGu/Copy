import {
  getLoc,
  getSearchDefaultWords,
  getSuggest,
  getMenuIcon,
  getHotSearchItems
} from "../../api";
import { buildHeaderMenuLeftZones } from "@/constants/videoZones";
import square01 from "../../assets/square_01.jpg";
import square02 from "../../assets/square_02.jpg";
import live01 from "../../assets/live_01.png";
import live02 from "../../assets/live_02.png";

const state = {
  leftNav: [
    //顶部左侧导航栏
    {
      name: "主站",
      class: "home",
      icon: "bili-icon",
      href: "/"
    },
    {
      name: "画友",
      class: "hbili",
      href: "https://www.bilibili.com/v/huayou"
    },
    {
      name: "游戏中心",
      class: "game",
      href: "https://game.bilibili.com"
    },
    {
      name: "直播",
      class: "live",
      href: "/#/minibili/live"
    },
    {
      name: "会员购",
      class: "buy",
      href: "https://show.bilibili.com"
    },
    {
      name: "赛事",
      href: "https://www.bilibili.com/v/esports"
    },
    {
      name: "BML",
      href: "https://www.bilibili.com/blackboard/bml"
    },
    {
      name: "下载APP",
      class: "mobile",
      icon: "bili-icon",
      href: "https://app.bilibili.com"
    }
  ],
  headBanner: [], //顶部背景、LOGO
  searchValue: "", //搜索框输入值
  searchWord: [], //默认搜索关键字
  suggest: { tag: [] }, //建议搜索
  menuLeft: [
    {
      name: "首页",
      class: "home",
      href: "/"
    },
    ...buildHeaderMenuLeftZones()
  ], //主要菜单左侧（分区与 constants/videoZones.js 一致）
  menuRight: [
    {
      name: "专栏",
      class: "zl",
      icon: "zhuanlan",
      href: "https://www.bilibili.com/v/column",
      fieldClass: "",
      fields: []
    },
    {
      name: "广场",
      class: "nav-square",
      icon: "square",
      href: "https://www.bilibili.com/v/cube",
      fieldClass: "square-wrap",
      fields: [
        {
          name: "会员购",
          icon: "icon-vip-buy",
          href: "https://show.bilibili.com"
        },
        {
          name: "活动中心",
          icon: "icon-activity",
          href: "https://www.bilibili.com/blackboard/activity"
        },
        {
          name: "游戏中心",
          icon: "icon-game",
          href: "https://game.bilibili.com"
        },
        {
          name: "新闻中心",
          icon: "icon-news",
          href: "https://www.bilibili.com/v/news"
        },
        {
          name: "画友",
          icon: "icon-hy",
          href: "https://www.bilibili.com/v/huayou"
        },
        {
          name: "芒果TV",
          icon: "icon-mango",
          href: "https://www.bilibili.com"
        }
      ],
      fieldImgClass: "square-field",
      fieldImg: [
        {
          title: "bilibili 活动",
          href: "https://www.bilibili.com/blackboard/activity",
          src: square01
        },
        {
          title: "话题列表",
          href: "https://www.bilibili.com/v/topic",
          src: square02
        }
      ]
    },
    {
      name: "直播",
      class: "",
      icon: "live",
      href: "/#/minibili/live",
      fieldClass: "nav-live",
      fields: [
        {
          name: "全部直播",
          href: "/#/minibili/live"
        },
        {
          name: "开播设置",
          href: "/#/minibili/live/create"
        }
      ],
      fieldImgClass: "live-field",
      fieldImg: [
        {
          title: "有文画",
          href: "https://www.bilibili.com/v/huayou",
          imgclass: "pic",
          src: live01
        },
        {
          title: "小视频",
          href: "https://www.bilibili.com/v/short",
          imgclass: "pic",
          src: live02
        }
      ]
    },
    {
      name: "小黑屋",
      class: "",
      icon: "blackroom",
      href: "https://www.bilibili.com/blackboard/blackroom",
      fieldClass: "",
      fields: []
    }
  ], //主要菜单右侧
  menuIcon: [], //主要菜单右侧icon
  hotSearchItems: [] //热搜榜原始条目 [{rank, title, badge}]
};

const getters = {};

const mutations = {
  SET_HEAD_BANNER: (state, data) => {
    state.headBanner = Object.assign({}, data[0]);
  },
  SET_SEARCH_DEFAULT_WORDS: (state, data) => {
    state.searchWord = Object.assign({}, data);
  },
  SET_MENUICON: (state, data) => {
    state.menuIcon = Object.assign({}, data);
  },
  SET_SEARCH_WORD: (state, data) => {
    state.searchValue = data;
  },
  SET_SUGGEST: (state, data) => {
    state.suggest = data;
  },
  SET_HOT_SEARCH_ITEMS: (state, data) => {
    state.hotSearchItems = Array.isArray(data) ? data : [];
  }
};

const actions = {
  setHeadBanner({ commit }, data) {
    getLoc(data).then(res => {
      commit("SET_HEAD_BANNER", res.data);
    });
  },
  setSearchDefaultWords({ commit }) {
    getSearchDefaultWords().then(res => {
      commit("SET_SEARCH_DEFAULT_WORDS", res.data);
    });
  },
  setSuggest({ commit, state }) {
    const term = String(state.searchValue || "").trim();
    if (!term) {
      commit("SET_SUGGEST", { tag: [] });
      return;
    }
    getSuggest(term).then(res => {
      const payload = res && res.result;
      if (payload && Array.isArray(payload.tag)) {
        commit("SET_SUGGEST", payload);
      } else if (Array.isArray(payload)) {
        commit("SET_SUGGEST", { tag: payload });
      } else {
        commit("SET_SUGGEST", { tag: [] });
      }
    });
  },
  setMenuIcon({ commit }) {
    getMenuIcon().then(res => {
      commit("SET_MENUICON", res.data);
    });
  },
  setHotSearchItems({ commit }) {
    getHotSearchItems(10).then(res => {
      commit("SET_HOT_SEARCH_ITEMS", res.items || []);
    });
  }
};

export default {
  namespaced: true, //注册header空间模块
  state,
  getters,
  actions,
  mutations
};
