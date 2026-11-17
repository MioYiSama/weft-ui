// Microtasks drain after the bundle and after every timer; a throwing one is logged.
const t = (s) => console.log(s);
Promise.resolve().then(() => t("promise"));
queueMicrotask(() => t("micro"));
queueMicrotask(() => {
  throw new Error("boom");
});
queueMicrotask(() => t("after boom"));
t("sync");
setTimeout(() => {
  t("timer 1");
  queueMicrotask(() => t("micro of timer 1"));
}, 0);
setTimeout(() => t("timer 2"), 0);
