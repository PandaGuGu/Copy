#!/usr/bin/env bash
# 构建并打包安卓 APK（把 Java 薄壳编进 classes.dex，再用原 keystore 重签）
#
#   bash scripts/build-apk.sh [aarch64-linux-android|x86_64-linux-android]
#
# cargo-apk 只会打包 Rust 的 .so 和清单，不会编译 Java。返回键的 MainActivity
# 需要单独 javac + d8 生成 classes.dex 塞进 APK 根目录，最后重新签名。
set -euo pipefail

export ANDROID_HOME=${ANDROID_HOME:-/c/environment/android/sdk}
export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | tail -1)}
TARGET=${1:-aarch64-linux-android}
BT=$(ls -d "$ANDROID_HOME"/build-tools/* | tail -1)
BIN() { if [ -f "$BT/$1.bat" ]; then echo "$BT/$1.bat"; elif [ -f "$BT/$1.exe" ]; then echo "$BT/$1.exe"; else echo "$BT/$1"; fi; }
D8=$(BIN d8); APKSIGNER=$(BIN apksigner); ZIPALIGN=$(BIN zipalign); AAPT2=$(BIN aapt2)
PLATFORM=$(ls -d "$ANDROID_HOME"/platforms/android-* | tail -1)

cd "$(dirname "$0")/.."

# ── 局域网直连（真机与电脑同一 Wi-Fi 时优先走 LAN，而不是 127.0.0.1）──
# 自动探测电脑的局域网 IP 烘焙进 APK（编译期 option_env!("CAKECAKE_LAN_API")）。
# 想手动指定：CAKECAKE_LAN_IP=192.168.x.x bash scripts/build-apk.sh
# 想禁用：CAKECAKE_LAN_API= bash scripts/build-apk.sh
if [ -z "${CAKECAKE_LAN_API+x}" ]; then
  LAN_IP=${CAKECAKE_LAN_IP:-}
  if [ -z "$LAN_IP" ] && command -v powershell.exe >/dev/null 2>&1; then
    # 只挑「已连接且有默认网关」的物理网卡（排除虚拟/断开网卡的残留 IP）
    LAN_IP=$(powershell.exe -NoProfile -Command \
      "Get-NetIPConfiguration | Where-Object { \$_.NetAdapter.Status -eq 'Up' -and \$_.IPv4DefaultGateway -ne \$null -and \$_.InterfaceAlias -notmatch 'vEthernet|WSL|Docker|VPN|TAP|Loopback' } | ForEach-Object { \$_.IPv4Address.IPAddress } | Select-Object -First 1" \
      2>/dev/null | tr -d '\r\n ')
  fi
  if [ -n "$LAN_IP" ]; then
    export CAKECAKE_LAN_API="http://$LAN_IP:8080"
  fi
fi
if [ -n "${CAKECAKE_LAN_API:-}" ]; then
  echo "▶ 局域网地址烘焙进包：$CAKECAKE_LAN_API（值变化时 cargo 会自动重编）"
else
  echo "▶ 未配置局域网地址（回退 127.0.0.1 / 模拟器网关探测）"
fi

echo "▶ cargo apk build ($TARGET)"
cargo apk build --lib --release --target "$TARGET" >/dev/null
APK=target/release/apk/cakecake-rs.apk

rm -rf build/java build/dex build/pack
mkdir -p build/java build/dex build/pack

echo "▶ javac"
javac -nowarn -source 8 -target 8 -bootclasspath "$PLATFORM/android.jar" \
  -d build/java android/java/com/cakecake/rust/MainBack.java 2>/dev/null

echo "▶ d8"
"$D8" --lib "$PLATFORM/android.jar" --release --min-api 24 \
  --output build/dex $(find build/java -name '*.class')

echo "▶ 重新打包"
( cd build/pack && unzip -q "../../$APK" && cp ../dex/classes.dex . )
python scripts/patch_manifest.py build/pack/AndroidManifest.xml
python scripts/rename_activity.py build/pack/AndroidManifest.xml
( cd build/pack && jar cMf ../patched.apk . )
"$ZIPALIGN" -f 4 build/patched.apk build/aligned.apk

echo "▶ 签名"
"$APKSIGNER" sign --ks cakecake.keystore \
  --ks-pass pass:cakecake123 --key-pass pass:cakecake123 \
  --out "$APK" build/aligned.apk

echo "✓ $APK"
"$AAPT2" dump xmltree --file AndroidManifest.xml "$APK" | grep -m1 -A1 "E: activity" || true
