import type { Script } from "../../conformance/harness.ts";
import { Theme } from "./Theme.weft";

export const script: Script<Theme> = {
  dir: import.meta.dirname,
  entry: "Theme.weft",
  component: "Theme",
  App: Theme,
  model: () => ({ flag: true }),
  setup() {},
  steps: [
    { name: "use a fixed color", run: (_, model) => { model.flag = false; } },
    { name: "use the accent token again", run: (_, model) => { model.flag = true; } },
  ],
};
