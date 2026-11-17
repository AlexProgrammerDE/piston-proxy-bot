import { cloudflare } from "@cloudflare/vite-plugin";
import { defineConfig } from "vite";

// Bundles the Rust Worker that scripts/build-worker.sh writes to build/.
export default defineConfig({ plugins: [cloudflare()] });
