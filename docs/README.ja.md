# coding-human

> **[English version](../README.md)**

WebSocket を通じて**クライアント**（質問する側）と**コーダー**（答える側）をリアルタイムで繋ぐ Q&A ツールです。[ratatui](https://ratatui.rs) による本格的な TUI をターミナル上で楽しめます。

---

## インストール

```sh
cargo install --git https://github.com/ut-code/coding-human --bin coding-human
```

ソースからビルドする場合:

```sh
git clone https://github.com/ut-code/coding-human
cd coding-human
cargo build --release
# バイナリは ./target/release/coding-human に生成されます
```

## クイックスタート

ターミナルを 2 つ用意してください — 1 つは**コーダー**用、もう 1 つは**クライアント**用です。

```sh
# ターミナル 1 — コーダーとして待機
coding-human coder "Alice (Rust / systems)"

# ターミナル 2 — クライアントとして接続
coding-human client "Bob"
```

クライアント側にインタラクティブな一覧が表示されるので、コーダーを選択するとチャット画面が開きます。

---

## クライアント向けガイド

### 接続する

```sh
coding-human client "あなたの名前"
```

利用可能なコーダーの一覧がピッカー画面に表示されます。**↑ / ↓**（または **j / k**）で選択し、**Enter** で接続します。

### 質問を送る

画面下のインプットボックスに質問を入力し、**Enter** を押します。

```
このライフタイム 'a は何を意味していますか？
```

回答が届くまでインプットは**ロック**され、「Waiting for answer…」が表示されます。回答はリアルタイムでログに流れてきます。

### ファイルを添付する

パスの先頭に `@` を付けると、質問の前にそのファイルの内容がコーダーへ送信されます。

```
@src/main.rs ボローチェッカーがここで怒る理由がわかりません
```

ファイルはコーダー側の `/tmp/coding-human/src/main.rs` に自動保存されます。

### コマンド実行リクエストへの応答

コーダーがシェルコマンドの実行を要求することがあります。その場合、次のように表示されます。

```
ⓘ  Run: ls -la?
ⓘ  [y/n]: y または n を入力して Enter を押してください
```

**y**（実行）か **n**（スキップ）を入力して **Enter** を押すと、出力が自動的にコーダーへ送られます。

### 差分（diff）への応答

コーダーがファイルの変更案を送ってくることがあります。ログに unified diff が表示されます。

```
ⓘ  Diff for src/main.rs:
   --- src/main.rs
   +++ src/main.rs
   @@ -10,3 +10,5 @@
   ...
ⓘ  apply? [y/n]: y または n を入力して Enter を押してください
```

**y** でパッチを適用、**n** で却下します。

### `--yes` で自動承認

`--yes` を付けると、すべてのコマンドと diff を自動的に承認します。

```sh
coding-human client "Bob" --yes
```

### キーボードショートカット

| キー | 操作 |
|------|------|
| **Enter** | メッセージを送信 |
| **Ctrl+C** | 終了 |
| **↑ / ↓** | ログを 1 行スクロール |
| **PgUp / PgDn** | ログを半ページスクロール |

---

## コーダー向けガイド

### キューに参加する

```sh
coding-human coder "Alice (Rust / systems)"
```

クライアントのピッカーにあなたの名前が表示されます。クライアントが接続するまでインプットはロックされています。

### 質問に回答する

クライアントが接続すると質問がログに表示され、インプットボックスがアンロックされます。1 行ずつ回答を入力し、**Enter** で送信します。各行がリアルタイムでクライアントに届きます。

### 回答を終了する

回答が完了したら **Ctrl+D** を押します。クライアントに「回答終了」の合図が送られ、次の質問が届くまでインプットは再びロックされます。

### クライアントのマシンでコマンドを実行する

行頭に `$ `（ドル + スペース）を付けると、クライアントにコマンド実行を依頼できます。クライアントが承認すると、出力が自動的に返ってきます。

```
$ ls -la
```

```
$ cat package.json
```

### クライアントのファイルをエディタで開く

クライアントが `@path` でファイルを送ってきた場合、`@` に続けてパスを入力するとそのファイルを `$EDITOR` で開けます。TUI は一時停止し、エディタ終了後に自動で復元されます。

```
@src/main.rs
```

### 差分をクライアントへ送る

ファイルを編集したあと、`$send-diff` で unified diff を生成してクライアントへ送信できます。

```
$send-diff src/main.rs
```

クライアントは diff を確認し、適用（y）または却下（n）を選択します。

### キーボードショートカット

| キー | 操作 |
|------|------|
| **Enter** | 回答の 1 行を送信 |
| **Ctrl+D** | 回答を終了（`Done` を送信） |
| **Ctrl+C** | 終了 |
| **↑ / ↓** | ログをスクロール |
| **PgUp / PgDn** | ログを半ページスクロール |

---

## 設定

サーバー URL のデフォルトは `http://localhost:8787` です。環境変数または `.env` ファイルで変更できます。

```sh
export SERVER_URL=https://your-worker.workers.dev
```

```sh
# .env
SERVER_URL=https://your-worker.workers.dev
```

---

## 関連資料

- [Developer guide (English)](development.md)
- [開発者ガイド（日本語）](development.ja.md)
- [User guide (English)](../README.md)
