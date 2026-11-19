import { cpSync, rmSync, statSync } from "node:fs";

const xcframework = "build/WeftCore.xcframework";
const kernels = {
  apple: [
    `${xcframework}/Info.plist`,
    ...["macos-arm64_x86_64", "ios-arm64", "ios-arm64_x86_64-simulator"].map(slice => `${xcframework}/${slice}/libWeftCore.a`),
  ],
  android: ["arm64-v8a", "armeabi-v7a", "x86_64", "x86"].map(abi => `build/jniLibs/${abi}/libweft.so`),
  windows: ["build/win-x64/weft_core.dll", "build/win-arm64/weft_core.dll"],
  linux: ["build/linux-x64/libweft_core.so", "build/linux-arm64/libweft_core.so", "build/linux-arm64/bin/hermesc"],
};

const [phase, target] = process.argv.slice(2);
if (!["prepack", "postpack", "prepublishOnly"].includes(phase) || !Object.hasOwn(kernels, target)) {
  console.error("usage: node native/pack.mjs <prepack|postpack|prepublishOnly> <apple|android|windows|linux>");
  process.exit(1);
}

const shell = new URL(`${target}/`, import.meta.url);
const license = new URL("LICENSE-hermes", shell);
const coreCopy = new URL("core/", shell);
switch (phase) {
  case "prepack":
    cpSync(new URL("core/LICENSE-hermes", import.meta.url), license);
    if (target === "linux") {
      cpSync(new URL("core/include/", import.meta.url), new URL("include/", coreCopy), { recursive: true });
      cpSync(new URL("core/src/ops.cpp", import.meta.url), new URL("src/ops.cpp", coreCopy));
    }
    break;
  case "postpack":
    rmSync(license, { force: true });
    if (target === "linux") rmSync(coreCopy, { recursive: true, force: true });
    break;
  case "prepublishOnly": {
    const missing = kernels[target].filter(path => !statSync(new URL(path, shell), { throwIfNoEntry: false })?.isFile());
    if (missing.length) {
      console.error(`weft: @weft-ui/native-${target} cannot be published; missing prebuilt files:\n${missing.map(path => `  ${path}`).join("\n")}`);
      process.exitCode = 1;
    }
    break;
  }
}
