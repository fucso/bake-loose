-- parameters テーブルを作成する
-- Parameter は Step に紐づくパラメーターを表す

CREATE TABLE parameters (
    id UUID PRIMARY KEY,
    step_id UUID NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    content JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- ステップ別の Parameter 一覧取得・カスケード削除用インデックス
CREATE INDEX idx_parameters_step_id ON parameters(step_id);
