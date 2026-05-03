import { describe, it, expect } from 'vitest'
import { cn } from '@/lib/utils'

describe('cn', () => {
  it('merges class names', () => {
    expect(cn('foo', 'bar')).toContain('foo')
    expect(cn('foo', 'bar')).toContain('bar')
  })

  it('handles conditional classes', () => {
    expect(cn('base', false && 'hidden')).toBe('base')
    expect(cn('base', true && 'visible')).toContain('visible')
  })

  it('merges tailwind conflicts', () => {
    const result = cn('px-2 py-1', 'px-4')
    expect(result).not.toContain('px-2')
    expect(result).toContain('px-4')
  })

  it('handles undefined and null', () => {
    expect(cn('base', undefined, null)).toBe('base')
  })

  it('handles empty inputs', () => {
    expect(cn()).toBe('')
  })

  it('handles arrays of classes', () => {
    expect(cn(['a', 'b'])).toContain('a')
    expect(cn(['a', 'b'])).toContain('b')
  })

  it('handles object syntax', () => {
    const result = cn({ active: true, inactive: false })
    expect(result).toContain('active')
    expect(result).not.toContain('inactive')
  })
})
