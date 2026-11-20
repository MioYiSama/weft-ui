import type { Script } from "../../conformance/harness.ts";
import { Language } from "./Language.weft";

export const script: Script<Language> = {
  dir: import.meta.dirname,
  entry: "Language.weft",
  component: "Language",
  App: Language,
  model: () => ({ name: "Gallery", items: [{ id: 1, kind: "text", name: "Oak" }], mode: "full", rating: 1 }),
  setup(app, log) {
    app.onRated((model, { value }) => {
      log(`Rated ${value}`);
      model.name = `Rated ${value}`;
    });
  },
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
    { name: "emit a rating through a callback", run: (ui) => ui.click("Button", 0) },
    { name: "set local rating and then emit it", run: (ui) => ui.click("Button", 4) },
    { name: "change model rating", run: (_, model) => { model.rating = 4; } },
    { name: "emit the newly bound element value", run: (ui) => ui.click("Button", 2) },
    { name: "replace the bound callback", run: (_, model) => { model.rating = 5; } },
    { name: "forward the newly bound callback", run: (ui) => ui.click("Button", 6) },
    { name: "call an omitted callback", run: (ui) => ui.click("Button", 9) },
  ],
};
