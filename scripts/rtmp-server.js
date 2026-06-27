const NodeMediaServer = require("node-media-server");
const http = require("http");
const path = require("path");

// Auto-detect project root from this script's location (two levels up from scripts/)
const PROJECT_ROOT = path.resolve(__dirname, "..");
const HLS_DIR = path.join(PROJECT_ROOT, "data", "live");
const GO_API = "http://127.0.0.1:8080/api/v1/live/callback";

// Try to find ffmpeg from common locations, fall back to PATH
const { execSync } = require("child_process");
let ffmpegPath = "ffmpeg";
try {
  const out = execSync("ffmpeg -version 2>&1", { encoding: "utf8", timeout: 5000 });
  if (out) ffmpegPath = "ffmpeg";
} catch (e) {
  // ffmpeg not on PATH — try Windows common locations
  const cmn = require("os").platform() === "win32" ? [
    "C:\\ffmpeg\\bin\\ffmpeg.exe",
    process.env.FFMPEG_PATH
  ] : [
    "/usr/bin/ffmpeg",
    "/usr/local/bin/ffmpeg",
    process.env.FFMPEG_PATH
  ];
  for (const p of cmn) {
    if (p) {
      try { execSync(`"${p}" -version`, { timeout: 3000 }); ffmpegPath = p; break; } catch (_) {}
    }
  }
}
console.log(`[Live] ffmpeg: ${ffmpegPath}`);

const config = {
  rtmp: {
    port: 1935,
    chunk_size: 60000,
    gop_cache: true,
    ping: 30,
    ping_timeout: 60
  },
  http: {
    port: 8000,
    mediaroot: HLS_DIR,
    allow_origin: "*"
  },
  trans: {
    ffmpeg: ffmpegPath,
    tasks: [
      {
        app: "live",
        hls: true,
        hlsFlags: "[hls_time=2:hls_list_size=5:hls_flags=delete_segments]",
        hlsKeep: false,
        dash: false
      }
    ]
  }
};

const nms = new NodeMediaServer(config);

// Extract stream key from event args
function getStreamKey(args) {
  for (const arg of args) {
    if (typeof arg === 'object' && arg) {
      if (arg.streamPath) {
        const p = arg.streamPath.replace(/^\//, '').split('/');
        return p[p.length - 1];
      }
      if (arg.streamName) return arg.streamName;
    }
  }
  return 'unknown';
}

nms.on('postPublish', (...args) => {
  const streamKey = getStreamKey(args);
  console.log(`[Live] Stream started: ${streamKey}`);

  if (streamKey !== 'unknown') {
    const postData = JSON.stringify({ stream_key: streamKey });
    const req = http.request(`${GO_API}/on_publish`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    }, (res) => {
      let body = '';
      res.on('data', (c) => (body += c));
      res.on('end', () => console.log(`[Live] Backend ok: ${body}`));
    });
    req.on('error', (e) => console.log(`[Live] Notify failed: ${e.message}`));
    req.write(postData);
    req.end();
  }
});

nms.on('donePublish', (...args) => {
  const streamKey = getStreamKey(args);
  console.log(`[Live] Stream ended: ${streamKey}`);

  if (streamKey !== 'unknown') {
    const postData = JSON.stringify({ stream_key: streamKey });
    const req = http.request(`${GO_API}/on_done`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    }, (res) => {
      let body = '';
      res.on('data', (c) => (body += c));
      res.on('end', () => console.log(`[Live] Backend ok: ${body}`));
    });
    req.on('error', (e) => console.log(`[Live] Notify failed: ${e.message}`));
    req.write(postData);
    req.end();
  }
});

nms.run();

console.log("============================================");
console.log("  RTMP Server Ready!");
console.log("  Push:  rtmp://localhost:1935/live");
console.log("  HLS:   http://localhost:8080/live-hls/live/{key}/index.m3u8");
console.log("============================================");
