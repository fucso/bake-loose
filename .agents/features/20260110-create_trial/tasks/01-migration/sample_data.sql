-- テスト用サンプルデータ
-- trials, steps, parameters テーブルのテストに使用

-- 前提: projects テーブルにテスト用プロジェクトが存在すること
-- INSERT INTO projects (id, name) VALUES ('11111111-1111-1111-1111-111111111111', 'バゲット研究');

-- Trial 1: 進行中の試行
INSERT INTO trials (id, project_id, status, memo) VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'in_progress', '初めてのバゲット');

-- Trial 1 の Steps
INSERT INTO steps (id, trial_id, name, position, started_at) VALUES
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'こね', 0, '2026-01-10 10:00:00+09'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb02', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '一次発酵', 1, '2026-01-10 10:15:00+09'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb03', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '焼成', 2, NULL);

-- Step 1 (こね) の Parameters
-- 順序は created_at でソートされる（position カラムなし）
INSERT INTO parameters (id, step_id, content_type, content) VALUES
    -- 粉: 300g
    ('cccccccc-cccc-cccc-cccc-cccccccccc01', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01', 'key_value',
     '{"key": "粉", "value": {"type": "quantity", "amount": 300, "unit": "gram"}}'),
    -- 水: 195g (65%)
    ('cccccccc-cccc-cccc-cccc-cccccccccc02', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01', 'key_value',
     '{"key": "水", "value": {"type": "quantity", "amount": 195, "unit": "gram"}}'),
    -- 水温: 28℃
    ('cccccccc-cccc-cccc-cccc-cccccccccc03', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01', 'key_value',
     '{"key": "水温", "value": {"type": "quantity", "amount": 28, "unit": "celsius"}}'),
    -- こね時間: 15分
    ('cccccccc-cccc-cccc-cccc-cccccccccc04', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01', 'duration_range',
     '{"duration_seconds": 900, "display_unit": "minute", "note": null}');

-- Step 2 (一次発酵) の Parameters
INSERT INTO parameters (id, step_id, content_type, content) VALUES
    -- 発酵時間: 90分
    ('cccccccc-cccc-cccc-cccc-cccccccccc05', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb02', 'duration_range',
     '{"duration_seconds": 5400, "display_unit": "minute", "note": "室温25度"}'),
    -- テキストメモ
    ('cccccccc-cccc-cccc-cccc-cccccccccc06', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb02', 'text',
     '{"value": "パンチは30分後と60分後に実施"}');

-- Step 3 (焼成) の Parameters
INSERT INTO parameters (id, step_id, content_type, content) VALUES
    -- 初期温度: 250℃
    ('cccccccc-cccc-cccc-cccc-cccccccccc07', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb03', 'key_value',
     '{"key": "初期温度", "value": {"type": "quantity", "amount": 250, "unit": "celsius"}}'),
    -- 焼成時間: 25分
    ('cccccccc-cccc-cccc-cccc-cccccccccc08', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb03', 'duration_range',
     '{"duration_seconds": 1500, "display_unit": "minute", "note": null}'),
    -- 10分後に温度変更
    ('cccccccc-cccc-cccc-cccc-cccccccccc09', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb03', 'time_point',
     '{"elapsed_seconds": 600, "display_unit": "minute", "note": "温度を230度に変更"}'),
    -- 15分後に天板入れ替え
    ('cccccccc-cccc-cccc-cccc-cccccccccc10', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb03', 'time_point',
     '{"elapsed_seconds": 900, "display_unit": "minute", "note": "天板の上下を入れ替え"}');

-- Trial 2: 完了済みの試行
INSERT INTO trials (id, project_id, status, memo) VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab', '11111111-1111-1111-1111-111111111111', 'completed', '2回目の試行');

INSERT INTO steps (id, trial_id, name, position, started_at) VALUES
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb04', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab', NULL, 0, '2026-01-09 09:00:00+09');

INSERT INTO parameters (id, step_id, content_type, content) VALUES
    ('cccccccc-cccc-cccc-cccc-cccccccccc11', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb04', 'text',
     '{"value": "シンプルな記録"}');
