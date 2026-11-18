import { defineConfig } from "vitest/config";
import weft from "@weft-ui/vite";

// The conformance Web driver imports `.weft` fixtures through the plugin;
// the other drivers and suites build their fixtures at test time. Keep tests
// in Node: the Web driver creates its own happy-dom window without replacing
// the native `URL` used for `file:` paths.
export default defineConfig({
  plugins: [weft()],
});
