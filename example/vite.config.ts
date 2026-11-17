import { defineConfig } from "vite";
import weft from "@weft-ui/vite";

// `WEFT_PLATFORM=<platform> vite build` builds one platform into apps/<platform>/.
export default defineConfig({
  plugins: [
    weft({
      platforms: ["android", "ios", "linux", "macos", "web", "wechat", "windows"],
      app: { id: "com.example.exp", name: "Exp" },
    }),
  ],
});
