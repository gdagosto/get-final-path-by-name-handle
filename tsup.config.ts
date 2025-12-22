// tsup.config.ts
import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["esm"], // Only ESM (Node.js 12+ supports it)
  dts: true,
  sourcemap: true,
  clean: true,
  minify: true,
  treeshake: true,
  platform: "node", // Explicitly target Node.js
  target: "node18", // Your minimum Node.js version
});
