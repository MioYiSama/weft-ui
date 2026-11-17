import { defineConfig } from "vitest/config";
import weft from "./src/index.ts";

export default defineConfig({
  plugins: [weft()],
  test: {
    environment: "happy-dom",
  },
});
