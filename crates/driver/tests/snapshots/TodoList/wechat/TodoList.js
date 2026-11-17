import * as $ from "./weft-runtime/wx.js";

export class Todo extends $.Model {
  static from(v) { return v instanceof Todo ? v : new Todo(v); }
  constructor(v) { super([v.id, v.name]); }
  get id() { return this.$v[0]; }
  set id(v) { this.$set(0, v); }
  get name() { return this.$v[1]; }
  set name(v) { this.$set(1, v); }
  toJSON() { return { id: this.$v[0], name: this.$v[1] }; }
}

export class TodoListModel extends $.Model {
  static from(v) { return v instanceof TodoListModel ? v : new TodoListModel(v); }
  constructor(v) { super([$c0(v.todos), v.draft]); }
  get todos() { return this.$v[0]; }
  set todos(v) { if (v !== this.$v[0]) this.$set(0, $c0(v)); }
  get draft() { return this.$v[1]; }
  set draft(v) { this.$set(1, v); }
  toJSON() { return { todos: this.$v[0], draft: this.$v[1] }; }
}

const $c0 = $.arrayOf(Todo.from);
const $s0 = [0, 1];
const $s1 = 2;

export function TodoList$c(app, args, children, depth) {
  const c = new $.Instance(app, 1, depth, args, children);
  c.pw[0] = $.watch($s0, c, args[0]);
  c.s[0] = false;
  c.sw[0] = $.watch($s1, c, c.s[0]);
  c.o.n0_0 = c.p[0].$v[1];
  c.on("e0", ($a) => { c.app.$emit("TextChanged", { value: $a }); }, "n0_0");
  c.on("e2", () => { c.state(0, (!c.s[0])); }, null);
  c.o.n1_0 = (c.s[0] ? "collapse" : "expand");
  const i8 = new $.Branches(c, "i8", "b8", (i) => {
    switch (i) {
      case 0: {
        const b = new $.Block();
        const l7 = new $.List(b, "l7", (item) => {
          const r0 = new $.Row(1, c.depth + 1, $s0, item);
          r0.o.n3_0 = r0.item.$v[1];
          r0.on("e5", () => { c.app.$emit("Remove", { id: r0.item.$v[0] }); }, null);
          r0.u = () => {
            if (r0.d[0] & 2) r0.set("n3_0", r0.item.$v[1]);
          };
          return r0;
        }, ($it) => $it.$v[0]);
        l7.set(c.p[0].$v[0]);
        b.u = () => {
          if (c.d[0] & 1) l7.set(c.p[0].$v[0]);
        };
        b.x = () => {
          l7.destroy();
        };
        return b;
      }
      default:
        return null;
    }
  });
  i8.set(c.s[0] ? 0 : -1);
  c.u = () => {
    if (c.d[0] & 2) c.set("n0_0", c.p[0].$v[1]);
    if (c.d[0] & 4) c.set("n1_0", (c.s[0] ? "collapse" : "expand"));
    if (c.d[0] & 4) i8.set(c.s[0] ? 0 : -1);
    if (c.d[0] & 1) i8.update();
  };
  c.x = () => {
    i8.destroy();
  };
  return c;
}

export class TodoList extends $.App {
  constructor(p0) { super(TodoList$c, [TodoListModel.from(p0)], "TodoList"); }
  onTextChanged(h) { return this.$on("TextChanged", h); }
  onRemove(h) { return this.$on("Remove", h); }
}
