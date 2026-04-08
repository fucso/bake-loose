-- trials, steps, parameters テーブルを作成する
-- Trial は Project に紐づく試行記録を表す
-- Step は Trial を構成する工程を表す
-- Parameter は Step に紐づくパラメーターを表す

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

CREATE TABLE steps (
    id UUID PRIMARY KEY,
    trial_id UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    position INTEGER NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (trial_id, position)
);

CREATE TABLE parameters (
    id UUID PRIMARY KEY,
    step_id UUID NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    content JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
