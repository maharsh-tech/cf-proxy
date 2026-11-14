import http from "node:http";
import { Readable } from "node:stream";

const PORT = process.env.PORT || 3000;
const AUTH_KEY = process.env.PROXY_KEY || null;

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization, X-Proxy-Key, X-Proxy-Url",
};

const BROWSER_HEADERS = {
  "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
  "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
  "Accept-Language": "en-US,en;q=0.9",
  "Sec-Ch-Ua": '"Chromium";v="128", "Not;A=Brand";v="24", "Google Chrome";v="128"',
  "Sec-Ch-Ua-Mobile": "?0",
  "Sec-Ch-Ua-Platform": '"Windows"',
  "Sec-Fetch-Dest": "document",
  "Sec-Fetch-Mode": "navigate",
  "Sec-Fetch-Site": "none",
  "Sec-Fetch-User": "?1",
  "Upgrade-Insecure-Requests": "1",
};

const server = http.createServer(async (req, res) => {
  // 1. Handle CORS Preflight
  if (req.method === "OPTIONS") {
    res.writeHead(204, CORS_HEADERS);
    return res.end();
  }

  const reqUrl = new URL(req.url, `http://${req.headers.host || "localhost"}`);
  const pathname = reqUrl.pathname;

  // 2. Health check
  if (pathname === "/health" || pathname === "/ping") {
    res.writeHead(200, { ...CORS_HEADERS, "Content-Type": "application/json" });
    return res.end(JSON.stringify({ status: "ok", uptime: process.uptime() }));
  }

  // 3. Optional Authentication
  if (AUTH_KEY) {
    const key = req.headers["x-proxy-key"] || reqUrl.searchParams.get("key");
    if (key !== AUTH_KEY) {
      res.writeHead(401, { ...CORS_HEADERS, "Content-Type": "application/json" });
      return res.end(JSON.stringify({ error: "Unauthorized: Invalid or missing PROXY_KEY" }));
    }
  }

  // 4. Extract Target URL
  const targetUrl = reqUrl.searchParams.get("url");
  if (!targetUrl) {
    res.writeHead(400, { ...CORS_HEADERS, "Content-Type": "application/json" });
    return res.end(JSON.stringify({
      error: "Missing 'url' query parameter",
      usage: "GET /?url=https://new14.hdhub4u.cl/...",
    }));
  }

  if (!/^https?:\/\//i.test(targetUrl)) {
    res.writeHead(400, { ...CORS_HEADERS, "Content-Type": "application/json" });
    return res.end(JSON.stringify({ error: "Invalid target URL: must start with http:// or https://" }));
  }

  try {
    // 5. Fetch Target with fake headers from non-Cloudflare IP
    const upstreamResp = await fetch(targetUrl, {
      method: "GET",
      headers: {
        ...BROWSER_HEADERS,
        "Referer": targetUrl,
        "Cookie": "xla=s4t; xyt=1",
      },
      redirect: "follow",
    });

    // 6. Forward headers & body
    const contentType = upstreamResp.headers.get("content-type") || "text/html; charset=utf-8";
    res.writeHead(upstreamResp.status, {
      ...CORS_HEADERS,
      "Content-Type": contentType,
      "X-Upstream-Url": upstreamResp.url,
      "X-Upstream-Status": String(upstreamResp.status),
    });

    if (upstreamResp.body) {
      Readable.fromWeb(upstreamResp.body).pipe(res);
    } else {
      res.end();
    }
  } catch (err) {
    res.writeHead(502, { ...CORS_HEADERS, "Content-Type": "application/json" });
    return res.end(JSON.stringify({
      error: "Failed to fetch upstream URL",
      details: err.message,
    }));
  }
});

server.listen(PORT, () => {
  console.log(`[Proxy] Server running on port ${PORT}`);
});
