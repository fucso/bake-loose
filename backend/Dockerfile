# ===== 開発環境用ステージ =====
FROM rust:1.92-slim-bookworm AS development

WORKDIR /app

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Rust コンポーネント（linter / formatter）
RUN rustup component add clippy rustfmt

# 開発用ツール
RUN cargo install cargo-watch
RUN cargo install sqlx-cli --no-default-features --features postgres

# ソースコードはボリュームマウントされる
# ホットリロードで起動
CMD ["cargo", "watch", "-x", "run"]

# ===== 依存関係キャッシュ用ステージ =====
FROM rust:1.92-slim-bookworm AS chef

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef
WORKDIR /app

# ===== 依存関係の計画 =====
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ===== ビルド用ステージ =====
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# 依存関係のみをビルド（キャッシュ効率化）
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

# ===== 本番環境用ステージ =====
FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bake-loose /usr/local/bin/

EXPOSE 8080

CMD ["bake-loose"]
