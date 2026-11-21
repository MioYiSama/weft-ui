import { readFileSync } from "node:fs";

export const platforms = ["web", "wechat", "ios", "macos", "android", "windows", "linux"] as const;
export type Platform = typeof platforms[number];

interface Page {
  id: string;
  title: string;
  component?: string;
  dark?: boolean;
  pending?: Platform[];
}

export function pages(platform: Platform): Page[] {
  const list: Page[] = JSON.parse(readFileSync(new URL("../pages.json", import.meta.url), "utf8"));
  return list.filter(({ pending }) => !pending?.includes(platform));
}
