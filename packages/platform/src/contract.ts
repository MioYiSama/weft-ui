import type * as Contract from "./index.d.ts";
import type * as Native from "./native.ts";
import type * as Web from "./web.ts";
import type * as Wechat from "./wechat.ts";

// Compare whole modules so new public exports are checked on every platform.
type Implements<Implementation extends typeof Contract> = Implementation;

type PlatformImplementations = [
  Implements<typeof Web>,
  Implements<typeof Wechat>,
  Implements<typeof Native>,
];
