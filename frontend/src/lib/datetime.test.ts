import { describe, expect, it } from 'vitest'

import { formatDateTime, fromDateTimeLocalValue, toDateTimeLocalValue } from './datetime'

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

describe('toDateTimeLocalValue', () => {
  it('ISO文字列をdatetime-localの値に変換する', () => {
    expect(toDateTimeLocalValue('2026-01-01T09:00:00+09:00')).toBe('2026-01-01T09:00')
  })

  it('閲覧環境のタイムゾーンに関わらずJSTの壁時計時刻に変換する', () => {
    expect(toDateTimeLocalValue('2026-01-01T00:00:00Z')).toBe('2026-01-01T09:00')
  })

  it('JSTの0時は24時ではなく00時として扱う', () => {
    expect(toDateTimeLocalValue('2026-01-01T00:00:00+09:00')).toBe('2026-01-01T00:00')
  })

  it('未設定・解釈できない文字列は空欄として扱う', () => {
    expect(toDateTimeLocalValue(null)).toBe('')
    expect(toDateTimeLocalValue('not-a-date')).toBe('')
  })
})

describe('fromDateTimeLocalValue', () => {
  it('datetime-localの値をJSTオフセット付きのISO文字列に変換する', () => {
    expect(fromDateTimeLocalValue('2026-01-01T09:00')).toBe('2026-01-01T09:00:00+09:00')
  })

  it('秒まで入力された値もそのまま保持する', () => {
    expect(fromDateTimeLocalValue('2026-01-01T09:00:30')).toBe('2026-01-01T09:00:30+09:00')
  })

  it('空欄・未完成の入力は未設定としてnullを返す', () => {
    expect(fromDateTimeLocalValue('')).toBeNull()
    expect(fromDateTimeLocalValue('2026-01-01')).toBeNull()
  })

  it('往復変換しても同じ日時を保つ', () => {
    const isoString = '2026-01-01T09:00:00+09:00'
    expect(fromDateTimeLocalValue(toDateTimeLocalValue(isoString))).toBe(isoString)
  })
})
