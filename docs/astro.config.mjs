// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import weft from "./src/weft.tmLanguage.json" with { type: "json" };

const repo = "https://github.com/MioYiSama/weft-ui";

export default defineConfig({
  site: "https://weftui.mioyi.net",
  integrations: [
    starlight({
      title: "Weft",
      description: "一份 .weft 视图加一份 JS/TS 逻辑，编译成七个平台的原生应用。",
      logo: {
        light: "./src/assets/logo-light.svg",
        dark: "./src/assets/logo-dark.svg",
      },
      favicon: "/favicon.svg",
      defaultLocale: "root",
      locales: { root: { label: "简体中文", lang: "zh-CN" } },
      social: [{ icon: "github", label: "GitHub", href: repo }],
      editLink: { baseUrl: `${repo}/edit/master/docs/` },
      sidebar: [{ label: "规范", items: ["grammar", "protocol"] }],
      customCss: [
        "@fontsource-variable/geist",
        "@fontsource-variable/geist-mono",
        "./src/styles/starlight.css",
      ],
      expressiveCode: { shiki: { langs: [weft] } },
    }),
  ],
});
