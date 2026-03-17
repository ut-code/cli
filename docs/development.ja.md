# 開発者ガイド

> **[English version](development.md)**

このドキュメントでは、`coding-human` プロジェクトの内部アーキテクチャ、モジュール構成、および開発ワークフローについて説明します。

---

## リポジトリ構成

```
coding-human/
├── Cargo.toml          # ワークスペースルート（メンバー: cli, worker）
├── Cargo.lock
├── .env                # ローカル環境変数上書き用（SERVER_URL）
├── flake.nix           # Nix dev-shell（任意）
│
├── cli/                # ネイティブバイナリ — coding-human
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs     # CLI エントリポイント（clap サブコマンド: client / coder）
│       ├── protocol.rs # WebSocket メッセージ型定義（WsMessage enum）
│       ├── tui.rs      # ratatui による本格的な TUI（ピッカー、チャット、入力ロックなど）
│       ├── client.rs   # クライアントモード: コーダー選択、Ｑ＆Ａループ
│       └── coder.rs    # コーダーモード: キュー登録、回答ループ
│
└── worker/             # Cloudflare Worker — リレーサーバー（Rust → WASM）
    ├── Cargo.toml
    ├── wrangler.jsonc
    └── src/
        └── lib.rs      # QueueDO + RoomSession + fetch ハンドラー
```

---

## ローカル開発

### 前提条件

| ツール | 用途 |
|------|---------|
| `rustup` + stable ツールチェーン | CLI のビルド |
| `cargo` | ビルド / チェック / テスト |
| Node.js + `npx` | `wrangler dev`（ワーカー）の実行 |
| `wrangler`（npx 経由） | Cloudflare Worker 開発用ローカルサーバー |

### ワーカーの起動

```sh
cd worker
npx wrangler dev          # http://localhost:8787 でリッスン
```

### ローカルワーカーに CLI を繋ぐ

```sh
# ターミナル 1 — コーダー
cargo run --bin coding-human -- coder "Alice"

# ターミナル 2 — クライアント
cargo run --bin coding-human -- client "Bob"
```

ワークスペースルートの `.env` ファイルは `dotenvy` により自動で読み込まれます。

```
SERVER_URL=http://localhost:8787
```

---

## アーキテクチャ

### WebSocket プロトコル (`protocol.rs`)

すべてのメッセージは `type` フィールドを持つ JSON としてシリアライズされます（`serde(tag = "type", rename_all = "snake_case")`）。

| バリアント | 方向 | 目的 |
|---------|-----------|---------|
| `Matched { client_name }` | Client → Coder | 接続時のハンドシェイク |
| `Question { from, text }` | Client → Coder | 質問テキスト |
| `File { path, content }` | Client → Coder | `@path` で添付されたファイルデータ |
| `Cmd { command }` | Coder → Client | クライアントへのコマンド実行要求 |
| `CmdResult { command, output }` | Client → Coder | コマンドの実行結果 |
| `Diff { path, diff }` | Coder → Client | 適用を提案する unified diff |
| `DiffResponse { accepted }` | Client → Coder | diff が適用されたかどうかの返答 |
| `Done` | Coder → Client | 現在の回答ストリームの終了合図（Ctrl+D）|

`WsMessage` としてパースできないメッセージは、すべて raw な回答テキストとして扱われます。

### TUI (`tui.rs`)

[ratatui](https://ratatui.rs) と [crossterm](https://docs.rs/crossterm) で構築されています。主要な型は以下の通りです。

#### `ChatMsg` (非同期タスク → TUI 描画ループ)

```rust
pub enum ChatMsg {
    Msg { role: ChatRole, text: String },
    SetWaiting(bool),      // 入力ボックスのロック（Wait状態）切り替え
    OpenEditor(String),    // TUIを一時停止して $EDITOR を開き、終了後に復元
}
```

#### `ChatEvent` (TUI → 非同期タスク)

```rust
pub enum ChatEvent {
    Line(String),     // ユーザーが Enter を押して入力確定
    Eof,              // Ctrl+D
    Quit,             // Ctrl+C
    EditorClosed,     // OpenEditor で開いたエディタプロセスが終了した
}
```

#### 並行処理モデル

```
┌──────────────────────────────┐   log_tx (ChatMsg)   ┌──────────────────┐
│   net_task (tokio::spawn)    │ ──────────────────►  │  run_chat        │
│                              │                       │  (メインタスク)   │
│  WebSocket I/O               │ ◄──────────────────  │                  │
│  コマンド実行                │   event_tx (ChatEvent)│  TUI描画ループ  │
│  パッチ適用                  │                       │  キーイベント   │
└──────────────────────────────┘                       └──────────────────┘
```

`run_chat` は `Terminal<CrosstermBackend<Stdout>>` の所有権を持っています。スクロールバーの位置をレンダリング行単位で正確に合わせるため、毎フレーム `Vec<Line<'static>>` に全メッセージを事前レンダリングしています。

#### スクロール制御

`scroll_offset` はメッセージのインデックスではなく**レンダリングされた実際の行数単位**で記録されます。`at_bottom = true` の状態では、常にログの末尾に追従するように `total_rendered_lines - inner_height` から表示位置が動的に計算されます。

#### エディタ連携（コーダー側）

コーダーが `@src/main.rs` と入力した場合、ネットワークタスクは直接エディタのプロセスを起動するのではなく、TUI タスクに向けて `ChatMsg::OpenEditor(path)` を送信します。`run_chat` はこれを受け取ると：

1. `leave(&mut term)` — raw モードと alternate screen をクリーンに解除します
2. `std::process::Command::new(editor).arg(path).status()` — クリーンなターミナル画面でエディタを実行します
3. 再度 raw モードと alternate screen に入り、`term.clear()` を呼び出して画面を復元します
4. `ChatEvent::EditorClosed` を送信して、待機していたネットワークタスクのブロックを解除します

この仕組みにより、ratatui がターミナルの制御を握ったまま外部プロセスが標準出力に書き込んで画面表示が崩れるバグを完全に防いでいます。

---

## 新しいメッセージタイプの追加方法

1. `protocol.rs` の `WsMessage` に新しいバリアントを追加します。
2. 送信側（`coder.rs` または `client.rs`）と受信側の両方で処理を実装します。
3. 必要に応じて、`log_tx.send(ChatMsg::sys(...))` を使って TUI ログにステータスを表示させます。

---

## ワーカーのデプロイ

Cloudflare Worker は [workers-rs](https://github.com/cloudflare/workers-rs) を通じて WASM にコンパイルされます。

```sh
# 最初の一回のみ worker-build をインストール
cargo install worker-build

cd worker
npx wrangler deploy
```

### Worker API 一覧

| メソッド | パス | 説明 |
|--------|------|-------------|
| `GET` | `/queue` | 待機中のコーダー一覧 `{ roomId: label }` を取得 |
| `POST` | `/queue` | コーダーを登録 `{ label }` → `{ roomId }` |
| `DELETE` | `/queue/:roomId` | コーダー登録の解除 |
| `GET` (WS) | `/rooms/:id/coder` | コーダー用 WebSocket エンドポイント |
| `GET` (WS) | `/rooms/:id/client` | クライアント用 WebSocket エンドポイント |

### Durable Objects の役割

| オブジェクト | インスタンス数 | 役割 |
|--------|-----------|---------|
| `QueueDO` | シングルトン | 待機中コーダーのキューを KV ストレージに永続化 |
| `RoomSession` | ルームごとに 1 つ | ハイバネータブル WebSocket を活用したコーダーとクライアント間のメッセージリレー |

---

## コーディング規約

- **エラーハンドリング**: 失敗する可能性のある関数はすべて `anyhow::Result` を返します。`?` 演算子を積極的に使い、最終的に `main` でキャッチしてエラー終了（コード 1）させます。
- **非同期ランタイム**: `full` 機能付きの `tokio` を使用します。ネットワーク I/O は専用の `tokio::spawn` タスクで、TUI の描画は呼び出し元のタスクで実行します。
- **チャンネル**: TUI ↔ 非同期タスクの連携には `mpsc::unbounded_channel` を使用します。TUI ループはメッセージ生成よりも遥かに速くキューを消化するため、bounded なチャンネルによるバックプレッシャーは不要です。
- **フォーマット**: 標準の `rustfmt` スタイルに準拠します。コミット前に `cargo fmt` を実行してください。
- **リント**: `cargo clippy -- -D warnings` で警告が出ないようにしてください。
