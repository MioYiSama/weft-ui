import type { Script } from "../../conformance/harness.ts";
import { Inputs } from "./Inputs.weft";

export const script: Script<Inputs> = {
  dir: import.meta.dirname,
  entry: "Inputs.weft",
  component: "Inputs",
  App: Inputs,
  model: () => ({ kind: "text", noteLimit: 5 }),
  setup(app, log) {
    app.onDraftEdited((_, { value }) => log(`draft ${value}`));
    app.onNoteEdited((_, { value }) => log(`note ${value}`));
    app.onFocusChanged((_, { value }) => log(`focus ${value}`));
    app.onSubmitted(() => log("submit"));
  },
  steps: [
    { name: "request focus", run: (ui) => ui.click("Button", 0) },
    { name: "limit a seven-character edit", run: (ui) => ui.edit("Input", 0, "abcdefg") },
    { name: "submit and clear the draft", run: (ui) => ui.click("Input", 0, "onSubmit") },
    { name: "release focus", run: (ui) => ui.click("Button", 0) },
    { name: "focus the note", run: (ui) => ui.focus("TextArea", 0, true) },
    { name: "count an emoji as two UTF-16 units", run: (ui) => ui.edit("TextArea", 0, "a😀bcde") },
    { name: "avoid splitting a surrogate pair", run: (ui) => ui.edit("TextArea", 0, "abcd😀e") },
    { name: "blur the note", run: (ui) => ui.focus("TextArea", 0, false) },
    { name: "switch to a numeric keyboard", run: (_, model) => { model.kind = "number"; } },
    { name: "switch from numeric to password", run: (_, model) => { model.kind = "password"; } },
  ],
};
