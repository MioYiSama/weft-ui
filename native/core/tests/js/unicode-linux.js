function check(condition, operation) {
  if (!condition) throw new Error(`Linux Unicode: ${operation}`);
}

check('\uD800ß𐐨\uDC00\uD800'.toUpperCase() === '\uD800SS𐐀\uDC00\uD800',
  'preserve lone surrogates and map pairs');
check('ß𐐨'.repeat(129).toUpperCase() === 'SS𐐀'.repeat(129), 'growing case output');
check('iı'.toUpperCase() === 'II' && 'Iİ'.toLowerCase() === 'ii\u0307', 'root case mapping');

const malformed = '\uD800e\uDC00\u0301';
for (const [form, input, expected] of [
  ['NFC', 'e\u0301', 'é'],
  ['NFD', 'é', 'e\u0301'],
  ['NFKC', '\u2460e\u0301', '1é'],
  ['NFKD', '\u2460é', '1e\u0301'],
]) {
  check((input + malformed).normalize(form) === expected + malformed, `${form} with lone surrogates`);
}
check('é'.repeat(129).normalize('NFD') === 'e\u0301'.repeat(129), 'growing normalization output');

for (const [timestamp, year] of [[8.64e15, '275760'], [-8.64e15, '271822']]) {
  check(new Date(timestamp).toLocaleString().replace(/[^0-9]/g, '').includes(year),
    `date at edge of range (${timestamp})`);
}

// Serbian's medium date uses a numeric month, but its BCE era displays the script.
const ancientDate = new Date(-8.64e15).toLocaleDateString();
console.log('iı'.toLocaleUpperCase(), 'Iİ'.toLocaleLowerCase(), '李'.localeCompare('张'),
  ancientDate.includes('p. n. e.'));
