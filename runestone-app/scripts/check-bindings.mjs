/**
 * Validates that src/lib/bindings.ts exists for IPC type safety.
 * Full specta codegen can replace this script in a follow-up.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const bindingsPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../src/lib/bindings.ts')

if (!fs.existsSync(bindingsPath)) {
  console.error('Missing bindings.ts — create src/lib/bindings.ts with shared IPC types')
  process.exit(1)
}

const content = fs.readFileSync(bindingsPath, 'utf8')
if (!content.includes('ScanVaultResult')) {
  console.error('bindings.ts must export ScanVaultResult')
  process.exit(1)
}

console.log('bindings.ts present')
