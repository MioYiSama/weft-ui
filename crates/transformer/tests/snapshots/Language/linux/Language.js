import * as $ from "@weft-ui/runtime/remote";

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

export function ModeLabel$c(app, args, children, depth) {
  const c = new $.Instance(app, 0, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  c.set(0, ((c.p[0] === "compact") ? "C" : "F"));
  c.u = () => {
    if (c.d[0] & 1) c.set(0, ((c.p[0] === "compact") ? "C" : "F"));
  };
  return c;
}

export class ModeLabel extends $.App {
  constructor(p0 = "full") { super(ModeLabel$c, [p0], "0491f0fb29968dc8"); }
}

export function Rating$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  c.set(0, ("Rating: " + $.str(c.p[0])));
  const $h2_onClick = 1;
  c.on(0, () => { c.p[1]?.($h2_onClick); }, -1);
  c.on(1, () => { c.p[1]?.(2); }, -1);
  let $h6_onClick = c.p[0];
  c.on(2, () => { c.p[1]?.($h6_onClick); }, -1);
  c.u = () => {
    if (c.d[0] & 1) c.set(0, ("Rating: " + $.str(c.p[0])));
    if (c.d[0] & 1) $h6_onClick = c.p[0];
  };
  return c;
}

export class Rating extends $.App {
  constructor(p0) { super(Rating$c, [p0, null], "0491f0fb29968dc8"); }
}

export function ForwardRating$c(app, args, children, depth) {
  const c = new $.Instance(app, 2, 1, depth, args, children);
  const k0 = Rating$c(c.app, [0, ($a) => { c.p[0]?.($a); }], null, c.depth + 1);
  c.attach(0, k0);
  c.x = () => {
    k0.destroy();
  };
  return c;
}

export class ForwardRating extends $.App {
  constructor() { super(ForwardRating$c, [null], "0491f0fb29968dc8"); }
  get model() { return undefined; }
}

export function Language$c(app, args, children, depth) {
  const c = new $.Instance(app, 3, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  c.s[0] = 0;
  c.sw[0] = $.watch($s2, c, c.s[0]);
  c.set(0, ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")"));
  const k1 = ModeLabel$c(c.app, [c.p[0].$v[2]], null, c.depth + 1);
  c.attach(0, k1);
  const k2 = Rating$c(c.app, [c.p[0].$v[3], ($a) => { c.app.$emit("Rated", { value: $a }); }], null, c.depth + 1);
  c.attach(1, k2);
  const k3 = Rating$c(c.app, [c.s[0], ($a) => { c.state(0, $a); c.app.$emit("Rated", { value: c.s[0] }); }], null, c.depth + 1);
  c.attach(2, k3);
  const k4 = ForwardRating$c(c.app, [(($b0) => { c.app.$emit("Rated", { value: $b0 }); }).bind(null, c.p[0].$v[3])], null, c.depth + 1);
  c.attach(3, k4);
  const k5 = Rating$c(c.app, [0, null], null, c.depth + 1);
  c.attach(4, k5);
  const l10 = new $.List(c, 5, (item) => {
    const r0 = new $.Row(c.app, 4, 1, c.depth + 1, $s3, item);
    r0.set(0, r0.item.$v[2].toUpperCase());
    const i9 = new $.Branches(r0, 0, (i) => {
      switch (i) {
        case 0: {
          const b = new $.Block(c.app, 5);
          return b;
        }
        case 1: {
          const b = new $.Block(c.app, 6);
          return b;
        }
        default:
          return null;
      }
    });
    i9.set((r0.item.$v[1] === "image") ? 0 : 1);
    r0.u = () => {
      if (r0.d[0] & 4) r0.set(0, r0.item.$v[2].toUpperCase());
      if (r0.d[0] & 2) i9.set((r0.item.$v[1] === "image") ? 0 : 1);
    };
    r0.x = () => {
      i9.destroy();
    };
    return r0;
  }, ($it) => $it.$v[0]);
  l10.set(c.p[0].$v[1]);
  c.u = () => {
    if (c.d[0] & 3) c.set(0, ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")"));
    if (c.d[0] & 4) k1.arg(0, c.p[0].$v[2]);
    if (c.d[0] & 8) k2.arg(0, c.p[0].$v[3]);
    if (c.d[0] & 16) k3.arg(0, c.s[0]);
    if (c.d[0] & 8) k4.arg(0, (($b0) => { c.app.$emit("Rated", { value: $b0 }); }).bind(null, c.p[0].$v[3]));
    if (c.d[0] & 2) l10.set(c.p[0].$v[1]);
  };
  c.x = () => {
    k1.destroy();
    k2.destroy();
    k3.destroy();
    k4.destroy();
    k5.destroy();
    l10.destroy();
  };
  return c;
}

export class Language extends $.App {
  constructor(p0) { super(Language$c, [LanguageModel.from(p0)], "0491f0fb29968dc8"); }
  onRated(h) { return this.$on("Rated", h); }
}
