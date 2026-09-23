# Spec: Trial 詳細（記録ワークベンチ）

> Feature Issue: #83, #85

## 概要

1 つの Trial（試行）に紐づく Step（工程）と Parameter（記録要素）を時系列で俯瞰し、
Trial 自体の編集・完了を行う画面。Trial 機能の中心となる画面であり、
調理中の記録操作はこの画面の上で完結させることを前提に情報設計している。

## 要件

### ルーティング

| パス | 画面 |
|------|------|
| `/projects/:id/trials/:trialId` | Trial 詳細（記録ワークベンチ） |

`:id` はプロジェクト ID、`:trialId` は Trial ID。「← 戻る」で `/projects/:id` に戻る。

### 表示内容

- ヘッダ: Trial の `name`・`memo`・`status` バッジ・`completedAt`（完了済みの場合）
  - `name` は任意項目のため、未設定時は「名称未設定の試行」を表示する
- タイムライン: Step を `position` 昇順に並べたカード列。各カードに配下の Parameter を表示する

### 操作

| 操作 | 使用する GraphQL |
|------|-----------------|
| name / memo の編集 | `updateTrial(id, input: UpdateTrialInput)` |
| Trial の完了 | `completeTrial(id)` |
| Parameter の追加 | `addParameter(trialId, stepId, content)` |
| Parameter の編集 | `updateParameter(trialId, stepId, parameterId, content)` |
| Parameter の削除 | `removeParameter(trialId, stepId, parameterId)` |

Step 自体の記録操作（追加・編集・完了）は本画面のスコープ外で、
このワークベンチの骨格の上に別途実装する。

### Parameter の記録

Step カードの中で、配下の Parameter を追加・編集・削除できる。

- **追加**: Step カード内の「パラメーター追加」からボトムシートを開き、
  種別を選んでから種別ごとのフォームを入力する。

  | 種別 | 表示名 | 入力項目 |
  |------|--------|---------|
  | `KEY_VALUE` | 項目と値 | 項目名（必須）＋ 値の種類（数値 / 文字列）。数値なら数量（必須・0 より大きい）＋ 単位（必須）、文字列なら値（必須） |
  | `DURATION` | 経過時間 | 内容（必須）＋ 時間（必須・0 以上）＋ 単位（日 / 時間 / 分 / 秒） |
  | `TIME_MARKER` | 時間マーカー | 内容（必須）＋ 経過時点（必須・0 以上）＋ 単位（日 / 時間 / 分 / 秒） |
  | `TEXT` | 自由記述 | 内容（必須） |

- **編集**: 既存 Parameter の末端の値だけを変更できる。種別は変更できず、
  `KEY_VALUE` の値の種類（数値 / 文字列）も変更できない。
- **削除**: 対象の行で削除を確認してから実行する。

記録操作は **Trial が記録中（`IN_PROGRESS`）かつ Step が未完了** の場合にだけ表示する。
バックエンドが完了済みの Trial・Step へのパラメーター操作を拒否するため、
実行できない操作は UI に出さない。

各操作の成功後は Trial を再取得し、タイムラインの表示に反映する。

### 状態表示

| 状態 | 表示 |
|------|------|
| 取得中 | `LoadingSpinner` |
| 取得失敗 | `ErrorState`（再試行ボタンあり） |
| `trial(id)` が null / `trialId` 不在 | `ErrorState`「指定された試行が見つかりません」 |
| Step が 0 件 | `EmptyState`「まだ工程が記録されていません」 |

## アーキテクチャ設計・実装方針

### 情報設計: 1 画面に集約される情報のコントロール

この画面には「複数 Step × 各 Step 複数 Parameter」が一度に載る。
モバイル（PWA）で調理中に片手操作されることを前提に、以下の方針で情報量を制御する。

#### 1. フォーカス外の情報を畳む

- Step カードは折りたたみ可能とし、**未完了の工程は展開・完了済みの工程は畳んだ状態**を初期表示とする。
  記録の焦点は「これから記録する工程」にあり、完了済みの工程は参照頻度が下がるため。
- Trial が完了済み（`status: COMPLETED`）の場合は焦点となる工程が存在しないため、
  **全工程を畳んだ俯瞰表示**を初期状態とする。
- 畳んだ状態でもヘッダに「工程名 / 状態 / パラメーター件数 / 開始・完了日時」のサマリーを常に出し、
  展開しなくても進捗を追えるようにする。
- 開閉はユーザーが手動で切り替えられる。**手動操作は初期表示の方針より優先**し、
  再取得でデータが更新されても開閉状態は維持する。
  そのためローディング表示は初回取得時のみとし、更新後の再取得では内容を表示したままにする。
  再取得のたびに内容を差し替えると、工程カードの開閉状態とスクロール位置が失われ記録操作を妨げるため。

#### 2. 現在記録中の工程へのフォーカス誘導

- `position` 昇順で**最初の未完了 Step** を「記録中の工程」とみなす。
- 記録中の工程はカード枠線を強調し、`aria-current="step"` を付与して支援技術にも伝える。
- 初期表示時および記録中の工程が切り替わったとき、その工程を画面内へスクロールする。
  工程数が増えても、画面を開いた直後に記録対象が視界に入る状態を保つ。

#### 3. モバイルでのスクロール量と可読性

- Parameter は種別によらず「ラベル + 値」の 1 行に正規化して表示し、カード内の縦方向の伸びを抑える。
- Trial のステータス（記録中 / 完了）と工程の状態（未着手 / 進行中 / 完了）は語彙を分け、
  同じ画面に並んだときにどちらの状態を指すか一目で区別できるようにする。

この情報設計は、後続で実装する Step 記録・Parameter 記録の UI 配置の前提になる。
新しい記録操作を追加する場合も「記録中の工程が常に展開・可視である」状態を崩さないこと。

#### 4. 記録操作の置き場所

Parameter の記録操作は、対象の Step カードの中に置く。記録は「どの工程の話か」と
不可分であり、別画面や画面外のボタンに出すと対象の取り違えが起きるため。

入力フォームは画面下端から開くボトムシート（`DialogSheetPopup`）に集約する。
調理中は片手での操作になるため、入力要素を親指の届く範囲に寄せる。
入力項目が増えてもシート内でスクロールさせ、画面全体を覆わないようにする。

一覧の各行には編集・削除をアイコンボタンで並べ、行の高さを増やさない。
削除は取り消せないため、同じ行で確認してから実行する。
別ダイアログを重ねると記録中の工程が見えなくなるため、確認は行内で完結させる。

### Parameter の記録操作

#### 操作を出す条件

バックエンドは `addParameter` / `updateParameter` / `removeParameter` のいずれについても
「Trial が記録中」かつ「Step が未完了」であることを要求する
（`backend/src/domain/actions/trial/*_parameter.rs`）。
この条件を満たす Step にだけ記録操作を表示し、満たさない Step は読み取り専用にする。
実行すれば必ず失敗する操作を出さないことで、記録中の誤操作とエラー表示を防ぐ。

#### 種別の不変性

`updateParameter` は `ParameterContent` のバリアント変更を拒否し、
`KEY_VALUE` については内部の値の種類（`text` / `quantity`）まで一致を要求する
（`backend/src/domain/validators/trial/parameter_variant_validator.rs`）。

このため編集フォームでは種別選択を出さず、`KEY_VALUE` の値の種類も固定する。
種別を変えたい場合は削除して追加し直す。

同じ理由で、フロントが知らない種別の Parameter は編集させない。
`content` は JSON スカラーのため未知の種別が先に届きうるが、入力フォームに写せないまま保存すると
種別一致検証で弾かれる。削除は `content` を伴わないため、未知の種別でも行える。

#### 入力の検証

送信前に、バックエンドのドメインバリデーションと同じ条件をフロントでも適用する
（数量は 0 より大きい / 時間量は 0 以上 / 数量の単位は空でない）。
調理中の記録でサーバーとの往復後に弾かれると入力し直しになるため、入力時点で気付けるようにする。

加えて、バックエンドが要求しない項目のうち以下をフロント側の必須項目とする。

| 項目 | 必須にする理由 |
|------|---------------|
| `KEY_VALUE` の `key` | 一覧表示のラベルになり、空だと値だけが並んで意味が読み取れないため |
| `KEY_VALUE` の値（text） | 値のない記録は記録として成立しないため |
| `DURATION` / `TIME_MARKER` の `note` | 一覧表示のラベルになり、空だと種別名しか手掛かりが残らないため |
| `TEXT` の値 | 同上 |

数値項目は入力途中の文字列（`"1."` など）をそのまま保持できるよう、
フォーム状態では string で持ち、送信時にまとめて数値へ変換・検証する。

#### 操作結果の反映

追加・編集・削除の成功後は Trial を再取得する。パラメーターの増減は Step カードの
サマリー（パラメーター件数）にも影響し、部分的な差し替えでは画面内の整合が取れないため。
再取得中も内容を表示したままにする方針（上記「開閉状態の同期」）により、
記録を続けたまま結果が反映される。

### データ取得

Step と Parameter は 1 画面で俯瞰するため、`trial(id)` クエリで
`steps { ... parameters { id parameterType content } }` まで一度に取得する。
工程ごとの追加リクエストは行わない。

「← 戻る」の遷移先には、取得した Trial の `projectId` を使う（未取得時のみ URL の `:id` にフォールバックする）。
URL の `:id` は利用者が書き換えられるため、実際の所属プロジェクトと食い違っていても正しい一覧へ戻れるようにする。

Parameter の種別判定には、バックエンドが公開する `parameterType`
（`KEY_VALUE` / `DURATION` / `TIME_MARKER` / `TEXT`）を使う。
これを TypeScript の判別子付き合併型のタグにすることで、JSON スカラーである
`content` の構造が種別ごとに型で絞り込まれる。

### Parameter の表示整形

各種別を「ラベル + 値」に正規化する。

| 種別 | ラベル | 値の例 |
|------|--------|--------|
| `KEY_VALUE` | `key` | `300g`（quantity） / `冷蔵庫`（text） |
| `DURATION` | `note`（空なら「経過時間」） | `90分` |
| `TIME_MARKER` | `note`（空なら「時間マーカー」） | `30分時点` |
| `TEXT` | なし | 記述内容そのまま |

時間量の単位は `day` / `hour` / `minute` / `second` を `日` / `時間` / `分` / `秒` に変換する。

`content` は JSON スカラーとして届くため、バックエンドに種別が追加されるとフロントの型より先に
未知の値が流れてくる。その場合は画面全体を落とさず、`content` をそのまま文字列化して表示する。

### 日時の表示

バックエンドは日時を JST オフセット付き ISO 8601 で返すため、
閲覧端末のタイムゾーン設定に関わらず JST（`Asia/Tokyo`）で `2026/01/01 09:00` 形式に整形する。

### name / memo の更新セマンティクス

`UpdateTrialInput` の `name` / `memo` は未指定なら変更なし、`null` ならクリアされる。
編集フォームは両方を常に送信し、**空欄は「未設定に戻す」意図として `null` を送る**。

## エンドポイント仕様

本画面は既存の GraphQL API のみを利用し、バックエンドの変更はない。

```graphql
query Trial($id: ID!) {
  trial(id: $id) {
    id
    projectId
    name
    memo
    status        # IN_PROGRESS | COMPLETED
    completedAt
    steps {
      id
      name
      position
      startedAt
      completedAt
      isCompleted
      parameters {
        id
        parameterType   # KEY_VALUE | DURATION | TIME_MARKER | TEXT
        content         # JSON スカラー
      }
    }
  }
}

mutation UpdateTrial($id: ID!, $input: UpdateTrialInput!) {
  updateTrial(id: $id, input: $input) { id name memo }
}

# completedAt を省略するとバックエンドが現在時刻を設定する
mutation CompleteTrial($id: ID!) {
  completeTrial(id: $id) { id status completedAt }
}

mutation AddParameter($trialId: ID!, $stepId: ID!, $content: JSON!) {
  addParameter(trialId: $trialId, stepId: $stepId, content: $content) {
    id
    parameterType
    content
  }
}

# 末端の値のみ更新できる。種別（KeyValue の値の種類を含む）は変更できない
mutation UpdateParameter($trialId: ID!, $stepId: ID!, $parameterId: ID!, $content: JSON!) {
  updateParameter(
    trialId: $trialId
    stepId: $stepId
    parameterId: $parameterId
    content: $content
  ) {
    id
    parameterType
    content
  }
}

mutation RemoveParameter($trialId: ID!, $stepId: ID!, $parameterId: ID!) {
  removeParameter(trialId: $trialId, stepId: $stepId, parameterId: $parameterId) { id }
}
```

`content` の JSON 構造は `backend/src/domain/models/parameter.rs` の定義に従う。

| 種別 | `content` の構造 |
|------|-----------------|
| `KEY_VALUE`（数値） | `{ "type": "key_value", "key": "強力粉", "value": { "type": "quantity", "amount": 300, "unit": "g" } }` |
| `KEY_VALUE`（文字列） | `{ "type": "key_value", "key": "発酵場所", "value": { "type": "text", "value": "冷蔵庫" } }` |
| `DURATION` | `{ "type": "duration", "duration": { "value": 90, "unit": "minute" }, "note": "一次発酵" }` |
| `TIME_MARKER` | `{ "type": "time_marker", "at": { "value": 30, "unit": "minute" }, "note": "焼成開始から" }` |
| `TEXT` | `{ "type": "text", "value": "打ち粉を追加" }` |

`unit`（`DurationUnit`）は `day` / `hour` / `minute` / `second`。

## 意思決定記録

| 決定事項 | 採用した方針 | 理由 | 関連 Issue |
|---------|------------|------|-----------|
| Step の初期開閉状態 | 未完了は展開・完了済みは畳む。完了済み Trial は全て畳む | 記録の焦点である未完了工程を常に可視にしつつ、モバイルでのスクロール量を抑えるため | #83 |
| 開閉状態の同期 | 手動操作を初期方針より優先し、再取得でも維持する | 記録中にデータを再取得するたび開閉が戻ると操作を妨げるため | #83 |
| 記録中の工程の定義 | `position` 昇順で最初の未完了 Step | Step 自体は「現在の工程」を持たないため、順序と完了状態から一意に導出する | #83 |
| Parameter の種別判定 | `content` の JSON タグではなく `parameterType` を使う | バックエンドが JSON を解析せず種別判定できるよう公開している列挙子であり、TypeScript の型の絞り込みにも使えるため | #83 |
| Step / Parameter の取得 | `trial(id)` で一括取得 | 1 画面で全工程を俯瞰する画面であり、工程ごとの追加リクエストは無駄になるため | #83 |
| 日時の表示タイムゾーン | 常に JST に固定 | バックエンドが JST を前提に日時を返しており、閲覧端末のロケールで記録時刻がずれて見えることを防ぐため | #83 |
| 空欄の name / memo | `null` を送りクリアする | いずれも任意項目であり、空欄にした意図は「未設定に戻す」であるため | #83 |
| 再取得中の表示 | ローディング表示は初回取得時のみ。更新後の再取得では内容を表示したままにする | 内容を差し替えると工程カードの開閉状態とスクロール位置が失われ、記録操作を妨げるため | #83 |
| 「← 戻る」の遷移先 | 取得した Trial の `projectId` を使い、未取得時のみ URL の `:id` にフォールバックする | URL の `:id` が実際の所属プロジェクトと食い違っていても正しい一覧へ戻せるため | #83 |
| 未知の Parameter 種別 | 画面を落とさず `content` をそのまま文字列化して表示する | `content` は JSON スカラーのため、バックエンドの種別追加がフロントの型より先に届きうるため | #83 |
| Trial と Step の状態ラベル | 語彙を分ける（Trial: 記録中 / 完了、Step: 未着手 / 進行中 / 完了） | 同じ画面に両方の状態が並ぶため、どちらを指すか区別できるようにする | #83 |
| Parameter 記録操作の置き場所 | 対象の Step カード内に配置し、入力はボトムシートに集約する | 記録は対象工程と不可分であり、モバイルでの片手操作を前提に入力要素を画面下端へ寄せるため | #85 |
| 記録操作を出す条件 | Trial が記録中かつ Step が未完了の場合にだけ表示する | バックエンドが完了済みの Trial・Step へのパラメーター操作を拒否するため、必ず失敗する操作を出さない | #85 |
| 編集時の種別変更 | 種別も `KEY_VALUE` の値の種類も固定する（変えたい場合は削除して追加し直す） | `updateParameter` が `ParameterContent` のバリアント変更を拒否するため | #85 |
| 未知の種別の扱い | 編集操作を出さず、削除だけ行えるようにする | 入力フォームに写せない種別を保存すると種別一致検証で弾かれる。削除は `content` を伴わないため種別によらず安全 | #85 |
| 入力値の検証 | バックエンドのドメインバリデーションと同じ条件を送信前にフロントでも適用する | 調理中にサーバーとの往復後で弾かれると入力し直しになるため | #85 |
| ラベルになる項目の必須化 | `key` / `note` / 各値をフロント側で必須にする（バックエンドは空を許容する） | 一覧では「ラベル + 値」に正規化して表示するため、空だと何の記録か読み取れなくなるため | #85 |
| 数値入力の保持 | フォーム状態では string で保持し、送信時にまとめて数値へ変換・検証する | 入力途中の文字列を数値に丸めると、入力しながら値が書き換わって記録を妨げるため | #85 |
| 記録操作後の反映 | 部分更新ではなく Trial を再取得する | パラメーターの増減は工程カードのサマリー（件数）にも影響し、部分的な差し替えでは画面内の整合が取れないため | #85 |
| 削除の確認 | 別ダイアログではなく対象の行内で確認する | 取り消せない操作であり、かつ確認のために記録中の工程が隠れることを避けるため | #85 |
