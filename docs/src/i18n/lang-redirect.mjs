// Inline head script shared by Starlight pages and the landing page.
// English is the root locale and Chinese lives under /zh/. The first visit
// follows the browser language; after that the language of the last page
// viewed wins, so switching languages by hand sticks.
export const langRedirect = `(() => {
  const key = "weft-lang";
  // From the path, not <html lang>: GitHub Pages serves the English 404 page
  // under /zh/ too, which must not redirect again.
  const zh = /^\\/zh(\\/|$)/.test(location.pathname);
  let stored = null;
  try { stored = localStorage.getItem(key); } catch {}
  if (stored === null && !zh && navigator.languages.some((l) => l.toLowerCase().startsWith("zh"))) {
    location.replace("/zh" + location.pathname + location.search + location.hash);
    return;
  }
  try { localStorage.setItem(key, zh ? "zh" : "en"); } catch {}
})();`;
