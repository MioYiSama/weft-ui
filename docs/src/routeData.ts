import { defineRouteMiddleware, type StarlightRouteData } from "@astrojs/starlight/route-data";

type SidebarLink = Extract<StarlightRouteData["sidebar"][number], { type: "link" }>;

function localizeSpecificationLink(link: SidebarLink | undefined) {
  if (link && (link.href.endsWith("/crates/parser/GRAMMAR.md") || link.href.endsWith("/protocol/SPEC.md"))) {
    link.href = link.href.replace(/\.md$/, ".zh-CN.md");
  }
}

function localizeSpecificationLinks(entries: StarlightRouteData["sidebar"]) {
  for (const entry of entries) {
    if (entry.type === "group") localizeSpecificationLinks(entry.entries);
    else localizeSpecificationLink(entry);
  }
}

export const onRequest = defineRouteMiddleware(({ locals }, next) => {
  const { locale, sidebar, pagination } = locals.starlightRoute;
  if (locale === "zh") {
    localizeSpecificationLinks(sidebar);
    localizeSpecificationLink(pagination.prev);
    localizeSpecificationLink(pagination.next);
  }
  return next();
});
