// Timer order: deadline, then arming order; cleared timers never run.
const t = (s) => console.log(s);
setTimeout(() => t("b 20"), 20);
setTimeout(() => t("a 0"), 0);
setTimeout((x, y) => t(`args ${x} ${y}`), 0, 1, "two");
clearTimeout(setTimeout(() => t("cancelled"), 5));
let z = 0;
const zero = setInterval(() => {
  t(`zero ${++z}`);
  if (z === 3) clearInterval(zero);
}, 0);
let n = 0;
const interval = setInterval(() => {
  t(`interval ${++n}`);
  if (n === 3) clearInterval(interval);
}, 1);
setTimeout(() => {
  t("outer");
  setTimeout(() => t("nested"), 0);
}, 10);
setTimeout(() => t("done"), 40);
