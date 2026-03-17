# coding-human

WebSocket を介してクライアントとコーダーがリアルタイムで質疑応答を行うための CLI ツールです。

コーダーはキューに自分を登録し、クライアントはキューを閲覧して好みのコーダーに接続します。

## リポジトリ構成

```
coding-human/
├── Cargo.toml      # ワークスペースルート
├── cli/            # ネイティブバイナリ (coding-human)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── client.rs  # クライアント: キュー閲覧、コーダー選択、質問送信
│       └── coder.rs   # コーダー: キュー登録、質問への回答
└── worker/         # Cloudflare Worker (workers-rs / WASM)
    ├── Cargo.toml
    ├── wrangler.jsonc
    └── src/
        └── lib.rs   # QueueDO + RoomSession + fetch ハンドラー
```

## インストール (CLI)

```sh
cargo install --git https://github.com/ut-code/cli --bin coding-human
```

## 使い方

### コーダー: キューに参加する

```sh
coding-human coder <ラベル>
```

指定した表示名でキューに登録し、ルームを作成してクライアントの接続を待ちます。接続が確立すると質問が自動的に届き、1 行ずつ回答を入力します。`Ctrl+D` を押して回答を送信し、次の質問を待ちます。

```sh
coding-human coder "Alice (Rust / systems)"
```

### クライアント: 質問する

```sh
coding-human client <名前>
```

利用可能なコーダーの一覧を取得し、1 人を選択してセッションを開始します。プロンプトに質問を入力すると、回答がリアルタイムで届きます。終了するには `/quit` を入力するか `Ctrl+D` を押してください。

## 設定 (CLI)

サーバー URL のデフォルトは `http://localhost:8787` です。`SERVER_URL` 環境変数または `.env` ファイルで上書きできます。

```
SERVER_URL=https://your-worker.example.com
```

## ローカル開発

```sh
# worker-build を一度インストール
cargo install worker-build

# http://localhost:8787 でワーカーをローカル起動
cd worker
npx wrangler dev
```

その後、CLI をデフォルト URL (`http://localhost:8787`) に向けて実行します。

```sh
# ターミナル 1 — コーダー
coding-human coder "Alice"

# ターミナル 2 — クライアント
coding-human client "Bob"
```

## デプロイ (Worker)

Cloudflare Worker は [workers-rs](https://github.com/cloudflare/workers-rs) を使用して Rust で書かれており、WASM にコンパイルされます。

```sh
# worker-build を一度インストール
cargo install worker-build

# ビルド + デプロイ
cd worker
npx wrangler deploy
```

ワーカーが公開するエンドポイント:

| メソッド | パス | 説明 |
|--------|------|-------------|
| `GET` | `/queue` | 待機中のコーダー一覧 `{ roomId: label }` |
| `POST` | `/queue` | コーダーを登録 `{ label }` → `{ roomId }` |
| `DELETE` | `/queue/:roomId` | コーダーを登録解除 |
| `GET` (WS) | `/rooms/:id/coder` | コーダー用 WebSocket |
| `GET` (WS) | `/rooms/:id/client` | クライアント用 WebSocket |

ワーカーは 2 つの Durable Object で支えられています。

- **`QueueDO`** — シングルトン。待機中のコーダーのキューを KV ストレージに永続化
- **`RoomSession`** — ルームごとに 1 つ。ハイバネータブル WebSocket を使用してコーダーとクライアント間のメッセージを中継
