const decoder = new TextDecoder();
console.log(`supports ${__weft.supports("echo")} ${__weft.supports("nope")} version ${__weft.version}`);
__weft.call("echo", "upper", "hi").then((bytes) => console.log(`resolved ${decoder.decode(bytes)}`));
__weft.call("echo", "fail", new Uint8Array([1, 2, 3]).subarray(1)).catch((e) => console.log(`rejected ${e.message}`));
__weft.call("echo", "empty").then((bytes) => console.log(`empty ${bytes.byteLength}`));
__weft.call("nope", "x").catch((e) => console.log(`unsupported ${e.message}`));
