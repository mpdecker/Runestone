import { invoke } from '@tauri-apps/api/core'

let cached: 'ios' | 'android' | 'desktop' | null = null

export async function detectPlatform(): Promise<'ios' | 'android' | 'desktop'> {
  if (cached) return cached
  try {
    const platform: string = await invoke('get_platform')
    if (platform === 'ios') cached = 'ios'
    else if (platform === 'android') cached = 'android'
    else cached = 'desktop'
  } catch {
    cached = 'desktop'
  }
  return cached
}

export function isMobile(): boolean {
  return cached === 'ios' || cached === 'android'
}

export function getPlatform(): 'ios' | 'android' | 'desktop' {
  return cached ?? 'desktop'
}
