# CF-Proxy (Ultra-Fast Rust & Node.js Relay)

High-performance, low-latency HTTP relay proxy designed to eliminate Cloudflare-to-Cloudflare Worker blocking (Cloudflare Error 1106).

- **Rust (Default with Dockerfile)**: Built on **Axum 0.7** + **Tokio** + **Reqwest** with HTTP connection pooling, keep-alive reuse, and zero-buffering stream piping. Runs in an ultra-lean Alpine container (~15MB image, <10MB RAM).
- **Node.js (Alternative)**: Native Node.js `server.js` with streaming pipe and zero dependencies.

---

## Deploying on Voroa (https://app.getvoroa.com/)

1. Choose **Web Service** in Voroa.
2. Select your repository: `maharsh-tech/cf-proxy`.
3. Voroa automatically detects the `Dockerfile` and builds the high-speed Rust binary.
   - **Port**: `3000` (or let Voroa inject `$PORT`)
4. Copy the public URL provided by Voroa (e.g. `https://cf-proxy.your-app.voroa.app`).

---

## Connecting to Cloudflare Worker

In your Cloudflare Worker Dashboard:
1. Go to **Settings** $\to$ **Variables and Secrets**.
2. Add an environment variable:
   - **Variable name**: `PROXY_URL`
   - **Value**: `https://cf-proxy.your-app.voroa.app`
3. Click **Deploy**.

---

## Endpoints

- `GET /health` — Health check status
- `GET /?url=https://hubcloud.pw/drive/...` — Proxies the requested URL with realistic browser headers and returns the upstream stream.
