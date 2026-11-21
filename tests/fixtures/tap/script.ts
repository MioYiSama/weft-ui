import type { Script } from "../../conformance/harness.ts";
import { Tap } from "./Tap.weft";

export const script: Script<Tap> = {
  dir: import.meta.dirname,
  entry: "Tap.weft",
  component: "Tap",
  App: Tap,
  model: () => ({ image: "tap.png" }),
  setup(app, log) {
    app.onActivated((_, { target }) => log(target));
  },
  steps: [
    { name: "tap the row", run: (ui) => ui.click("Row", 0, "onTap") },
    { name: "tap only the nested button", run: (ui) => ui.click("Button", 0, "onClick") },
    { name: "tap only the nested text", run: (ui) => ui.click("Text", 2, "onTap") },
    { name: "tap the image", run: (ui) => ui.click("Image", 0, "onTap") },
  ],
};
