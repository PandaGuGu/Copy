#!/usr/bin/env bash
# ============================================================
#  Cakecake — Linux/macOS 一键启动脚本
#
#  用法:
#    ./scripts/start.sh               # 自动启动所有服务
#    ./scripts/start.sh --docker      # 使用 Docker Compose
#    ./scripts/start.sh --skip-front  # 不启动前端
# ============================================================
set -e

# ── 参数解析 ──
USE_DOCKER=false
SKIP_FRONT=false
for arg in "$@"; do
  case "$arg" in
    --docker)    USE_DOCKER=true ;;
    --skip-front) SKIP_FRONT=true ;;
  esac
done

# ── 颜色 ──
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; GRAY='\033[0;90m'; NC='\033[0m'

# ── 自动定位项目根目录 ──
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

# ── 环境文件检查 ──
if [ ! -f ".env" ]; then
  echo -e "${RED}[!]${NC} 未找到 .env 文件"
  echo -e "${YELLOW}    请执行: cp .env.example .env  然后编辑 JWT_SECRET${NC}"
  exit 1
fi

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}    Cakecake 一键启动${NC}"
echo -e "${GRAY}    项目: $ROOT${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# ── 检查端口是否监听 ──
port_listening() {
  if command -v ss &>/dev/null; then
    ss -tlnp 2>/dev/null | grep -q ":$1 "
  elif command -v netstat &>/dev/null; then
    netstat -an 2>/dev/null | grep -q ":$1 "
  else
    lsof -i :$1 2>/dev/null | grep -q LISTEN
  fi
}

# ── Docker 模式 ──
if $USE_DOCKER; then
  echo -e "${CYAN}[Docker]${NC} 启动所有容器..."
  docker compose up -d
  if [ $? -eq 0 ]; then
    echo -e "${GREEN}[Docker] 启动成功! http://localhost${NC}"
    if command -v open &>/dev/null; then open http://localhost
    elif command -v xdg-open &>/dev/null; then xdg-open http://localhost; fi
  else
    echo -e "${RED}[Docker] 启动失败${NC}"
  fi
  exit 0
fi

# ════════════════════════════════════════
#  [1/6] MySQL
# ════════════════════════════════════════
echo -ne "[1/6] MySQL (3306)..."
if port_listening 3306; then
  echo -e " ${GREEN}OK${NC}"
else
  echo -e " ${YELLOW}未运行${NC}"
  echo -e "      请手动启动: systemctl start mysql / mysqld"
fi
sleep 1

# ════════════════════════════════════════
#  [2/6] Redis
# ════════════════════════════════════════
echo -ne "[2/6] Redis (6379)..."
if port_listening 6379; then
  echo -e " ${GREEN}OK${NC}"
else
  if command -v redis-server &>/dev/null; then
    redis-server --port 6379 --daemonize yes 2>/dev/null &
    sleep 1
    echo -e " ${GREEN}OK${NC}"
  else
    echo -e " ${YELLOW}未安装 — 请安装 redis${NC}"
  fi
fi
sleep 1

# ════════════════════════════════════════
#  [3/6] RabbitMQ
# ════════════════════════════════════════
echo -ne "[3/6] RabbitMQ (5672)..."
if port_listening 5672; then
  echo -e " ${GREEN}OK${NC}"
else
  if command -v rabbitmq-server &>/dev/null; then
    rabbitmq-server -detached 2>/dev/null &
    sleep 5
    echo -e " ${GREEN}OK${NC}"
  elif systemctl is-active rabbitmq-server &>/dev/null 2>&1; then
    sudo systemctl start rabbitmq-server 2>/dev/null && echo -e " ${GREEN}OK${NC}" || echo -e " ${YELLOW}需要 sudo 权限${NC}"
  else
    echo -e " ${YELLOW}未安装 — 请安装 rabbitmq-server${NC}"
  fi
fi

# ════════════════════════════════════════
#  [4/6] Go 后端
# ════════════════════════════════════════
echo -ne "[4/6] Backend (8080)..."
if port_listening 8080; then
  echo -e " ${GREEN}OK (已运行)${NC}"
else
  BIN="$ROOT/bin/mini-bili"
  if [ ! -f "$BIN" ]; then
    echo -ne " ${CYAN}构建中...${NC}"
    go build -o "$BIN" "$ROOT/cmd/mini-bili/" 2>/dev/null
  fi
  if [ -f "$BIN" ]; then
    "$BIN" &
    sleep 4
    port_listening 8080 && echo -e " ${GREEN}OK${NC}" || echo -e " ${YELLOW}启动中（检查 logs/）${NC}"
  else
    echo -e " ${RED}构建失败 — 请执行 go build -o ./bin/mini-bili ./cmd/mini-bili/${NC}"
  fi
fi

# ════════════════════════════════════════
#  [5/6] RTMP 直播服务器
# ════════════════════════════════════════
echo -ne "[5/6] RTMP Server (1935)..."
if port_listening 1935; then
  echo -e " ${GREEN}OK (已运行)${NC}"
else
  RTMP_JS="$ROOT/scripts/rtmp-server.js"
  if [ -d "$ROOT/scripts/node_modules/node-media-server" ] && command -v node &>/dev/null; then
    node "$RTMP_JS" &
    sleep 3
    port_listening 1935 && echo -e " ${GREEN}OK${NC}" || echo -e " ${YELLOW}启动中...${NC}"
  elif command -v srs &>/dev/null; then
    srs -c "$ROOT/deploy/srs-docker.conf" &
    sleep 3
    echo -e " ${GREEN}OK (SRS)${NC}"
  else
    echo -e " ${YELLOW}未安装 — 跳过直播功能${NC}"
    echo -e "      (安装: cd scripts && npm install 或使用 SRS)"
  fi
fi

# ════════════════════════════════════════
#  [6/6] 前端 (Vite dev server)
# ════════════════════════════════════════
if $SKIP_FRONT; then
  echo -e "[6/6] Frontend — ${GRAY}已跳过 (--skip-front)${NC}"
else
  echo -ne "[6/6] Frontend (8888)..."
  if port_listening 8888; then
    echo -e " ${GREEN}OK (已运行)${NC}"
  elif command -v node &>/dev/null; then
    VUE_DIR="$ROOT/cakecake-vue/bilibili-vue"
    if [ ! -d "$VUE_DIR/node_modules" ]; then
      echo -ne " ${CYAN}安装依赖...${NC}"
      npm install --prefix "$VUE_DIR" 2>/dev/null
    fi
    cd "$VUE_DIR" && npx vite --host 0.0.0.0 --port 8888 &
    cd "$ROOT"
    sleep 4
    echo -e " ${GREEN}OK${NC}"
  else
    echo -e " ${RED}Node.js 未安装${NC}"
  fi
fi

# ════════════════════════════════════════
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}  启动完成!${NC}"
echo -e "${CYAN}  前端:  http://localhost:8888${NC}"
echo -e "${CYAN}  后台:  http://localhost:8888/#/admin${NC}"
echo -e "${GRAY}  API:   http://localhost:8080/api/v1/health${NC}"
echo -e "${CYAN}========================================${NC}"
