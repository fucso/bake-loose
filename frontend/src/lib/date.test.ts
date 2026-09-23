import { describe, expect, it } from 'vitest'

import { formatDateTime } from './date'

describe('formatDateTime', () => {
  it('JSTのISO 8601文字列を表示用の文字列に整形する', () => {
    expect(formatDateTime('2026-03-01T09:30:00+09:00')).toBe('2026/03/01 09:30')
  })

  it('UTC表記の日時をJSTに変換して整形する', () => {
    expect(formatDateTime('2026-03-01T00:30:00Z')).toBe('2026/03/01 09:30')
  })

  it('パースできない文字列の場合はnullを返す', () => {
    expect(formatDateTime('not-a-date')).toBeNull()
  })
})
