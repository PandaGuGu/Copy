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
