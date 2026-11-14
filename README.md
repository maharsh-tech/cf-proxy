# CF-Proxy (Ultra-Fast Node.js 20 Streaming Relay)

High-performance, low-latency HTTP relay proxy designed to eliminate Cloudflare-to-Cloudflare Worker blocking (Cloudflare Error 1106 / Turnstile challenge).

- **Runtime**: Native Node.js 20+ (Zero dependencies, instant deployment)
- **Streaming**: Native `Readable.fromWeb(body).pipe(res)` for immediate chunk streaming without memory buffering.
- **Headers**: Spoofs realistic Chrome client hints, referer, and cookies to bypass Cloudflare Bot Management.

---

## Deploying on Voroa (https://app.getvoroa.com/)

1. Choose **Web Service** in Voroa.
2. Select your repository: `maharsh-tech/cf-proxy`.
3. Configuration:
   - **Runtime**: `Node.js 20`
   - **Start Command**: `npm start` (or `node server.js`)
   - **Port**: `3000` (or leave default `$PORT`)
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
