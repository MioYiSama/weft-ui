import type { Script } from "../../conformance/harness.ts";
import { Controls } from "./Controls.weft";

export const script: Script<Controls> = {
  dir: import.meta.dirname,
  entry: "Controls.weft",
  component: "Controls",
  App: Controls,
  model: () => ({
    options: ["Local", "mïlk 🥛"],
    placeholder: "Choose a location",
    progress: 0.3,
    locked: false,
  }),
  setup(app, log) {
    app.onSwitchChanged((_, { value }) => log(`switch ${value}`));
    app.onSliderChanged((_, { value }) => log(`slider ${value}`));
    app.onPickerChanged((_, { value }) => log(`picker ${value}`));
    app.onNoteChanged((_, { value }) => log(`note ${value}`));
  },
  steps: [
    { name: "toggle the switch", run: (ui) => ui.edit("Switch", 0, true) },
    { name: "move the slider", run: (ui) => ui.edit("Slider", 0, 45) },
    { name: "choose the first option", run: (ui) => ui.edit("Picker", 0, 0) },
    { name: "edit multiple lines", run: (ui) => ui.edit("TextArea", 0, "First\nSecond") },
    { name: "advance progress and relabel the selection", run: (_, model) => {
      model.progress = 0.8;
      model.options = ["Remote <site>", "mïlk 🥛"];
    } },
    { name: "reject a selection without changing state", run: (ui, model) => {
      model.locked = true;
      ui.edit("Picker", 0, 1);
    } },
    { name: "replace the options with an empty list", run: (_, model) => {
      model.options = [];
      model.placeholder = "No locations";
    } },
  ],
};
