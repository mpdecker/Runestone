import { describe, it, expect, vi, beforeEach } from 'vitest'

const mockInvoke = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

describe('platform', () => {
  beforeEach(async () => {
    vi.clearAllMocks()
    vi.resetModules()
  })

  describe('detectPlatform', () => {
    it('returns desktop when invoke returns desktop platform', async () => {
      mockInvoke.mockResolvedValue('windows')
      const { detectPlatform } = await import('@/lib/platform')

      const platform = await detectPlatform()

      expect(platform).toBe('desktop')
      expect(mockInvoke).toHaveBeenCalledWith('get_platform')
    })

    it('returns ios when invoke returns ios', async () => {
      mockInvoke.mockResolvedValue('ios')
      const { detectPlatform } = await import('@/lib/platform')

      const platform = await detectPlatform()

      expect(platform).toBe('ios')
    })

    it('returns android when invoke returns android', async () => {
      mockInvoke.mockResolvedValue('android')
      const { detectPlatform } = await import('@/lib/platform')

      const platform = await detectPlatform()

      expect(platform).toBe('android')
    })

    it('returns desktop when invoke throws', async () => {
      mockInvoke.mockRejectedValue(new Error('not available'))
      const { detectPlatform } = await import('@/lib/platform')

      const platform = await detectPlatform()

      expect(platform).toBe('desktop')
    })

    it('caches platform after first call', async () => {
      mockInvoke.mockResolvedValue('macos')
      const { detectPlatform } = await import('@/lib/platform')

      await detectPlatform()
      await detectPlatform()

      expect(mockInvoke).toHaveBeenCalledTimes(1)
    })
  })

  describe('isMobile', () => {
    it('returns true after detecting ios', async () => {
      mockInvoke.mockResolvedValue('ios')
      const { detectPlatform, isMobile } = await import('@/lib/platform')

      await detectPlatform()

      expect(isMobile()).toBe(true)
    })

    it('returns true after detecting android', async () => {
      mockInvoke.mockResolvedValue('android')
      const { detectPlatform, isMobile } = await import('@/lib/platform')

      await detectPlatform()

      expect(isMobile()).toBe(true)
    })

    it('returns false after detecting desktop', async () => {
      mockInvoke.mockResolvedValue('linux')
      const { detectPlatform, isMobile } = await import('@/lib/platform')

      await detectPlatform()

      expect(isMobile()).toBe(false)
    })
  })

  describe('getPlatform', () => {
    it('returns cached platform after detection', async () => {
      mockInvoke.mockResolvedValue('macos')
      const { detectPlatform, getPlatform } = await import('@/lib/platform')

      await detectPlatform()

      expect(getPlatform()).toBe('desktop')
    })

    it('returns desktop when no detection has run', async () => {
      const { getPlatform } = await import('@/lib/platform')

      expect(getPlatform()).toBe('desktop')
    })
  })
})
