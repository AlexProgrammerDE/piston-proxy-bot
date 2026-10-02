import { defineConfig } from "cf/config";

export default defineConfig({
	worker: {
		name: "piston-proxy-bot",
		compatibilityDate: "2026-07-23",
		entrypoint: "build/worker/shim.mjs",
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
