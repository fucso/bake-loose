# ===== 開発環境用ステージ =====
FROM node:22-alpine AS development

WORKDIR /app

# pnpm を有効化
RUN corepack enable && corepack prepare pnpm@10 --activate

# 依存関係ファイルをコピー
COPY package.json pnpm-lock.yaml* ./

# 依存関係をインストール
RUN pnpm install

EXPOSE 3000

# Vite 開発サーバー（ホットリロード）
CMD ["pnpm", "dev", "--host", "0.0.0.0"]
