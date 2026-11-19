# Repository instructions

## Required Lefthook setup

- Use Lefthook as the default Git hook manager for this repository.
- Before other project work, install the dependencies needed by the hooks.
- From the repository root, run `bunx --no-install lefthook install`.
- Run `bunx --no-install lefthook validate` and `bunx --no-install lefthook check-install` successfully.
- Make sure that Git's effective hook directory contains executable Lefthook hooks, including every configured commit hook.
- If hooks are missing, outdated, disabled, or fail installation checks, refuse all other project work.
- While blocked, only perform the setup and diagnostics needed to install and verify the hooks.
- Never bypass hooks with `--no-verify`, `-n`, environment variables, or configuration changes.
- Let the hooks finish and fix failures before committing or pushing.

## What the hooks run

- Before each commit, Lefthook checks staged whitespace. It runs the same Rust and Oxfmt format checks as CI when matching files are staged.
- The commit message hook checks the first line for Conventional Commits syntax. It also accepts Git-generated merge, revert, fixup, squash, and amend subjects.
- Before a push, Lefthook runs Clippy for Rust changes. For Worker or lint input changes, it builds the Worker before running Oxlint. The generated entry point is required for type checking.
- CI runs the deploy dry-run.

## Cloudflare tooling

- Use `cf` for direct Cloudflare CLI commands. Prefer existing package scripts for project workflows.
- For new Cloudflare configuration, prefer `cloudflare.config.ts`. Preserve existing configuration formats during unrelated work.
