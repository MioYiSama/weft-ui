import * as $ from "@weft-ui/runtime";

export class LanguageItem extends $.Model {
  static from(v) { return v instanceof LanguageItem ? v : new LanguageItem(v); }
  constructor(v) { super([v.id, v.kind, v.name]); }
  get id() { return this.$v[0]; }
  set id(v) { this.$set(0, v); }
  get kind() { return this.$v[1]; }
  set kind(v) { this.$set(1, v); }
  get name() { return this.$v[2]; }
  set name(v) { this.$set(2, v); }
  toJSON() { return { id: this.$v[0], kind: this.$v[1], name: this.$v[2] }; }
}

export class LanguageModel extends $.Model {
  static from(v) { return v instanceof LanguageModel ? v : new LanguageModel(v); }
  constructor(v) { super([v.name, $c0(v.items), v.mode, v.rating]); }
  get name() { return this.$v[0]; }
  set name(v) { this.$set(0, v); }
  get items() { return this.$v[1]; }
  set items(v) { if (v !== this.$v[1]) this.$set(1, $c0(v)); }
  get mode() { return this.$v[2]; }
  set mode(v) { this.$set(2, v); }
  get rating() { return this.$v[3]; }
  set rating(v) { this.$set(3, v); }
  toJSON() { return { name: this.$v[0], items: this.$v[1], mode: this.$v[2], rating: this.$v[3] }; }
}

const $c0 = $.arrayOf(LanguageItem.from);
const $s0 = 0;
const $s1 = [0, 1, 2, 3];
const $s2 = 4;
const $s3 = [0, 1, 2];
const $t0 = $.template("<span></span>");
const $t1 = $.template("<div style=\"display:flex;flex-direction:row;align-items:flex-start\"><span></span><button type=\"button\"><span>1</span></button><button type=\"button\"><span>2</span></button><button type=\"button\"><span>Repeat</span></button></div>");
const $t2 = $.template("<!----><!---->");
const $t3 = $.template("<div style=\"display:flex;flex-direction:column;align-items:flex-start\"><span></span><!----><!----><!----><!----><!----><!----></div>");
const $t4 = $.template("<span></span><!---->");
const $t5 = $.template("<span>image</span>");
const $t6 = $.template("<span>text</span>");

export function ModeLabel$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  const f = $t0();
  const n0 = f.firstChild;
  n0.textContent = ((c.p[0] === "compact") ? "C" : "F");
  c.init(f);
  c.u = () => {
    if (c.d[0] & 1) n0.textContent = ((c.p[0] === "compact") ? "C" : "F");
  };
  return c;
}

export class ModeLabel extends $.App {
  constructor(p0 = "full") { super(ModeLabel$c, [p0]); }
}

export function Rating$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  const f = $t1();
  const n7 = f.firstChild;
  const n0 = n7.firstChild;
  const n2 = n0.nextSibling;
  const n1 = n2.firstChild;
  const n4 = n2.nextSibling;
  const n3 = n4.firstChild;
  const n6 = n4.nextSibling;
  const n5 = n6.firstChild;
  n0.textContent = ("Rating: " + $.str(c.p[0]));
  const $h2_onClick = 1;
  n2.addEventListener("click", () => { c.p[1]?.($h2_onClick); });
  n4.addEventListener("click", () => { c.p[1]?.(2); });
  let $h6_onClick = c.p[0];
  n6.addEventListener("click", () => { c.p[1]?.($h6_onClick); });
  c.init(f);
  c.u = () => {
    if (c.d[0] & 1) n0.textContent = ("Rating: " + $.str(c.p[0]));
    if (c.d[0] & 1) $h6_onClick = c.p[0];
  };
  return c;
}

export class Rating extends $.App {
  constructor(p0) { super(Rating$c, [p0, null]); }
}

export function ForwardRating$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  const f = $t2();
  const n0 = f.firstChild.nextSibling;
  const k0 = Rating$c(c.app, [0, ($a) => { c.p[0]?.($a); }], null, c.depth + 1);
  k0.mount(n0.parentNode, n0);
  c.init(f);
  c.x = () => {
    k0.destroy(false);
  };
  return c;
}

export class ForwardRating extends $.App {
  constructor() { super(ForwardRating$c, [null]); }
  get model() { return undefined; }
}

export function Language$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  c.s[0] = 0;
  c.sw[0] = $.watch($s2, c, c.s[0]);
  const f = $t3();
  const n11 = f.firstChild;
  const n0 = n11.firstChild;
  const n1 = n0.nextSibling;
  const n2 = n1.nextSibling;
  const n3 = n2.nextSibling;
  const n4 = n3.nextSibling;
  const n5 = n4.nextSibling;
  const n10 = n5.nextSibling;
  n0.textContent = ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")");
  const k1 = ModeLabel$c(c.app, [c.p[0].$v[2]], null, c.depth + 1);
  k1.mount(n1.parentNode, n1);
  const k2 = Rating$c(c.app, [c.p[0].$v[3], ($a) => { c.app.$emit("Rated", { value: $a }); }], null, c.depth + 1);
  k2.mount(n2.parentNode, n2);
  const k3 = Rating$c(c.app, [c.s[0], ($a) => { c.state(0, $a); c.app.$emit("Rated", { value: c.s[0] }); }], null, c.depth + 1);
  k3.mount(n3.parentNode, n3);
  const k4 = ForwardRating$c(c.app, [(($b0) => { c.app.$emit("Rated", { value: $b0 }); }).bind(null, c.p[0].$v[3])], null, c.depth + 1);
  k4.mount(n4.parentNode, n4);
  const k5 = Rating$c(c.app, [0, null], null, c.depth + 1);
  k5.mount(n5.parentNode, n5);
  const l10 = new $.List(n10, (item) => {
    const r0 = new $.Row(1, c.depth + 1, $s3, item);
    const f = $t4();
    const n6 = f.firstChild;
    const n9 = n6.nextSibling;
    n6.textContent = r0.item.$v[2].toUpperCase();
    const i9 = new $.Branches(n9, (i) => {
      switch (i) {
        case 0: {
          const b = new $.Block();
          const f = $t5();
          const n7 = f.firstChild;
          b.init(f);
          return b;
        }
        case 1: {
          const b = new $.Block();
          const f = $t6();
          const n8 = f.firstChild;
          b.init(f);
          return b;
        }
        default:
          return null;
      }
    });
    i9.set((r0.item.$v[1] === "image") ? 0 : 1);
    r0.init(f);
    r0.u = () => {
      if (r0.d[0] & 4) n6.textContent = r0.item.$v[2].toUpperCase();
      if (r0.d[0] & 2) i9.set((r0.item.$v[1] === "image") ? 0 : 1);
    };
    r0.x = () => {
      i9.destroy();
    };
    return r0;
  }, ($it) => $it.$v[0]);
  l10.set(c.p[0].$v[1]);
  c.init(f);
  c.u = () => {
    if (c.d[0] & 3) n0.textContent = ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")");
    if (c.d[0] & 4) k1.arg(0, c.p[0].$v[2]);
    if (c.d[0] & 8) k2.arg(0, c.p[0].$v[3]);
    if (c.d[0] & 16) k3.arg(0, c.s[0]);
    if (c.d[0] & 8) k4.arg(0, (($b0) => { c.app.$emit("Rated", { value: $b0 }); }).bind(null, c.p[0].$v[3]));
    if (c.d[0] & 2) l10.set(c.p[0].$v[1]);
  };
  c.x = () => {
    k1.destroy(false);
    k2.destroy(false);
    k3.destroy(false);
    k4.destroy(false);
    k5.destroy(false);
    l10.destroy();
  };
  return c;
}

export class Language extends $.App {
  constructor(p0) { super(Language$c, [LanguageModel.from(p0)]); }
  onRated(h) { return this.$on("Rated", h); }
}
