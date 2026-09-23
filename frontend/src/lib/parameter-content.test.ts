import { describe, expect, it } from 'vitest'

import { buildParameterContent, createEmptyFormState, toFormState } from './parameter-content'
import type { ParameterFormState } from './parameter-content'
import type { Parameter } from './trial'

const formWith = (overrides: Partial<ParameterFormState>): ParameterFormState => ({
  ...createEmptyFormState(),
  ...overrides,
})

describe('buildParameterContent', () => {
  describe('KEY_VALUE', () => {
    it('数量の値を組み立てる', () => {
      const result = buildParameterContent(
        formWith({ key: ' 強力粉 ', valueKind: 'quantity', amount: ' 300 ', unit: ' g ' }),
      )

      expect(result).toEqual({
        ok: true,
        content: {
          type: 'key_value',
          key: '強力粉',
          value: { type: 'quantity', amount: 300, unit: 'g' },
        },
      })
    })

    it('文字列の値を組み立てる', () => {
      const result = buildParameterContent(
        formWith({ key: '発酵場所', valueKind: 'text', keyValueText: '冷蔵庫' }),
      )

      expect(result).toEqual({
        ok: true,
        content: {
          type: 'key_value',
          key: '発酵場所',
          value: { type: 'text', value: '冷蔵庫' },
        },
      })
    })

    it('項目名が空の場合はエラーを返す', () => {
      const result = buildParameterContent(
        formWith({ key: '   ', valueKind: 'quantity', amount: '300', unit: 'g' }),
      )

      expect(result).toEqual({ ok: false, message: '項目名を入力してください' })
    })

    it.each([
      ['未入力', ''],
      ['0', '0'],
      ['負の値', '-1'],
      ['数値でない', 'たくさん'],
    ])('数量が%sの場合はエラーを返す', (_case, amount) => {
      const result = buildParameterContent(
        formWith({ key: '強力粉', valueKind: 'quantity', amount, unit: 'g' }),
      )

      expect(result).toEqual({
        ok: false,
        message: '数量には0より大きい数値を入力してください',
      })
    })

    it('単位が空の場合はエラーを返す', () => {
      const result = buildParameterContent(
        formWith({ key: '強力粉', valueKind: 'quantity', amount: '300', unit: ' ' }),
      )

      expect(result).toEqual({ ok: false, message: '単位を入力してください' })
    })

    it('文字列の値が空の場合はエラーを返す', () => {
      const result = buildParameterContent(
        formWith({ key: '発酵場所', valueKind: 'text', keyValueText: '  ' }),
      )

      expect(result).toEqual({ ok: false, message: '値を入力してください' })
    })
  })

  describe('DURATION / TIME_MARKER', () => {
    it('経過時間を組み立てる', () => {
      const result = buildParameterContent(
        formWith({ parameterType: 'DURATION', time: '90', timeUnit: 'minute', note: '一次発酵' }),
      )

      expect(result).toEqual({
        ok: true,
        content: {
          type: 'duration',
          duration: { value: 90, unit: 'minute' },
          note: '一次発酵',
        },
      })
    })

    it('時間マーカーを組み立てる', () => {
      const result = buildParameterContent(
        formWith({
          parameterType: 'TIME_MARKER',
          time: '30',
          timeUnit: 'minute',
          note: '焼成開始から',
        }),
      )

      expect(result).toEqual({
        ok: true,
        content: {
          type: 'time_marker',
          at: { value: 30, unit: 'minute' },
          note: '焼成開始から',
        },
      })
    })

    it('0は経過時点として有効な値とする', () => {
      const result = buildParameterContent(
        formWith({ parameterType: 'TIME_MARKER', time: '0', note: '開始時点' }),
      )

      expect(result).toEqual({
        ok: true,
        content: { type: 'time_marker', at: { value: 0, unit: 'minute' }, note: '開始時点' },
      })
    })

    it('内容が空の場合はエラーを返す', () => {
      const result = buildParameterContent(
        formWith({ parameterType: 'DURATION', time: '90', note: '  ' }),
      )

      expect(result).toEqual({ ok: false, message: '内容を入力してください' })
    })

    it.each([
      ['未入力', ''],
      ['負の値', '-1'],
      ['数値でない', 'しばらく'],
    ])('時間が%sの場合はエラーを返す', (_case, time) => {
      const result = buildParameterContent(
        formWith({ parameterType: 'DURATION', time, note: '一次発酵' }),
      )

      expect(result).toEqual({
        ok: false,
        message: '時間には0以上の数値を入力してください',
      })
    })

    it('時間マーカーのエラーメッセージは経過時点と表記する', () => {
      const result = buildParameterContent(
        formWith({ parameterType: 'TIME_MARKER', time: '', note: '焼成開始から' }),
      )

      expect(result).toEqual({
        ok: false,
        message: '経過時点には0以上の数値を入力してください',
      })
    })
  })

  describe('TEXT', () => {
    it('自由記述を組み立てる', () => {
      const result = buildParameterContent(
        formWith({ parameterType: 'TEXT', text: ' 打ち粉を追加 ' }),
      )

      expect(result).toEqual({ ok: true, content: { type: 'text', value: '打ち粉を追加' } })
    })

    it('内容が空の場合はエラーを返す', () => {
      const result = buildParameterContent(formWith({ parameterType: 'TEXT', text: '  ' }))

      expect(result).toEqual({ ok: false, message: '内容を入力してください' })
    })
  })
})

describe('toFormState', () => {
  it.each<[string, Parameter, Partial<ParameterFormState>]>([
    [
      '数量のKeyValue',
      {
        id: 'p1',
        parameterType: 'KEY_VALUE',
        content: {
          type: 'key_value',
          key: '強力粉',
          value: { type: 'quantity', amount: 300, unit: 'g' },
        },
      },
      { parameterType: 'KEY_VALUE', key: '強力粉', valueKind: 'quantity', amount: '300', unit: 'g' },
    ],
    [
      '文字列のKeyValue',
      {
        id: 'p2',
        parameterType: 'KEY_VALUE',
        content: { type: 'key_value', key: '発酵場所', value: { type: 'text', value: '冷蔵庫' } },
      },
      { parameterType: 'KEY_VALUE', key: '発酵場所', valueKind: 'text', keyValueText: '冷蔵庫' },
    ],
    [
      '経過時間',
      {
        id: 'p3',
        parameterType: 'DURATION',
        content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
      },
      { parameterType: 'DURATION', time: '90', timeUnit: 'minute', note: '一次発酵' },
    ],
    [
      '時間マーカー',
      {
        id: 'p4',
        parameterType: 'TIME_MARKER',
        content: { type: 'time_marker', at: { value: 2, unit: 'hour' }, note: '焼成開始から' },
      },
      { parameterType: 'TIME_MARKER', time: '2', timeUnit: 'hour', note: '焼成開始から' },
    ],
    [
      '自由記述',
      { id: 'p5', parameterType: 'TEXT', content: { type: 'text', value: '打ち粉を追加' } },
      { parameterType: 'TEXT', text: '打ち粉を追加' },
    ],
  ])('%sを編集フォームの初期状態に変換する', (_case, parameter, expected) => {
    expect(toFormState(parameter)).toMatchObject(expected)
  })

  it('変換した状態はそのまま元のcontentへ戻せる', () => {
    const parameter: Parameter = {
      id: 'p1',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
    }

    expect(buildParameterContent(toFormState(parameter))).toEqual({
      ok: true,
      content: parameter.content,
    })
  })
})
