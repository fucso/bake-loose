-- テスト用Trial（projects.sql の Project に紐づく）
INSERT INTO trials (id, project_id, name, memo, status, created_at, updated_at)
VALUES
    ('33333333-3333-3333-3333-333333333333', '11111111-1111-1111-1111-111111111111', 'Test Trial 1', 'Test Memo', 'in_progress', NOW(), NOW()),
    ('44444444-4444-4444-4444-444444444444', '11111111-1111-1111-1111-111111111111', 'Test Trial 2', NULL, 'in_progress', NOW(), NOW()),
    ('66666666-6666-6666-6666-666666666666', '11111111-1111-1111-1111-111111111111', 'Completed Trial', NULL, 'completed', NOW(), NOW()),
    ('55555555-5555-5555-5555-555555555555', '22222222-2222-2222-2222-222222222222', 'Other Project Trial', NULL, 'in_progress', NOW(), NOW());
