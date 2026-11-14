# Stream Relay Proxy

Ultra-lightweight HTTP proxy server designed to bypass Cloudflare-to-Cloudflare Worker blocking (Cloudflare Error 1106).

## Deployment on Voroa (https://app.getvoroa.com/)

1. Choose **Web Service** in Voroa.
2. Connect your GitHub repository (or push the `proxy` folder as a repository).
3. Configuration settings:
   - **Environment / Runtime**: `Node.js` (Node 18+)
   - **Build Command**: Leave empty or `npm install`
   - **Start Command**: `node server.js` (or `npm start`)
   - **Port**: `3000` (or let Voroa inject `$PORT`)
4. Once deployed, Voroa will provide you with a public URL, for example:
   `https://my-proxy-app.voroa.app`

## Usage with your Cloudflare Worker

In your Cloudflare Worker dashboard:
1. Go to **Settings** -> **Variables and Secrets**.
2. Add an environment variable:
   - **Variable name**: `PROXY_URL`
   - **Value**: `https://my-proxy-app.voroa.app`
3. Click **Deploy**.

Now all outbound scraping requests from your Cloudflare Worker will seamlessly route through your Voroa proxy, completely eliminating Cloudflare Error 1106.
