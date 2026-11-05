import { readdir, readFile } from 'node:fs/promises';
import { extname, join, relative } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('../', import.meta.url);
const web = new URL('../web/', import.meta.url);

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

const webPath = web.pathname;
const rootPath = root.pathname;
const files = await walk(webPath);
const javascript = files.filter((file) => ['.js', '.mjs'].includes(extname(file)));
let failed = false;

for (const file of javascript) {
  const result = spawnSync(process.execPath, ['--check', file], { encoding: 'utf8' });
  if (result.status !== 0) {
    failed = true;
    console.error(`Syntax check failed: ${relative(rootPath, file)}`);
    console.error(result.stderr.trim());
  }
}

const dangerousTokens = [
  ['inner' + 'HTML', 'HTML string injection API'],
  ['outer' + 'HTML', 'HTML string injection API'],
  ['insertAdjacent' + 'HTML', 'HTML string injection API'],
  ['eval' + '(', 'dynamic code execution'],
  ['new ' + 'Function', 'dynamic code execution'],
];

for (const file of javascript.filter((path) => !path.includes('/tests/'))) {
  const source = await readFile(file, 'utf8');
  for (const [token, reason] of dangerousTokens) {
    if (source.includes(token)) {
      failed = true;
      console.error(`${relative(rootPath, file)} uses forbidden ${reason}: ${token}`);
    }
  }
}

const htmlPath = new URL('../web/index.html', import.meta.url).pathname;
const html = await readFile(htmlPath, 'utf8');
const htmlChecks = [
  [/\son[a-z]+\s*=/i, 'inline event handlers'],
  [/(?:src|href)=["']https?:\/\//i, 'remote script/style assets'],
  [/(?:user-scalable\s*=\s*no|maximum-scale\s*=\s*1)/i, 'disabled browser zoom'],
];
for (const [pattern, description] of htmlChecks) {
  if (pattern.test(html)) {
    failed = true;
    console.error(`web/index.html contains forbidden ${description}.`);
  }
}

if (failed) process.exit(1);
console.log(`Frontend lint passed (${javascript.length} JavaScript modules checked).`);
