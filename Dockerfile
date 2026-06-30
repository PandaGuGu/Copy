# ─── Stage 1: Build Go binary ───
FROM golang:1.24-alpine AS builder

RUN apk add --no-cache git ca-certificates

WORKDIR /build

# Cache dependencies
COPY go.mod go.sum ./
RUN go mod download

# Build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o mini-bili ./cmd/mini-bili/

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
RUN mkdir -p /app/data/tmp /app/logs && chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 \
    CMD wget -qO- http://localhost:8080/api/v1/health || exit 1

ENTRYPOINT ["./mini-bili"]
