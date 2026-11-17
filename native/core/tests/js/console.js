console.log("text", 1, 1.5, true, null, undefined, { a: [1, "b"] });
console.debug("d");
console.info("i");
console.warn("w");
console.error("e");
const cyclic = {};
cyclic.self = cyclic;
console.log(cyclic);
console.log(new Error("bad"));
