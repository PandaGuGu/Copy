# cakecake-rs v0.4.1 — 安全修复：更换签名密钥

> **这一版必须卸载旧版再装**：签名密钥已更换（旧密钥曾随仓库公开过，视为泄漏），
> Android 不允许同一包名换签名覆盖安装，会报 `INSTALL_FAILED_UPDATE_INCOMPATIBLE`。
>
> ```bash
> adb uninstall com.cakecake.rust
> adb install -r cakecake-rs-0.4.1.apk
> ```

功能与 v0.4.0 完全一致（直播间播放器、弹幕、创作中心那一批），只有签名变化：

| 项 | v0.4.0 | v0.4.1 |
|---|---|---|
| 签名 SHA256 | `24:60:C3:7D:…:38:DD`（已泄漏，废弃） | `D0:44:F2:F1:…:D4:AF:73` |
| 密钥/口令存放 | 密钥曾入库 | **只在本机**，`.gitignore` 排除；口令走 `CAKECAKE_KS_PASS` 或 `.keystore-pass` |
| 版本号 | 0.4.0 | 0.4.1 |

同时完成的其它安全处理（同批提交）：

- 仓库转私有，历史重写剔除 `LOCAL-DEPLOY.md`（含账号口令/内网 IP）与 `cakecake.keystore`，
  旧提交已不可达；
- 后端 `JWT_SECRET` 已轮换（旧 token 全部失效，需要重新登录）；
- 本地部署手册改为不入库，另留脱敏模板 `LOCAL-DEPLOY.example.md`；
- `build-apk.sh` 改为从环境变量 / 本地文件取签名口令，构建时临时写入 `Cargo.toml`，退出即还原。

> 本地 MySQL / 后台管理员 / 测试账号的口令轮换需要服务在运行时进行，
> 用 `scripts/rotate-local-secrets.ps1`（先启动 MySQL，再跑一次即可）。
