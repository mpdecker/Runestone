/**
 * Optional server integration smoke test.
 * Skips gracefully when RUNESTONE_SERVER_URL is unset or server is unreachable.
 */
const serverUrl = process.env.RUNESTONE_SERVER_URL ?? 'http://127.0.0.1:3000'
const apiToken = process.env.RUNESTONE_API_TOKEN ?? ''

async function waitForHealth(url, timeoutMs = 15_000) {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`${url}/api/health`, {
        headers: apiToken ? { Authorization: `Bearer ${apiToken}` } : {},
      })
      if (res.ok) return
    } catch {
      // not ready
    }
    await new Promise((r) => setTimeout(r, 500))
  }
  throw new Error(`Server health check failed for ${url}`)
}

async function main() {
  if (process.env.SKIP_SERVER_E2E === '1') {
    console.log('Skipping server integration test (SKIP_SERVER_E2E=1)')
    return
  }

  try {
    await waitForHealth(serverUrl)
  } catch (err) {
    console.log(`Server integration skipped: ${err.message}`)
    return
  }

  const headers = { 'Content-Type': 'application/json' }
  if (apiToken) headers.Authorization = `Bearer ${apiToken}`

  const res = await fetch(`${serverUrl}/api/invoke/list_vaults`, {
    method: 'POST',
    headers,
    body: '{}',
  })

  if (res.status === 401) {
    throw new Error('list_vaults returned 401 — check RUNESTONE_API_TOKEN')
  }

  if (!res.ok) {
    const text = await res.text()
    throw new Error(`list_vaults failed (${res.status}): ${text}`)
  }

  const body = await res.json()
  if (!Array.isArray(body)) {
    throw new Error('list_vaults response was not an array')
  }

  console.log(`Server integration passed: list_vaults returned ${body.length} vault(s)`)
}

main().catch((err) => {
  console.error('Server integration test failed:', err.message)
  process.exit(1)
})
