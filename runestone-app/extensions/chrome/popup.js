document.getElementById('port').addEventListener('change', (e) => {
  chrome.storage.local.set({ port: parseInt(e.target.value) || 9876 })
})

chrome.storage.local.get(['port'], (r) => {
  if (r.port) document.getElementById('port').value = r.port
})

document.getElementById('clipBtn').addEventListener('click', async () => {
  const status = document.getElementById('status')
  status.textContent = 'Clipping...'
  status.className = ''

  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true })
    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => {
        const sel = window.getSelection()?.toString().trim()
        const article = document.querySelector('article') || document.querySelector('main') || document.body
        const clone = article.cloneNode(true)
        clone.querySelectorAll('script, style, nav, footer, header, iframe, noscript').forEach(el => el.remove())
        return {
          title: document.title,
          url: window.location.href,
          content: sel || clone.innerHTML.slice(0, 50000)
        }
      }
    })

    const data = results[0]?.result
    if (!data) throw new Error('No content extracted')

    chrome.runtime.sendMessage({ action: 'clip', ...data }, (response) => {
      if (response?.success) {
        status.textContent = 'Clipped successfully!'
        status.className = 'success'
        setTimeout(() => window.close(), 1500)
      } else {
        status.textContent = 'Failed: ' + (response?.error || 'unknown error')
        status.className = 'error'
      }
    })
  } catch (e) {
    status.textContent = 'Error: ' + e.message
    status.className = 'error'
  }
})
