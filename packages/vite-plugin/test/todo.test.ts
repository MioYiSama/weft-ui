import { tick } from "@weft-ui/runtime";
import { expect, test } from "vitest";
import { TodoList } from "./fixtures/TodoList.weft";

function mount(options: { handlers?: boolean } = {}) {
  const target = document.createElement("div");
  document.body.append(target);
  const app = new TodoList({
    draft: "",
    todos: [
      { id: 1, name: "a" },
      { id: 2, name: "b" },
      { id: 3, name: "c" },
    ],
  });
  if (options.handlers !== false) {
    app.onTextChanged((model, e) => {
      model.draft = e.value;
    });
    app.onRemove((model, e) => {
      model.todos = model.todos.filter((t) => t.id !== e.id);
    });
  }
  app.mount(target);
  const input = target.querySelector("input")!;
  const toggle = target.querySelector("button")!;
  const column = target.firstElementChild!;
  const rows = () => [...column.querySelectorAll<HTMLElement>(":scope > div")];
  const names = () => rows().map((row) => row.querySelector("span")!.textContent);
  return { app, target, input, toggle, rows, names };
}

/** DOM mutations caused by `action` and the flush it triggers. */
async function mutations(target: Node, action: () => void): Promise<MutationRecord[]> {
  const records: MutationRecord[] = [];
  const observer = new MutationObserver((batch) => records.push(...batch));
  observer.observe(target, { subtree: true, childList: true, characterData: true, attributes: true });
  action();
  await tick();
  // Observers are notified asynchronously; let pending deliveries run.
  const { promise, resolve } = Promise.withResolvers<void>();
  setTimeout(resolve);
  await promise;
  records.push(...observer.takeRecords());
  observer.disconnect();
  return records;
}

function typeInto(input: HTMLInputElement, value: string) {
  input.value = value;
  input.dispatchEvent(new Event("input"));
}

test("toggling state creates and destroys the list", async () => {
  const { toggle, rows, names } = mount();
  expect(toggle.textContent).toBe("expand");
  expect(rows()).toHaveLength(0);

  toggle.click();
  await tick();
  expect(toggle.textContent).toBe("collapse");
  expect(names()).toEqual(["a", "b", "c"]);

  toggle.click();
  await tick();
  expect(toggle.textContent).toBe("expand");
  expect(rows()).toHaveLength(0);
});

test("typing only touches the input", async () => {
  const { app, target, input, toggle } = mount();
  toggle.click();
  await tick();

  const records = await mutations(target, () => typeInto(input, "hello"));
  expect(app.model.draft).toBe("hello");
  expect(input.value).toBe("hello");
  expect(records).toEqual([]);
});

test("the input shows the model, not unaccepted edits", async () => {
  const { app, input } = mount({ handlers: false });
  typeInto(input, "rejected");
  await tick();
  expect(input.value).toBe("");

  app.model.draft = "from host";
  await tick();
  expect(input.value).toBe("from host");
});

test("removing an item removes exactly its row", async () => {
  const { target, toggle, rows, names } = mount();
  toggle.click();
  await tick();
  const [first, second, third] = rows();

  const records = await mutations(target, () => second.querySelector("button")!.click());
  expect(names()).toEqual(["a", "c"]);
  expect(rows()).toEqual([first, third]);
  expect(records).toHaveLength(1);
  expect([...records[0].removedNodes]).toEqual([second]);
  expect(records[0].addedNodes).toHaveLength(0);
});

test("changing one item field updates only its text", async () => {
  const { app, target, toggle, rows, names } = mount();
  toggle.click();
  await tick();
  const third = rows()[2].querySelector("span")!;

  const records = await mutations(target, () => {
    app.model.todos[2].name = "z";
  });
  expect(names()).toEqual(["a", "b", "z"]);
  expect(records.length).toBeGreaterThan(0);
  expect(records.every((r) => r.target === third || r.target.parentNode === third)).toBe(true);
});

test("reordering moves existing rows by key", async () => {
  const { app, target, toggle, rows, names } = mount();
  toggle.click();
  await tick();
  const before = rows();

  const records = await mutations(target, () => {
    app.model.todos = [...app.model.todos].reverse();
  });
  expect(names()).toEqual(["c", "b", "a"]);
  expect(rows()).toEqual([...before].reverse());
  // Moves only: no text was rewritten and no row was created.
  expect(records.every((r) => r.type === "childList" && r.target === before[0].parentNode)).toBe(true);
  const added = records.flatMap((r) => [...r.addedNodes]);
  expect(added.every((node) => before.includes(node as HTMLElement))).toBe(true);
  expect(added.length).toBeLessThanOrEqual(2);
});

test("replacing an item with an equal-keyed one only rewrites changed fields", async () => {
  const { app, target, toggle, rows } = mount();
  toggle.click();
  await tick();
  const before = rows();

  const records = await mutations(target, () => {
    app.model.todos = app.model.todos.map((t) => (t.id === 1 ? { id: 1, name: "a2" } : t));
  });
  expect(rows()).toEqual(before);
  const first = before[0].querySelector("span")!;
  expect(first.textContent).toBe("a2");
  expect(records.every((r) => r.target === first)).toBe(true);
});

test("model arrays are frozen", () => {
  const { app } = mount();
  expect(() => (app.model.todos as unknown as unknown[]).push({ id: 4, name: "d" })).toThrow(TypeError);
});

test("unmount removes the view and stops tracking", async () => {
  const { app, target } = mount();
  app.unmount();
  expect(target.childNodes).toHaveLength(0);
  app.model.draft = "after";
  await tick();
  expect(target.childNodes).toHaveLength(0);
});
