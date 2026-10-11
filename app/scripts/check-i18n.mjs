// Falha se algum texto da interface não tem tradução para inglês.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const files = [];
const walk = (d) => {
  for (const f of readdirSync(d)) {
    const p = join(d, f);
    if (statSync(p).isDirectory()) walk(p);
    else if (/\.(svelte|ts)$/.test(f)) files.push(p);
  }
};
walk("src");

const dict = readFileSync("src/lib/i18n.svelte.ts", "utf8");
const have = new Set([...dict.matchAll(/^\s*"([^"]+)":/gm)].map((m) => m[1]));
const used = new Set();
for (const f of files) {
  const s = readFileSync(f, "utf8");
  for (const m of s.matchAll(/\bt\("([^"]+)"\)/g)) used.add(m[1]);
  // tabelas [valor, "rótulo"] passadas por t(label)
  for (const m of s.matchAll(/\[\s*(?:"[^"]*"|0x[0-9a-f]+|\d+|\[[^\]]*\])\s*,\s*"([^"]+)"\s*\]/g)) used.add(m[1]);
}
const ignore = /^(Hz\d+|\d+ Hz|Ctrl|Shift|Alt|Win\/Cmd)$/;
const missing = [...used].filter((k) => !have.has(k) && !ignore.test(k)).sort();
if (missing.length) {
  console.error(`textos sem tradução (${missing.length}):\n` + missing.join("\n"));
  process.exit(1);
}
console.log(`i18n ok: ${used.size} textos`);
