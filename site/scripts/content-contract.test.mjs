import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';

test('launch page preserves the approved content contract', async () => {
  const html = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
  assert.equal((html.match(/<h1\b/g) ?? []).length, 1);
  assert.doesNotMatch(html, /ascii-title/);
  assert.match(html, /fonts\.googleapis\.com\/css2\?family=Press\+Start\+2P.*family=DotGothic16/);
  const css = await readFile(new URL('../src/styles/global.css', import.meta.url), 'utf8');
  assert.match(css, /cursor:\s*url\("\.\.\/assets\/cursor\.svg"\)/);
  for (const phrase of [
    'What is best in code?',
    'To crush your bugs.',
    'See your agents driven before you.',
    'Hear the lamentations of your token budget.',
    'https://github.com/opsydyn/herdr-questmancer',
    'herdr plugin action invoke opsydyn.questmancer.open',
  ]) assert.ok(html.includes(phrase), 'missing approved phrase: ' + phrase);
  assert.doesNotMatch(html, /cargo install questmancer/i);
});
