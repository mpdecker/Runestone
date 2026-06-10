/**
 * Minimal E2E smoke test: build the app, serve via vite preview, verify the shell loads.
 * Tauri-driver is not required for CI; this validates the web bundle boots.
 */
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const appRoot = path.resolve(__dirname, '../..')
const previewPort = 4173
const previewUrl = `http://127.0.0.1:${previewPort}/`

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: appRoot,
      stdio: 'inherit',
      shell: process.platform === 'win32',
      ...options,
    })
    child.on('error', reject)
    child.on('close', (code) => {
      if (code === 0) resolve()
      else reject(new Error(`${command} ${args.join(' ')} exited with code ${code}`))
    })
  })
}

async function waitForServer(url, timeoutMs = 45_000) {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url)
      if (res.ok) return res
    } catch {
      // server not ready yet
    }
    await new Promise((r) => setTimeout(r, 500))
  }
  throw new Error(`Timed out waiting for ${url}`)
}

async function main() {
  console.log('Building app...')
  await run('pnpm', ['build'])

  console.log(`Starting preview on ${previewUrl}`)
  const preview = spawn('pnpm', ['exec', 'vite', 'preview', '--port', String(previewPort), '--strictPort'], {
    cwd: appRoot,
    stdio: 'pipe',
    shell: process.platform === 'win32',
  })

  let previewOutput = ''
  preview.stdout?.on('data', (chunk) => {
    previewOutput += chunk.toString()
  })
  preview.stderr?.on('data', (chunk) => {
    previewOutput += chunk.toString()
  })

  try {
    const res = await waitForServer(previewUrl)
    const html = await res.text()
    if (!html.includes('root') && !html.toLowerCase().includes('runestone')) {
      throw new Error('Preview page did not contain expected app shell markup')
    }
    console.log('Smoke test passed: preview server responded with app HTML')
  } finally {
    preview.kill('SIGTERM')
    await new Promise((r) => preview.on('close', r))
  }
}

main().catch((err) => {
  console.error('E2E smoke test failed:', err.message)
  process.exit(1)
})
