#!/usr/bin/env bash
# adb-install-auto.sh — 盯着屏幕装 APK：观察 → 判断 → 动作，不猜不盲点
#
# 国产 ROM（vivo 安全守护 / OPPO 安装校验 / MIUI 风险提示）在 adb 装非商店包时
# 会额外弹一层确认页：先勾「已了解…」，再点「继续安装」，否则 adb install 阻塞。
#
# 本脚本不缓存坐标、不预设等待：每轮先看焦点窗口（~125ms），确认在安装页上
# 才 dump 控件树（~2s）读取真实控件状态，看到未勾选就勾、看到按钮可点就点。
# 每步都打印带时间戳的观察结果。
#
# 用法: bash scripts/adb-install-auto.sh <apk路径> [设备序列号]
# 退出: 0=成功 1=失败 2=参数/环境问题

export MSYS_NO_PATHCONV=1
set -u

APK="${1:-}"; DEV="${2:-}"
[ -z "$APK" ] && { echo "usage: $0 <apk> [device-serial]"; exit 2; }
[ -f "$APK" ] || { echo "apk not found: $APK"; exit 2; }
[ -z "$DEV" ] && DEV=$(adb devices | awk 'NR>1 && $2=="device"{print $1; exit}')
[ -z "${DEV:-}" ] && { echo "no adb device online"; exit 2; }

WINAPK=$(cygpath -w "$APK" 2>/dev/null || printf '%s' "$APK")
TMPD=${TMPDIR:-/tmp}/adb-auto-install
mkdir -p "$TMPD"
LOG="$TMPD/install.log"; : > "$LOG"
UI="$TMPD/ui.xml"

T0=$(date +%s%3N)
log() { printf '[%5dms] %s\n' $(( $(date +%s%3N) - T0 )) "$*"; }

focus() {
  adb -s "$DEV" shell "dumpsys window 2>/dev/null | grep -m1 mCurrentFocus" 2>/dev/null | tr -d '\r'
}

# 读一次真实控件树，返回 "<动作> <x> <y>"
#   CK x y  = 风险复选框存在但未勾选 → 需要点它
#   BTN x y = 复选框已勾选、安装按钮在 → 需要点按钮
#   NONE    = 还没到可操作状态
observe() {
  adb -s "$DEV" shell "uiautomator dump /sdcard/_ai.xml" >/dev/null 2>&1
  adb -s "$DEV" shell "cat /sdcard/_ai.xml" 2>/dev/null > "$UI"
  python - "$(cygpath -w "$UI")" <<'PY'
import sys, re
try:
    xml = open(sys.argv[1], encoding='utf-8', errors='ignore').read()
except Exception:
    print('ERR'); sys.exit()
ck = btn = None
for m in re.finditer(r'<node[^>]*>', xml):
    s = m.group(0)
    t = re.search(r'text="([^"]*)"', s)
    b = re.search(r'bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"', s)
    if not (t and b):
        continue
    tx = t.group(1).strip()
    x1, y1, x2, y2 = map(int, b.groups())
    cx, cy = (x1 + x2) // 2, (y1 + y2) // 2
    if ck is None and ('已了解' in tx or '风险检测' in tx):
        ck = (cx, cy, re.search(r'checked="(\w+)"', s) is not None and
              re.search(r'checked="(\w+)"', s).group(1) == 'true')
    if btn is None and tx in ('继续安装', '仍要安装', '安装'):
        btn = (cx, cy)
if ck and not ck[2]:
    print(f'CK {ck[0]} {ck[1]}')
elif btn:
    print(f'BTN {btn[0]} {btn[1]}')
else:
    print('NONE')
PY
}

log "▶ 提交安装 $(basename "$APK")  →  $DEV"
timeout 60 adb -s "$DEV" install -r "$WINAPK" >>"$LOG" 2>&1 &
PID=$!

DUMPED=0
while kill -0 "$PID" 2>/dev/null; do
  if [ $(( $(date +%s%3N) - T0 )) -gt 90000 ]; then
    log "⚠ 超过 90s 未完成，强制中止并归还界面"
    kill -9 "$PID" 2>/dev/null
    adb -s "$DEV" shell "input keyevent 4" >/dev/null 2>&1
    break
  fi
  F=$(focus)
  if [[ "$F" == *installer* ]]; then
    DUMPED=$((DUMPED + 1))
    read -r ACT X Y <<< "$(observe)"
    case "$ACT" in
      CK)
        adb -s "$DEV" shell "input tap $X $Y" >/dev/null 2>&1
        log "看到「已了解…」未勾选 → 已勾选 ($X,$Y)"
        ;;
      BTN)
        adb -s "$DEV" shell "input tap $X $Y" >/dev/null 2>&1
        log "看到「继续安装」可用 → 已点击 ($X,$Y)"
        ;;
      *)
        log "确认页在，但控件还没就绪 → 继续观察"
        ;;
    esac
    continue
  fi
  sleep 0.2
done

wait "$PID" 2>/dev/null
MS=$(( $(date +%s%3N) - T0 ))

if grep -q "Success" "$LOG"; then
  log "✓ 安装成功，总耗时 ${MS}ms（观察 ${DUMPED} 次）"
  adb -s "$DEV" shell "rm -f /sdcard/_ai.xml /data/local/tmp/_adbinstall.apk" >/dev/null 2>&1
  exit 0
fi

log "✗ 安装失败，总耗时 ${MS}ms"
tail -4 "$LOG"
adb -s "$DEV" shell "rm -f /data/local/tmp/_adbinstall.apk" >/dev/null 2>&1
exit 1
