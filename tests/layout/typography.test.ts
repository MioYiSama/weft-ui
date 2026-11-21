import { tick } from "@weft-ui/runtime";
import { Window } from "happy-dom";
import { expect, test, vi } from "vitest";
import { Typography } from "../../crates/transformer/tests/fixtures/Typography.weft";

test("text updates independently of its clamp, visibility and inline runs", async () => {
  const doc = new Window().document as unknown as Document;
  vi.stubGlobal("document", doc);
  const app = new Typography({
    value: "alpha", weight: "light", decoration: "underline",
    lines: 2, hidden: true, clip: false, italic: false,
  });
  const target = doc.createElement("main");
  doc.body.append(target);

  try {
    app.mount(target);
    const [plain, rich] = target.querySelectorAll<HTMLElement>(".w-text");
    const [lead, middle, tail] = rich.querySelectorAll<HTMLElement>(":scope > span.w-span");
    expect(rich.textContent).toBe("One alpha two");
    expect(lead.style.fontWeight).toBe("600");
    expect(tail.style.getPropertyValue("text-decoration-line")).toBe("line-through");
    expect(plain.style.fontWeight).toBe("300");
    expect(plain.style.overflow).toBe("hidden");
    expect(plain.classList.contains("w-text-hidden")).toBe(true);
    expect(plain.classList.contains("w-text-clip")).toBe(false);

    app.model.value = "beta";
    app.model.weight = "bold";
    app.model.decoration = "lineThrough";
    app.model.lines = 0;
    app.model.clip = true;
    app.model.italic = true;
    await tick();

    expect(plain.textContent).toBe("beta");
    expect(plain.style.display).toBe("");
    expect(plain.style.overflow).toBe("");
    expect(plain.style.getPropertyValue("text-decoration-line")).toBe("line-through");
    expect(plain.style.fontWeight).toBe("700");
    expect(plain.style.fontStyle).toBe("italic");
    expect(plain.classList.contains("w-text-hidden")).toBe(true);
    expect(plain.classList.contains("w-text-clip")).toBe(true);
    expect(rich.textContent).toBe("One beta two");
    expect(rich.children[1]).toBe(middle);

    app.model.lines = 3;
    app.model.hidden = false;
    await tick();
    expect(plain.style.overflow).toBe("hidden");
    expect(plain.classList.contains("w-text-hidden")).toBe(false);
  } finally {
    app.unmount();
    vi.unstubAllGlobals();
  }
});
