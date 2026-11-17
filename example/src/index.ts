import { App } from "./App.weft";
// Platform APIs share one signature everywhere; unsupported ones reject with PlatformError.
import { run, storage } from "@weft-ui/platform";

const app = new App({
  clicked: false,
});

app.onMount(async (model) => {
  model.clicked = (await storage.get("clicked")) === "true";
});

app.onClick((model) => {
  model.clicked = true;
  void storage.set("clicked", "true");
});

run(app);
