# ICTSC Discord Bot

ICTSCコンテストサーバー管理用のDiscord Bot。チーム管理、チャンネル作成、問題再展開機能を提供します。

## 機能

- チーム管理とロール割り当て
- チームと問題用のチャンネル自動作成
- チーム交流用スラッシュコマンド
- RStateとの問題再展開連携
- スタッフ権限管理

## 利用可能なコマンド

### グローバルコマンド
- `/ping` - シンプルなpingコマンド
- `/join <team_code>` - チームコードを使用してチームに参加

### ギルドコマンド
- `/archive` - チャンネルをアーカイブ
- `/ask` - スタッフに質問
- `/redeploy` - 問題の再展開（スタッフのみ）

## ビルド

Dockerが必要です：

```bash
make build
```

## 設定

サンプル設定をコピー：

```bash
cp bot.sample.yaml bot.yaml
```

`bot.yaml` を編集してDiscord Botの認証情報とコンテスト設定を入力してください。

## 実行

### Botデーモンの開始

```bash
make start
```

### ロールとチャンネルの同期

```bash
make sync
```

### Botの停止

```bash
make stop
```

### ログの確認

```bash
make logs
```

### クリーンアップ（全ロール、チャンネル、コマンドを削除）

```bash
make flush
```

## 開発

### コードフォーマット

```bash
make fmt
```

### ローカルビルド

```bash
cargo build --release
```

### ローカル実行

```bash
./target/release/bot -f bot.yaml <subcommand>
```
