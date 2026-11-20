import type { Script } from "../../conformance/harness.ts";
import { Language } from "./Language.weft";

export const script: Script<Language> = {
  dir: import.meta.dirname,
  entry: "Language.weft",
  component: "Language",
  App: Language,
  model: () => ({ items: [{ id: 1, kind: "text" }], mode: "full" }),
  setup() {},
  steps: [
    {
      name: "switch item kind to image",
      run: (_, model) => {
        model.items[0]!.kind = "image";
      },
    },
    {
      name: "switch mode to compact",
      run: (_, model) => {
        model.mode = "compact";
      },
    },
  ],
};
