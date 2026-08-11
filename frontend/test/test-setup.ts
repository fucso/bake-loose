import { afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import '@testing-library/jest-dom/vitest'

// 各テスト後にレンダリング結果をアンマウントし、DOMをクリーンな状態に戻す
// また vi.stubGlobal 等で差し替えたグローバルも元に戻し、テスト間の状態漏れを防ぐ
afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})
