-- trials テーブルに completed_at カラムを追加する
-- Trial の完了日時を記録する（Step.completed_at と同様、JSTに正規化された日時をUTCで保持する）

ALTER TABLE trials ADD COLUMN completed_at TIMESTAMP WITH TIME ZONE;
