console.log(__weft.environment);
__weft.listen(() => console.log("replaced listener"));
__weft.listen(() => {
  console.log(__weft.environment);
  queueMicrotask(() => console.log("after change"));
  throw new Error("listener error");
});
