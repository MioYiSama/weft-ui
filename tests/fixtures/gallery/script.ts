import type { Script } from "../../conformance/harness.ts";
import { Gallery } from "./Gallery.weft";

export const script: Script<Gallery> = {
  dir: import.meta.dirname,
  entry: "Gallery.weft",
  component: "Gallery",
  App: Gallery,
  model: () => ({
    style: {
      padding: 4,
      gap: 6,
      width: 240,
      height: 60,
      background: "#ace",
      color: "#135",
      size: 14,
      bold: false,
    },
    checked: false,
    imageUrl: "https://example.test/gallery/day.png",
    labels: ["First", "Second"],
  }),
  setup(app, log) {
    app.onChecked((model, { value }) => {
      log(`Checked ${value}`);
      model.checked = value;
    });
  },
  steps: [
    { name: "check to show the slot list", run: (ui) => ui.edit("Checkbox", 0, true) },
    {
      name: "change the image URL",
      run: (_, model) => {
        model.imageUrl = "https://example.test/gallery/night.png";
      },
    },
    {
      name: "change all bound styles",
      run: (_, model) => {
        model.style = {
          padding: 8,
          gap: 10,
          width: 280,
          height: 80,
          background: "#246",
          color: "#fff",
          size: 18,
          bold: true,
        };
      },
    },
    { name: "edit the caption in the slot", run: (ui) => ui.edit("Input", 0, "New caption") },
    {
      name: "reorder the slot list",
      run: (_, model) => {
        model.labels = [...model.labels].reverse();
      },
    },
    { name: "click the panel button", run: (ui) => ui.click("Button", 0, "onClick") },
    { name: "uncheck to hide the slot list", run: (ui) => ui.edit("Checkbox", 0, false) },
  ],
};
