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
  constructor(v) { super([v.name, $c0(v.items), v.mode]); }
  get name() { return this.$v[0]; }
  set name(v) { this.$set(0, v); }
  get items() { return this.$v[1]; }
  set items(v) { if (v !== this.$v[1]) this.$set(1, $c0(v)); }
  get mode() { return this.$v[2]; }
  set mode(v) { this.$set(2, v); }
  toJSON() { return { name: this.$v[0], items: this.$v[1], mode: this.$v[2] }; }
}

const $c0 = $.arrayOf(LanguageItem.from);
const $s0 = 0;
const $s1 = [0, 1, 2];

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
  constructor(p0 = "full") { super(ModeLabel$c, [p0], "518bf39fdd8340b7"); }
}

export function Language$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  c.set(0, ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")"));
  const k1 = ModeLabel$c(c.app, [c.p[0].$v[2]], null, c.depth + 1);
  c.attach(0, k1);
  const l6 = new $.List(c, 1, (item) => {
    const r0 = new $.Row(c.app, 2, 1, c.depth + 1, $s1, item);
    r0.set(0, r0.item.$v[2].toUpperCase());
    const i5 = new $.Branches(r0, 0, (i) => {
      switch (i) {
        case 0: {
          const b = new $.Block(c.app, 3);
          return b;
        }
        case 1: {
          const b = new $.Block(c.app, 4);
          return b;
        }
        default:
          return null;
      }
    });
    i5.set((r0.item.$v[1] === "image") ? 0 : 1);
    r0.u = () => {
      if (r0.d[0] & 4) r0.set(0, r0.item.$v[2].toUpperCase());
      if (r0.d[0] & 2) i5.set((r0.item.$v[1] === "image") ? 0 : 1);
    };
    r0.x = () => {
      i5.destroy();
    };
    return r0;
  }, ($it) => $it.$v[0]);
  l6.set(c.p[0].$v[1]);
  c.u = () => {
    if (c.d[0] & 3) c.set(0, ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")"));
    if (c.d[0] & 4) k1.arg(0, c.p[0].$v[2]);
    if (c.d[0] & 2) l6.set(c.p[0].$v[1]);
  };
  c.x = () => {
    k1.destroy();
    l6.destroy();
  };
  return c;
}

export class Language extends $.App {
  constructor(p0) { super(Language$c, [LanguageModel.from(p0)], "518bf39fdd8340b7"); }
}
