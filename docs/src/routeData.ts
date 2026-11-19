import { defineRouteMiddleware, type StarlightRouteData } from "@astrojs/starlight/route-data";

function localizeSpecificationLinks(entries: StarlightRouteData["sidebar"]) {
  for (const entry of entries) {
    if (entry.type === "group") {
      localizeSpecificationLinks(entry.entries);
    } else if (
      entry.href.endsWith("/crates/parser/GRAMMAR.md") ||
      entry.href.endsWith("/protocol/SPEC.md")
    ) {
      entry.href = entry.href.replace(/\.md$/, ".zh-CN.md");
    }
  }
}

export const onRequest = defineRouteMiddleware(({ locals }, next) => {
  const { locale, sidebar } = locals.starlightRoute;
  if (locale === "zh") localizeSpecificationLinks(sidebar);
  return next();
});
