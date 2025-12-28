-- テスト用データベースを作成
-- PostgreSQLコンテナ起動時に自動実行される

CREATE DATABASE bakeloose_test;

-- テスト用データベースに対して同じユーザーに権限を付与
GRANT ALL PRIVILEGES ON DATABASE bakeloose_test TO bakeloose;
