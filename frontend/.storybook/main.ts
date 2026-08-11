import type { StorybookConfig } from '@storybook/react-vite'

type VitePlugin = { name?: string } | VitePlugin[]

// PWA用のService Worker生成（vite-plugin-pwa）はStorybookのバンドル（manager等）を
// precacheマニフェストの対象に含めてしまい、workboxのサイズ上限を超えてビルドが失敗するため、
// Storybookのビルドでは除外する。
const excludePwaPlugin = (plugins: VitePlugin[]): VitePlugin[] =>
  plugins
    .flatMap((plugin) => (Array.isArray(plugin) ? excludePwaPlugin(plugin) : plugin))
    .filter((plugin) => !plugin?.name?.startsWith('vite-plugin-pwa'))

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
  addons: [],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
  async viteFinal(config) {
    return {
      ...config,
      plugins: excludePwaPlugin(config.plugins ?? []),
    }
  },
}

export default config
