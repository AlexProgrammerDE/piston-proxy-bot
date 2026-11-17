import { bindings, defineConfig } from "cf/config";

import * as entrypoint from "./build/index.js" with { type: "cf-worker" };

export default defineConfig({
  worker: {
    name: "piston-proxy-bot",
    compatibilityDate: "2026-07-23",
    entrypoint,
    env: {
      DISCORD_APPLICATION_ID: bindings.secret(),
      DISCORD_PUBLIC_KEY: bindings.secret(),
      PROXY_API_URL: bindings.secret(),
    },
    observability: {
      issues: {
        enabled: true,
      },
      logs: {
        enabled: true,
        invocationLogs: true,
      },
    },
  },
});
