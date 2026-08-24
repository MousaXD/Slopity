import { readdir, readFile } from 'node:fs/promises';
import { extname, join, relative } from 'node:path';

const root = new URL('../', import.meta.url);
const rootPath = root.pathname;

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await walk(path));
    else files.push(path);
  }
  return files;
}

const candidates = [
  ...await walk(new URL('../web/', import.meta.url).pathname),
  ...await walk(new URL('./', import.meta.url).pathname),
  new URL('../package.json', import.meta.url).pathname,
].filter((file) => ['.js', '.mjs', '.html', '.css', '.json'].includes(extname(file)));

let failed = false;
for (const file of candidates) {
  const source = await readFile(file, 'utf8');
  const problems = [];
  if (source.includes('\r')) problems.push('CRLF line endings');
  if (!source.endsWith('\n')) problems.push('missing final newline');
  if (/\t/.test(source)) problems.push('tab indentation');
  if (/[ \t]+$/m.test(source)) problems.push('trailing whitespace');
  if (problems.length) {
    failed = true;
    console.error(`${relative(rootPath, file)}: ${problems.join(', ')}`);
  }
}

if (failed) process.exit(1);
console.log(`Format check passed (${candidates.length} frontend files checked).`);
