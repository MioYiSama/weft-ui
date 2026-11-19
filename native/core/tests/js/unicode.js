function check(condition, operation) {
  if (!condition) throw new Error(`Unicode: ${operation}`);
}

check('é'.toUpperCase() === 'É', 'accented uppercase');
check('ß'.toUpperCase() === 'SS', 'expanding uppercase');
check('İ'.toLowerCase() === 'i\u0307', 'locale-independent lowercase');
check('a\uD800é\uDC00'.toUpperCase() === 'A\uD800É\uDC00', 'preserve lone surrogates');

check('e\u0301'.normalize() === 'é', 'NFC');
check('é'.normalize('NFD') === 'e\u0301', 'NFD');
check('\uFB03'.normalize('NFKC') === 'ffi', 'NFKC');
check('\u2460é'.normalize('NFKD') === '1e\u0301', 'NFKD');

check('é'.localeCompare('e\u0301') === 0, 'canonical collation equivalence');
check('a'.localeCompare('ä') < 0 && 'a'.localeCompare('b') < 0, 'localized collation order');

const date = new Date(Date.UTC(2026, 10, 19, 12, 34, 56));
const combined = date.toLocaleString();
const dateOnly = date.toLocaleDateString();
const timeOnly = date.toLocaleTimeString();
check(combined.includes('2026') && combined.includes('56'), 'medium local date and time');
check(dateOnly.includes('2026') && timeOnly.includes('56'), 'separate local date and time');
check(new Date(NaN).toLocaleString() === 'Invalid Date', 'invalid date');

console.log('Unicode OK');
