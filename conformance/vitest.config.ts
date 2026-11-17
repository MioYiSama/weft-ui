import { defineConfig } from "vitest/config";
import weft from "@weft-ui/vite";

// The Web side imports the fixtures as `.weft` modules; the mini program and
// headless sides build them at test time. Tests run in Node: the Web driver
// brings its own happy-dom window, whose `URL` would break `file:` URLs here.
export default defineConfig({
  plugins: [weft()],
});
