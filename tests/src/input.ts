/** Simulates the text host's UTF-16 limit without splitting a surrogate pair. */
export function limitInput(value: string, maxLength: number): string {
  if (maxLength < 0 || value.length <= maxLength) return value;
  const high = value.charCodeAt(maxLength - 1);
  const low = value.charCodeAt(maxLength);
  const splitsPair = high >= 0xd800 && high <= 0xdbff && low >= 0xdc00 && low <= 0xdfff;
  return value.slice(0, splitsPair ? maxLength - 1 : maxLength);
}
