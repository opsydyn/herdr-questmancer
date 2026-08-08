import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { basename, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const auditedExtensions = new Set(['.html', '.css', '.js']);

function filesBelow(directory) {
  const entries = readdirSync(directory, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const entryPath = join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...filesBelow(entryPath));
    } else if (auditedExtensions.has(extname(entry.name))) {
      files.push(entryPath);
    }
  }

  return files;
}

function localReferences(source) {
  const references = [];
  const attributePattern = /\b(?:src|href|action)\s*=\s*(?:"([^"]+)"|'([^']+)'|([^\s>]+))/gi;
  const cssUrlPattern = /\burl\(\s*(?:"([^"]+)"|'([^']+)'|([^\s)]+))\s*\)/gi;

  for (const pattern of [attributePattern, cssUrlPattern]) {
    for (const match of source.matchAll(pattern)) {
      const reference = match[1] ?? match[2] ?? match[3];
      if (reference?.startsWith('/') && !reference.startsWith('//')) {
        references.push(reference);
      }
    }
  }

  return references;
}

function isBasePrefixed(reference, basePath) {
  const pathname = reference.split(/[?#]/, 1)[0];
  return pathname === basePath || pathname.startsWith(`${basePath}/`);
}

/**
 * Audit a generated Astro directory for base-safe local references and
 * expected hashed assets.
 *
 * @param {string} buildDir
 * @param {string} basePath
 * @param {string[]} expectedAssetStems
 */
export function validateBuild(buildDir, basePath, expectedAssetStems) {
  if (!existsSync(buildDir) || !statSync(buildDir).isDirectory()) {
    throw new Error(`build directory is missing: ${buildDir}`);
  }

  const files = filesBelow(buildDir);
  for (const file of files) {
    const source = readFileSync(file, 'utf8');
    for (const reference of localReferences(source)) {
      if (!isBasePrefixed(reference, basePath)) {
        throw new Error(`local reference bypasses base path: ${reference}`);
      }
    }
  }

  const outputEntries = readdirSync(buildDir, { withFileTypes: true });
  const outputFiles = [];
  function collectAll(directory, entries) {
    for (const entry of entries) {
      const entryPath = join(directory, entry.name);
      if (entry.isDirectory()) {
        collectAll(entryPath, readdirSync(entryPath, { withFileTypes: true }));
      } else {
        outputFiles.push(entryPath);
      }
    }
  }
  collectAll(buildDir, outputEntries);

  for (const expectedStem of expectedAssetStems) {
    if (!outputFiles.some((file) => basename(file).startsWith(expectedStem))) {
      throw new Error(`missing expected built asset: ${expectedStem}`);
    }
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  validateBuild('dist', '/herdr-questmancer', [
    'hero',
    'guild-hall',
    'delve',
    'ledger',
    'home',
    'bell',
    'heart',
    'lock',
  ]);
  console.log('base-path audit passed');
}
