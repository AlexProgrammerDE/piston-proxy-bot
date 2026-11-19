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
- Before a push, Lefthook runs `cargo deny --locked --workspace check` if a Cargo manifest, lockfile, or `deny.toml` changed.
- CI runs the deploy dry-run.

## Cloudflare tooling

- Use `cf` for direct Cloudflare CLI commands. Prefer existing package scripts for project workflows.
- For new Cloudflare configuration, prefer `cloudflare.config.ts`. Preserve existing configuration formats during unrelated work.

## Fuzzing

- The `fuzz/` crate holds cargo-fuzz targets. It is not part of the main workspace, so stable Clippy, tests, and cargo-deny skip it.
- `multipart_round_trip` decodes the Worker's multipart response and checks that the payload and attachment come back unchanged.
- `verify_signature` checks that the Worker accepts genuine Discord signatures and rejects malformed, truncated, or tampered ones.
- `register_application_id` parses `.dev.vars` and the registration utility's application ID.
- Fuzzing needs `nightly-2026-11-01` and cargo-fuzz 0.13.2. Do not run it from Lefthook.
- Run the commands from `fuzz/`:

```bash
cargo +nightly-2026-11-01 fuzz list
mkdir -p corpus/<target>
cargo +nightly-2026-11-01 fuzz run <target> corpus/<target> seeds/<target> -- -max_total_time=300
cargo +nightly-2026-11-01 fuzz run <target> artifacts/<target>/<crash-file>
cargo +nightly-2026-11-01 fuzz tmin <target> artifacts/<target>/<crash-file>
```

- Keep `seeds/` small. The `corpus/` and `artifacts/` directories are ignored by Git.
- The Fuzz workflow runs each target for 60 seconds on related pull requests and pushes, and for 10 minutes every week.
