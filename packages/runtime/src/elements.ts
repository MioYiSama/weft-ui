// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

import type { Css } from "./css.ts";

export type CrossAlign = "start" | "center" | "end" | "stretch";

export type Justify = "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly";

export type Alignment = "topStart" | "top" | "topEnd" | "start" | "center" | "end" | "bottomStart" | "bottom" | "bottomEnd";

export type Axis = "vertical" | "horizontal";

export const ELEMENTS: Record<string, { main: string | null; children: "any" | "none" | readonly string[] }> = {
  Column: { main: null, children: "any" },
  Row: { main: null, children: "any" },
  Text: { main: "value", children: "none" },
  Button: { main: null, children: "any" },
  Input: { main: "value", children: "none" },
  Checkbox: { main: "checked", children: "none" },
  Image: { main: "src", children: "none" },
  Stack: { main: null, children: "any" },
  Scroll: { main: null, children: "any" },
  Spacer: { main: null, children: "none" },
};

export const CSS_PROPS: readonly Css[] = [
  {"property":"width","unit":"px"},
  {"property":"height","unit":"px"},
  {"unit":"custom","custom":"grow"},
  {"property":"min-width","unit":"px"},
  {"property":"max-width","unit":"px"},
  {"property":"min-height","unit":"px"},
  {"property":"max-height","unit":"px"},
  {"unit":"custom","custom":"fill"},
  {"property":"aspect-ratio","unit":"number"},
  {"properties":["margin-top","margin-right","margin-bottom","margin-left"],"unit":"px"},
  {"properties":["margin-left","margin-right"],"unit":"px"},
  {"properties":["margin-top","margin-bottom"],"unit":"px"},
  {"property":"margin-top","unit":"px"},
  {"property":"margin-bottom","unit":"px"},
  {"property":"margin-left","unit":"px"},
  {"property":"margin-right","unit":"px"},
  {"properties":["padding-top","padding-right","padding-bottom","padding-left"],"unit":"px"},
  {"properties":["padding-left","padding-right"],"unit":"px"},
  {"properties":["padding-top","padding-bottom"],"unit":"px"},
  {"property":"padding-top","unit":"px"},
  {"property":"padding-bottom","unit":"px"},
  {"property":"padding-left","unit":"px"},
  {"property":"padding-right","unit":"px"},
  {"property":"background","unit":"color"},
  {"property":"gap","unit":"px"},
  {"property":"align-items","unit":"keyword","keywords":{"start":"flex-start","end":"flex-end"}},
  {"property":"justify-content","unit":"keyword","keywords":{"start":"flex-start","end":"flex-end","spaceBetween":"space-between","spaceAround":"space-around","spaceEvenly":"space-evenly"}},
  {"property":"color","unit":"color"},
  {"property":"font-size","unit":"px"},
  {"property":"font-weight","unit":"keyword","keywords":{"true":"bold","false":"normal"}},
  {"property":"place-items","unit":"keyword","keywords":{"topStart":"start start","top":"start center","topEnd":"start end","start":"center start","center":"center center","end":"center end","bottomStart":"end start","bottom":"end center","bottomEnd":"end end"}},
  {"property":"scrollbar-width","unit":"keyword","keywords":{"true":"auto","false":"none"}},
  {"unit":"custom","custom":"spacer"},
];

export const CSS = ".w-root{display:flex;flex-direction:column;align-items:flex-start;box-sizing:border-box;width:100%;height:100%;overflow:hidden}\n.w-column{display:flex;flex-direction:column;align-items:flex-start}\n.w-row{display:flex;flex-direction:row;align-items:flex-start}\n.w-stack{display:grid;position:relative;grid-template:minmax(0,1fr)/minmax(0,1fr);place-items:start start}\n.w-in-stack{grid-area:1/1/2/2;position:relative}\n.w-in-stack.w-fill{position:absolute;inset:0}\n.w-scroll-v,.w-scroll-h{display:flex;align-items:flex-start;flex:1 1 0px;align-self:stretch;min-width:0;min-height:0}\n.w-scroll-v{flex-direction:column;overflow-x:hidden;overflow-y:auto}\n.w-scroll-h{flex-direction:row;overflow-x:auto;overflow-y:hidden}\n.w-in-scroll{flex-shrink:0}\n/* A percentage basis falls back to content when the main size is indefinite. */\n.w-grow{flex-basis:0%!important}\n.w-spacer{width:0;height:0;flex:1 1 0px}\n.w-text{display:inline-block}\n.w-button.w-wx{margin:0}\n.w-input.w-wx{border:1px solid #ccc;padding:4px 8px}\n/* The mini program scroll-view owns its viewport; its child owns the layout. */\n.w-scroll-content-v,.w-scroll-content-h{display:flex;align-items:flex-start;flex:none}\n.w-scroll-content-v{flex-direction:column;min-height:100%}\n.w-scroll-content-h{flex-direction:row;width:max-content;min-width:100%;height:100%}\n";
