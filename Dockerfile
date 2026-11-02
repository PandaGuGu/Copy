# ─── Stage 1: Build Go binary ───
# 与 go.mod 的 `go 1.25.0` 对齐：低于 1.25 会直接报
# "go.mod requires go >= 1.25.0"
FROM golang:1.25-alpine AS builder

RUN apk add --no-cache git ca-certificates

WORKDIR /build

# Use vendored dependencies (no network required during build)
COPY go.mod go.sum ./
COPY vendor/ ./vendor/
COPY . .

RUN CGO_ENABLED=0 GOOS=linux go build -mod=vendor -ldflags="-s -w" -o mini-bili ./cmd/mini-bili/

# ─── Stage 2: Runtime ───
FROM alpine:3.21

RUN apk add --no-cache ca-certificates ffmpeg tzdata curl

# Create non-root user
RUN addgroup -g 1000 appgroup && adduser -u 1000 -G appgroup -D appuser

WORKDIR /app

# Binary
COPY --from=builder /build/mini-bili .

# Runtime config files
COPY configs/ ./configs/

# Create writable directories & set ownership
# NOTE: /app/data/uploads must exist here, otherwise Docker creates the empty
# named-volume mountpoint as root:root and every upload fails with
# "mkdir ... permission denied" (container runs as uid 1000).
RUN mkdir -p /app/data/tmp /app/data/uploads /app/logs && chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
    CMD wget -qO- http://localhost:8080/api/v1/health || exit 1

ENTRYPOINT ["./mini-bili"]
