import { defineConfig } from "vite";
import weft from "@weft-ui/vite";

export default defineConfig({
  plugins: [weft({
    app: { id: "net.mioyi.weftui.showcase", name: "Weft Showcase" },
    // Follow only the imports of the generated catalog. A broad glob would
    // compile pages still pending on the selected platform.
    include: "src/App.weft",
  })],
});
