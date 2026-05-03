chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'clip') {
    chrome.storage.local.get(['port'], (result) => {
      const port = result.port || 9876
      fetch(`http://localhost:${port}/clip`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
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
