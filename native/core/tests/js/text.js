// TextEncoder / TextDecoder come with Hermes.
const bytes = new TextEncoder().encode("héllo ✓");
console.log(bytes.length, new TextDecoder().decode(bytes));
