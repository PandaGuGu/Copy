#!/bin/bash
# ============================================================
# install-hooks.sh — 一键安装 Git Hooks
# 将 .githooks/ 目录配置为 Git 的 hooks 路径
# 所有协作者 clone 仓库后执行一次即可
# ============================================================

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.githooks"

echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}  Git Hooks 安装脚本${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""

# 检查是否在 Git 仓库中
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo -e "${RED}✗ 未找到 .git 目录，请在仓库根目录执行${NC}"
    exit 1
fi

# 检查 hooks 目录
if [ ! -d "$HOOKS_DIR" ]; then
    echo -e "${RED}✗ 未找到 .githooks 目录: $HOOKS_DIR${NC}"
    exit 1
fi

# 设置 Git hooks 路径
echo -e "${YELLOW}→ 配置 core.hooksPath...${NC}"
git config core.hooksPath .githooks
echo -e "${GREEN}  ✓ core.hooksPath = .githooks${NC}"

# 确保 hooks 有执行权限
echo -e "${YELLOW}→ 设置执行权限...${NC}"
chmod +x "$HOOKS_DIR"/* 2>/dev/null || true
echo -e "${GREEN}  ✓ hooks 已设置为可执行${NC}"

# 列出已安装的 hooks
echo ""
echo -e "${YELLOW}已安装的 hooks:${NC}"
for hook in "$HOOKS_DIR"/*; do
    if [ -f "$hook" ] && [ -x "$hook" ]; then
        hook_name=$(basename "$hook")
        case "$hook_name" in
            pre-commit)  desc="提交前检查（敏感信息 + go vet + 调试代码 + 大文件）" ;;
            commit-msg)  desc="提交信息格式校验（Conventional Commits）" ;;
            pre-push)    desc="推送前检查（禁止直推 main + 编译 + 测试）" ;;
            *)           desc="自定义 hook" ;;
        esac
        echo -e "  ${GREEN}✓${NC} $hook_name — $desc"
    fi
done

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  安装完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}使用说明:${NC}"
echo -e "  • pre-commit:  git commit 时自动触发"
echo -e "  • commit-msg:  git commit 时自动校验提交信息"
echo -e "  • pre-push:    git push 时自动触发"
echo ""
echo -e "${YELLOW}提交信息格式:${NC}"
echo -e "  <type>(<scope>): <description>"
echo -e "  示例: feat(video): 添加弹幕发送功能"
echo ""
echo -e "${YELLOW}跳过检查（不推荐）:${NC}"
echo -e "  git commit --no-verify    # 跳过 pre-commit + commit-msg"
echo -e "  BYPASS_PUSH=1 git push    # 跳过 pre-push"
echo ""
echo -e "${YELLOW}卸载:${NC}"
echo -e "  git config --unset core.hooksPath"
echo ""
