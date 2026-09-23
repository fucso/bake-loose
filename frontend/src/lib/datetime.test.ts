import { describe, expect, it } from 'vitest'

import { formatDateTime } from './datetime'

describe('formatDateTime', () => {
  it('JSTオフセット付きのISO文字列を日時表記に整形する', () => {
    expect(formatDateTime('2026-01-01T09:00:00+09:00')).toBe('2026/01/01 09:00')
  })

  it('閲覧環境のタイムゾーンに関わらずJSTで表示する', () => {
    // UTC 表記の同時刻も JST に変換して表示する
    expect(formatDateTime('2026-01-01T00:00:00Z')).toBe('2026/01/01 09:00')
  })

  it('解釈できない文字列は元の文字列を返す', () => {
    expect(formatDateTime('not-a-date')).toBe('not-a-date')
  })
})
