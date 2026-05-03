import { useState, useEffect } from 'react'
import { DesktopApp } from '@/features/layout/DesktopApp'
import { MobileApp } from '@/features/layout/MobileApp'
import { detectPlatform } from '@/lib/platform'

function App() {
  const [platform, setPlatform] = useState<'ios' | 'android' | 'desktop' | null>(null)

  useEffect(() => {
    detectPlatform().then(setPlatform)
  }, [])

  if (!platform) {
    return (
      <div className="h-screen flex items-center justify-center bg-background text-foreground">
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
          <p className="text-sm text-muted-foreground">Loading Runestone...</p>
        </div>
      </div>
    )
  }

  if (platform === 'ios' || platform === 'android') {
    return <MobileApp />
  }

  return <DesktopApp />
}

export default App
