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

Install Bun, then run `bun install` to get the [`cf` CLI](https://developers.cloudflare.com/cf/)
and the Cloudflare Vite plugin. `scripts/build-worker.sh` installs the pinned
Rust toolchain and `worker-build` when they are missing.
Install cargo-deny 0.20.2 to run the workspace dependency check.

## Configure local development

Create `.dev.vars` in the repository root:

```dotenv
DISCORD_APPLICATION_ID=your_application_id
DISCORD_PUBLIC_KEY=your_public_key
DISCORD_TOKEN=your_bot_token
PROXY_API_URL=https://example.com/proxies
```

`cf dev` provides the application ID, public key, and proxy API URL to the
Worker. The registration utility reads the application ID and bot token
directly from the same file. Do not commit `.dev.vars`.

## Run locally

Start the Worker:

```bash
bun run dev
```

The script builds the Rust Worker first. The health endpoint is at `/` on the
local URL that the dev server prints. Discord
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
cargo deny --locked --workspace check
worker-build --release
```

Oxlint lints the TypeScript configuration files. Oxfmt checks the formatting
of the TypeScript, Markdown, JSON, YAML, and TOML files:

```bash
bun run lint
bun run format:check
```

## Deploy

Store `DISCORD_APPLICATION_ID`, `DISCORD_PUBLIC_KEY`, and `PROXY_API_URL` as
Worker secrets in Cloudflare. Declare them in `cloudflare.config.ts`.

Validate the Worker before deployment:

```bash
bun install
bun run build
bun cf deploy --prebuilt --dry-run
```

Pushes to `main` deploy through the Build and Test workflow. It needs the
`CLOUDFLARE_API_TOKEN` secret, an account API token that can edit Workers
scripts. To deploy by hand, sign in with `bunx cf auth login` and run:

```bash
bun run deploy
```

`DISCORD_TOKEN` is only needed by the local registration utility. The deployed
Worker does not need the bot token.

## Workspace layout

- `src/` contains the Cloudflare Worker, signature verification, proxy
  formatting, and multipart response encoder.
- `crates/commands/` contains the shared Discord command definitions.
- `crates/register/` contains the native command-registration utility.
- `cloudflare.config.ts` defines the Worker and its secrets. `scripts/build-worker.sh` builds the
  Rust Worker into `build/`, and the Cloudflare Vite plugin in `vite.config.ts` bundles it.

## Git hooks

`bun install` installs Lefthook for this checkout and preserves the existing preparation steps.
Run `bun run pre-commit` to run the commit checks manually.
The hooks use the existing project tools. Before pushing, they run the relevant project checks for changed files.
