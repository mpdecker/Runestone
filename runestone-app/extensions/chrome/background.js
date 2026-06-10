chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'clip') {
    chrome.storage.local.get(['port', 'clipperToken'], (result) => {
      const port = result.port || 9876
      const headers = { 'Content-Type': 'application/json' }
      if (result.clipperToken) {
        headers['X-Clipper-Token'] = result.clipperToken
      }
      fetch(`http://localhost:${port}/clip`, {
        method: 'POST',
        headers,
        body: JSON.stringify({
          title: request.title,
          url: request.url,
          content: request.content
        })
      })
      .then(r => r.json())
      .then(data => {
        sendResponse({ success: true, id: data.id })
      })
      .catch(err => {
        sendResponse({ success: false, error: err.message })
      })
    })
    return true // keep message channel open for async response
  }
})
