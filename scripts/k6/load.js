// k6 压测脚本 —— CakeCake/MiniBili 性能回归门槛
//
// 用法：
//   k6 run scripts/k6/load.js
//   k6 run scripts/k6/load.js -e BASE_URL=http://127.0.0.1:8080
//
// 设计：
//   - 两个 stage：health（无状态、测纯吞吐）与 videos（DB+限流+追踪全链路）
//   - SLO 门槛（thresholds）：p95 / p99 / 错误率，超限即 exit code 99，可作 CI 性能门槛
//   - 参考 docs/PERF-REPORT.md 既有基线，脚本不追求极致绝对值，重点是可复现的回归门槛
import http from 'k6/http';
import { check } from 'k6';
import { Rate } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:18080';
const api = BASE_URL.replace(/\/$/, '') + '/api/v1';

// 只统计服务端 5xx 的"真错误率"。
// 注意：不用内置 http_req_failed，因为它会把 /videos 因限流返回的 4xx(429) 也算失败，
// 造成压测误报。这里将 4xx 视为业务限流（设计内），仅 5xx 计入失败。
const serverErrors = new Rate('server_errors');

export const options = {
  // 短压测：先 5s 爬升到 30 并发，再维持 30s，够检出回归又不至于长期压死本机
  scenarios: {
    smoke: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5s', target: 30 },
        { duration: '30s', target: 30 },
        { duration: '5s', target: 0 },
      ],
    },
  },
  thresholds: {
    // 无状态 health：p95 < 200ms，p99 < 500ms（宽松，回归门槛）
    'http_req_duration{name:health}': ['p(95)<200', 'p(99)<500'],
    // 全链路 videos：p95 < 500ms（DB 查询 + 限流 + 追踪）
    'http_req_duration{name:videos}': ['p(95)<500'],
    // 整体"真服务端错误率"（仅 5xx）门槛 <1%；4xx 视为业务限流，不误报
    server_errors: ['rate<0.01'],
    // 吞吐下限：整体 RPS 不低于 60（和 PERF-REPORT 干净样本 183 QPS 保持合理余量）
    http_reqs: ['rate>60'],
  },
};

export default function () {
  // 1) 健康检查（无状态，限流豁免）——测纯吞吐
  const h = http.get(api + '/health', {
    tags: { name: 'health' },
  });
  check(h, { 'health: 200': (r) => r.status === 200 });

  // 2) 视频列表（DB + 限流 + 追踪全链路）——可能偶发 429，属设计内；只统计 5xx 为错误
  const v = http.get(api + '/videos?limit=20', {
    tags: { name: 'videos' },
  });
  serverErrors.add(v.status >= 500);
  check(v, {
    'videos: not error (no 5xx)': (r) => r.status < 500,
    'videos: 200 or 429': (r) => r.status === 200 || r.status === 429,
  });

  // 3) 监控端点（限流豁免，验证可观测性可达）
  const m = http.get(api + '/metrics', {
    tags: { name: 'metrics' },
  });
  check(m, { 'metrics: 200': (r) => r.status === 200 });
}