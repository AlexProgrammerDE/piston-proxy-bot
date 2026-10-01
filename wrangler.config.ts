import { defineWranglerConfig } from "wrangler/experimental-config";

export default defineWranglerConfig({
	build: {
		command: "./scripts/build-worker.sh",
	},
	types: {
		generate: false,
	},
});
