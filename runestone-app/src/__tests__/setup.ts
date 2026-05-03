import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockRejectedValue(new Error('Tauri API not available in tests')),
}))

vi.mock('@tauri-apps/plugin-os', () => ({
  getPlatform: vi.fn().mockResolvedValue('linux'),
}))
