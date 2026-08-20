# ============================================================
#  Cakecake — 媒体冷备脚本（不可再生资产）
#
#  备份目标（按重要性排序）：
#    1. data/uploads/          用户上传的视频/封面/头像（不可再生）
#    2. MySQL                   全库逻辑备份（mysqldump）
#
#  用法:
#    ./scripts/backup.sh                # 全量备份到 ./backups/YYYYMMDD-HHMMSS/
#    BACKUP_DIR=/d/backups ./scripts/backup.sh   # 指定备份目录
#
#  建议: 挂到 Windows 任务计划程序每日执行（见文件底部注释）
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="${BACKUP_DIR:-$ROOT/backups}"
DEST="$BACKUP_DIR/$STAMP"

# 保留最近 N 份完整备份（默认 7）
KEEP="${BACKUP_KEEP:-7}"

mkdir -p "$DEST"
echo "[backup] 目标目录: $DEST"

# ── 1. 媒体文件（uploads 目录）────────────────────────────
if [ -d "$ROOT/data/uploads" ]; then
  echo "[backup] 拷贝 uploads/ ..."
  # cp -a 保留权限/时间戳；--parents 保留相对路径便于恢复
  (cd "$ROOT" && cp -a --parents "data/uploads/." "$DEST/" 2>/dev/null || cp -a "data/uploads" "$DEST/")
  SIZE_MB=$(du -sm "$DEST/data/uploads" 2>/dev/null | cut -f1)
  echo "[backup] uploads 完成: ${SIZE_MB:-?} MB"
else
  echo "[backup] 未发现 data/uploads（可能使用 OSS），跳过媒体备份"
fi

# ── 2. MySQL 逻辑备份（读 .env 的 MYSQL_DSN）───────────────
if [ -f "$ROOT/.env" ]; then
  DSN=$(grep -E '^MYSQL_DSN=' "$ROOT/.env" | head -1 | cut -d= -f2- | tr -d '"' || true)
fi
if [ -n "${DSN:-}" ]; then
  # 解析 user:pass@tcp(host:port)/dbname?params
  USER_PASS="${DSN%%@*}"
  DB_USER="${USER_PASS%%:*}"
  DB_PASS="${USER_PASS#*:}"
  REST="${DSN#*@tcp(}"
  DB_HOST="${REST%%:*}"
  REST2="${REST#*:}"
  DB_PORT="${REST2%%/*}"
  DB_NAME="${REST2#*/}"
  DB_NAME="${DB_NAME%%\?*}"

  echo "[backup] mysqldump $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME ..."
  if command -v mysqldump >/dev/null 2>&1; then
    MYSQL_PWD="$DB_PASS" mysqldump \
      -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USER" \
      --single-transaction --routines --triggers \
      "$DB_NAME" > "$DEST/db.sql" 2>/dev/null \
      || { echo "[backup][warn] mysqldump 失败，跳过数据库备份"; rm -f "$DEST/db.sql"; }
    [ -f "$DEST/db.sql" ] && echo "[backup] db.sql 完成: $(du -h "$DEST/db.sql" | cut -f1)"
  else
    echo "[backup][warn] 未找到 mysqldump 命令，跳过数据库备份"
  fi
else
  echo "[backup][warn] .env 无 MYSQL_DSN，跳过数据库备份"
fi

# ── 3. 清理旧备份 ─────────────────────────────────────────
cd "$BACKUP_DIR" 2>/dev/null || exit 0
COUNT=$(ls -d 2*/ 2>/dev/null | wc -l)
if [ "$COUNT" -gt "$KEEP" ]; then
  # 按目录名（时间戳）排序，删除最旧的
  ls -d 2*/ 2>/dev/null | sort | head -n $((COUNT - KEEP)) | while read -r old; do
    echo "[backup] 清理旧备份: $old"
    rm -rf "$old"
  done
fi

echo "[backup] 完成: $DEST"
echo ""
echo "恢复方法:"
echo "  媒体: 将 data/uploads 拷回项目根目录"
echo "  DB:   mysql -u<user> -p <dbname> < backups/<stamp>/db.sql"
echo ""
echo "Windows 定时（每日 03:00）:"
echo "  schtasks /Create /TN CakecakeBackup /TR \"bash -lc '/c/Users/Administrator/Desktop/cakecake-project/scripts/backup.sh'\" /SC DAILY /ST 03:00"
