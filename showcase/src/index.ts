import { run } from "@weft-ui/platform";
import manifest from "../pages.json" with { type: "json" };
import { Showcase } from "./App.weft";

// The catalog generator validates these keys and declares the matching model fields.
type Model = ConstructorParameters<typeof Showcase>[0];
const model = Object.fromEntries(manifest.flatMap(({ props }) => Object.entries(props ?? {}))) as Record<keyof Model, string[]>;
run(new Showcase(model));
