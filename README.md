# piston-proxy-bot

A Rust Cloudflare Worker that shares proxy lists through Discord application
commands.

Invite: https://discord.com/oauth2/authorize?client_id=1332702321397727303

## Commands

- `/http` returns HTTP proxies.
- `/https` returns HTTPS proxies.
- `/socks4` returns SOCKS4 proxies.
- `/socks5` returns SOCKS5 proxies.
- `/all` returns every proxy with its URL scheme.
- `/invite` returns the Discord installation link.

Proxy commands respond with a text-file attachment. The upstream response is
cached at Cloudflare's edge for two hours.

## Prerequisites

The repository pins its Rust toolchain in `rust-toolchain.toml`. Install the
Cloudflare Worker build helper and Wrangler before running the Worker:

```bash
cargo install worker-build --version 0.8.5 --locked
```

Wrangler must be available as the `wrangler` command and authenticated with
your Cloudflare account for remote development or deployment.

## Configure local development

Create `.dev.vars` in the repository root:

```dotenv
DISCORD_APPLICATION_ID=your_application_id
DISCORD_PUBLIC_KEY=your_public_key
DISCORD_TOKEN=your_bot_token
PROXY_API_URL=https://example.com/proxies
```

Wrangler provides the application ID, public key, and proxy API URL to the
Worker. The registration utility reads the application ID and bot token
directly from the same file. Do not commit `.dev.vars`.

## Run locally

Start the Worker:

```bash
wrangler dev
```

The health endpoint is available at `http://localhost:8787/`. Discord
interactions are handled at `/interactions`.

## Register Discord commands

Register all commands globally:

```bash
cargo run -p piston-proxy-register
```

Global Discord command updates can take time to appear everywhere.

## Check the workspace

Run the same checks used by continuous integration:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
worker-build --release
```

## Deploy

Store the Worker bindings in Cloudflare:

```bash
wrangler secret put DISCORD_APPLICATION_ID
wrangler secret put DISCORD_PUBLIC_KEY
wrangler secret put PROXY_API_URL
```

Then deploy:

```bash
wrangler deploy
```

`DISCORD_TOKEN` is only needed by the local registration utility. The deployed
Worker does not need the bot token.

## Workspace layout

- `src/` contains the Cloudflare Worker, signature verification, proxy
  formatting, and multipart response encoder.
- `crates/commands/` contains the shared Discord command definitions.
- `crates/register/` contains the native command-registration utility.
- `wrangler.toml` defines the Worker build and deployment configuration.
