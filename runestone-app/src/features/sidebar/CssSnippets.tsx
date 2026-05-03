import { useState } from 'react'
import { useStore } from '@/store'

export function CssSnippets() {
  const { userCss, setUserCss } = useStore()
  const [open, setOpen] = useState(false)

  return (
    <div className="border-t p-2 space-y-1">
      <button
        className="w-full text-left text-[10px] text-muted-foreground uppercase tracking-wider hover:text-foreground"
        onClick={() => setOpen(!open)}
      >
        {open ? '\u25BC' : '\u25B6'} Custom CSS
      </button>
      {open && (
        <textarea
          className="w-full h-24 text-[10px] font-mono p-1 border rounded bg-background resize-y"
          placeholder="/* Custom CSS snippets */"
          value={userCss}
          onChange={(e) => setUserCss(e.target.value)}
          spellCheck={false}
        />
      )}
    </div>
  )
}
