function extractContent() {
  const selection = window.getSelection()
  if (selection && selection.toString().trim().length > 0) {
    const range = selection.getRangeAt(0)
    const container = document.createElement('div')
    container.appendChild(range.cloneContents())
    return container.innerHTML
  }

  const article = document.querySelector('article') || document.querySelector('main') || document.body
  const clone = article.cloneNode(true)
  const elements = clone.querySelectorAll('script, style, nav, footer, header, iframe, noscript, [aria-hidden="true"]')
  elements.forEach(el => el.remove())
  return clone.innerHTML.slice(0, 50000)
}

const result = {
  title: document.title,
  url: window.location.href,
  content: extractContent(),
  selection: window.getSelection()?.toString().trim() || ''
}

result
