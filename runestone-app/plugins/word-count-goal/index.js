var totalWords = 0
var goalWords = 500
var panelEl = null

function renderPanel(container) {
  panelEl = document.createElement('div')
  panelEl.style.padding = '8px'
  panelEl.style.borderTop = '1px solid var(--border)'

  var title = document.createElement('p')
  title.textContent = 'WORD GOAL'
  title.style.cssText = 'font-size:10px;text-transform:uppercase;color:var(--muted-foreground);margin-bottom:4px'
  panelEl.appendChild(title)

  var progress = document.createElement('div')
  progress.style.cssText = 'height:6px;background:var(--muted);border-radius:3px;overflow:hidden;margin-bottom:4px'
  var bar = document.createElement('div')
  var pct = Math.min((totalWords / goalWords) * 100, 100)
  bar.style.cssText = 'height:100%;background:var(--primary);width:' + pct + '%'
  progress.appendChild(bar)
  panelEl.appendChild(progress)

  var info = document.createElement('p')
  info.textContent = totalWords + ' / ' + goalWords + ' words'
  info.style.cssText = 'font-size:10px;color:var(--muted-foreground)'
  panelEl.appendChild(info)

  var goalInput = document.createElement('input')
  goalInput.type = 'number'
  goalInput.value = goalWords
  goalInput.style.cssText = 'width:100%;padding:2px 4px;font-size:9px;border:1px solid var(--border);border-radius:3px;background:var(--background);margin-top:4px'
  goalInput.placeholder = 'Word goal per session'
  goalInput.addEventListener('change', function() {
    goalWords = parseInt(goalInput.value) || 500
    renderPanel(container)
  })
  panelEl.appendChild(goalInput)

  container.innerHTML = ''
  container.appendChild(panelEl)
}

exports.activate = function(api) {
  var currentNode = api.store.currentNode || { word_count: 0 }
  totalWords = currentNode.word_count || 0

  api.hooks.on('node-saved', function() {
    var node = api.store.currentNode || { word_count: 0 }
    totalWords = node.word_count || 0
    if (panelEl) renderPanel(panelEl.parentElement || document.body)
  })

  var sidebar = document.querySelector('[data-testid="sidebar"]')
  var container = document.createElement('div')
  container.id = 'plugin-word-count-goal'
  sidebar.appendChild(container)
  renderPanel(container)
  panelEl = container
}

exports.deactivate = function() {
  var el = document.getElementById('plugin-word-count-goal')
  if (el) el.remove()
  panelEl = null
}
