-- trials テーブルを作成する
-- Trial は一連の調理工程（例: ピザ生地 v1）を表す

CREATE TABLE trials (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress',
    memo TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- プロジェクトIDによる検索を高速化するためのインデックス
CREATE INDEX idx_trials_project_id ON trials(project_id);
