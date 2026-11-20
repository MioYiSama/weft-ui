import * as $ from "@weft-ui/runtime";
$.theme({
  ["accent"]: ["#0a84ffff", "#409cffff"],
});

export class Model extends $.Model {
  static from(v) { return v instanceof Model ? v : new Model(v); }
  constructor(v) { super([v.flag]); }
  get flag() { return this.$v[0]; }
  set flag(v) { this.$set(0, v); }
  toJSON() { return { flag: this.$v[0] }; }
}

const $s0 = 0;
const $s1 = [0];
const $t0 = $.template("<span>badge</span>");
const $t1 = $.template("<div style=\"display:flex;flex-direction:column;align-items:flex-start;gap:8px\"><span style=\"color:var(--weft-accent)\">accent</span><span style=\"color:var(--weft-accent)\">styled</span><span>flag</span><span></span><!----></div>");

export function Badge$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  const f = $t0();
  const n0 = f.firstChild;
  n0.style.setProperty("color", $.color(c.p[0]));
  c.init(f);
  c.u = () => {
    if (c.d[0] & 1) n0.style.setProperty("color", $.color(c.p[0]));
  };
  return c;
}

export class Badge extends $.App {
  constructor(p0) { super(Badge$c, [p0]); }
}

export function Theme$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  const f = $t1();
  const n5 = f.firstChild;
  const n0 = n5.firstChild;
  const n1 = n0.nextSibling;
  const n2 = n1.nextSibling;
  const n3 = n2.nextSibling;
  const n4 = n3.nextSibling;
  n2.style.setProperty("color", $.color((c.p[0].$v[0] ? "@accent" : "#000000")));
  n3.textContent = $.str((-1.25));
  const k4 = Badge$c(c.app, ["@accent"], [], c.depth + 1);
  k4.mount(n4.parentNode, n4);
  c.init(f);
  c.u = () => {
    if (c.d[0] & 1) n2.style.setProperty("color", $.color((c.p[0].$v[0] ? "@accent" : "#000000")));
  };
  c.x = () => {
    k4.destroy(false);
  };
  return c;
}

export class Theme extends $.App {
  constructor(p0) { super(Theme$c, [Model.from(p0)]); }
}
