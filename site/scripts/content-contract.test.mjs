import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';

test('launch page preserves the approved content contract', async () => {
  const html = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
  assert.equal((html.match(/<h1\b/g) ?? []).length, 1);
  assert.doesNotMatch(html, /ascii-title/);
  assert.match(html, /fonts\.googleapis\.com\/css2\?family=Press\+Start\+2P.*family=DotGothic16/);
  assert.equal((html.match(/loading="eager"/g) ?? []).length, 7);
  assert.equal((html.match(/<svg\b/g) ?? []).length, 4);
  const gallery = await readFile(new URL('../src/components/EvidenceGallery.astro', import.meta.url), 'utf8');
  assert.match(gallery, /import\s+\{\s*Image\s*\}\s+from\s+['"]astro:assets['"]/);
  assert.match(gallery, /<Image\s+src=\{item\.src\}/);
  assert.doesNotMatch(gallery, /src=\{item\.src\.src\}/);
  const hero = await readFile(new URL('../src/components/HeroProof.astro', import.meta.url), 'utf8');
  assert.match(hero, /import\s+\{\s*Image\s*\}\s+from\s+['"]astro:assets['"]/);
  assert.equal((hero.match(/<Image\b/g) ?? []).length, 3);
  assert.doesNotMatch(hero, /src=\{(?:hero|guildHall|delve)\.src\}/);
  const mappings = await readFile(new URL('../src/components/MappingCards.astro', import.meta.url), 'utf8');
  for (const iconStem of ['sword', 'castle', 'bell', 'coins']) {
    assert.match(mappings, new RegExp(`assets/icons/${iconStem}\\.svg`));
  }
  assert.match(mappings, /<mapping\.icon\b/);
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
  assert.match(html, /Prerequisite/);
  assert.match(html, /Install Herdr first/);
  assert.match(html, /href="https:\/\/herdr\.dev\/docs\/install\/"[^>]*target="_blank"/);
  assert.doesNotMatch(html, /cargo install questmancer/i);
});
