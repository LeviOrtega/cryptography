# Cryptography

A shared, multiplayer cryptex. Everyone who opens the page is turning the same
eight cylinders, watching the same holographic coded string, and trying to
open the same lock.

## How the puzzle works

The server holds a secret 8-letter `password` and a constant 8-letter `key`
(the pad). It shows players the **coded** string:

```
coded[i] = (25 - (password[i] + key[i])) mod 26
```

Players set the cylinders to what they think the password is and submit. If
they are wrong, the server generates a new random password, re-derives the
coded string with the same key, and everyone's hologram shifts. If they are
right, the cryptex opens for everyone and the victory message appears.

## Layout

```
backend/   Rust (axum) — WebSocket server at /ws, also serves the built frontend
frontend/  React + Vite — the cryptex UI
Dockerfile Single image: builds frontend, builds backend, ships one binary
fly.toml   Fly.io config
```

### Backend modules

| file              | role |
| ----------------- | ---- |
| `src/cipher.rs`   | key parsing, `derive_coded`, encoding, random password |
| `src/protocol.rs` | JSON message types (client → server, server → client) |
| `src/state.rs`    | the single shared `Game` + broadcast channel |
| `src/ws.rs`       | per-connection task: apply inbound, relay broadcasts |
| `src/main.rs`     | config from env, router, static files, serve |

### WebSocket protocol

Client → server:

```json
{ "type": "rotate", "ring": 3, "index": 17 }
{ "type": "attempt" }
{ "type": "reset", "token": "<CRYPTEX_ADMIN_TOKEN>" }
```

Server → client:

```json
{ "type": "state", "coded": "sjlonqex", "rings": [0,0,0,0,0,0,0,0], "attempts": 4, "solved": false }
{ "type": "solved", "values": { "password": "hologram", "coded": "sjlonqex", "attempts": 5, "solvedAt": "2026-10-18T19:04:11Z" } }
{ "type": "error", "message": "ring 9 out of range" }
```

`state` is sent on connect and after every change from any client. The
`attempt` message carries no payload: the server validates whatever the rings
currently show, so a client can't submit positions it isn't displaying.

## Configuration (env vars)

| var                   | required | meaning |
| --------------------- | -------- | ------- |
| `CRYPTEX_KEY`         | yes      | the pad: `HOLOGRAM` or `7,14,11,14,6,17,0,12` |
| `CRYPTEX_ADMIN_TOKEN` | no       | enables `reset` messages carrying this token |
| `PORT`                | no       | default `8080` |
| `STATIC_DIR`          | no       | built frontend dir, default `../frontend/dist` |
| `RUST_LOG`            | no       | default `info,tower_http=warn` |

Copy `backend/.env.example` to `backend/.env` for local dev.

## Local development

Terminal 1 — server:

```sh
cd backend
cp .env.example .env        # first time only
cargo run
```

Terminal 2 — frontend with hot reload (Vite proxies `/ws` to the Rust server):

```sh
cd frontend
npm install
npm run dev                 # http://localhost:5173
```

Open the URL in two browser windows to see cylinder turns sync.

To test the production layout locally (Rust serving the built frontend):

```sh
cd frontend && npm run build && cd ../backend && cargo run
# http://localhost:8080
```

Tests: `cd backend && cargo test`.

## Customising the victory message

Edit `frontend/src/victoryMessage.tsx`. It receives `values` with
`password`, `coded`, `attempts`, and `solvedAt`; return any JSX. Styles are in
`index.css` under `.victory-message`.

## Deploying to Fly.io

```sh
brew install flyctl && fly auth login
fly launch --no-deploy            # accept the existing fly.toml / Dockerfile
fly secrets set CRYPTEX_KEY=HOLOGRAM CRYPTEX_ADMIN_TOKEN=$(openssl rand -hex 16)
fly deploy
```

Notes:

- `fly.toml` pins **one** always-on machine (`min_machines_running = 1`,
  `auto_stop_machines = "off"`). Puzzle state is in memory, so don't scale
  to multiple machines without adding shared storage.
- Fly terminates TLS; the browser connects over `wss://` automatically.
- A restart resets the puzzle (new random password, rings back to `A`). If you
  want state to survive restarts, add Postgres (`fly postgres create`) and
  persist `Game` in `state.rs`; the sqlx deps are commented in `Cargo.toml`.

### Alternative: Railway

Point a Railway service at this repo; it will pick up the `Dockerfile`. Set
`CRYPTEX_KEY` (and optionally `CRYPTEX_ADMIN_TOKEN`) under Variables.

### Alternative: split hosting

Deploy only the backend to Fly and host `frontend/dist` on Vercel / Netlify /
Cloudflare Pages, building with `VITE_WS_URL=wss://<app>.fly.dev/ws`. Browser
WebSockets are not subject to CORS, so no server changes are needed.

## Resetting after a solve

With `CRYPTEX_ADMIN_TOKEN` set, from the browser devtools on the page:

```js
// the hook exposes reset(); quickest is a raw socket:
const s = new WebSocket(`${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws`);
s.onopen = () => s.send(JSON.stringify({ type: "reset", token: "<token>" }));
```

Or simply restart the machine (`fly machine restart`).
