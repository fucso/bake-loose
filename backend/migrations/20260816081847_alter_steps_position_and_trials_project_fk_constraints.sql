-- steps テーブルの UNIQUE (trial_id, position) 制約を DEFERRABLE にする
-- 将来の Step 並び替え機能で、同一トランザクション内で position の一時的な重複を許容できるようにする
ALTER TABLE steps DROP CONSTRAINT steps_trial_id_position_key;
ALTER TABLE steps
    ADD CONSTRAINT steps_trial_id_position_key
    UNIQUE (trial_id, position) DEFERRABLE INITIALLY IMMEDIATE;

-- trials.project_id の外部キーに ON DELETE 挙動を明示する
-- Project の削除アクションは現状未実装のため、既存の暗黙的な NO ACTION 相当の挙動を変更せずに
-- RESTRICT として明示する（Project 削除機能の実装時に改めて挙動を検討する）
ALTER TABLE trials DROP CONSTRAINT trials_project_id_fkey;
ALTER TABLE trials
    ADD CONSTRAINT trials_project_id_fkey
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT;
