-- steps テーブルを作成する
-- Step は Trial 内の各工程（例: 生地作り、一次発酵）を表す

CREATE TABLE steps (
    id UUID PRIMARY KEY,
    trial_id UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    name VARCHAR(100),
    position INTEGER NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(trial_id, position)
);

-- trial_id による検索を高速化するためのインデックス
CREATE INDEX idx_steps_trial_id ON steps(trial_id);
