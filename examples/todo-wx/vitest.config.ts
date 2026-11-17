import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // Builds miniprogram/weft before the tests import it.
    globalSetup: "./test/setup.ts",
  },
});
