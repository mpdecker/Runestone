import { useStore } from '@/store'

export function TabBar() {
  const { openTabs, activeTabId, switchToTab, closeTab } = useStore()

  if (openTabs.length === 0) return null

  return (
    <div className="flex items-center border-b bg-card overflow-x-auto shrink-0">
      {openTabs.map((tab) => (
        <div
          key={tab.id}
          className={`group flex items-center gap-1 px-3 py-1.5 text-xs cursor-pointer border-r whitespace-nowrap transition-colors ${
            activeTabId === tab.id
              ? 'bg-background text-foreground border-b-2 border-b-primary'
              : 'text-muted-foreground hover:bg-muted'
          }`}
          onClick={() => switchToTab(tab.id)}
        >
          <span className="truncate max-w-[120px]">{tab.title}</span>
          <button
            className="text-[10px] text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 shrink-0"
            onClick={(e) => {
              e.stopPropagation()
              closeTab(tab.id)
            }}
            title="Close tab"
          >
            &times;
          </button>
        </div>
      ))}
    </div>
  )
}
