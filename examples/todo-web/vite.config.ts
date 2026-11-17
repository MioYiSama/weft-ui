import { defineConfig } from "vitest/config";
import weft from "@weft-ui/vite-plugin";

export default defineConfig({
  plugins: [weft()],
  test: {
    environment: "happy-dom",
  },
});
