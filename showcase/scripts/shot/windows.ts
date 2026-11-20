import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { build, command, prepareShots } from "./cases.ts";

const { directory, shots } = prepareShots("windows");
if (shots.length === 0) process.exit(0);

if (process.platform === "win32") {
  build("windows");
  command("dotnet", ["test", "apps/windows/UITests", "--filter", "FullyQualifiedName~Screenshots", "-nologo"], {
    env: {
      ...process.env,
      WEFT_PAGES: shots.map(({ id, scheme }) => `${id}:${scheme}`).join(","),
      WEFT_SHOT_DIR: directory,
    },
  });
} else if (process.platform === "darwin") {
  command("/tmp/weft-win.sh", ["/tmp/win-sync.ps1"]);
  command("/tmp/weft-win.sh", [resolve("scripts/shot/windows.ps1")]);
} else {
  throw new Error("Run shot:windows on Windows or from macOS through /tmp/weft-win.sh");
}
for (const { path } of shots) {
  if (!existsSync(path)) throw new Error(`Missing screenshot: ${path}`);
}
