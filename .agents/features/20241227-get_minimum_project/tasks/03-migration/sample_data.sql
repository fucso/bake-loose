-- サンプルデータ投入用SQL
-- マイグレーション実行後、手動で実行してください
--
-- 使用方法:
-- psql -h localhost -U bakeloose -d bakeloose -f sample_data.sql
-- または
-- docker compose exec db psql -U bakeloose -d bakeloose -c "$(cat sample_data.sql)"

INSERT INTO projects (id, name) VALUES
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'ピザ生地研究'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'カンパーニュ'),
  ('c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'フォカッチャ')
ON CONFLICT (id) DO NOTHING;
