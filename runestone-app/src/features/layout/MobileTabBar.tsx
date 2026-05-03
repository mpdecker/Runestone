import { LayoutGrid, FileText, Search, Settings } from 'lucide-react'

type Tab = 'graph' | 'notes' | 'search' | 'settings'

interface MobileTabBarProps {
  activeTab: Tab
  onTabChange: (tab: Tab) => void
}

const tabs: { key: Tab; label: string; icon: React.ReactNode }[] = [
  { key: 'graph', label: 'Graph', icon: <LayoutGrid className="w-5 h-5" /> },
  { key: 'notes', label: 'Notes', icon: <FileText className="w-5 h-5" /> },
  { key: 'search', label: 'Search', icon: <Search className="w-5 h-5" /> },
  { key: 'settings', label: 'Settings', icon: <Settings className="w-5 h-5" /> },
]

export function MobileTabBar({ activeTab, onTabChange }: MobileTabBarProps) {
  return (
    <nav
      className="flex items-center justify-around border-t bg-card shrink-0"
      style={{
        height: '56px',
        paddingBottom: 'env(safe-area-inset-bottom, 0px)',
      }}
    >
      {tabs.map(({ key, label, icon }) => (
        <button
          key={key}
          onClick={() => onTabChange(key)}
          className={`flex flex-col items-center justify-center gap-0.5 flex-1 h-full min-h-[44px] min-w-[44px] ${
            activeTab === key
              ? 'text-primary'
              : 'text-muted-foreground'
          }`}
        >
          {icon}
          <span className="text-[10px] font-medium">{label}</span>
        </button>
      ))}
    </nav>
  )
}
