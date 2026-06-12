<template>
  <div class="login">
    <div class="complain-mask" @click="setLoginShow()" />
    <div class="login-stage">
      <div class="login-form">
        <div class="login-close" @click="setLoginShow()">
          <i class="iconfont icon-close" />
        </div>
        <div class="login-body">
          <!-- 左侧：二维码 -->
          <div class="login-qr">
            <div class="qr-box">
              <div class="qr-placeholder">
                <p class="qr-tip">二维码已过期</p>
                <p class="qr-tip">请点击刷新</p>
              </div>
            </div>
          </div>
          <!-- 右侧：登录表单 -->
          <div class="login-pwd">
            <!-- 顶部 Tab -->
            <div class="login-tabs">
              <span
                v-for="(t, i) in modeTabs"
                :key="i"
                :class="{ active: activeMode === i }"
                @click="switchMode(i)"
              >{{ t }}</span>
            </div>
            <template v-if="showRegister">
              <!-- 注册表单 -->
              <div class="register-user">
                <p v-if="isMinibiliMode" class="register-rule-hint">
                  用户名：{{ MINIBILI_USERNAME_RULE_HINT }}；密码 {{ MINIBILI_REGISTER_PASSWORD_HINT }}。
                </p>
                <div class="login-content">
                  <div class="field" :class="{ on: user !== '' }">
                    <input v-model="user" type="text"
                      :placeholder="isMinibiliMode ? MINIBILI_USERNAME_PLACEHOLDER : '昵称（例：哔哩哔哩）'"
                      :maxlength="isMinibiliMode ? 32 : 50" autocomplete="off" class="username" />
                    <p v-if="isMinibiliMode" class="field-hint" :class="{ 'field-hint--error': registerUsernameFieldError }">
                      {{ registerUsernameFieldError || MINIBILI_USERNAME_RULE_HINT }}
                    </p>
                  </div>
                  <div class="field password--with-toggle" :class="{ on: password !== '' }">
                    <input v-model="password"
                      :type="registerPasswordRevealed ? 'text' : 'password'"
                      :placeholder="isMinibiliMode ? '密码（至少 8 位）' : '密码（6-16个字符组成，区分大小写）'"
                      class="userpassword userpassword--padded" autocomplete="new-password" />
                    <button type="button" class="pwd-toggle"
                      :aria-pressed="registerPasswordRevealed"
                      :aria-label="registerPasswordRevealed ? '隐藏密码' : '显示密码'"
                      @click.prevent="registerPasswordRevealed = !registerPasswordRevealed">
                      <svg v-if="!registerPasswordRevealed" class="pwd-toggle__svg" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" d="M2 12s4-6 10-6 10 6 10 6-4 6-10 6S2 12 2 12z"/><circle cx="12" cy="12" r="2.75" fill="currentColor"/></svg>
                      <svg v-else class="pwd-toggle__svg" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" d="M3 3l18 18M10.6 10.6a2 2 0 002.8 2.8M9.9 5.3C10.6 5.1 11.3 5 12 5c6 0 10 7 10 7a18.9 18.9 0 01-3.5 4.2M6.2 6.2A18.5 18.5 0 002 12s4 7 10 7c1.1 0 2.1-.2 3.1-.5"/></svg>
                    </button>
                  </div>
                </div>
                <div class="btn-row">
                  <span class="btn-outline" @click="showRegister = false">返回登录</span>
                  <span class="btn-primary" :class="{ on: registerSubmitLooksReady }" @click="onRegister()">注册</span>
                </div>
              </div>
            </template>
            <template v-else-if="activeMode === 1">
              <!-- 短信登录占位 -->
              <div class="login-user">
                <div class="login-content">
                  <div class="field" :class="{ on: user !== '' }">
                    <input v-model="user" type="text" placeholder="手机号" maxlength="11" autocomplete="off" class="username" />
                  </div>
                  <div class="field sms-row" :class="{ on: password !== '' }">
                    <input v-model="password" type="text" placeholder="短信验证码" maxlength="6" class="sms-input" autocomplete="off" />
                    <span class="sms-btn">获取验证码</span>
                  </div>
                </div>
                <div class="btn-error">{{ btnErrorText }}</div>
                <span class="btn-primary" @click="onLogin()">登录</span>
              </div>
            </template>
            <template v-else>
              <!-- 密码登录 -->
              <div class="login-user">
                <div class="login-content">
                  <div class="field" :class="{ on: user !== '' }">
                    <input v-model="user" type="text"
                      :placeholder="isMinibiliMode ? '用户名（支持中文）' : '你的手机号/邮箱'"
                      maxlength="50" autocomplete="off" class="username" />
                    <p v-if="!isMinibiliMode" class="error-inline">{{ userError.errorText }}</p>
                  </div>
                  <div class="field password--with-toggle" :class="{ on: password !== '' }">
                    <input v-model="password"
                      :type="passwordRevealed ? 'text' : 'password'"
                      placeholder="密码"
                      class="userpassword userpassword--padded" autocomplete="current-password" />
                    <button type="button" class="pwd-toggle"
                      :aria-pressed="passwordRevealed"
                      :aria-label="passwordRevealed ? '隐藏密码' : '显示密码'"
                      @click.prevent="passwordRevealed = !passwordRevealed">
                      <svg v-if="!passwordRevealed" class="pwd-toggle__svg" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" d="M2 12s4-6 10-6 10 6 10 6-4 6-10 6S2 12 2 12z"/><circle cx="12" cy="12" r="2.75" fill="currentColor"/></svg>
                      <svg v-else class="pwd-toggle__svg" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" d="M3 3l18 18M10.6 10.6a2 2 0 002.8 2.8M9.9 5.3C10.6 5.1 11.3 5 12 5c6 0 10 7 10 7a18.9 18.9 0 01-3.5 4.2M6.2 6.2A18.5 18.5 0 002 12s4 7 10 7c1.1 0 2.1-.2 3.1-.5"/></svg>
                    </button>
                    <p v-if="!isMinibiliMode" class="error-inline">{{ passError.errorText }}</p>
                  </div>
                </div>
                <a class="login-forget" href="javascript:;">忘记密码？</a>
                <div class="btn-row">
                  <span class="btn-outline" @click="showRegister = true">注册</span>
                  <span class="btn-primary" :class="{ on: loginSubmitLooksReady }" @click="onLogin()">登录</span>
                </div>
                <div class="btn-error">{{ btnErrorText }}</div>
              </div>
            </template>
            <!-- 第三方登录 -->
            <div class="social-login">
              <p class="social-title">其他方式登录</p>
              <div class="social-icons">
                <a class="social-btn social-wx" title="微信登录"><svg viewBox="0 0 24 24" width="22" height="22"><path fill="#09BB07" d="M8.69 3.46c-3.96 0-7.17 3.06-7.17 6.84 0 2.16 1.04 4.08 2.67 5.35l-.66 2.01 2.33-1.15c.85.24 1.78.37 2.83.37.42 0 .84-.03 1.25-.08a5.96 5.96 0 01-.14-1.2c0-3.22 2.71-5.84 6.04-5.84.26 0 .51.02.77.05A7.17 7.17 0 008.69 3.46zm-2.02 4.1c.53 0 .96.41.96.92s-.43.91-.96.91a.94.94 0 01-.96-.91c0-.51.43-.92.96-.92zm4.03 0c.53 0 .96.41.96.92s-.43.91-.96.91a.94.94 0 01-.96-.91c0-.51.43-.92.96-.92z"/><path fill="#09BB07" d="M15.23 13.02c-3.33 0-6.04 2.61-6.04 5.84s2.71 5.84 6.04 5.84c.88 0 1.72-.19 2.5-.52l2.08 1.02-.55-1.76a5.56 5.56 0 002.06-4.58c0-3.23-2.71-5.84-6.09-5.84zm-2.2 3.07c.42 0 .76.33.76.73s-.34.73-.76.73c-.42 0-.76-.33-.76-.73s.34-.73.76-.73zm4.41 0c.42 0 .76.33.76.73s-.34.73-.76.73c-.42 0-.76-.33-.76-.73s.34-.73.76-.73z"/></svg><span>微信登录</span></a>
                <a class="social-btn social-wb" title="微博登录"><svg viewBox="0 0 24 24" width="22" height="22"><path fill="#E6162D" d="M20.19 14.66c-.35-.17-2.08-.98-2.4-1.09-.31-.1-.54-.16-.76.18-.23.34-.88 1.09-1.08 1.31-.19.22-.39.24-.74.08-.34-.16-1.46-.52-2.78-1.65a10.2 10.2 0 01-1.93-2.32c-.19-.34-.02-.52.16-.69.16-.15.35-.39.53-.59.17-.19.23-.34.35-.56.11-.22.06-.42-.03-.59-.09-.16-.76-1.77-1.04-2.43-.28-.63-.55-.54-.76-.55-.19 0-.42-.02-.65-.02-.23 0-.59.08-.9.42-.31.34-1.18 1.12-1.18 2.73 0 1.61 1.21 3.17 1.38 3.39.16.22 2.36 3.69 5.81 5.02.81.38 1.45.61 1.94.78.82.26 1.56.22 2.15.13.65-.09 2.08-.82 2.37-1.61.29-.79.29-1.47.2-1.61-.08-.14-.33-.23-.68-.41zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/></svg><span>微博登录</span></a>
                <a class="social-btn social-qq" title="QQ登录"><svg viewBox="0 0 24 24" width="22" height="22"><path fill="#12B7F5" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm.42 14.5c-.85.15-1.73-.12-2.18-.65-.2-.23-.29-.55-.18-.82.12-.3.45-.46.79-.44.56.03.92.2 1.26.2.05 0 .1 0 .15-.01.06-.01.1-.03.08-.1-.05-.2-.34-.48-.6-.74-.44-.43-1-.94-1.36-1.58-.23-.4-.37-.82-.42-1.22-.05-.38-.03-.75.09-1.09.14-.39.39-.7.74-.87.35-.16.78-.16 1.17.01.38.17.64.51.78.9.07.2.1.41.08.63-.02.22-.1.43-.24.61-.21.26-.53.44-.86.52-.19.04-.37.03-.55-.02-.06-.02-.1-.06-.1-.12.02-.18.17-.33.3-.48.07-.07.11-.15.11-.24.01-.11-.06-.22-.16-.26-.2-.08-.53 0-.72.22-.17.19-.25.43-.24.66.01.23.1.46.26.66.27.34.66.6 1.07.82.31.17.64.28.99.35.66.13 1.38.14 2.01-.08.31-.11.58-.34.72-.64.07-.15.07-.3.03-.45-.05-.18-.18-.32-.35-.39-.04-.02-.1-.03-.12.02-.04.09-.11.16-.19.23-.13.1-.29.15-.44.14-.14-.01-.27-.08-.36-.19-.08-.11-.11-.25-.08-.38.02-.11.09-.21.18-.27.13-.08.27-.11.41-.11.17.01.34.06.49.17.12.09.21.21.27.34.05.12.07.26.05.39-.02.16-.09.32-.21.44a3.03 3.03 0 01-2.78.97z"/></svg><span>QQ登录</span></a>
              </div>
            </div>
          </div>
        </div>
      <!-- 角色图 + 协议 -->
      <img class="login-char login-char--left" src="@/assets/login-char-left.png" alt="" />
      <img class="login-char login-char--right" src="@/assets/login-char-right.png" alt="" />
      <div class="login-agreement">
        未注册过哔哩哔哩的手机号，我们将自动帮你注册账号<br />
        登录或完成注册即代表你同意 <a href="javascript:;">用户协议</a> 和 <a href="javascript:;">隐私政策</a>
      </div>
    </div>
  </div>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { createNamespacedHelpers } from "vuex";
import { getUserInfo } from "../../api";
import { mbLogin, mbRegisterThenLogin } from "@/api/minibili";
import { consumeMinibiliPostLoginRedirect } from "@/utils/authTokens";
import {
  MINIBILI_REGISTER_PASSWORD_HINT,
  MINIBILI_USERNAME_PLACEHOLDER,
  MINIBILI_USERNAME_RULE_HINT,
  validateMinibiliUsername,
  validateMinibiliRegisterPassword,
  minibiliErrorMessage,
  mapMinibiliLoginFailureMessage
} from "@/utils/minibiliAuthRules";

const { mapState, mapMutations, mapActions } = createNamespacedHelpers("login");

export default {
  data() {
    return {
      modeTabs: ["密码登录", "短信登录"],
      activeMode: 0,
      showRegister: false,
      btnErrorText: "",
      userFlag: false,
      passFlag: false,
      passwordRevealed: false,
      registerPasswordRevealed: false,
      MINIBILI_USERNAME_RULE_HINT,
      MINIBILI_USERNAME_PLACEHOLDER,
      MINIBILI_REGISTER_PASSWORD_HINT
    };
  },
  computed: {
    ...mapState({
      nowindex: state => state.nowindex
    }),
    isMinibiliMode() {
      return (
        import.meta.env.VITE_MINIBILI_API === "true" ||
        import.meta.env.VITE_MINIBILI_API === "1"
      );
    },
    user: {
      get() { return this.$store.state.login.userName; },
      set(value) { this.updateUserName(value); }
    },
    password: {
      get() { return this.$store.state.login.password; },
      set(value) { this.updatePassword(value); }
    },
    userError() {
      let status, errorText;
      if (!/^\d{6,}$/g.test(this.user)) { status = false; errorText = "用户名不足六位"; }
      else { status = true; errorText = ""; }
      if (!this.userFlag) { this.userFlag = true; errorText = ""; }
      return { status, errorText };
    },
    passError() {
      let status, errorText;
      if (!/^\w{1,6}$/g.test(this.password)) { status = false; errorText = "密码超过六位"; }
      else { status = true; errorText = ""; }
      if (!this.passFlag) { this.passFlag = true; errorText = ""; }
      return { status, errorText };
    },
    minibiliLoginBtnReady() {
      const u = String(this.user || "").trim();
      const p = this.password || "";
      if (!u || !p) return false;
      return !validateMinibiliUsername(u);
    },
    registerUsernameFieldError() {
      if (!this.isMinibiliMode) return "";
      const u = String(this.user || "").trim();
      if (!u) return "";
      return validateMinibiliUsername(u);
    },
    minibiliRegisterBtnReady() {
      const u = String(this.user || "").trim();
      const p = this.password || "";
      if (!u || !p) return false;
      if (validateMinibiliUsername(u)) return false;
      if (validateMinibiliRegisterPassword(p)) return false;
      return true;
    },
    legacyLoginBtnReady() { return this.userError.status && this.passError.status; },
    legacyRegisterBtnReady() {
      const u = String(this.user || "").trim();
      const p = this.password || "";
      if (!u || !p) return false;
      return p.length >= 6 && p.length <= 16;
    },
    loginSubmitLooksReady() {
      return this.isMinibiliMode ? this.minibiliLoginBtnReady : this.legacyLoginBtnReady;
    },
    registerSubmitLooksReady() {
      return this.isMinibiliMode ? this.minibiliRegisterBtnReady : this.legacyRegisterBtnReady;
    }
  },
  watch: {
    activeMode() { this.btnErrorText = ""; },
    showRegister() { this.btnErrorText = ""; }
  },
  methods: {
    ...mapMutations({
      setLoginShow: "SET_LOGIN_SHOW",
      updateUserName: "SET_USERNAME",
      updatePassword: "SET_PASSWORD"
    }),
    ...mapActions(["setSignIn", "setVipInfo", "refreshMinibiliMe"]),
    switchMode(i) {
      this.activeMode = i;
      this.showRegister = false;
    },
    onLogin() {
      if (!this.loginSubmitLooksReady) return;
      this.btnErrorText = "";
      if (this.isMinibiliMode) {
        const u = String(this.user || "").trim();
        const p = this.password;
        if (!u || !p) { this.btnErrorText = "请输入账号和密码"; return; }
        const nameErr = validateMinibiliUsername(u);
        if (nameErr) { this.btnErrorText = nameErr; return; }
        mbLogin(u, p).then(() => {
          localStorage.setItem("signIn", "1");
          this.setSignIn({ signIn: "1" });
          this.$store.commit("login/CLOSE_LOGIN_MODAL");
          this.setVipInfo().catch(() => {});
          void this.refreshMinibiliMe().catch(() => {});
          const nextPath = consumeMinibiliPostLoginRedirect();
          const target = nextPath || this.$route.path;
          this.$router.replace(target).catch(() => {});
        }).catch(e => {
          this.btnErrorText = mapMinibiliLoginFailureMessage(e);
        });
        return;
      }
      sessionStorage.setItem("signIn", 0);
      if (!this.userError.status || !this.passError.status) {
        this.btnErrorText = "部分选项未通过";
      } else {
        getUserInfo().then(res => {
          localStorage.setItem("userName", this.user);
          localStorage.setItem("password", this.password);
          localStorage.setItem("signIn", 1);
          this.setSignIn({ signIn: localStorage.getItem("signIn") });
          this.$store.commit("login/SET_USER_INFO", { proInfo: res.data });
          this.setLoginShow();
          this.setVipInfo();
        });
      }
    },
    async onRegister() {
      if (!this.registerSubmitLooksReady) return;
      if (this.isMinibiliMode) {
        const u = String(this.user || "").trim();
        const p = this.password;
        if (!u || !p) { ElMessage.warning("请输入用户名和密码"); return; }
        const nameErr = validateMinibiliUsername(u);
        if (nameErr) { this.btnErrorText = nameErr; return; }
        const passErr = validateMinibiliRegisterPassword(p);
        if (passErr) { this.btnErrorText = passErr; return; }
        this.btnErrorText = "";
        try {
          await mbRegisterThenLogin(u, p);
          localStorage.setItem("signIn", "1");
          this.setSignIn({ signIn: "1" });
          this.$store.commit("login/CLOSE_LOGIN_MODAL");
          this.setVipInfo().catch(() => {});
          await this.refreshMinibiliMe().catch(() => {});
          const nextPath = consumeMinibiliPostLoginRedirect();
          const target = nextPath || this.$route.path;
          this.$router.replace(target).catch(() => {});
        } catch (e) {
          ElMessage.error(minibiliErrorMessage(e, "注册失败"));
        }
        return;
      }
      ElMessage.info("演示模式下暂不支持注册，请配置 cakecake 后端 API 后使用");
    }
  }
};
</script>

<style lang="scss">
@import "../../style/mixin";

.login { position: absolute; top: 0; @include wh(100%, 100%); }
.complain-mask {
  background: rgba(0, 0, 0, 0.8);
  @include wh(100%, 100%);
  position: fixed; z-index: 999; display: block; top: 0; left: 0;
}

/* === 舞台 === */
.login-stage {
  position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
  z-index: 9999; width: 750px; min-height: 460px; pointer-events: none;
}

/* === 角色图：对齐表单左/右下角 === */
.login-char {
  position: absolute; bottom: 0; pointer-events: none; user-select: none; z-index: 10;
}
.login-char--left { left: 0; width: 100px; height: auto; }
.login-char--right { right: 0; width: 100px; height: auto; }

/* === 白框 === */
.login-form {
  position: relative; width: 100%; min-height: 430px; background: $white; @include borderRadius(8px);
  pointer-events: auto; box-shadow: 0 4px 24px rgba(0,0,0,0.15);
  overflow: hidden;
  .login-close {
    position: absolute; cursor: pointer; right: 16px; top: 16px; z-index: 10;
    .icon-close { @include sc(22px, #909399); &:hover { color: $blue; } }
  }
}

/* === 主体：左扫码+右表单 === */
.login-body {
  display: flex; padding: 0 0 50px;
}

/* 二维码 */
.login-qr {
  flex: 0 0 280px; display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 12px 0; background: #f6f7f8; position: relative;
  &::after { content:""; position:absolute; right:0; top:20px; bottom:20px; width:1px; background:#e8e8e8; }
}
.qr-box {
  width: 180px; height: 180px; background: $white; border-radius: 6px;
  display: flex; align-items: center; justify-content: center; border: 1px solid #e8e8e8;
}
.qr-placeholder { text-align: center; }
.qr-tip { font-size: 12px; color: #999; line-height: 1.6; margin: 0; }
.qr-desc { font-size: 12px; color: #999; line-height: 1.5; margin: 2px 0 0; }

/* 右侧表单区 */
.login-pwd {
  flex: 1; padding: 32px 30px 0; display: flex; flex-direction: column; min-width: 0;
}
.login-tabs {
  display: flex; justify-content: center; gap: 32px; font-size: 15px; flex-shrink: 0;
  span {
    cursor: pointer; color: #666; padding-bottom: 6px; border-bottom: 2px solid transparent;
    transition: 0.2s;
    &.active { color: $blue; border-bottom-color: $blue; font-weight: 600; }
    &:hover { color: $blue; }
  }
}

/* 输入框 */
.login-user, .register-user { flex: 1; display: flex; flex-direction: column; }
.login-content, .register-content { margin-top: 6px; width: 100%; }
.field { position: relative; }
.field input {
  box-sizing: border-box; border: 1px solid #dcdfe0; border-radius: 6px;
  padding: 10px 12px 0; margin: 8px 0 0; @include wh(100%, 44px); font-size: 14px;
  background: #f7f8fa;
}
.field.on input, .field input:focus { border-color: $blue; background: $white; }
.error-inline { @include sc(11px, $pink); margin-top: 2px; padding-left: 4px; }

/* 密码可见切换 */
.password--with-toggle { position: relative; }
.password--with-toggle .pwd-toggle {
  position: absolute; right: 6px; top: 50%; transform: translateY(-50%);
  display: flex; align-items: center; justify-content: center;
  width: 36px; height: 36px; padding: 0; border: none; background: transparent; color: #9499a0;
  cursor: pointer; border-radius: 6px; z-index: 1;
  &:hover { color: $blue; background: rgba(0,161,214,0.08); }
}
.pwd-toggle__svg { width: 22px; height: 22px; }
.userpassword--padded { padding-right: 44px; }

/* 短信 */
.sms-row { display: flex; align-items: flex-end; gap: 8px; }
.sms-input { flex: 1; }
.sms-btn {
  flex-shrink: 0; font-size: 13px; color: $blue; cursor: pointer; line-height: 44px; white-space: nowrap;
}

/* 忘记密码 */
.login-forget {
  display: block; text-align: right; @include sc(12px, $blue); margin: 6px 0 0; flex-shrink: 0;
}

/* 按钮行 */
.btn-row { display: flex; gap: 12px; margin-top: 14px; flex-shrink: 0; }
.btn-outline {
  flex: 1; text-align: center; line-height: 38px; @include borderRadius(6px); cursor: pointer;
  border: 1px solid #ccc; @include sc(14px, #666); user-select: none;
  &:hover { border-color: $blue; color: $blue; }
}
.btn-primary {
  flex: 1; text-align: center; line-height: 38px; @include borderRadius(6px); cursor: pointer;
  background: #d1d1d1; @include sc(14px, $white); user-select: none;
  &.on { background: $blue; }
}

.btn-error { height: 18px; line-height: 18px; @include sc(12px, $pink); text-align: right; flex-shrink: 0; }

/* 第三方登录 */
.social-login { flex-shrink: 0; margin-top: auto; padding-bottom: 4px; }
.social-title { text-align: center; @include sc(12px, #999); margin: 0 0 8px; }
.social-icons { display: flex; justify-content: center; gap: 24px; }
.social-btn {
  display: flex; flex-direction: column; align-items: center; gap: 4px; cursor: pointer; text-decoration: none;
  padding: 8px 16px; border: 1px solid #e8e8e8; border-radius: 8px; min-width: 64px;
  span { font-size: 11px; color: #666; }
  &:hover { border-color: $blue; span { color: $blue; } }
}

/* 注册 */
.register-user .field-hint { margin: 4px 0 0; padding: 0 2px; font-size: 12px; line-height: 1.4; color: #9499a0; }
.register-user .field-hint--error { color: $pink; }
.register-rule-hint { margin: 4px 0 0; padding: 0 2px; font-size: 11px; line-height: 1.5; color: #9499a0; flex-shrink: 0; }

/* 底部协议（2233 中间填白） */
.login-agreement {
  position: absolute; bottom: 0; left: 100px; right: 100px; z-index: 11;
  padding: 0 16px 6px; text-align: center; @include sc(12px, #999); line-height: 1.6;
  background: $white;
  pointer-events: auto;
  a { color: $blue; text-decoration: none; &:hover { text-decoration: underline; } }
}
</style>
