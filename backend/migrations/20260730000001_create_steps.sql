-- steps テーブルを作成する
-- Step は Trial を構成する工程を表す

CREATE TABLE steps (
    id UUID PRIMARY KEY,
    trial_id UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    position SMALLINT NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (trial_id, position)
);
