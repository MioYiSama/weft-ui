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
const $t0 = $.template("<span></span>");
const $t1 = $.template("<div style=\"display:flex;flex-direction:column;align-items:flex-start\"><span></span><!----><!----></div>");
const $t2 = $.template("<span></span><!---->");
const $t3 = $.template("<span>image</span>");
const $t4 = $.template("<span>text</span>");

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

export function Language$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s1, c, args[0]);
  const f = $t1();
  const n7 = f.firstChild;
  const n0 = n7.firstChild;
  const n1 = n0.nextSibling;
  const n6 = n1.nextSibling;
  n0.textContent = ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")");
  const k1 = ModeLabel$c(c.app, [c.p[0].$v[2]], null, c.depth + 1);
  k1.mount(n1.parentNode, n1);
  const l6 = new $.List(n6, (item) => {
    const r0 = new $.Row(1, c.depth + 1, $s1, item);
    const f = $t2();
    const n2 = f.firstChild;
    const n5 = n2.nextSibling;
    n2.textContent = r0.item.$v[2].toUpperCase();
    const i5 = new $.Branches(n5, (i) => {
      switch (i) {
        case 0: {
          const b = new $.Block();
          const f = $t3();
          const n3 = f.firstChild;
          b.init(f);
          return b;
        }
        case 1: {
          const b = new $.Block();
          const f = $t4();
          const n4 = f.firstChild;
          b.init(f);
          return b;
        }
        default:
          return null;
      }
    });
    i5.set((r0.item.$v[1] === "image") ? 0 : 1);
    r0.init(f);
    r0.u = () => {
      if (r0.d[0] & 4) n2.textContent = r0.item.$v[2].toUpperCase();
      if (r0.d[0] & 2) i5.set((r0.item.$v[1] === "image") ? 0 : 1);
    };
    r0.x = () => {
      i5.destroy();
    };
    return r0;
  }, ($it) => $it.$v[0]);
  l6.set(c.p[0].$v[1]);
  c.init(f);
  c.u = () => {
    if (c.d[0] & 3) n0.textContent = ($.str(c.p[0].$v[0]) + " (" + $.str(c.p[0].$v[1].length) + ")");
    if (c.d[0] & 4) k1.arg(0, c.p[0].$v[2]);
    if (c.d[0] & 2) l6.set(c.p[0].$v[1]);
  };
  c.x = () => {
    k1.destroy(false);
    l6.destroy();
  };
  return c;
}

export class Language extends $.App {
  constructor(p0) { super(Language$c, [LanguageModel.from(p0)]); }
}
