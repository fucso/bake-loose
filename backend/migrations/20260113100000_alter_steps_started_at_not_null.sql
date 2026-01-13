-- started_at を NOT NULL に変更
-- 既存データがある場合は created_at の値で埋める

UPDATE steps SET started_at = created_at WHERE started_at IS NULL;

ALTER TABLE steps ALTER COLUMN started_at SET NOT NULL;
