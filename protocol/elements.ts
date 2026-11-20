import { readFileSync } from "node:fs";

type Doc = { en: string; zh: string };
type Locale = keyof Doc;
type Member = Prop | Event;
type Children = "any" | "none" | string[];
type CssUnit = "px" | "color" | "bold";
type Level = "no" | "ignored";
type Support = Record<string, Level>;
type Platforms = ReadonlyMap<string, string>;

interface Feature {
  doc: Doc;
  support?: Support;
}

interface Prop extends Feature {
  name: string;
  type: string;
  positional?: boolean;
  required?: boolean;
  controlledBy?: string;
  default?: unknown;
  css?: { property: string; unit: CssUnit };
  event?: never;
}

interface Event extends Feature {
  name: string;
  event: true;
  payload?: string | null;
}

interface Element extends Feature {
  name: string;
  groups?: string[];
  members?: Member[];
  children?: Children;
  parents?: string[];
  content?: string;
}

interface Schema {
  unions?: Record<string, string[]>;
  groups?: Record<string, Member[]>;
  features?: Record<string, Feature>;
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

function validateSupport(support: Support | undefined, path: string, platforms: Platforms): void {
  if (support === undefined) return;
  if (support === null || typeof support !== "object" || Array.isArray(support)) {
    invalid(path, "expected an object mapping platforms to support levels");
  }
  for (const [platform, level] of Object.entries(support)) {
    if (!platforms.has(platform)) invalid(`${path}.${platform}`, `unknown platform ${quote(platform)}`);
    if (level !== "no" && level !== "ignored") {
      invalid(`${path}.${platform}`, `expected \"no\" or \"ignored\", got ${quote(level)}`);
    }
  }
}

function validateMembers(members: LocatedMember[], unions: Record<string, string[]>, platforms: Platforms): void {
  const names = new Set<string>();
  for (const { member, path } of members) {
    unique(names, member.name, `${path}.name`);
    validateSupport(member.support, `${path}.support`, platforms);
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

function expand(schema: Schema, platforms: Platforms): ExpandedElement[] {
  const unions = schema.unions ?? {};
  for (const [name, feature] of Object.entries(schema.features ?? {})) {
    validateSupport(feature.support, `features.${name}.support`, platforms);
  }
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
    validateMembers(located, unions, platforms);
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
    validateSupport(element.support, `${path}.support`, platforms);
    validateMembers(members, unions, platforms);

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

function rustSupport(support: Support | undefined, platforms: Platforms): string {
  const entries = Array.from(platforms).flatMap(([name, variant]) => {
    const level = support?.[name];
    return level ? [`(Platform::${variant}, Level::${level === "no" ? "No" : "Ignored"})`] : [];
  });
  return entries.length < 2 ? `&[${entries.join("")}]` : `&[\n            ${entries.join(",\n            ")},\n        ]`;
}

function rustMember(member: Member, platforms: Platforms): string {
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
        support: ${rustSupport(member.support, platforms)},
    },`;
}

function rustChildren(children: Children = "none"): string {
  if (children === "none") return "Children::None";
  if (children === "any") return "Children::Any";
  return `Children::Only(&[${children.map((name) => `Element::${name}`).join(", ")}])`;
}

function rust(elements: ExpandedElement[], features: Record<string, Feature>, platforms: Platforms): string {
  // Elements with identical expanded members share one slice.
  const shared = new Map<string, string>();
  function props(members: Member[]): string {
    if (members.length === 0) return "&[]";
    const rendered = members.map((member) => rustMember(member, platforms)).join("\n");
    const name = shared.get(rendered) ?? `PROPS_${shared.size}`;
    shared.set(rendered, name);
    return name;
  }
  const specs = elements.map((element) => `    ElementSpec {
        element: Element::${element.name},
        name: ${quote(element.name)},
        props: ${props(element.members)},
        children: ${rustChildren(element.children)},
        support: ${rustSupport(element.support, platforms)},
    },`).join("\n");
  const members = [...shared].map(([rendered, name]) => `const ${name}: &[PropSpec] = &[
${rendered}
];`).join("\n\n");
  const featureSpecs = Object.entries(features).map(([name, feature]) => `    FeatureSpec {
        name: ${quote(name)},
        support: ${rustSupport(feature.support, platforms)},
    },`).join("\n");
  const usesPlatform = [specs, members, featureSpecs].some((code) => code.includes("Platform::"));
  const platformImport = usesPlatform ? "use crate::protocol::Platform;\n" : "";
  return `${header}
use super::{Children, Css, CssUnit, ElementSpec, FeatureSpec, PropKind, PropSpec, PropType};
${platformImport}
/// Support exceptions for a target platform; omitted entries are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    No,
    #[allow(dead_code, reason = "the catalog may omit ignored entries")]
    Ignored,
}

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

#[expect(dead_code, reason = "feature checks land in L6 and R7")]
pub(crate) const FEATURES: [FeatureSpec; ${Object.keys(features).length}] = [
${featureSpecs}
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

const markdownHeader = "<!-- Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit. -->";
const locales: Record<Locale, {
  title: string;
  introduction: string;
  lifecycle: string;
  properties: string[];
  events: string[];
  summary: string[];
  required: string;
  positional: string;
  children: Record<"none" | "any", string>;
  directChildren: string;
  directParents: string;
  support: string;
  levels: Record<Level, string>;
  features: string;
  reference: string;
}> = {
  en: {
    title: "Built-in elements",
    introduction: "Elements, properties, and events are supported on all target platforms unless noted. Unsupported uses cause a compilation error; ignored uses are accepted but have no effect.",
    lifecycle: "All elements also accept `onMount` and `onUnmount`, which have no payload.",
    properties: ["Property", "Type", "Default", "Description", "Platform exceptions"],
    events: ["Event", "Payload", "Description", "Platform exceptions"],
    summary: ["Element", "Positional property", "Events", "Children"],
    required: "Required",
    positional: "positional",
    children: { none: "None", any: "Any" },
    directChildren: "Only these direct child elements are accepted:",
    directParents: "Must be a direct child of:",
    support: "Platform exceptions",
    levels: { no: "Unsupported", ignored: "Ignored" },
    features: "Feature support",
    reference: "See the [element reference](https://weftui.mioyi.net/reference/elements/) for full properties, events, and platform support.",
  },
  zh: {
    title: "内置元素",
    introduction: "除非另有说明，所有平台都支持列出的元素、属性和事件。使用标为「不支持」的功能会导致编译错误；标为「忽略」的功能可以编译，但没有效果。",
    lifecycle: "所有元素还接受 `onMount` 和 `onUnmount`，这两个事件都没有载荷。",
    properties: ["属性", "类型", "默认值", "说明", "平台例外"],
    events: ["事件", "载荷", "说明", "平台例外"],
    summary: ["元素", "位置属性", "事件", "子节点"],
    required: "必填",
    positional: "位置参数",
    children: { none: "无", any: "不限" },
    directChildren: "仅接受以下直接子元素：",
    directParents: "必须是以下元素之一的直接子元素：",
    support: "平台例外",
    levels: { no: "不支持", ignored: "忽略" },
    features: "功能支持",
    reference: "完整的属性、事件及各平台的支持情况见[元素参考](https://weftui.mioyi.net/zh/reference/elements/)。",
  },
};

const code = (text: string): string => `\`${text}\``;
const codeList = (items: string[]): string => items.map(code).join(", ");
const isEvent = (member: Member): member is Event => member.event === true;
const isProp = (member: Member): member is Prop => !isEvent(member);

function table(header: string[], rows: string[][]): string {
  return [header, header.map(() => "---"), ...rows]
    .map((row) => `| ${row.map((cell) => cell.replaceAll("|", "\\|")).join(" | ")} |`)
    .join("\n");
}

function childrenLabel(children: Children = "none", locale: Locale): string {
  return Array.isArray(children) ? codeList(children) : locales[locale].children[children];
}

function supportLabel(support: Support | undefined, locale: Locale, platforms: Platforms): string {
  return Array.from(platforms.keys()).flatMap((platform) => {
    const level = support?.[platform];
    return level ? [`${locales[locale].levels[level]}: ${code(platform)}`] : [];
  }).join("; ");
}

function reference(
  elements: ExpandedElement[], features: Record<string, Feature>, locale: Locale, platforms: Platforms,
): string {
  const text = locales[locale];
  const out = [
    `---\ntitle: ${text.title}\n---\n${markdownHeader}`,
    text.introduction,
    text.lifecycle,
  ];
  for (const element of elements) {
    out.push(`## ${element.name}`, element.doc[locale]);
    const support = supportLabel(element.support, locale, platforms);
    if (support) out.push(`**${text.support}:** ${support}.`);
    out.push(Array.isArray(element.children)
      ? `${text.directChildren} ${codeList(element.children)}.`
      : `**${text.summary[3]}:** ${childrenLabel(element.children, locale)}.`);
    if (element.parents) out.push(`${text.directParents} ${codeList(element.parents)}.`);
    const props = element.members.filter(isProp);
    if (props.length > 0) {
      out.push(table(text.properties, props.map((prop) => [
        prop.positional ? `${code(prop.name)} (${text.positional})` : code(prop.name),
        code(prop.type),
        prop.default !== undefined ? code(quote(prop.default)) : prop.required ? text.required : "—",
        prop.doc[locale],
        supportLabel(prop.support, locale, platforms) || "—",
      ])));
    }
    const events = element.members.filter(isEvent);
    if (events.length > 0) {
      out.push(table(text.events, events.map((event) => [
        code(event.name),
        event.payload ? code(event.payload) : "—",
        event.doc[locale],
        supportLabel(event.support, locale, platforms) || "—",
      ])));
    }
  }
  const featureNotes = Object.entries(features).flatMap(([name, feature]) => {
    const support = supportLabel(feature.support, locale, platforms);
    return support ? [`- ${code(name)}: ${feature.doc[locale]} ${support}.`] : [];
  });
  if (featureNotes.length > 0) out.push(`## ${text.features}`, featureNotes.join("\n"));
  return out.join("\n\n") + "\n";
}

function grammar(path: string, elements: ExpandedElement[], locale: Locale): string {
  const source = readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
  const block = /(<!-- elements:begin -->)[\s\S]*?(<!-- elements:end -->)/;
  if (!block.test(source)) throw new Error(`${path}: missing elements:begin/end markers`);
  const text = locales[locale];
  const summary = table(text.summary, elements.map((element) => [
    code(element.name),
    element.main === null ? "—" : code(element.main),
    element.members.filter(isEvent).map((event) =>
      code(event.payload ? `${event.name}(${event.payload})` : event.name)).join(", ") || "—",
    childrenLabel(element.children, locale),
  ]));
  return source.replace(block, (_match, begin, end) =>
    `${begin}\n${markdownHeader}\n\n${summary}\n\n${text.reference}\n${end}`);
}

export function generateElements(platforms: Platforms): Record<string, string> {
  const schema: Schema = JSON.parse(readFileSync(new URL("weft.elements.json", import.meta.url), "utf8"));
  const elements = expand(schema, platforms);
  const features = schema.features ?? {};
  const outputs: Record<string, string> = {
    "crates/transformer/src/elements/catalog.rs": rust(elements, features, platforms),
    "packages/runtime/src/elements.ts": typescript(elements, schema.unions ?? {}),
  };
  for (const locale of ["en", "zh"] as const) {
    const prefix = locale === "zh" ? "zh/" : "";
    outputs[`docs/src/content/docs/${prefix}reference/elements.md`] = reference(elements, features, locale, platforms);
    const grammarPath = `crates/parser/GRAMMAR${locale === "zh" ? ".zh-CN" : ""}.md`;
    outputs[grammarPath] = grammar(grammarPath, elements, locale);
  }
  return outputs;
}
