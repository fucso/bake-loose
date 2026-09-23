//! ビルドスクリプト
//!
//! `sqlx::migrate!` / `#[sqlx::test(migrations = "./migrations")]` は
//! コンパイル時にマイグレーション SQL を実行ファイルへ埋め込む。
//! マイグレーションを **新規追加** しただけでは Rust のソースが変わらないため、
//! これがないと cargo が再コンパイルを行わず、新しいマイグレーションを含まない
//! 古いテストバイナリがそのまま実行されてしまう。
//!
//! `migrations` ディレクトリを再ビルドのトリガーとして登録することで、
//! マイグレーション追加時にも必ずテストバイナリが作り直され、
//! `backend/tests/schema_naming.rs` のスキーマ命名規約検証が新しい SQL に対して働く。

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
