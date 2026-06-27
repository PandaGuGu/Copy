const NodeMediaServer = require("node-media-server");
const http = require("http");

const HLS_DIR = "C:/Users/Administrator/Desktop/cakecake-project/data/live";
const GO_API = "http://127.0.0.1:8080/api/v1/live/callback";

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
    ffmpeg: "C:/Users/Administrator/AppData/Local/Microsoft/WinGet/Links/ffmpeg",
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
