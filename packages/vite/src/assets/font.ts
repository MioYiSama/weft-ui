/** Reads the typographic family (or legacy family) from a TrueType/OpenType name table. */
export function fontFamily(font: Buffer): string {
  const invalid = (): never => { throw new Error("invalid sfnt name table"); };
  if (font.length < 12) invalid();
  const signature = font.readUInt32BE(0);
  if (signature !== 0x00010000 && signature !== 0x4f54544f && signature !== 0x74727565) invalid();
  const directoryEnd = 12 + font.readUInt16BE(4) * 16;
  if (directoryEnd > font.length) invalid();

  for (let entry = 12; entry < directoryEnd; entry += 16) {
    if (font.toString("ascii", entry, entry + 4) !== "name") continue;
    const start = font.readUInt32BE(entry + 8);
    const length = font.readUInt32BE(entry + 12);
    const end = start + length;
    if (length < 6 || start < directoryEnd || end > font.length) invalid();
    const recordsEnd = start + 6 + font.readUInt16BE(start + 2) * 12;
    const strings = start + font.readUInt16BE(start + 4);
    if (recordsEnd > strings || strings > end) invalid();

    let family = "";
    let preferred = Infinity;
    for (let record = start + 6; record < recordsEnd; record += 12) {
      const name = font.readUInt16BE(record + 6);
      if (name !== 16 && name !== 1) continue;
      const platform = font.readUInt16BE(record);
      const encoding = font.readUInt16BE(record + 2);
      const windows = platform === 3 && encoding === 1;
      if (!windows && !(platform === 1 && encoding === 0)) continue;
      const language = font.readUInt16BE(record + 4);
      // A preferred family beats a legacy one. Prefer Unicode, then English
      // within each encoding so the name is independent of the build locale.
      const priority = (name === 16 ? 0 : 4) + (windows ? 0 : 2)
        + (language === (windows ? 0x0409 : 0) ? 0 : 1);
      if (priority >= preferred) continue;
      const size = font.readUInt16BE(record + 8);
      const offset = strings + font.readUInt16BE(record + 10);
      if (offset + size > end || (windows && size % 2 !== 0)) invalid();
      const text = new TextDecoder(windows ? "utf-16be" : "macintosh", { fatal: true })
        .decode(font.subarray(offset, offset + size)).trim();
      if (text) {
        family = text;
        preferred = priority;
      }
    }
    if (family) return family;
    break;
  }
  throw new Error("no supported family name (name ID 16 or 1)");
}

export type FontWeight = 300 | 400 | 500 | 600 | 700;

/** The part of a font face before its URL, which Web builds may compute at runtime. */
export function fontFaceStart(family: string, weight: FontWeight, italic: boolean): string {
  const quoted = family.replace(/[\p{Cc}"\\]/gu, (c) => `\\${c.charCodeAt(0).toString(16)} `);
  return `@font-face { font-family: "${quoted}"; font-weight: ${weight}; font-style: ${italic ? "italic" : "normal"}; src: url(`;
}
