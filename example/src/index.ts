import { App } from "./App.weft";
// 将平台API都抽象出来
import { run, fetch } from "@weft-ui/platform";

const app = new App({
  clicked: false,
});

app.onMount(async (model, event) => {
  await fetch("https://example.com");
});

app.onClick((model, event) => {
  model.clicked = true;
});

run(app);
