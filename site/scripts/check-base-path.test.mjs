import { mkdtemp, mkdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import { validateBuild } from './check-base-path.mjs';

test('accepts base-prefixed local references and expected outputs', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await mkdir(join(root, '_astro'), { recursive: true });
  await writeFile(join(root, 'index.html'), '<img src="/herdr-questmancer/_astro/hero.abc.png">');
  await writeFile(join(root, '_astro/hero.abc.png'), 'fixture');
  assert.doesNotThrow(() => validateBuild(root, '/herdr-questmancer', ['hero']));
});

test('rejects a local root-relative asset path', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await writeFile(join(root, 'index.html'), '<img src="/assets/hero.png">');
  assert.throws(() => validateBuild(root, '/herdr-questmancer', ['hero']), /base path/);
});

test('rejects a missing expected output', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await writeFile(join(root, 'index.html'), '<main>ok</main>');
  assert.throws(() => validateBuild(root, '/herdr-questmancer', ['hero']), /missing/);
});
