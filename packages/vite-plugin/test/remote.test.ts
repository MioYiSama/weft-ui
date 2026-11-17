import { hold, tick } from "@weft-ui/runtime";
import { connect } from "@weft-ui/runtime/remote";
import { afterEach, expect, test } from "vitest";
import { RemoteView } from "./fixtures/Remote.weft";

function snapshot() {
  return {
    user: { name: "ann" },
    draft: "bound",
    accepted: false,
    todos: [
      { id: 1, name: "first", owner: { name: "ann" }, watchers: [{ name: "bob" }] },
      { id: 2, name: "second", owner: { name: "bob" }, watchers: [] },
    ],
  };
}

const cleanups: (() => void)[] = [];
afterEach(async () => {
  for (const cleanup of cleanups.splice(0)) cleanup();
  await tick();
});

function mount() {
  const target = document.createElement("div");
  document.body.append(target);
  const messages: string[] = [];
  const remote = connect(RemoteView, ["TextChanged", "Toggled", "Select"], target, (message) => messages.push(message));
  remote.start({ args: [snapshot(), "from Rust"] });
  cleanups.push(() => {
    remote.apply({ patches: [], acks: messages.length });
    target.remove();
  });
  const input = target.querySelector<HTMLInputElement>("input:not([type=checkbox])")!;
  const checkbox = target.querySelector<HTMLInputElement>("input[type=checkbox]")!;
  const column = target.firstElementChild!;
  const rows = () => [...column.querySelectorAll<HTMLElement>(":scope > div")];
  return { target, messages, remote, input, checkbox, column, rows };
}

function typeInto(input: HTMLInputElement, text: string) {
  input.value = text;
  input.dispatchEvent(new Event("input"));
}

test("connect announces readiness, spreads start args and forwards events", async () => {
  const { messages, remote, column, input } = mount();
  expect(globalThis.__weft).toBe(remote);
  expect(messages).toEqual(['{"type":"ready"}']);
  expect(column.children[0].textContent).toBe("from Rust");
  expect(column.children[1].textContent).toBe("ann");
  expect(input.value).toBe("bound");

  typeInto(input, "typed");
  expect(JSON.parse(messages[1])).toEqual({ type: "event", name: "TextChanged", payload: { value: "typed" } });
  await tick();
  expect(input.value).toBe("typed");
});

test("event model payloads serialize nested models and arrays as plain fields", () => {
  const { messages, rows } = mount();
  rows()[0].querySelector("button")!.click();
  expect(JSON.parse(messages[1])).toEqual({
    type: "event",
    name: "Select",
    payload: { todo: snapshot().todos[0] },
  });
});

test("controlled text waits for a patch and acknowledgement without flickering", async () => {
  const { remote, input } = mount();
  typeInto(input, "accepted text");
  await tick();
  expect(input.value).toBe("accepted text");
  remote.apply({ patches: [[["draft"], "accepted text"]], acks: 1 });
  await tick();
  expect(input.value).toBe("accepted text");

  typeInto(input, "rejected text");
  await tick();
  expect(input.value).toBe("rejected text");
  remote.apply({ patches: [], acks: 1 });
  await tick();
  expect(input.value).toBe("accepted text");
});

test("all event holds must be acknowledged before controlled values restore", async () => {
  const { remote, input } = mount();
  typeInto(input, "first edit");
  typeInto(input, "second edit");
  await tick();
  remote.apply({ patches: [], acks: 1 });
  await tick();
  expect(input.value).toBe("second edit");
  remote.apply({ patches: [], acks: 1 });
  await tick();
  expect(input.value).toBe("bound");
});

test("checkbox restores are held until the host accepts or rejects them", async () => {
  const { remote, checkbox, messages } = mount();
  checkbox.click();
  await tick();
  expect(checkbox.checked).toBe(true);
  expect(JSON.parse(messages[1])).toEqual({ type: "event", name: "Toggled", payload: { value: true } });
  remote.apply({ patches: [[["accepted"], true]], acks: 1 });
  await tick();
  expect(checkbox.checked).toBe(true);
  checkbox.click();
  await tick();
  expect(checkbox.checked).toBe(false);
  remote.apply({ patches: [], acks: 1 });
  await tick();
  expect(checkbox.checked).toBe(true);
});

test("hold releases are idempotent and tick still resolves while held", async () => {
  const { input } = mount();
  const first = hold();
  const second = hold();
  cleanups.push(first, second);
  // Acknowledge the forwarded event; the manual holds still defer its restore.
  input.value = "local";
  input.dispatchEvent(new Event("input"));
  globalThis.__weft.apply({ patches: [], acks: 1 });
  await tick();
  first();
  first();
  await tick();
  expect(input.value).toBe("local");
  second();
  await tick();
  expect(input.value).toBe("bound");
});

test("a new start unmounts the old view and releases its outstanding holds", async () => {
  const { remote, target, input } = mount();
  typeInto(input, "unacknowledged");
  await tick();
  remote.start({ args: [{ ...snapshot(), draft: "restarted" }, "new view"] });
  await tick();
  expect(target.contains(input)).toBe(false);
  expect(input.value).toBe("bound");
  expect(target.querySelector("input")!.value).toBe("restarted");
  expect(target.querySelectorAll(":scope > div")).toHaveLength(1);
});

/** Captures precisely the DOM touched by one batch of remote patches. */
async function mutations(target: Node, action: () => void): Promise<MutationRecord[]> {
  const records: MutationRecord[] = [];
  const { promise, resolve } = Promise.withResolvers<void>();
  const observer = new MutationObserver((batch) => {
    records.push(...batch);
    resolve();
  });
  observer.observe(target, { subtree: true, childList: true, characterData: true, attributes: true });
  action();
  await tick();
  await promise;
  records.push(...observer.takeRecords());
  observer.disconnect();
  return records;
}

test("nested patches and array replacements update only affected DOM and reuse keyed rows", async () => {
  const { remote, target, column, rows, input } = mount();
  const before = rows();
  const user = column.children[1];
  let records = await mutations(target, () => {
    remote.apply({ patches: [[["user", "name"], "carol"]], acks: 0 });
  });
  expect(user.textContent).toBe("carol");
  expect(rows()).toEqual(before);
  expect(input.value).toBe("bound");
  expect(records.length).toBeGreaterThan(0);
  expect(records.every((record) => record.target === user || record.target.parentNode === user)).toBe(true);

  const todos = snapshot().todos;
  records = await mutations(target, () => {
    remote.apply({ patches: [[["todos"], [{ ...todos[0], name: "renamed" }, todos[1]]]], acks: 0 });
  });
  const firstLabel = before[0].querySelector("span")!;
  expect(firstLabel.textContent).toBe("renamed");
  expect(rows()).toEqual(before);
  expect(records.length).toBeGreaterThan(0);
  expect(records.every((record) => record.target === firstLabel || record.target.parentNode === firstLabel)).toBe(true);

  records = await mutations(target, () => {
    remote.apply({ patches: [[["todos"], [todos[1], { ...todos[0], name: "renamed" }]]], acks: 0 });
  });
  expect(rows()).toEqual([...before].reverse());
  expect(records.length).toBeGreaterThan(0);
  expect(records.every((record) => record.type === "childList" && record.target === column)).toBe(true);
  expect(records.flatMap((record) => [...record.addedNodes]).every((node) => before.includes(node as HTMLElement))).toBe(true);
});
