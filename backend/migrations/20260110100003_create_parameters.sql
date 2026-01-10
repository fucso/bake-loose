-- parameters テーブルを作成する
-- Parameter は Step ごとのパラメータ（例: 粉の量、発酵時間）を表す

CREATE TABLE parameters (
    id UUID PRIMARY KEY,
    step_id UUID NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    content_type VARCHAR(20) NOT NULL,
    content JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- step_id による検索を高速化するためのインデックス
CREATE INDEX idx_parameters_step_id ON parameters(step_id);
