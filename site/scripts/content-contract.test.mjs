import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';

test('launch page preserves the approved content contract', async () => {
  const html = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
  assert.equal((html.match(/<h1\b/g) ?? []).length, 1);
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
