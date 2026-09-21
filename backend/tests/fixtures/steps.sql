-- テスト用Step / Parameter（trials.sql の Trial 1 に紐づく）
--
-- trials.sql を使う既存テストの期待値を壊さないようフィクスチャを分離している。
-- 利用するテストのみ fixtures("projects.sql", "trials.sql", "steps.sql") の順で読み込むこと。
--
-- Step は position 昇順、Parameter は投入順（= id 昇順）で取得される前提で
-- 完全なJSONによる検証ができるよう、固定UUID・固定日時のみを使用する。
--
-- Parameter は parameterType の全バリアント（text / key_value / duration / time_marker）を含める。

INSERT INTO steps (id, trial_id, name, position, started_at, completed_at, created_at, updated_at)
VALUES
    ('77777777-7777-7777-7777-777777777777', '33333333-3333-3333-3333-333333333333', 'こね', 0, '2026-01-01T09:00:00+09:00', '2026-01-01T09:30:00+09:00', NOW(), NOW()),
    ('88888888-8888-8888-8888-888888888888', '33333333-3333-3333-3333-333333333333', '一次発酵', 1, NULL, NULL, NOW(), NOW());

INSERT INTO parameters (id, step_id, content, created_at, updated_at)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1', '77777777-7777-7777-7777-777777777777', '{"type": "text", "value": "打ち粉を追加"}'::jsonb, NOW(), NOW()),
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2', '77777777-7777-7777-7777-777777777777', '{"type": "key_value", "key": "強力粉", "value": {"type": "quantity", "amount": 300, "unit": "g"}}'::jsonb, NOW(), NOW()),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1', '88888888-8888-8888-8888-888888888888', '{"type": "duration", "duration": {"value": 90, "unit": "minute"}, "note": "一次発酵"}'::jsonb, NOW(), NOW()),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2', '88888888-8888-8888-8888-888888888888', '{"type": "time_marker", "at": {"value": 60, "unit": "minute"}, "note": "生地の膨らみを確認"}'::jsonb, NOW(), NOW());
