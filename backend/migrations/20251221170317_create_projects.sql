-- projects テーブルを作成する
-- Project は調理テーマ（例: ピザ生地研究、カンパーニュ）を表す

CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 名前の重複を防ぐユニークインデックス
CREATE UNIQUE INDEX idx_projects_name ON projects(name);
