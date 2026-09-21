-- trials テーブルを作成する
-- Trial は Project に紐づく試行記録を表す

CREATE TABLE trials (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id),
    name VARCHAR(100),
    memo TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- プロジェクト別の Trial 一覧取得用インデックス
CREATE INDEX idx_trials_project_id ON trials(project_id);
