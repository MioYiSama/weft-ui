// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import { langRedirect } from "./src/i18n/lang-redirect.mjs";
import weft from "./src/weft.tmLanguage.json" with { type: "json" };

const repo = "https://github.com/MioYiSama/weft-ui";

export default defineConfig({
  site: "https://weftui.mioyi.net",
  integrations: [
    starlight({
      title: "Weft",
      description: "One .weft view plus one JS/TS codebase, compiled to native apps on seven platforms.",
      logo: {
        light: "./src/assets/logo-light.svg",
        dark: "./src/assets/logo-dark.svg",
      },
      favicon: "/favicon.svg",
      head: [{ tag: "script", content: langRedirect }],
      defaultLocale: "root",
      locales: {
        root: { label: "English", lang: "en" },
        zh: { label: "简体中文", lang: "zh-CN" },
      },
      social: [{ icon: "github", label: "GitHub", href: repo }],
      editLink: { baseUrl: `${repo}/edit/master/docs/` },
      sidebar: [
        {
          label: "Guides",
          translations: { "zh-CN": "指南" },
          items: [{ autogenerate: { directory: "guides" } }],
        },
        {
          label: "Specification",
          translations: { "zh-CN": "规范" },
          items: [
            {
              label: "Grammar",
              translations: { "zh-CN": "语法" },
              link: `${repo}/blob/master/crates/parser/GRAMMAR.md`,
            },
            {
              label: "View protocol",
              translations: { "zh-CN": "视图协议" },
              link: `${repo}/blob/master/protocol/SPEC.md`,
            },
          ],
        },
      ],
      customCss: [
        "@fontsource-variable/geist",
        "@fontsource-variable/geist-mono",
        "./src/styles/starlight.css",
      ],
      expressiveCode: { shiki: { langs: [weft] } },
    }),
  ],
});
