import { run } from "@weft-ui/platform";
import { Showcase } from "./App.weft";

function createApp() {
  return new Showcase();
}

run(createApp());
