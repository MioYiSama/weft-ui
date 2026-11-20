import { readFileSync } from "node:fs";

type Doc = { en: string; zh: string };
type Member = Prop | Event;
type Children = "any" | "none" | string[];
type CssUnit = "px" | "color" | "bold";

interface Prop {
  name: string;
  type: string;
  positional?: boolean;
  required?: boolean;
  controlledBy?: string;
  css?: { property: string; unit: CssUnit };
  doc: Doc;
  event?: never;
}

interface Event {
  name: string;
  event: true;
  payload?: string | null;
  doc: Doc;
}

interface Element {
  name: string;
  doc: Doc;
  groups?: string[];
  members?: Member[];
  children?: Children;
  parents?: string[];
  content?: string;
}

interface Schema {
  unions?: Record<string, string[]>;
  groups?: Record<string, Member[]>;
  elements: Element[];
}

interface ExpandedElement extends Element {
  members: Member[];
  main: string | null;
}

interface LocatedMember {
  member: Member;
  path: string;
}

const header = "// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.\n";
const propTypes = { Int: true, Bool: true, String: true, Color: true };
const payloadTypes = { Int: true, Bool: true, String: true };
const cssUnits: Record<CssUnit, string> = { px: "Px", color: "Color", bold: "Bold" };
const quote = JSON.stringify;

function invalid(path: string, message: string): never {
  throw new Error(`protocol/weft.elements.json: ${path}: ${message}`);
}

function unique(names: Set<string>, name: string, path: string): void {
  if (names.has(name)) invalid(path, `duplicate name ${quote(name)}`);
  names.add(name);
}

function locate(members: Member[], path: string): LocatedMember[] {
  return members.map((member, i) => ({ member, path: `${path}[${i}]` }));
}

function validateMembers(members: LocatedMember[], unions: Record<string, string[]>): void {
  const names = new Set<string>();
  for (const { member, path } of members) {
    unique(names, member.name, `${path}.name`);
    if (member.event) {
      if (member.payload != null && !Object.hasOwn(payloadTypes, member.payload)) {
        invalid(`${path}.payload`, `unknown event payload type ${quote(member.payload)}`);
      }
      continue;
    }
    if (!Object.hasOwn(propTypes, member.type)) {
      const message = Object.hasOwn(unions, member.type) || member.type === "Float" || member.type === "Strings"
        ? "unsupported property type"
        : "unknown property type";
      invalid(`${path}.type`, `${message} ${quote(member.type)}`);
    }
    if (member.css && !Object.hasOwn(cssUnits, member.css.unit)) {
      invalid(`${path}.css.unit`, `unknown CSS unit ${quote(member.css.unit)}`);
    }
  }
}

function expand(schema: Schema): ExpandedElement[] {
  const unions = schema.unions ?? {};
  for (const [name, values] of Object.entries(unions)) {
    const path = `unions.${name}`;
    if (!/^[A-Z][A-Za-z0-9]*$/.test(name)) invalid(path, "expected a PascalCase union name");
    if (Object.hasOwn(propTypes, name) || name === "Float" || name === "Strings") {
      invalid(path, `duplicate name ${quote(name)}`);
    }
    if (values.length === 0) invalid(path, "expected a nonempty union");
    const names = new Set<string>();
    for (const [i, value] of values.entries()) unique(names, value, `${path}[${i}]`);
  }
  const groups = new Map<string, LocatedMember[]>();
  for (const [name, members] of Object.entries(schema.groups ?? {})) {
    const located = locate(members, `groups.${name}`);
    validateMembers(located, unions);
    groups.set(name, located);
  }

  const names = new Set<string>();
  for (const [i, { name }] of schema.elements.entries()) {
    unique(names, name, `elements[${i}].name`);
  }

  return schema.elements.map((element, i) => {
    const path = `elements[${i}]`;
    const members = (element.groups ?? []).flatMap((name, j) => {
      const group = groups.get(name);
      if (group === undefined) invalid(`${path}.groups[${j}]`, `unknown group ${quote(name)}`);
      return group;
    });
    members.push(...locate(element.members ?? [], `${path}.members`));
    validateMembers(members, unions);

    let main: Prop | undefined;
    for (const { member, path: memberPath } of members) {
      if (member.event) continue;
      if (member.positional) {
        if (main) invalid(`${memberPath}.positional`, `already has positional property ${quote(main.name)}`);
        main = member;
      }
      if (member.controlledBy !== undefined && !members.some(({ member: event }) =>
        event.event && event.name === member.controlledBy)) {
        invalid(`${memberPath}.controlledBy`, `unknown event ${quote(member.controlledBy)} on ${element.name}`);
      }
    }
    if (element.content !== undefined && element.content !== main?.name) {
      invalid(`${path}.content`, `${quote(element.content)} is not a positional property`);
    }
    for (const field of ["parents", "children"] as const) {
      const list = element[field];
      if (!Array.isArray(list)) continue;
      for (const [j, name] of list.entries()) {
        if (!names.has(name)) invalid(`${path}.${field}[${j}]`, `unknown element ${quote(name)}`);
      }
    }
    return { ...element, members: members.map(({ member }) => member), main: main?.name ?? null };
  });
}

function rustMember(member: Member): string {
  const prop = member.event ? undefined : member;
  const kind = member.event
    ? member.payload == null
      ? "PropKind::Handler { payload: None }"
      : `PropKind::Handler {\n            payload: Some(PropType::${member.payload}),\n        }`
    : `PropKind::Value(PropType::${member.type})`;
  const css = prop?.css;
  return `    PropSpec {
        name: ${quote(member.name)},
        kind: ${kind},
        positional: ${prop?.positional ?? false},
        required: ${prop?.required ?? false},
        css: ${css ? `Some(Css {
            property: ${quote(css.property)},
            unit: CssUnit::${cssUnits[css.unit]},
        })` : "None"},
        controlled_by: ${prop?.controlledBy === undefined ? "None" : `Some(${quote(prop.controlledBy)})`},
    },`;
}

function rustChildren(children: Children = "none"): string {
  if (children === "none") return "Children::None";
  if (children === "any") return "Children::Any";
  return `Children::Only(&[${children.map((name) => `Element::${name}`).join(", ")}])`;
}

function rust(elements: ExpandedElement[]): string {
  // Elements with identical expanded members share one slice.
  const shared = new Map<string, string>();
  function props(members: Member[]): string {
    if (members.length === 0) return "&[]";
    const rendered = members.map(rustMember).join("\n");
    const name = shared.get(rendered) ?? `PROPS_${shared.size}`;
    shared.set(rendered, name);
    return name;
  }
  const specs = elements.map((element) => `    ElementSpec {
        element: Element::${element.name},
        name: ${quote(element.name)},
        props: ${props(element.members)},
        children: ${rustChildren(element.children)},
    },`).join("\n");
  const members = [...shared].map(([rendered, name]) => `const ${name}: &[PropSpec] = &[
${rendered}
];`).join("\n\n");
  return `${header}
use super::{Children, Css, CssUnit, ElementSpec, PropKind, PropSpec, PropType};

/// Built-in elements available in Weft views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
${elements.map(({ name }) => `    ${name},`).join("\n")}
}

impl Element {
    /// The element's name in a Weft view.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.spec().name
    }
}

${members}

pub(crate) const ELEMENTS: [ElementSpec; ${elements.length}] = [
${specs}
];
`;
}

function typescript(elements: ExpandedElement[], unions: Record<string, string[]>): string {
  const types = Object.entries(unions).map(([name, values]) =>
    `export type ${name} = ${values.map((value) => quote(value)).join(" | ")};\n\n`).join("");
  const entries = elements.map(({ name, main, children = "none" }) =>
    `  ${name}: { main: ${quote(main)}, children: ${quote(children)} },`).join("\n");
  return `${header}
${types}export const ELEMENTS: Record<string, { main: string | null; children: "any" | "none" | readonly string[] }> = {
${entries}
};
`;
}

export function generateElements(): Record<string, string> {
  const schema: Schema = JSON.parse(readFileSync(new URL("weft.elements.json", import.meta.url), "utf8"));
  const elements = expand(schema);
  return {
    "crates/transformer/src/elements/catalog.rs": rust(elements),
    "packages/runtime/src/elements.ts": typescript(elements, schema.unions ?? {}),
  };
}
