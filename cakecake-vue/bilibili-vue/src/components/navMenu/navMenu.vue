<template>
  <div class="nav-menu" :class="{ 'nav-menu--solid-top': useSolidTopNav }">
    <div
      v-if="menuShow && isHomeRoute"
      class="blur-bg"
      :style="{ background: 'url(' + headBanner.pic + ')' }"
    ></div>
    <div class="nav-mask"></div>
    <div class="nav-inner">
      <div class="nav-con nav-con--left">
        <ul>
          <li
            class="nav-item"
            v-for="(item, index) in leftNav"
            :key="`leftNav_${index}`"
            :class="item.class"
          >
            <a class="t" :href="item.href" target="_blank">
              <i :class="item.icon" v-if="item.icon"></i>
              {{ item.name }}
            </a>
          </li>
        </ul>
      </div>
      <!-- 搜索框（移入顶部导航栏） -->
      <div class="nav-search">
        <div class="searchform">
          <input
            v-model="searchValue"
            type="text"
            :placeholder="searchPlaceholder"
            @keyup.enter="searchALL()"
            @input="onSearchInput"
            @focus="onSearchFocus"
            @blur="onSearchBlur"
            class="search-keyword"
          />
          <button
            type="submit"
            class="search-submit"
            @click="searchALL()"
          ></button>
        </div>
        <!-- 搜索建议（有输入时） -->
        <ul v-if="suggestShow" class="bilibili-suggest">
          <li class="kw">
            <div class="b-line">
              <p><span>关键词</span></p>
            </div>
          </li>
          <li
            class="suggest-item"
            v-for="(item, index) in suggestTagList"
            :key="`suggest_item_${index}`"
          >
            <a
              href="javascript:;"
              @mousedown.prevent="searchByHistory(item.value)"
              v-html="item.name"
            ></a>
          </li>
        </ul>
        <!-- 热搜榜（无输入且聚焦时） -->
        <div
          v-else-if="hotSearchPanelShow"
          class="bilibili-suggest hot-search-panel"
        >
          <div class="hot-search-head">
            <span class="hot-search-title">热搜榜</span>
          </div>
          <div class="hot-search-grid">
            <a
              href="javascript:;"
              class="hot-search-item"
              v-for="item in hotSearchItems"
              :key="`hot_${item.rank}`"
              @mousedown.prevent="searchByHistory(item.title)"
            >
              <span
                class="hot-search-rank"
                :class="{ 'top3': item.rank <= 3 }"
              >{{ item.rank }}</span>
              <span class="hot-search-text">{{ item.title }}</span>
              <span v-if="item.badge" class="hot-search-badge">{{ item.badge }}</span>
            </a>
          </div>
        </div>
        <!-- 历史搜索（无输入、无热搜时） -->
        <div
          v-else-if="historyPanelShow"
          class="bilibili-suggest search-history-panel"
        >
          <div class="search-history-head">
            <div class="b-line">
              <p><span>历史搜索</span></p>
            </div>
          </div>
          <ul class="search-history-list">
            <li
              v-for="(kw, index) in searchHistory"
              :key="`search_hist_${index}_${kw}`"
              class="search-history-item"
            >
              <a
                href="javascript:;"
                class="search-history-link"
                @mousedown.prevent="searchByHistory(kw)"
                >{{ kw }}</a
              >
              <button
                type="button"
                class="search-history-del"
                aria-label="删除"
                @mousedown.prevent="removeHistoryItem(index)"
              >
                ×
              </button>
            </li>
          </ul>
        </div>
      </div>
      <div class="nav-con nav-con--right">
        <ul>
          <!-- 头像 -->
          <li
            class="nav-item profile-info"
            :class="{ on: signIn == 1 }"
            @mouseenter="profileFadeIn"
            @mouseleave="profileFadeOut"
          >
            <router-link
              v-if="signIn == 1 && isMinibiliMode && minibiliSpaceTo"
              class="t"
              :to="minibiliSpaceTo"
            >
              <div class="i-face">
                <img v-if="navFaceSrc" :src="navFaceSrc" class="face" />
                <img class="pendant" />
              </div>
            </router-link>
            <a
              class="t"
              v-else-if="signIn == 1"
              :href="profileSpaceHref"
              target="_blank"
            >
              <div class="i-face">
                <img v-if="navFaceSrc" :src="navFaceSrc" class="face" />
                <img class="pendant" />
              </div>
            </a>
            <a
              class="t"
              v-else
              @click="
                setLoginShow();
                setLoginTab(0);
              "
            >
              <div class="i-face">
                <img src="../../assets/akari.jpg" class="face" />
              </div>
            </a>
            <transition name="nav-trans">
              <div
                class="profile-m dd-bubble"
                v-if="signIn == 1"
                v-show="profileShow"
              >
                <div class="header-u-info" v-if="navProfileReady">
                  <div class="header-uname">
                    <b class="">{{ navDisplayName }}</b>
                  </div>
                  <div class="btns-profile clearfix">
                    <div class="coin fl">
                      <a
                        href="https://account.bilibili.com/site/coin"
                        target="_blank"
                        title="硬币"
                      >
                        <i class="bili-icon bi"></i>
                        <i class="bili-icon jia"></i>
                        <span class="num">{{ navCoinDisplay }}</span>
                        <span class="num-move">{{ navCoinRaw }}</span>
                        <span title="" class="num-tip">登录奖励</span>
                      </a>
                    </div>
                    <div class="currency fl">
                      <a
                        href="https://pay.bilibili.com/bb_balance.html"
                        target="_blank"
                        title="B币"
                      >
                        <i class="bili-icon"></i>
                        <span class="num">{{ navBcoinDisplay }}</span>
                      </a>
                    </div>
                    <div class="ver phone fr verified">
                      <a
                        href="https://passport.bilibili.com/site/site.html"
                        target="_blank"
                      >
                        <i class="bili-icon"></i>
                        <span class="tips">已绑定</span>
                      </a>
                    </div>
                    <div class="ver email fr verified">
                      <a
                        href="https://passport.bilibili.com/site/site.html"
                        target="_blank"
                      >
                        <i class="bili-icon"></i>
                        <span class="tips">已绑定</span>
                      </a>
                    </div>
                    <div class="link-to-bind-mobile"></div>
                  </div>
                  <div class="grade clearfix">
                    <span class="hd fl">等级</span>
                    <a
                      href="https://account.bilibili.com/site/record.html"
                      target="_blank"
                    >
                      <div class="bar fr">
                        <div class="lt" :class="level" aria-hidden="true">
                          <span class="lt-num">{{ navLevelDisplay }}</span>
                        </div>
                        <div
                          class="rate"
                          :style="{ width: navLevelFillPct + '%' }"
                        ></div>
                        <div class="num">
                          <div v-if="navLevelInfo">
                            {{ navLevelInfo.current_exp }}
                            <span>{{ "/" + navLevelInfo.next_exp }}</span>
                          </div>
                        </div>
                      </div>
                    </a>
                    <div class="desc-tips">
                      <span class="arrow-left"></span>
                      <div class="lv-row">
                        作为<strong>LV{{ navLevelDisplay }}</strong>，你可以：
                      </div>
                      <div>
                        <div
                          v-for="(line, idx) in navLevelPrivilegeLines"
                          :key="idx"
                        >
                          {{ idx + 1 }}、{{ line }}
                        </div>
                      </div>
                      <a
                        :href="userLevelHelpUrl"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="help-link"
                        >会员等级相关说明 &gt;</a
                      >
                    </div>
                  </div>
                </div>
                <div class="member-menu">
                  <ul class="clearfix">
                    <li>
                      <router-link
                        v-if="isMinibiliMode"
                        to="/minibili/account"
                        class="account"
                      >
                        <i class="bili-icon b-icon-p-account"></i>
                        个人中心
                      </router-link>
                      <a
                        v-else
                        href="https://account.bilibili.com/account/home"
                        target="_blank"
                        class="account"
                      >
                        <i class="bili-icon b-icon-p-account"></i>
                        个人中心
                      </a>
                    </li>
                    <li>
                      <router-link
                        v-if="isMinibiliMode"
                        :to="{ name: 'upload' }"
                        class="member"
                      >
                        <i class="bili-icon b-icon-p-member"></i>
                        投稿管理
                      </router-link>
                      <a
                        v-else
                        href="https://member.bilibili.com/v2#/home"
                        target="_blank"
                        class="member"
                      >
                        <i class="bili-icon b-icon-p-member"></i>
                        投稿管理
                      </a>
                    </li>
                    <li>
                      <a
                        href="https://pay.bilibili.com/paywallet-fe/bb_balance.html"
                        target="_blank"
                        class="wallet"
                      >
                        <i class="bili-icon b-icon-p-wallet"></i>
                        B币钱包
                      </a>
                    </li>
                    <li>
                      <router-link
                        to="/minibili/live/create"
                        class="live"
                      >
                        <i class="bili-icon b-icon-p-live"></i>
                        直播中心
                      </router-link>
                    </li>
                    <li>
                      <a
                        href="https://show.bilibili.com/orderlist"
                        target="_blank"
                        class="bml"
                      >
                        <i class="bili-icon b-icon-p-ticket"></i>
                        订单中心
                      </a>
                    </li>
                    <li></li>
                  </ul>
                </div>
                <div class="member-bottom">
                  <a href="#" class="logout" @click="signOut()">退出</a>
                </div>
              </div>
            </transition>
            <div class="i_menu i_menu_login" v-if="signIn == 0">
              <p class="tip">
                登录后你可以：
              </p>
              <div class="img">
                <img src="../../assets/danmu.png" />
                <img src="../../assets/danmu.png" />
              </div>
              <a
                class="login-btn"
                @click="
                  setLoginShow();
                  setLoginTab(0);
                "
                >登录</a
              >
              <p class="reg">
                首次使用？<a
                  @click="
                    setLoginShow();
                    setLoginTab(1);
                  "
                  >点我去注册</a
                >
              </p>
            </div>
          </li>
          <!-- 大会员、消息、动态、收藏、历史、创作中心（头像右侧，带图标） -->
          <li class="nav-item nav-item--icon">
            <a
              href="https://account.bilibili.com/big"
              target="_blank"
              rel="noopener noreferrer"
              class="t"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/><text x="12" y="17" text-anchor="middle" font-size="11" font-weight="bold" fill="currentColor">大</text></svg>
              </span>
              <span class="nav-label">大会员</span>
            </a>
          </li>
          <li
            class="nav-item nav-item--icon"
            @mouseenter="messageFadeIn"
            @mouseleave="messageFadeOut"
          >
            <router-link
              v-if="isMinibiliMode"
              to="/minibili/messages?cat=my_message"
              class="t"
              title="消息"
              @click="messageShow = false"
            >
              <div v-if="messageUnreadTotalLabel" class="num">
                {{ messageUnreadTotalLabel }}
              </div>
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" stroke-width="1.5"/><path d="M3 7L12 14L21 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
              </span>
              <span class="nav-label">消息</span>
            </router-link>
            <a
              v-else
              href="https://message.bilibili.com"
              target="_blank"
              title="消息"
              class="t"
            >
              <div class="num">
                1
              </div>
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" stroke-width="1.5"/><path d="M3 7L12 14L21 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
              </span>
              <span class="nav-label">消息</span>
            </a>
            <transition name="nav-trans">
              <div class="im-list-box" v-show="messageShow">
                <template v-for="item in messageNavItems" :key="item.cat">
                  <router-link
                    v-if="isMinibiliMode"
                    class="im-list"
                    :to="`/minibili/messages?cat=${item.cat}`"
                    @click="messageShow = false"
                  >
                    {{ item.label }}
                    <div
                      v-if="formatMessageUnreadBadge(msgUnread[item.cat])"
                      class="im-notify im-number im-center"
                    >
                      {{ formatMessageUnreadBadge(msgUnread[item.cat]) }}
                    </div>
                  </router-link>
                  <a
                    v-else
                    class="im-list"
                    target="_blank"
                    href="https://message.bilibili.com"
                  >
                    {{ item.label }}
                  </a>
                </template>
              </div>
            </transition>
          </li>
          <li
            class="nav-item nav-item--icon"
            @mouseenter="dynamicFadeIn"
            @mouseleave="dynamicFadeOut"
          >
            <router-link
              v-if="isMinibiliMode && minibiliDynamicsTo"
              class="t"
              :to="minibiliDynamicsTo"
              title="动态"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="3" fill="currentColor"/><ellipse cx="12" cy="4" rx="2.5" ry="4" fill="currentColor" opacity="0.7"/><ellipse cx="12" cy="20" rx="2.5" ry="4" fill="currentColor" opacity="0.7"/><ellipse cx="4" cy="12" rx="4" ry="2.5" fill="currentColor" opacity="0.7"/><ellipse cx="20" cy="12" rx="4" ry="2.5" fill="currentColor" opacity="0.7"/></svg>
              </span>
              <span class="nav-label">动态</span>
            </router-link>
            <a
              v-else
              href="#"
              class="t"
              @click.prevent="dynamicShow = !dynamicShow"
              title="动态"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="3" fill="currentColor"/><ellipse cx="12" cy="4" rx="2.5" ry="4" fill="currentColor" opacity="0.7"/><ellipse cx="12" cy="20" rx="2.5" ry="4" fill="currentColor" opacity="0.7"/><ellipse cx="4" cy="12" rx="4" ry="2.5" fill="currentColor" opacity="0.7"/><ellipse cx="20" cy="12" rx="4" ry="2.5" fill="currentColor" opacity="0.7"/></svg>
              </span>
              <span class="nav-label">动态</span>
            </a>
            <transition name="nav-trans">
              <div class="dynamic-list-box" v-show="dynamicShow">
                <router-link v-if="minibiliDynamicsTo" class="dyn-panel-more" :to="minibiliDynamicsTo" @click.native="dynamicShow = false">查看更多 &gt;</router-link>
                <a v-else href="#" class="dyn-panel-more" @click.prevent>查看更多 &gt;</a>
                <!-- 正在直播（有数据时显示） -->
                <div class="dyn-live-section" v-if="dynamicLiveList.length > 0">
                  <div class="dyn-live-header">
                    <span class="dyn-live-title">正在直播</span>
                  </div>
                  <div class="dyn-live-users">
                    <div class="dyn-live-user" v-for="(lu, idx) in dynamicLiveList.slice(0, 6)" :key="'live-'+idx">
                      <div class="dyn-live-avatar">
                        <img :src="lu.face || lu.avatar" :alt="lu.uname || lu.name" />
                      </div>
                      <span class="dyn-live-name">{{ lu.uname || lu.name }}</span>
                    </div>
                  </div>
                </div>
                <!-- 历史动态 -->
                <div class="dyn-feed-section">
                  <div class="dyn-feed-divider" v-if="dynamicLiveList.length > 0"></div>
                  <div class="dyn-feed-title">历史动态</div>
                  <div class="dyn-feed-list">
                    <div class="dyn-feed-item" v-for="(feed, idx) in dynamicFeedList" :key="'feed-'+idx">
                      <div class="dyn-feed-left">
                        <img class="dyn-feed-avatar" :src="feed.user?.face || feed.face || ''" :alt="feed.user?.uname || feed.uname || '用户'" />
                      </div>
                      <div class="dyn-feed-body">
                        <div class="dyn-feed-username">{{ feed.user?.uname || feed.uname || '用户' }}</div>
                        <div class="dyn-feed-content">{{ feed.desc || feed.content || feed.title || '' }}</div>
                        <div class="dyn-feed-time">{{ formatFeedTime(feed.ctime || feed.timestamp || feed.time || '') }}</div>
                      </div>
                      <div class="dyn-feed-right" v-if="feed.cover || feed.pic">
                        <img class="dyn-feed-cover" :src="feed.cover || feed.pic" alt="" />
                      </div>
                    </div>
                    <!-- 空状态 -->
                    <div class="dyn-feed-empty" v-if="dynamicFeedList.length === 0">
                      <span>暂无动态</span>
                    </div>
                  </div>
                </div>
              </div>
            </transition>
          </li>
          <li
            class="nav-item nav-item--icon"
            @mouseenter="collectFadeIn"
            @mouseleave="collectFadeOut"
          >
            <router-link
              v-if="minibiliCollectTo"
              class="t"
              :to="minibiliCollectTo"
              title="收藏"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><path d="M12 2L15 9L22 9.5L16.5 14L18 21L12 17.5L6 21L7.5 14L2 9.5L9 9L12 2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/><circle cx="10" cy="13" r="1" fill="currentColor"/><circle cx="14" cy="13" r="1" fill="currentColor"/><path d="M10 16Q12 18 14 16" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
              </span>
              <span class="nav-label">收藏</span>
            </router-link>
            <a
              v-else
              href="//www.bilibili.com"
              target="_blank"
              class="t"
              title="收藏"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><path d="M12 2L15 9L22 9.5L16.5 14L18 21L12 17.5L6 21L7.5 14L2 9.5L9 9L12 2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/><circle cx="10" cy="13" r="1" fill="currentColor"/><circle cx="14" cy="13" r="1" fill="currentColor"/><path d="M10 16Q12 18 14 16" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
              </span>
              <span class="nav-label">收藏</span>
            </a>
            <transition name="nav-trans">
              <div class="collect-list-box" v-show="collectShow">
                <!-- 左侧：收藏夹列表 -->
                <div class="collect-sidebar">
                  <div
                    class="collect-folder"
                    :class="{ active: collectActiveId === f.id }"
                    v-for="(f, idx) in collectFolders"
                    :key="'cf-'+idx"
                    @click="selectCollectFolder(f)"
                  >
                    <span class="cf-name">{{ f.title || f.name }}</span>
                    <span class="cf-count">{{ f.video_count || f.count || 0 }}</span>
                  </div>
                  <div class="collect-empty" v-if="collectFolders.length === 0">暂无收藏夹</div>
                </div>
                <!-- 右侧：选中收藏夹的视频列表 -->
                <div class="collect-main">
                  <div class="collect-video-list" v-if="collectVideos.length > 0">
                    <div class="collect-video-item" v-for="(v, idx) in collectVideos" :key="'cv-'+idx" @click="goCollectVideo(v)">
                      <img class="cv-cover" :src="v.cover_url || v.cover || ''" alt="" />
                      <div class="cv-info">
                        <div class="cv-title">{{ v.title }}</div>
                        <div class="cv-meta">{{ formatDuration(v.duration || v.duration_sec || 0) }} {{ v.uploader || v.author || '' }}</div>
                      </div>
                    </div>
                  </div>
                  <div class="collect-empty-right" v-else>暂无视频</div>
                  <!-- 底部操作栏 -->
                  <div class="collect-footer">
                    <router-link v-if="minibiliCollectTo" class="cf-btn cf-view-all" :to="minibiliCollectTo">查看全部</router-link>
                    <a v-else href="#" class="cf-btn cf-view-all" @click.prevent>查看全部</a>
                    <a href="#" class="cf-btn cf-play-all" @click.prevent>▶ 播放全部</a>
                  </div>
                </div>
              </div>
            </transition>
          </li>
          <li
            class="nav-item nav-item--icon"
            @mouseenter="historyFadeIn"
            @mouseleave="historyFadeOut"
          >
            <a href="#" class="t" @click.prevent="historyShow = !historyShow" title="历史">
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/><path d="M12 6V12L16 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
              </span>
              <span class="nav-label">历史</span>
            </a>
            <transition name="nav-trans">
              <div class="history-list-box" v-show="historyShow">
                <!-- 顶部Tab -->
                <div class="hist-tabs">
                  <span
                    class="hist-tab"
                    :class="{ active: historyActiveTab === t }"
                    v-for="(t, idx) in historyTabs"
                    :key="'ht-'+idx"
                    @click="switchHistoryTab(t)"
                  >{{ t }}</span>
                  <router-link v-if="minibiliHistoryTo" class="hist-more" :to="minibiliHistoryTo" @click.native="historyShow = false">查看全部 &gt;</router-link>
                </div>
                <!-- 历史记录列表 -->
                <div class="hist-body">
                  <!-- 按时间分组 -->
                  <template v-if="historyGrouped.length > 0">
                    <div class="hist-group" v-for="(group, gIdx) in historyGrouped" :key="'hg-'+gIdx">
                      <div class="hist-date-label">{{ group.label }}</div>
                      <div
                        class="hist-item"
                        v-for="(item, iIdx) in group.items"
                        :key="'hi-'+gIdx+'-'+iIdx"
                        @click="goHistoryItem(item)"
                      >
                        <img class="hist-cover" :src="item.cover || ''" alt="" />
                        <div class="hist-info">
                          <div class="hist-title">{{ item.title }}</div>
                          <div class="hist-meta">
                            <span class="hist-duration">{{ formatDuration(item.duration_sec || item.duration || 0) }}</span>
                            <span class="hist-time">{{ formatHistTime(item.viewed_at || item.ctime || '') }}</span>
                            <span class="hist-uploader" v-if="item.uploader || item.author"><span class="hist-up-icon">UP</span> {{ item.uploader || item.author }}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </template>
                  <div class="hist-empty" v-else>暂无历史记录</div>
                </div>
              </div>
            </transition>
          </li>
          <li class="nav-item nav-item--icon">
            <router-link
              v-if="isMinibiliMode"
              class="t"
              :to="{ name: 'upload' }"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><path d="M10 21H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M9 18H15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M12 2C8 2 6 5.5 6 9C6 12 7.5 14 9 15.5V17H15V15.5C16.5 14 18 12 18 9C18 5.5 16 2 12 2Z" stroke="currentColor" stroke-width="1.3"/><path d="M12 6L10 10.5H11.5L11 14L14 9.5H12.5L13 6H12Z" fill="currentColor"/></svg>
              </span>
              <span class="nav-label">创作中心</span>
            </router-link>
            <a
              v-else
              href="https://member.bilibili.com/v2#/home"
              target="_blank"
              class="t"
            >
              <span class="nav-icon">
                <svg viewBox="0 0 24 24" fill="none"><path d="M10 21H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M9 18H15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M12 2C8 2 6 5.5 6 9C6 12 7.5 14 9 15.5V17H15V15.5C16.5 14 18 12 18 9C18 5.5 16 2 12 2Z" stroke="currentColor" stroke-width="1.3"/><path d="M12 6L10 10.5H11.5L11 14L14 9.5H12.5L13 6H12Z" fill="currentColor"/></svg>
              </span>
              <span class="nav-label">创作中心</span>
            </a>
          </li>
        </ul>
      </div>
      <div class="up-load">
        <a
          v-if="minibiliUploadOpensModal"
          href="javascript:;"
          class="u-link"
          @click.prevent="onMbUploadNavClick"
          >投 稿</a
        >
        <router-link v-else class="u-link" :to="uploadNavTo">投 稿</router-link>
      </div>
    </div>
  </div>
</template>

<script>
import { createNamespacedHelpers } from "vuex";
import { setMinibiliPostLoginRedirect } from "@/utils/authTokens";
import akariFace from "../../assets/akari.jpg";
import http from "../../utils/http";
import {
  minibiliUploadOpensLoginModal,
  resolveMinibiliUploadNavTo
} from "@/utils/minibiliUploadNav";
import {
  minibiliDynamicsRoute,
  minibiliViewHistoryRoute,
  minibiliUserSpaceCollectRoute,
  minibiliUserSpaceRoute,
  minibiliLiveRoomRoute,
  minibiliArticleReadRoute,
  minibiliVideoPlayRoute,
  shouldShowMinibiliCompactHeader
} from "@/utils/minibiliRoutes";
import { formatCoinBalance, coinBalanceNumber } from "@/utils/coinBalance";
import {
  USER_LEVEL_HELP_URL,
  levelFillPct,
  levelPrivilegeLines
} from "@/utils/userLevel";
import {
  MESSAGE_CATEGORIES,
  formatMessageUnreadBadge,
  sumMessageUnread
} from "@/utils/messageCategories";
import {
  refreshMessageUnread,
  subscribeMessageUnread
} from "@/utils/messageUnread";
import {
  addSearchHistory,
  loadSearchHistoryAsync,
  removeSearchHistoryAt
} from "@/utils/searchHistory";

const { mapState, mapMutations, mapActions } = createNamespacedHelpers("login");

export default {
  props: {
    leftNav: {
      default: []
    },
    headBanner: {
      default: []
    },
    menuShow: {
      default: []
    }
  },
  data() {
    return {
      profileShow: false, //个人信息默认隐藏
      messageShow: false, //消息通知默认隐藏
      dynamicShow: false, //动态下拉默认隐藏
      // 动态下拉数据
      dynamicLiveList: [],   // 正在直播列表
      dynamicFeedList: [],   // 历史动态列表
      // 收藏夹下拉数据
      collectShow: false,    // 收藏夹下拉默认隐藏
      collectActiveId: 0,    // 当前选中的收藏夹ID
      collectFolders: [],    // 收藏夹列表
      collectVideos: [],     // 当前收藏夹的视频列表
      // 历史记录下拉数据
      historyShow: false,
      historyActiveTab: "视频",
      historyTabs: ["视频", "直播", "专栏"],
      historyRawItems: [],   // 原始历史数据
      userLevelHelpUrl: USER_LEVEL_HELP_URL,
      msgUnread: {},
      _msgUnreadUnsub: null,
      // 搜索相关
      suggestShow: false,
      historyPanelVisible: false,
      hotSearchVisible: false,
      searchHistory: [],
      _hideSearchPanelTimer: null,
      hotPlaceholderIndex: 0,
      _hotPlaceholderTimer: null
    };
  },
  computed: {
    /** 仅首页保留顶栏毛玻璃透明叠在头图上；其余路由顶栏纯白 */
    isHomeRoute() {
      return this.$route.name === "home";
    },
    /** 消息中心 / 个人空间等：纯白顶栏，无毛玻璃头图 */
    useSolidTopNav() {
      return !this.isHomeRoute || shouldShowMinibiliCompactHeader(this.$route);
    },
    isMinibiliMode() {
      return (
        import.meta.env.VITE_MINIBILI_API === "true" ||
        import.meta.env.VITE_MINIBILI_API === "1"
      );
    },
    // 搜索相关 computed
    searchValue: {
      get() {
        return this.$store.state.header.searchValue;
      },
      set(value) {
        this.$store.commit("header/SET_SEARCH_WORD", value);
      }
    },
    searchPlaceholder() {
      const list = this.hotPlaceholderList;
      if (!list.length) {
        return "搜索";
      }
      return list[this.hotPlaceholderIndex % list.length];
    },
    defaultSearchKeyword() {
      return (
        this.searchPlaceholder ||
        String((this.searchWord && this.searchWord.word) || "").trim()
      );
    },
    hotPlaceholderList() {
      const list = this.searchWord && this.searchWord.hot_list;
      if (Array.isArray(list) && list.length) {
        return list.map(s => String(s).trim()).filter(Boolean);
      }
      const one = String(
        (this.searchWord && this.searchWord.show_name) || ""
      ).trim();
      return one ? [one] : [];
    },
    suggestTagList() {
      const tags = this.suggest && this.suggest.tag;
      return Array.isArray(tags) ? tags : [];
    },
    historyPanelShow() {
      return (
        this.historyPanelVisible &&
        !this.suggestShow &&
        !this.hotSearchPanelShow &&
        this.searchHistory.length > 0
      );
    },
    hotSearchPanelShow() {
      return (
        this.hotSearchVisible &&
        !this.suggestShow &&
        this.hotSearchItems.length > 0
      );
    },
    // 使用对象展开运算符将此对象混入到外部对象中
    ...mapState({
      //命名空间获取state
      signIn: state => state.signIn, //登录状态获取
      proInfo: state => state.proInfo //个人信息获取
    }),
    // header 命名空间的搜索相关 state
    searchWord() {
      return this.$store.state.header.searchWord;
    },
    suggest() {
      return this.$store.state.header.suggest;
    },
    hotSearchItems() {
      return this.$store.state.header.hotSearchItems || [];
    },
    /** 顶栏用：兼容 proInfo 初始为 [] */
    navProfileRecord() {
      const p = this.proInfo;
      return p && typeof p === "object" && !Array.isArray(p) ? p : null;
    },
    navProfileReady() {
      return !!this.navProfileRecord;
    },
    navFaceSrc() {
      const p = this.navProfileRecord;
      if (p && p.face) {
        return p.face;
      }
      return this.signIn == 1 ? akariFace : "";
    },
    navDisplayName() {
      const p = this.navProfileRecord;
      return (p && p.uname) || "";
    },
    minibiliSpaceTo() {
      if (!this.isMinibiliMode || this.signIn != 1) {
        return null;
      }
      const p = this.navProfileRecord;
      if (!p || p.mid == null) {
        return null;
      }
      return minibiliUserSpaceRoute(p.mid);
    },
    minibiliCollectTo() {
      if (!this.isMinibiliMode || this.signIn != 1) {
        return null;
      }
      const p = this.navProfileRecord;
      if (!p || p.mid == null) {
        return null;
      }
      return minibiliUserSpaceCollectRoute(p.mid);
    },
    minibiliDynamicsTo() {
      if (!this.isMinibiliMode || this.signIn != 1) {
        return null;
      }
      return minibiliDynamicsRoute();
    },
    minibiliHistoryTo() {
      if (!this.isMinibiliMode) {
        return "/";
      }
      return minibiliViewHistoryRoute();
    },
    profileSpaceHref() {
      const p = this.navProfileRecord;
      if (p && p.mid != null) {
        return `https://space.bilibili.com/${p.mid}`;
      }
      return "/";
    },
    messageNavItems() {
      return MESSAGE_CATEGORIES;
    },
    messageUnreadTotal() {
      return sumMessageUnread(this.msgUnread);
    },
    messageUnreadTotalLabel() {
      return formatMessageUnreadBadge(this.messageUnreadTotal);
    },
    navCoinDisplay() {
      const p = this.navProfileRecord;
      if (p && typeof p.money === "number") {
        return formatCoinBalance(p.money);
      }
      return "0";
    },
    navCoinRaw() {
      const p = this.navProfileRecord;
      if (p && typeof p.money === "number") {
        return coinBalanceNumber(p.money);
      }
      return 0;
    },
    navBcoinDisplay() {
      const p = this.navProfileRecord;
      const w = p && p.wallet;
      if (w && typeof w.bcoin_balance === "number") {
        return w.bcoin_balance;
      }
      return 0;
    },
    navMoralPct() {
      const p = this.navProfileRecord;
      if (p && typeof p.moral === "number") {
        return p.moral;
      }
      return 0;
    },
    navLevelInfo() {
      const p = this.navProfileRecord;
      return p && p.level_info ? p.level_info : null;
    },
    navLevelDisplay() {
      const li = this.navLevelInfo;
      if (li && li.current_level != null) {
        const n = Number(li.current_level);
        if (Number.isFinite(n) && n >= 1) {
          return Math.min(6, Math.max(1, Math.floor(n)));
        }
      }
      return 1;
    },
    navLevelFillPct() {
      return levelFillPct(this.navLevelInfo);
    },
    navLevelPrivilegeLines() {
      return levelPrivilegeLines(this.navLevelDisplay);
    },
    //个人等级
    level() {
      const li = this.navLevelInfo;
      if (li && li.current_level != null) {
        return "lv" + li.current_level;
      }
      return "";
    },
    /** 最右侧「投稿」：Mini-Bili 未登录 → 弹主站登录窗 */
    minibiliUploadOpensModal() {
      void this.$route.fullPath;
      return minibiliUploadOpensLoginModal();
    },
    /** 已登录 Mini-Bili 或非 Mini：router-link 目标 */
    uploadNavTo() {
      void this.$route.fullPath;
      return resolveMinibiliUploadNavTo();
    },
    // 按当前 tab 筛选历史记录
    historyFiltered() {
      if (!this.historyRawItems.length) return [];
      if (this.historyActiveTab === "视频") {
        return this.historyRawItems.filter(item => !item.live_room_id && !item.article_id);
      } else if (this.historyActiveTab === "直播") {
        return this.historyRawItems.filter(item => item.media_type === "live" || item.live_room_id > 0);
      } else if (this.historyActiveTab === "专栏") {
        return this.historyRawItems.filter(item => item.media_type === "article" || item.article_id > 0);
      }
      return this.historyRawItems;
    },
    // 按时间分组的历史记录（计算属性，模板可直接引用）
    historyGrouped() {
      if (!this.historyFiltered.length) return [];
      const groups = {};
      const now = new Date();
      const todayStr = now.getFullYear() + "-" + String(now.getMonth()+1).padStart(2,"0") + "-" + String(now.getDate()).padStart(2,"0");
      const yesterday = new Date(now - 86400000);
      const yesterdayStr = yesterday.getFullYear() + "-" + String(yesterday.getMonth()+1).padStart(2,"0") + "-" + String(yesterday.getDate()).padStart(2,"0");

      this.historyFiltered.forEach(item => {
        const ts = item.viewed_at || item.ctime || item.created_at || "";
        let label = "";
        if (ts) {
          const d = typeof ts === "number" ? (ts > 9999999999 ? new Date(ts) : new Date(ts * 1000)) : new Date(ts);
          const ds = d.getFullYear() + "-" + String(d.getMonth()+1).padStart(2,"0") + "-" + String(d.getDate()).padStart(2,"0");
          if (ds === todayStr) label = "今天";
          else if (ds === yesterdayStr) label = "昨天";
          else label = d.getFullYear() + "年" + (d.getMonth()+1) + "月" + d.getDate() + "日";
        }
        if (!groups[label]) groups[label] = [];
        groups[label].push(item);
      });

      return Object.keys(groups).map(label => ({ label, items: groups[label] }));
    }
  },
  methods: {
    onMbUploadNavClick() {
      setMinibiliPostLoginRedirect("/upload");
      this.$store.commit("login/SET_LOGIN_TAB", 0);
      this.$store.commit("login/OPEN_LOGIN_MODAL");
    },
    ...mapMutations({
      setLoginShow: "SET_LOGIN_SHOW", //登录弹窗显示隐藏
      setLoginTab: "SET_LOGIN_TAB" //注册登录tab状态
    }),
    ...mapActions([
      "setSignIn", //登录
      "setUserInfo", //获取个人信息
      "refreshMinibiliMe",
      "signOut" //退出登录
    ]),
    //个人信息显示隐藏
    profileFadeIn() {
      this.profileShow = true;
    },
    profileFadeOut() {
      this.profileShow = false;
    },
    formatMessageUnreadBadge,
    onMsgUnreadSummary(summary) {
      this.msgUnread = summary || {};
    },
    //消息通知显示隐藏
    messageFadeIn() {
      this.messageShow = true;
      if (this.isMinibiliMode && this.signIn == 1) {
        void refreshMessageUnread();
      }
    },
    messageFadeOut() {
      this.messageShow = false;
    },
    //动态下拉显示隐藏
    dynamicFadeIn() {
      this.dynamicShow = true;
      this.loadDynamicData();
    },
    dynamicFadeOut() {
      this.dynamicShow = false;
      // 关闭时重置标记，下次 hover 重新拉取，实现准实时更新
      this._dynamicLoaded = false;
    },
    // 加载动态下拉数据（合并个人动态 + 最近投稿视频，无数据则留空）
    async loadDynamicData() {
      if (this._dynamicLoaded) return;
      // 需要登录才能获取个人数据
      if (this.signIn != 1) return;
      let ok = false;
      try {
        const feedRows = [];

        // 1. 拉取我的动态
        const dynRes = await http.get("/api/v1/users/me/dynamics", { params: { page_size: 4 } }).catch(() => null);
        if (dynRes) {
          const dynData = dynRes.data || dynRes || {};
          const dynItems = dynData.items || [];
          for (const item of dynItems) {
            feedRows.push({
              id: item.id,
              title: item.title || "",
              desc: item.content || item.title || "",
              content: item.content || "",
              ctime: item.created_at || "",
              cover: (item.images && item.images.length > 0) ? item.images[0] : "",
              pic: (item.images && item.images.length > 0) ? item.images[0] : "",
              user: {
                uname: this.navDisplayName || "用户",
                face: this.navFaceSrc || ""
              },
              uname: this.navDisplayName || "用户",
              face: this.navFaceSrc || ""
            });
          }
        }

        // 2. 拉取我的投稿视频（published），与动态合并展示
        const vidRes = await http.get("/api/v1/users/me/videos", {
          params: { page_size: 4, status: "passed" }
        }).catch(() => null);
        if (vidRes) {
          const vidData = vidRes.data || vidRes || {};
          const vidItems = vidData.items || [];
          for (const item of vidItems) {
            feedRows.push({
              id: item.id,
              title: item.title || "",
              desc: "投稿了视频",
              content: item.title || "",
              ctime: item.created_at || "",
              cover: item.cover_url || "",
              pic: item.cover_url || "",
              user: {
                uname: this.navDisplayName || "用户",
                face: this.navFaceSrc || ""
              },
              uname: this.navDisplayName || "用户",
              face: this.navFaceSrc || ""
            });
          }
        }

        // 3. 按时间倒序，取最近6条
        feedRows.sort((a, b) => {
          const ta = a.ctime ? new Date(a.ctime).getTime() : 0;
          const tb = b.ctime ? new Date(b.ctime).getTime() : 0;
          return tb - ta;
        });
        this.dynamicFeedList = feedRows.slice(0, 6);
        ok = true;
      } catch (e) {
        // 后端无数据则留空
      }
      if (ok) this._dynamicLoaded = true;
    },
    // 格式化动态时间
    formatFeedTime(ts) {
      if (!ts) return "";
      const now = Date.now();
      const t = typeof ts === "number" ? (ts > 9999999999 ? ts : ts * 1000) : new Date(ts).getTime();
      const diff = Math.floor((now - t) / 1000);
      if (diff < 60) return "刚刚";
      if (diff < 3600) return Math.floor(diff / 60) + "分钟前";
      if (diff < 86400) return Math.floor(diff / 3600) + "小时前";
      if (diff < 2592000) return Math.floor(diff / 86400) + "天前";
      return new Date(t).toLocaleDateString("zh-CN");
    },
    //收藏夹下拉显示隐藏
    collectFadeIn() {
      this.collectShow = true;
      this.loadCollectData();
    },
    collectFadeOut() {
      this.collectShow = false;
    },
    // 加载收藏夹数据
    async loadCollectData() {
      if (this._collectLoaded) return;
      const p = this.navProfileRecord;
      // 已登录用 mid，未登录用默认用户 3
      const userId = p && p.mid ? p.mid : 3;
      try {
        const res = await http.get("/api/v1/space/" + userId + "/favorite-folders").catch(() => null);
        console.log("[收藏夹] API返回:", res);
        if (res && res.data) {
          const folders = res.data.items || [];
          if (folders.length > 0) {
            this.collectFolders = folders;
            if (!this.collectActiveId) {
              this.selectCollectFolder(folders[0]);
            }
          }
        }
      } catch (e) { console.error("[收藏夹] 加载失败:", e); return; }
      this._collectLoaded = true;
    },
    // 选择收藏夹并加载视频
    async selectCollectFolder(f) {
      this.collectActiveId = f.id;
      this.collectVideos = [];
      if (!f.id) return;
      const p = this.navProfileRecord;
      const userId = p && p.mid ? p.mid : 3;
      try {
        const res = await http.get("/api/v1/space/" + userId + "/favorites", { params: { folder_id: f.id, limit: 6 } }).catch(() => null);
        console.log("[收藏夹视频] API返回:", res);
        if (res && res.data) {
          this.collectVideos = res.data.items || [];
        }
      } catch (e) { console.error("[收藏夹视频] 加载失败:", e); }
    },
    // 格式化视频时长
    formatDuration(sec) {
      sec = Number(sec) || 0;
      const h = Math.floor(sec / 3600);
      const m = Math.floor((sec % 3600) / 60);
      const s = Math.floor(sec % 60);
      if (h > 0) return h + ":" + String(m).padStart(2, "0") + ":" + String(s).padStart(2, "0");
      return m + ":" + String(s).padStart(2, "0");
    },
    // 跳转视频
    goCollectVideo(v) {
      const id = v.id || v.aid || v.video_id;
      if (id) {
        const router = this.$router;
        router.push("/video/BV" + id);
      }
      this.collectShow = false;
    },
    //历史下拉显示隐藏
    historyFadeIn() {
      this.historyShow = true;
      this.loadHistoryData();
    },
    historyFadeOut() {
      this.historyShow = false;
      // 关闭时重置标记，下次 hover 重新拉取，实现准实时更新
      this._historyLoaded = false;
    },
    // 加载历史数据
    async loadHistoryData() {
      if (this._historyLoaded) return;
      // 需要登录才能获取浏览历史
      if (this.signIn != 1) return;
      let ok = false;
      try {
        const res = await http.get("/api/v1/users/me/view-history", { params: { limit: 20 } }).catch(() => null);
        if (res) {
          const data = res.data || res || {};
          const rawItems = data.items || [];
          if (rawItems.length > 0) {
            this.historyRawItems = rawItems.map(item => ({
              id: item.live_room_id || item.video_id || item.article_id || 0,
              video_id: item.video_id || 0,
              article_id: item.article_id || 0,
              live_room_id: item.live_room_id || 0,
              title: item.title || "",
              cover: item.cover_url || "",
              duration_sec: item.duration_sec || 0,
              duration: item.duration_sec || 0,
              viewed_at: item.viewed_at || "",
              ctime: item.viewed_at || "",
              uploader: item.uploader_name || "",
              author: item.uploader_name || "",
              media_type: item.media_type || "video",
              progress_sec: item.progress_sec || 0,
              category: item.category || ""
            }));
            ok = true;
          } else {
            // API 成功但数据为空，也标记已加载
            ok = true;
            this.historyRawItems = [];
          }
        }
      } catch (e) { /* 留空 */ }
      if (ok) this._historyLoaded = true;
    },
    // 切换Tab
    switchHistoryTab(tab) {
      this.historyActiveTab = tab;
    },
    // 格式化历史时间（如"今天 16:35"）
    formatHistTime(ts) {
      if (!ts) return "";
      const now = new Date();
      const t = typeof ts === "number" ? (ts > 9999999999 ? new Date(ts) : new Date(ts * 1000)) : new Date(ts);
      const h = String(t.getHours()).padStart(2,"0");
      const m = String(t.getMinutes()).padStart(2,"0");
      const td = now.getFullYear() + "-" + String(now.getMonth()+1).padStart(2,"0") + "-" + String(now.getDate()).padStart(2,"0");
      const td2 = t.getFullYear() + "-" + String(t.getMonth()+1).padStart(2,"0") + "-" + String(t.getDate()).padStart(2,"0");
      if (td === td2) return "今天 " + h + ":" + m;
      return h + ":" + m;
    },
    // 跳转历史记录
    goHistoryItem(item) {
      if (!this.$router) return;
      // 直播
      if (item.live_room_id) {
        const route = minibiliLiveRoomRoute(item.live_room_id);
        if (route) this.$router.push(route);
      }
      // 专栏
      else if (item.article_id) {
        const route = minibiliArticleReadRoute(item.article_id);
        if (route) this.$router.push(route);
      }
      // 视频
      else if (item.video_id) {
        const route = minibiliVideoPlayRoute(item.video_id);
        if (route) this.$router.push(route);
      }
      this.historyShow = false;
    },
    // 搜索相关方法
    clearHideSearchPanelTimer() {
      if (this._hideSearchPanelTimer) {
        clearTimeout(this._hideSearchPanelTimer);
        this._hideSearchPanelTimer = null;
      }
    },
    startHotPlaceholderRotate() {
      const list = this.hotPlaceholderList;
      if (list.length <= 1) {
        return;
      }
      this._hotPlaceholderTimer = setInterval(() => {
        this.hotPlaceholderIndex =
          (this.hotPlaceholderIndex + 1) % list.length;
      }, 3500);
    },
    stopHotPlaceholderRotate() {
      if (this._hotPlaceholderTimer) {
        clearInterval(this._hotPlaceholderTimer);
        this._hotPlaceholderTimer = null;
      }
    },
    syncSearchPanels() {
      const hasInput = String(this.searchValue || "").length > 0;
      if (hasInput) {
        this.suggestShow = true;
        this.hotSearchVisible = false;
        this.historyPanelVisible = false;
      } else {
        this.suggestShow = false;
        // 无输入时优先显示热搜，其次显示历史
        if (this.hotSearchItems.length > 0) {
          this.hotSearchVisible = true;
          this.historyPanelVisible = false;
        } else {
          this.hotSearchVisible = false;
          this.historyPanelVisible = this.searchHistory.length > 0;
        }
      }
    },
    onSearchFocus() {
      this.clearHideSearchPanelTimer();
      // 聚焦时拉取最新热搜
      this.$store.dispatch("header/setHotSearchItems");
      void loadSearchHistoryAsync().then(list => {
        this.searchHistory = list;
        this.syncSearchPanels();
      });
    },
    onSearchBlur() {
      this.clearHideSearchPanelTimer();
      this._hideSearchPanelTimer = setTimeout(() => {
        this.suggestShow = false;
        this.hotSearchVisible = false;
        this.historyPanelVisible = false;
        this._hideSearchPanelTimer = null;
      }, 180);
    },
    onSearchInput() {
      this.$store.dispatch("header/setSuggest");
      this.syncSearchPanels();
    },
    removeHistoryItem(index) {
      this.searchHistory = removeSearchHistoryAt(index);
      if (!this.searchHistory.length) {
        this.historyPanelVisible = false;
      }
    },
    searchByHistory(keyword) {
      const kw = String(keyword || "").trim();
      if (!kw) {
        return;
      }
      this.searchValue = kw;
      this.searchHistory = addSearchHistory(kw);
      this.suggestShow = false;
      this.hotSearchVisible = false;
      this.historyPanelVisible = false;
      this.$router.push({ path: "/search/all", query: { keyword: kw } });
    },
    searchALL() {
      const raw = String(this.searchValue || "").trim();
      const kw = raw || String(this.defaultSearchKeyword || "").trim();
      if (kw) {
        this.searchHistory = addSearchHistory(kw);
        if (!raw) {
          this.searchValue = kw;
        }
      }
      this.suggestShow = false;
      this.hotSearchVisible = false;
      this.historyPanelVisible = false;
      this.$router.push({ path: "/search/all", query: { keyword: kw } });
    }
  },
  watch: {
    signIn(v) {
      if (this.isMinibiliMode && v == 1) {
        void refreshMessageUnread();
      } else if (v != 1) {
        this.msgUnread = {};
      }
    },
    $route() {
      if (this.isMinibiliMode && this.signIn == 1) {
        void refreshMessageUnread();
      }
    },
    hotPlaceholderList: {
      immediate: true,
      handler() {
        this.hotPlaceholderIndex = 0;
        this.stopHotPlaceholderRotate();
        this.startHotPlaceholderRotate();
      }
    }
  },
  mounted() {
    this._msgUnreadUnsub = subscribeMessageUnread(this.onMsgUnreadSummary);
    if (this.isMinibiliMode && this.signIn == 1) {
      void refreshMessageUnread();
    }
  },
  beforeUnmount() {
    if (this._msgUnreadUnsub) {
      this._msgUnreadUnsub();
      this._msgUnreadUnsub = null;
    }
    this.clearHideSearchPanelTimer();
    this.stopHotPlaceholderRotate();
  },
  async created() {
    // 初始化搜索热词
    this.$store.dispatch("header/setSearchDefaultWords");
    this.$store.dispatch("header/setHotSearchItems");
    this.searchHistory = await loadSearchHistoryAsync();

    const login = localStorage.getItem("signIn"); //读取缓存登录状态
    if (!login) {
      //无状态即未登录状态，修改state值
      this.setSignIn({
        signIn: localStorage.setItem("signIn", 0)
      });
    } else {
      //已登录状态
      //读取缓存状态
      this.setSignIn({
        signIn: localStorage.getItem("signIn")
      });
      if (this.isMinibiliMode && String(login) === "1") {
        await this.refreshMinibiliMe().catch(() => {});
      }
    }
  }
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss" scoped>
@import "../../style/mixin";

//菜单偏移，透明度过渡效果
.nav-trans-enter,
.nav-trans-leave-to {
  transform: translateY(5px);
  opacity: 0;
}
.nav-trans-enter-to,
.nav-trans-leave {
  transform: translateY(0px);
  opacity: 1;
}
.nav-trans-enter-active,
.nav-trans-leave-active {
  transition: all 0.3s ease;
}
//app-header
.app-header {
  position: relative;
  background: $white;
  .bili-wrapper {
    margin: 0 auto;
    width: 1160px;
  }
  // 导航栏搜索框（居中，宽度与悬浮面板一致）
  .nav-search {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    width: 420px;
    height: 42px;
    display: flex;
    align-items: center;
    z-index: 10;
    .searchform {
      position: relative;
      width: 100%;
      height: 30px;
      background-color: hsla(0, 0%, 100%, 0.8);
      border-radius: 4px;
      overflow: hidden;
      display: flex;
      align-items: center;
    }
    .search-keyword {
      flex: 1;
      height: 30px;
      line-height: 30px;
      padding: 0 40px 0 10px;
      border: 0;
      background: transparent;
      color: $black;
      font-size: 12px;
      outline: none;
      box-shadow: none;
      min-width: 0;
    }
    button.search-submit {
      position: absolute;
      right: 0;
      top: 0;
      width: 36px;
      height: 30px;
      min-width: 0;
      cursor: pointer;
      background: url(../../assets/icons.png) -653px -720px;
      margin: 0;
      padding: 0;
      border: 0;
      &:hover {
        background-position: -718px -720px;
      }
    }
    .bilibili-suggest {
      position: absolute;
      top: 100%;
      left: 0;
      width: 420px;
      border: 1px solid #e5e9ef;
      background: $white;
      z-index: 99999;
      border-radius: 0 0 4px 4px;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.16);
      padding-bottom: 5px;
      font-size: 12px;
      // 热搜面板
      &.hot-search-panel {
        width: 420px;
        padding: 0;
      }
      .hot-search-head {
        padding: 10px 12px 6px;
        border-bottom: 1px solid #e5e9ef;
      }
      .hot-search-title {
        font-size: 13px;
        font-weight: bold;
        color: #222;
      }
      .hot-search-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0;
        padding: 4px 0 6px;
      }
      .hot-search-item {
        display: flex;
        align-items: center;
        height: 32px;
        padding: 0 12px;
        cursor: pointer;
        text-decoration: none;
        color: #222;
        transition: background-color 0.15s;
        &:hover {
          background-color: #e5e9ef;
          .hot-search-text {
            color: #00a1d6;
          }
        }
      }
      .hot-search-rank {
        flex: 0 0 18px;
        width: 18px;
        height: 16px;
        line-height: 16px;
        text-align: center;
        font-size: 12px;
        font-weight: bold;
        color: #999;
        margin-right: 8px;
        &.top3 {
          color: $pink;
        }
      }
      .hot-search-text {
        flex: 1;
        min-width: 0;
        font-size: 12px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: #222;
        transition: color 0.15s;
      }
      .hot-search-badge {
        flex: 0 0 auto;
        margin-left: 4px;
        padding: 0 4px;
        height: 16px;
        line-height: 16px;
        font-size: 10px;
        color: $pink;
        background: #fff0f5;
        border-radius: 2px;
      }
      .b-line {
        border-top: 1px solid #e5e9ef;
        position: relative;
        height: 10px;
        margin: 10px 10px 0;
        p {
          margin-top: -10px;
          text-align: center;
        }
        span {
          display: inline-block;
          padding: 0 10px;
          height: 18px;
          font-size: 12px;
          text-align: center;
          cursor: pointer;
          color: $grau;
          background: $white;
        }
      }
      .suggest-item {
        padding: 8px 10px;
        cursor: pointer;
        word-wrap: break-word;
        word-break: break-all;
        display: block;
        color: $black;
        position: relative;
        a {
          color: $black;
          display: block;
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        &:hover {
          background-color: #e5e9ef;
        }
      }
      &.search-history-panel {
        padding-bottom: 8px;
      }
      .search-history-list {
        list-style: none;
        margin: 0;
        padding: 4px 0 0;
      }
      .search-history-item {
        display: flex;
        align-items: center;
        min-height: 32px;
        padding: 0 8px 0 10px;
        cursor: default;
        &:hover {
          background-color: #e5e9ef;
        }
      }
      .search-history-link {
        flex: 1;
        min-width: 0;
        display: block;
        height: 32px;
        line-height: 32px;
        color: #222;
        text-decoration: none;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        &:hover {
          color: #00a1d6;
        }
      }
      .search-history-del {
        flex: 0 0 28px;
        width: 28px;
        height: 28px;
        margin: 0;
        padding: 0;
        border: none;
        background: transparent;
        color: #99a2aa;
        font-size: 16px;
        line-height: 28px;
        text-align: center;
        cursor: pointer;
        &:hover {
          color: #00a1d6;
        }
      }
    }
  }
  .nav-menu {
    position: relative;
    z-index: 200;
    height: 42px;
    color: $black;
    .blur-bg {
      position: absolute;
      top: 0;
      left: 0;
      @include wh(100%, 100%);
      background-position: center -10px;
      background-repeat: no-repeat;
      -webkit-filter: blur(4px);
      filter: blur(4px);
    }
    .nav-mask {
      position: absolute;
      top: 0;
      left: 0;
      @include wh(100%, 100%);
      background-color: hsla(0, 0%, 100%, 0.4);
      -webkit-box-shadow: rgba(0, 0, 0, 0.1) 0 1px 2px;
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
      pointer-events: none;
    }
    &.nav-menu--solid-top {
      .blur-bg {
        display: none;
      }
      .nav-mask {
        background-color: #fff;
        -webkit-box-shadow: none;
        box-shadow: none;
      }
      .nav-inner .nav-con .nav-item {
        background-color: #fff !important;
        &:hover {
          background-color: #fff !important;
        }
      }
    }
    .nav-inner {
      position: relative;
      z-index: 1;
      display: flex;
      align-items: center;
      width: 100%;
      max-width: 1856px;
      margin: 0 auto;
      padding: 0 20px;
      box-sizing: border-box;
    }
    .nav-con {
      &--left {
        flex-shrink: 0;
      }
      &--right {
        flex-shrink: 0;
        margin-left: auto;
      }
      .nav-item {
        float: left;
        text-align: center;
        line-height: 42px;
        height: 42px;
        position: relative;
        background-color: hsla(0, 0%, 100%, 0);
        @include transition(0.3s);
        &:hover {
          background-color: hsla(0, 0%, 100%, 0.3);
        }
        a {
          &.t {
            color: $black;
            height: 100%;
            display: block;
            padding: 0 11px;
          }
        }
        // icon + label 纵向布局
        &--icon {
          a.t {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            line-height: 1;
            padding: 0 9px;
          }
          .nav-icon {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 20px;
            height: 20px;
            color: #555;
            svg {
              width: 20px;
              height: 20px;
            }
          }
          .nav-label {
            font-size: 11px;
            margin-top: 2px;
            color: #555;
            line-height: 1;
          }
          &:hover {
            .nav-icon { color: #00a1d6; }
            .nav-label { color: #00a1d6; }
          }
        }
        &.home {
          margin-left: -10px;
          padding-left: 12px;
          a {
            padding-left: 20px;
          }
          i {
            position: absolute;
            @include wh(17px, 14px);
            left: 10px;
            top: 12px;
            background-position: -919px -88px;
          }
        }
        &.mobile {
          i {
            display: inline-block;
            vertical-align: middle;
            background-position: -1367px -1175px;
            @include wh(21px, 21px);
          }
        }
      }
    }
    .nav-con {
      .nav-item {
        .t {
          .num {
            height: 12px;
            line-height: 12px;
            background-color: $pink;
            position: absolute;
            padding: 1px 2px;
            @include sc(12px, $white);
            @include borderRadius(10px);
            top: 1px;
            right: -4px;
            min-width: 16px;
            z-index: 30;
            text-align: center;
          }
        }
        .im-list-box {
          width: 110px;
          position: absolute;
          top: 100%;
          left: calc(50% - 55px);
          background: $white;
          box-shadow: rgba(0, 0, 0, 0.16) 0px 2px 4px;
          border-radius: 0 0 4px 4px;
          overflow: hidden;
          transition: all 300ms;
        }
        // 动态下拉面板
        .dynamic-list-box {
          width: 340px;
          position: absolute;
          top: 100%;
          left: calc(50% - 170px);
          background: $white;
          box-shadow: rgba(0, 0, 0, 0.16) 0px 2px 8px;
          border-radius: 8px;
          overflow: hidden;
          z-index: 500;
          padding-bottom: 8px;

          // 面板右上角「查看更多」（始终显示）
          .dyn-panel-more {
            position: absolute;
            right: 14px;
            top: 12px;
            font-size: 12px;
            color: #99a2aa;
            text-decoration: none;
            z-index: 10;
            &:hover { color: $blue; }
          }

          // 正在直播区域
          .dyn-live-section {
            padding: 14px 16px 10px;
            .dyn-live-header {
              display: flex;
              justify-content: space-between;
              align-items: center;
              margin-bottom: 12px;
              .dyn-live-title {
                font-size: 15px;
                font-weight: bold;
                color: #222;
              }
              .dyn-live-more {
                font-size: 12px;
                color: #99a2aa;
                text-decoration: none;
                &:hover { color: $blue; }
              }
            }
            .dyn-live-users {
              display: flex;
              justify-content: space-between;
              gap: 4px;
            }
            .dyn-live-user {
              display: flex;
              flex-direction: column;
              align-items: center;
              width: 48px;
              cursor: pointer;
              .dyn-live-avatar {
                width: 42px;
                height: 42px;
                border-radius: 50%;
                border: 2px solid #ff6699;
                overflow: hidden;
                background: #e5e9ef;
                img { width: 100%; height: 100%; object-fit: cover; display: block; }
              }
              .dyn-live-name {
                font-size: 11px;
                color: #222;
                margin-top: 4px;
                max-width: 52px;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                text-align: center;
              }
            }
          }

          // 历史动态区域
          .dyn-feed-section {
            .dyn-feed-divider {
              height: 1px;
              background: #eee;
              margin: 0 16px;
            }
            .dyn-feed-title {
              text-align: center;
              font-size: 13px;
              color: #99a2aa;
              padding: 10px 0 6px;
            }
            .dyn-feed-list {
              padding: 0 14px;
            }
            .dyn-feed-item {
              display: flex;
              align-items: flex-start;
              padding: 8px 4px;
              border-bottom: 1px solid #f5f5f5;
              cursor: pointer;
              transition: background 0.15s;
              &:hover { background: #fafafa; }
              &:last-child { border-bottom: none; }
              .dyn-feed-left {
                flex-shrink: 0;
                margin-right: 10px;
                .dyn-feed-avatar {
                  width: 36px;
                  height: 36px;
                  border-radius: 50%;
                  background: #e5e9ef;
                  object-fit: cover;
                }
              }
              .dyn-feed-body {
                flex: 1;
                min-width: 0;
                .dyn-feed-username {
                  font-size: 13px;
                  font-weight: bold;
                  color: #222;
                  line-height: 1.3;
                }
                .dyn-feed-content {
                  font-size: 13px;
                  color: #99a2aa;
                  line-height: 1.5;
                  margin-top: 2px;
                  display: -webkit-box;
                  -webkit-line-clamp: 2;
                  -webkit-box-orient: vertical;
                  overflow: hidden;
                }
                .dyn-feed-time {
                  font-size: 11px;
                  color: #99a2aa;
                  margin-top: 3px;
                }
              }
              .dyn-feed-right {
                flex-shrink: 0;
                margin-left: 8px;
                .dyn-feed-cover {
                  width: 80px;
                  height: 54px;
                  border-radius: 4px;
                  object-fit: cover;
                  background: #eee;
                }
              }
            }
            .dyn-feed-empty {
              text-align: center;
              padding: 20px 0;
              font-size: 13px;
              color: #99a2aa;
            }
          }
        }
        // 收藏夹下拉面板
        .collect-list-box {
          width: 360px;
          position: absolute;
          top: 100%;
          left: calc(50% - 180px);
          background: $white;
          box-shadow: rgba(0, 0, 0, 0.16) 0px 2px 8px;
          border-radius: 8px;
          overflow: hidden;
          z-index: 500;
          display: flex;

          // 左侧收藏夹列表
          .collect-sidebar {
            width: 110px;
            flex-shrink: 0;
            background: #f7f8fa;
            border-right: 1px solid #eee;
            max-height: 320px;
            overflow-y: auto;
            padding: 4px 0;
            .collect-folder {
              display: flex;
              justify-content: space-between;
              align-items: center;
              padding: 7px 10px;
              cursor: pointer;
              transition: background 0.15s;
              &:hover { background: #ebedf0; }
              &.active {
                background: $blue;
                .cf-name, .cf-count { color: $white; }
                .cf-name { font-weight: bold; }
              }
              .cf-name {
                font-size: 12px;
                color: #222;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
                max-width: 70px;
              }
              .cf-count {
                font-size: 11px;
                color: #99a2aa;
                flex-shrink: 0;
              }
            }
            .collect-empty {
              text-align: center;
              padding: 20px 8px;
              font-size: 12px;
              color: #99a2aa;
            }
          }

          // 右侧视频列表
          .collect-main {
            flex: 1;
            display: flex;
            flex-direction: column;
            min-width: 0;
            .collect-video-list {
              flex: 1;
              overflow-y: auto;
              padding: 6px 8px;
              .collect-video-item {
                display: flex;
                align-items: flex-start;
                padding: 4px 4px;
                cursor: pointer;
                border-radius: 4px;
                transition: background 0.15s;
                &:hover { background: #fafafa; }
                .cv-cover {
                  width: 72px;
                  height: 40px;
                  border-radius: 4px;
                  object-fit: cover;
                  flex-shrink: 0;
                  background: #e5e9ef;
                }
                .cv-info {
                  margin-left: 8px;
                  min-width: 0;
                  flex: 1;
                  .cv-title {
                    font-size: 12px;
                    color: #222;
                    line-height: 1.3;
                    display: -webkit-box;
                    -webkit-line-clamp: 2;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                  }
                  .cv-meta {
                    font-size: 10px;
                    color: #999;
                    margin-top: 2px;
                  }
                }
              }
            }
            .collect-empty-right {
              display: flex;
              align-items: center;
              justify-content: center;
              flex: 1;
              font-size: 13px;
              color: #99a2aa;
            }

            // 底部操作栏
            .collect-footer {
              display: flex;
              justify-content: space-between;
              align-items: center;
              padding: 7px 10px;
              border-top: 1px solid #f0f0f0;
              .cf-btn {
                font-size: 12px;
                text-decoration: none;
                &.cf-view-all { color: #222; }
                &.cf-play-all { color: $blue; }
                &:hover { opacity: 0.75; }
              }
            }
          }
        }
        // 历史记录下拉面板
        .history-list-box {
          width: 300px;
          position: absolute;
          top: 100%;
          left: calc(50% - 150px);
          background: $white;
          box-shadow: rgba(0, 0, 0, 0.16) 0px 2px 8px;
          border-radius: 8px;
          overflow: hidden;
          z-index: 500;

          // 顶部Tab栏
          .hist-tabs {
            display: flex;
            justify-content: center;
            border-bottom: 1px solid #f0f0f0;
            .hist-tab {
              font-size: 13px;
              color: #222;
              padding: 10px 14px;
              cursor: pointer;
              position: relative;
              transition: color 0.15s;
              &:hover { color: $blue; }
              &.active {
                color: $blue;
                font-weight: bold;
                &::after {
                  content: "";
                  position: absolute;
                  bottom: -1px;
                  left: 20%;
                  right: 20%;
                  height: 2px;
                  background: $blue;
                  border-radius: 1px;
                }
              }
            }
            .hist-more {
              margin-left: auto;
              font-size: 12px;
              color: #999;
              padding: 10px 8px;
              text-decoration: none;
              white-space: nowrap;
              &:hover { color: $blue; }
            }
          }

          // 历史列表区域
          .hist-body {
            max-height: 280px;
            overflow-y: auto;
            padding: 4px 0;

            .hist-group {
              .hist-date-label {
                font-size: 12px;
                color: #222;
                font-weight: bold;
                padding: 8px 12px 4px;
              }
              .hist-item {
                display: flex;
                align-items: flex-start;
                padding: 6px 10px;
                cursor: pointer;
                transition: background 0.15s;
                &:hover { background: #fafafa; }
                .hist-cover {
                  width: 72px;
                  height: 40px;
                  border-radius: 3px;
                  object-fit: cover;
                  flex-shrink: 0;
                  background: #e5e9ef;
                }
                .hist-info {
                  margin-left: 8px;
                  min-width: 0;
                  flex: 1;
                  .hist-title {
                    font-size: 12px;
                    color: #222;
                    line-height: 1.35;
                    display: -webkit-box;
                    -webkit-line-clamp: 2;
                    -webkit-box-orient: vertical;
                    overflow: hidden;
                  }
                  .hist-meta {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    font-size: 10px;
                    color: #999;
                    margin-top: 2px;
                    flex-wrap: wrap;
                    .hist-duration { color: #222; }
                    .hist-time { color: #999; }
                    .hist-uploader {
                      display: inline-flex;
                      align-items: center;
                      gap: 2px;
                      .hist-up-icon {
                        display: inline-block;
                        background: $blue;
                        color: $white;
                        font-size: 9px;
                        padding: 0 3px;
                        border-radius: 2px;
                        line-height: 14px;
                      }
                    }
                  }
                }
              }
            }

            .hist-empty {
              text-align: center;
              padding: 30px 0;
              font-size: 13px;
              color: #99a2aa;
            }
          }
        }
        .reg {
          a {
            display: initial;
            cursor: pointer;
            padding: 0;
            color: $blue;
          }
        }
      }
    }
    .dd-bubble {
      position: absolute;
      z-index: 1;
    }
    .up-load {
      position: relative;
      flex-shrink: 0;
      @include wh(58px, 42px);
      .u-link {
        position: relative;
        display: block;
        @include wh(100%, 48px);
        @include sc(14px, $white);
        line-height: 42px;
        text-align: center;
        z-index: 0;
        &:after {
          position: absolute;
          left: 0;
          content: "";
          @include wh(100%, 100%);
          background: $pink;
          border-bottom-left-radius: 5px;
          border-bottom-right-radius: 5px;
          z-index: -1;
        }
      }
    }
  }
  //右侧
  .profile-info {
    width: 58px;
    .i-face {
      position: absolute;
      z-index: 20;
      @include wh(36px, 36px);
      left: 11px;
      top: 0;
      @include transition(0.3s);
      .face {
        border: 0 solid $white;
        @include wh(100%, 100%);
        @include borderRadius(50%);
      }
      .pendant {
        position: absolute;
        @include wh(84px, 84px);
        left: -11px;
        bottom: -3px;
        visibility: hidden;
        -webkit-transition-delay: 0s;
        -o-transition-delay: 0s;
        transition-delay: 0s;
      }
    }
    &.on {
      &:hover {
        .i-face {
          left: -4px;
          top: 15px;
          @include wh(64px, 64px);
          .face {
            border: 2px solid $white;
          }
        }
      }
    }
    &:hover {
      .i_menu_login {
        display: block;
        opacity: 1;
        transition: all 0.3s;
      }
    }
  }
  //个人信息开始
  .profile-m {
    left: 50%;
    margin-left: -130px;
    width: 260px;
    padding: 50px 0 0;
    top: 42px;
    background: $white;
    -webkit-box-shadow: rgba(0, 0, 0, 0.16) 0 2px 4px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.16);
    border-radius: 0 0 4px 4px;
    line-height: normal;
    .header-u-info {
      a {
        color: $black;
      }
    }
    .header-uname {
      padding-bottom: 15px;
      b {
        display: block;
        margin-bottom: 8px;
        font-weight: bold;
      }
    }
    .btns-profile {
      position: relative;
      margin: 0 20px;
      height: 18px;
      .bili-icon {
        display: inline-block;
        @include wh(18px, 18px);
        vertical-align: middle;
        background-repeat: no-repeat;
      }
      .coin {
        .bi {
          background-position: -343px -471px;
          margin-right: 2px;
          position: relative;
          z-index: 2;
        }
        .jia {
          z-index: 1;
          left: 0;
          position: absolute;
          top: 0;
          @include wh(18px, 18px);
          background-position: -279px -1495px;
        }
      }
      .num {
        vertical-align: middle;
        display: inline-block;
        @include transition(2s);
      }
      .num-move {
        position: absolute;
        @include transition(2s);
        left: 23px;
        top: -10px;
        opacity: 0;
        line-height: 14px;
      }
      .num-tip {
        color: #2cc06f;
        position: absolute;
        @include transition(0.3s);
        left: 60px;
        top: -18px;
        opacity: 0;
        background: $white;
        padding: 3px 5px;
        z-index: 10;
      }
      .currency {
        position: absolute;
        left: 58px;
        z-index: 0;
        .bili-icon {
          background-position: -407px -471px;
          margin: 0 5px 0 8px;
        }
      }
      .ver {
        position: relative;
        a {
          display: block;
        }
        .tips {
          display: none;
          padding: 0 6px;
          height: 20px;
          line-height: 20px;
          border: 1px solid #ccc;
          @include borderRadius(4px);
          position: absolute;
          right: 30px;
          top: -2px;
          white-space: nowrap;
          background-color: $white;
          color: $black;
          z-index: 10;
          &:after {
            content: "";
            position: absolute;
            @include wh(8px, 8px);
            background: url(../../assets/horn.png);
            right: -8px;
            top: 6px;
          }
        }
        &:hover {
          .tips {
            display: block;
          }
        }
      }
      .phone {
        &.verified {
          .bili-icon {
            background-position: -343px -599px;
          }
        }
      }
      .email {
        margin-right: 10px;
        &.verified {
          .bili-icon {
            background-position: -343px -534px;
          }
        }
      }
    }
    .grade {
      position: relative;
      margin: 24px 0 30px;
      height: 16px;
      padding: 0 20px;
      .bar {
        position: relative;
        top: 6px;
        @include wh(170px, 8px);
        background: #eee;
        .lt {
          @include wh(18px, 18px);
          @include borderRadius(9px);
          position: absolute;
          left: -17px;
          top: -6px;
          display: flex;
          align-items: center;
          justify-content: center;
          background-color: #f3cb85;
          background-image: none;
          z-index: 1;
          &.lv1 {
            background-color: #94def5;
          }
          &.lv2 {
            background-color: #94def5;
          }
          &.lv3 {
            background-color: #6dc781;
          }
          &.lv4 {
            background-color: #f3cb85;
          }
          &.lv5 {
            background-color: #ff9f3f;
          }
          &.lv6 {
            background-color: #ff7f24;
          }
          .lt-num {
            display: block;
            font-size: 12px;
            font-weight: 700;
            line-height: 1;
            font-family: Arial, "Helvetica Neue", Helvetica, sans-serif;
            color: #fff;
            -webkit-text-stroke: 1.5px #000;
            paint-order: stroke fill;
            text-shadow:
              1px 0 0 #000,
              -1px 0 0 #000,
              0 1px 0 #000,
              0 -1px 0 #000;
          }
        }
        .rate {
          @include wh(20%, 8px);
          background-color: #f3cb85;
        }
        .num {
          position: absolute;
          right: 0;
          bottom: -18px;
          span {
            color: #ccc;
          }
        }
      }
      .desc-tips {
        display: none;
        padding: 15px 15px 15px 20px;
        position: absolute;
        top: -16px;
        left: 260px;
        @include borderRadius(2px);
        background-color: $white;
        z-index: 100;
        width: 220px;
        line-height: 24px;
        word-break: break-word;
        word-wrap: break-word;
        min-height: 65px;
        color: #676b73;
        -webkit-box-shadow: 0 0 2px 0 rgba(0, 0, 0, 0.25);
        box-shadow: 0 0 2px 0 rgba(0, 0, 0, 0.25);
        text-align: left;
        .arrow-left {
          position: absolute;
          display: inline-block;
          top: 16px;
          left: -10px;
          @include wh(10px, 20px);
          background: transparent url(../../assets/level.png) -182px -224px
            no-repeat;
        }
        .lv-row {
          margin-bottom: 10px;
          strong {
            @include sc(14px, $black);
            padding: 0 3px;
          }
        }
        .help-link {
          margin-top: 15px;
          float: right;
          color: $blue;
        }
      }
      &:hover {
        .desc-tips {
          display: block;
        }
      }
    }
    .member-menu {
      border-top: 1px solid #e5e9ef;
      padding: 10px 20px 40px;
      overflow: hidden;
      ul {
        width: 240px;
        clear: both;
        zoom: 1;
      }
      li {
        float: left;
        width: 100px;
        margin-right: 20px;
        position: relative;
        a {
          white-space: nowrap;
          color: $black;
          text-align: left;
          margin: 0 auto;
          display: block;
          padding: 5px 0;
          line-height: 16px;
          &:hover {
            color: $blue;
            .bili-icon {
              &.b-icon-p-account {
                background-position: -536px -407px;
              }
              &.b-icon-p-member {
                background-position: -601px -1046px;
              }
              &.b-icon-p-wallet {
                background-position: -536px -472px;
              }
              &.b-icon-p-live {
                background-position: -537px -855px;
              }
              &.b-icon-p-ticket {
                background-position: -535px -2075px;
              }
            }
          }
          .bili-icon {
            @include wh(16px, 16px);
            margin-right: 10px;
            vertical-align: top;
            &.b-icon-p-account {
              background-position: -472px -407px;
            }
            &.b-icon-p-member {
              background-position: -536px -1046px;
            }
            &.b-icon-p-wallet {
              background-position: -472px -472px;
            }
            &.b-icon-p-live {
              background-position: -473px -855px;
            }
            &.b-icon-p-ticket {
              @include wh(18px, 15px);
              background-position: -471px -2075px;
            }
          }
        }
      }
    }
    .member-bottom {
      position: absolute;
      bottom: 0;
      left: 0;
      @include wh(100%, 30px);
      line-height: 30px;
      background-color: #f4f5f7;
      border-radius: 0 0 4px 4px;
      .logout {
        float: right;
        padding-right: 20px;
        color: $black;
      }
    }
  }
  //大会员开始
  &.vip-m {
    width: 260px;
    margin-left: -107px;
    position: absolute;
    border-radius: 0 0 4px 4px;
    background-color: $white;
    -webkit-box-shadow: rgba(0, 0, 0, 0.16) 0 2px 4px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.16);
    border: 1px solid #e5e9ef;
    text-align: left;
    z-index: 7000;
  }
  .bubble-traditional {
    padding: 14px;
    .recommand {
      .title {
        @include sc(14px, #212121);
        margin: 5px 0 12px;
        font-weight: 900;
        .more {
          float: right;
          -webkit-box-sizing: border-box;
          box-sizing: border-box;
          border: 1px solid $border_color;
          font-weight: 400;
          text-align: center;
          @include borderRadius(4px);
          @include wh(52px, 22px);
          @include sc(12px, #6d757a);
          line-height: 22px;
          -webkit-transition: background 0.2s;
          -o-transition: background 0.2s;
          transition: background 0.2s;
        }
      }
      .bubble-col {
        display: flex;
        margin-bottom: 7px;
        .item {
          flex: 1;
          .pic {
            display: inline-block;
          }
          .recommand-link {
            display: block;
            margin-top: 10px;
            @include sc(12px, $black);
            text-align: left;
            line-height: 18px;
            height: 36px;
            overflow: hidden;
            -o-text-overflow: ellipsis;
            text-overflow: ellipsis;
            -webkit-line-clamp: 2;
            display: -webkit-box;
            -webkit-box-orient: vertical;
            &:hover {
              color: #fb7299;
            }
          }
        }
        &.bubble-col-3 {
          img {
            @include wh(72px, 94px);
            @include borderRadius(4px);
            background: #ccc;
          }
        }
      }
    }
  }
  .b-icon {
    &.b-icon-arrow-r {
      background-position: -478px -218px;
      @include wh(6px, 12px);
      margin: -2px 0 0 5px;
    }
  }
  img {
    border: none;
    vertical-align: middle;
  }
  .i_menu_login {
    opacity: 0;
    display: none;
    background: $white;
    left: 50%;
    margin-left: -130px;
    padding-bottom: 0;
    padding-top: 50px;
    border-top: none;
    width: 320px;
    margin-left: -160px;
    padding: 12px;
    text-align: left;
    line-height: normal;
    border: 1px solid #e5e9ef;
    @include transition(0.3s);
    .tip {
      @include sc(14px, #666);
    }
    .img {
      @include wh(320px, 200px);
      margin: 12px 0;
      overflow: hidden;
      position: relative;
      background: url(../../assets/danmu_bg.png) no-repeat 50%;
      img {
        &:first-child {
          @include wh(320px, 200px);
          position: absolute;
          left: 0;
          top: 0;
          animation: one 5s linear infinite;
        }
        &:last-child {
          @include wh(320px, 200px);
          position: absolute;
          left: 320px;
          top: 0;
          animation: two 5s linear infinite;
        }
      }
    }
    .reg {
      margin-top: 8px;
      text-align: center;
      @include sc(12px, #282828);
    }
  }
}
//动态开始
.im-list {
  display: block;
  text-align: center;
  position: relative;
  line-height: 42px;
  height: 42px;
  color: #99a2aa;
  &:hover {
    color: $blue;
    background-color: #e5e9ef;
  }
}
.im-notify {
  position: absolute;
  background-color: #fb7299;
  &.im-number {
    height: 14px;
    line-height: 15px;
    border-radius: 10px;
    padding: 1px 3px;
    @include sc(12px, $white);
    min-width: 20px;
    text-align: center;
    &.im-center {
      top: 13px;
      left: 80px;
    }
  }
}
@keyframes one {
  0% {
    left: 0;
  }

  100% {
    left: -320px;
  }
}
@keyframes two {
  0% {
    left: 320px;
  }

  100% {
    left: 0px;
  }
}
.app-header {
  .nav-menu {
    .nav-con {
      .nav-item {
        .login-btn {
          display: block;
          height: 43px;
          line-height: 43px;
          text-align: center;
          background: $blue;
          @include borderRadius(4px);
          @include sc(14px, $white);
          cursor: pointer;
        }
      }
    }
  }
}
</style>
