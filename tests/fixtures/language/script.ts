import type { Script } from "../../conformance/harness.ts";
import { Language } from "./Language.weft";

export const script: Script<Language> = {
  dir: import.meta.dirname,
  entry: "Language.weft",
  component: "Language",
  App: Language,
  model: () => ({ name: "Gallery", items: [{ id: 1, kind: "text", name: "Oak" }], mode: "full" }),
  setup() {},
  steps: [
    {
      name: "rename the gallery",
      run: (_, model) => {
        model.name = "Library";
      },
    },
    {
      name: "rename one item",
      run: (_, model) => {
        model.items[0]!.name = "Elm";
      },
    },
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
