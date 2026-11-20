import * as $ from "@weft-ui/runtime/remote";

export class LanguageItem extends $.Model {
  static from(v) { return v instanceof LanguageItem ? v : new LanguageItem(v); }
  constructor(v) { super([v.id, v.kind]); }
  get id() { return this.$v[0]; }
  set id(v) { this.$set(0, v); }
  get kind() { return this.$v[1]; }
  set kind(v) { this.$set(1, v); }
  toJSON() { return { id: this.$v[0], kind: this.$v[1] }; }
}

export class LanguageModel extends $.Model {
  static from(v) { return v instanceof LanguageModel ? v : new LanguageModel(v); }
  constructor(v) { super([$c0(v.items), v.mode]); }
  get items() { return this.$v[0]; }
  set items(v) { if (v !== this.$v[0]) this.$set(0, $c0(v)); }
  get mode() { return this.$v[1]; }
  set mode(v) { this.$set(1, v); }
  toJSON() { return { items: this.$v[0], mode: this.$v[1] }; }
}

const $c0 = $.arrayOf(LanguageItem.from);
const $s0 = 0;
const $s1 = [0, 1];

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
  constructor(p0 = "full") { super(ModeLabel$c, [p0], "d7ad73a72b4b61c5"); }
}

export function Language$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  const k0 = ModeLabel$c(c.app, [c.p[0].$v[1]], null, c.depth + 1);
  c.attach(0, k0);
  const l4 = new $.List(c, 1, (item) => {
    const r0 = new $.Row(c.app, 2, 1, c.depth + 1, $s1, item);
    const i3 = new $.Branches(r0, 0, (i) => {
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
    i3.set((r0.item.$v[1] === "image") ? 0 : 1);
    r0.u = () => {
      if (r0.d[0] & 2) i3.set((r0.item.$v[1] === "image") ? 0 : 1);
    };
    r0.x = () => {
      i3.destroy();
    };
    return r0;
  }, ($it) => $it.$v[0]);
  l4.set(c.p[0].$v[0]);
  c.u = () => {
    if (c.d[0] & 2) k0.arg(0, c.p[0].$v[1]);
    if (c.d[0] & 1) l4.set(c.p[0].$v[0]);
  };
  c.x = () => {
    k0.destroy();
    l4.destroy();
  };
  return c;
}

export class Language extends $.App {
  constructor(p0) { super(Language$c, [LanguageModel.from(p0)], "d7ad73a72b4b61c5"); }
}
