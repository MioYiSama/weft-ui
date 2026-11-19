import { defineConfig } from "vite";
import weft from "@weft-ui/vite";

// `WEFT_PLATFORM=<platform> vite build` builds one platform into apps/<platform>/.
export default defineConfig({
  plugins: [
    weft({
      app: { id: "net.mioyi.weftui", name: "Exp" },
    }),
  ],
});
