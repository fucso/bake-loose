import { describe, expect, it } from 'vitest'

import { formatParameter } from './parameter-format'
import type { Parameter } from './trial'

describe('formatParameter', () => {
  it('KeyValue(quantity)はキーをラベル、数量と単位を値にする', () => {
    const parameter: Parameter = {
      id: '1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    }

    expect(formatParameter(parameter)).toEqual({ label: '強力粉', value: '300g' })
  })

  it('KeyValue(text)はキーをラベル、テキストを値にする', () => {
    const parameter: Parameter = {
      id: '2',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '発酵場所',
        value: { type: 'text', value: '冷蔵庫' },
      },
    }

    expect(formatParameter(parameter)).toEqual({ label: '発酵場所', value: '冷蔵庫' })
  })

  it('Durationはnoteをラベル、単位を日本語化した時間量を値にする', () => {
    const parameter: Parameter = {
      id: '3',
      parameterType: 'DURATION',
      content: {
        type: 'duration',
        duration: { value: 90, unit: 'minute' },
        note: '一次発酵',
      },
    }

    expect(formatParameter(parameter)).toEqual({ label: '一次発酵', value: '90分' })
  })

  it('TimeMarkerはnoteをラベル、時点であることが分かる値にする', () => {
    const parameter: Parameter = {
      id: '4',
      parameterType: 'TIME_MARKER',
      content: {
        type: 'time_marker',
        at: { value: 30, unit: 'minute' },
        note: '焼成開始から',
      },
    }

    expect(formatParameter(parameter)).toEqual({ label: '焼成開始から', value: '30分時点' })
  })

  it('Textはラベルを持たない', () => {
    const parameter: Parameter = {
      id: '5',
      parameterType: 'TEXT',
      content: { type: 'text', value: '生地がべたつく場合は打ち粉を追加' },
    }

    expect(formatParameter(parameter)).toEqual({
      label: null,
      value: '生地がべたつく場合は打ち粉を追加',
    })
  })

  it('型にない種別が届いても内容を落とさずに表示する', () => {
    // バックエンドに種別が追加された場合を模した、フロントの型にはない値
    const unknown = {
      id: '99',
      parameterType: 'HUMIDITY',
      content: { type: 'humidity', percent: 60 },
    } as unknown as Parameter

    expect(formatParameter(unknown)).toEqual({
      label: null,
      value: '{"type":"humidity","percent":60}',
    })
  })

  it('noteが空のDuration/TimeMarkerは種別名をラベルに使う', () => {
    const duration: Parameter = {
      id: '6',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 2, unit: 'hour' }, note: '  ' },
    }
    const timeMarker: Parameter = {
      id: '7',
      parameterType: 'TIME_MARKER',
      content: { type: 'time_marker', at: { value: 1, unit: 'day' }, note: '' },
    }

    expect(formatParameter(duration)).toEqual({ label: '経過時間', value: '2時間' })
    expect(formatParameter(timeMarker)).toEqual({ label: '時間マーカー', value: '1日時点' })
  })
})
