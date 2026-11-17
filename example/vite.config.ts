import { defineConfig } from "vite";
import weft from "@weft-ui/vite";

// 充分利用Vite Environment API，用环境变量指定平台
export default defineConfig({
  plugins: [
    weft({
      platforms: ["android", "ios", "linux", "macos", "web", "wechat", "windows"],
    }),
  ],
});
