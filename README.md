# RustでGitを作るハンズオン

Gitと同じ形式でリポジトリを読み書きするツール`rgit`を，Rustで一から作るハンズオンである．
12回のIterationで1つのプログラムを育てながら，Rustならではの設計とGitの内部の仕組みを学ぶ．

```console
$ rgit add .
$ rgit commit -m first
[main (root-commit) 6c04901] first
$ git log --oneline
6c04901 first
```

## 学ぶこと

- Rust：所有権と借用，ライフタイム，ニュータイプと`enum`による型の設計，型状態パターン，トレイトとジェネリクス，`Iterator`の実装，`Drop`とRAII，`Result`によるエラー処理，スレッドと`Send`/`Sync`
- Git：内容アドレスとオブジェクト(blob，tree，commit)，zlibによる格納，インデックス，参照と`HEAD`，コミットのグラフ，`status`と`diff`の仕組み
- 開発の進め方：テストリストから始めるテスト駆動開発と，型とモジュールの図による設計

## 前提

- Rust以外のプログラミング言語(Python，JavaScript，Go，Javaなど)で，プログラムを書いた経験がある．
- 単体テストを書いたことがある．
- Gitを日常的に使っている．Gitの内部とRustは知らなくてよい．

## 環境の準備

VS CodeのDev Containersで開くと，必要なものがそろう．

1. このリポジトリをクローンし，VS Codeで開く．
2. コマンドパレットで「Dev Containers: Reopen in Container」を実行する．初回は`mise run setup`が実行される．
3. ターミナルで次のコマンドを実行し，版が表示されることを確かめる．

```console
$ cargo --version
cargo 1.98.1 (797e8a9bc 2026-08-05)
$ rustowl --version
RustOwl v0.4.0
```

Dev Containerには，Rust，`git`，`xxd`，所有権を可視化するRustOwl，VS Codeの拡張機能(rust-analyzer，RustOwl，デバッガーのCodeLLDB，Mermaidのプレビュー)が入っている．
`mise tasks`で，使えるタスクの一覧を表示できる．

Dev Containersを使わない場合は，次を用意する．

- Rust 1.98.1(`rustup`または`mise`で入れる)
- CのリンカーとCの標準ライブラリ(Debian系では`gcc`と`libc6-dev`)
- `git`と`xxd`
- Node.jsとpnpm(`pnpm install`を実行する．Mermaidの図の検査に使う)
- RustOwl 0.4.0(任意．`rustowl toolchain install`まで実行する)

## 進め方

各Iterationは`iterations/iteration-NN/`にある．

- `exercise/`：受講者が作業する場所．`exercise/README.md`から始める．
- `solution/`：演習を終えた状態と模範解答．自分の答えと見比べる．

どのIterationも，テストリスト → 図の更新 → テスト駆動の実装 → 設計レビューの順に進める．

- [ロードマップ](docs/ROADMAP.md)：各Iterationで作る機能と学ぶこと
- [テスト駆動開発とテストリスト](docs/tdd.md)
- [型とモジュールの図の書き方](docs/design.md)
- [Rustのノート](docs/rust/README.md)：各Iterationで初めて使う文法と概念
- [Gitのノート](docs/git/README.md)：各Iterationで初めて扱うGitの仕組み

## Iterationの一覧

| # | 作る機能 |
| --- | --- |
| [0](iterations/iteration-00/exercise/README.md) | blobのハッシュの計算 |
| [1](iterations/iteration-01/exercise/README.md) | オブジェクトID |
| [2](iterations/iteration-02/exercise/README.md) | `init`と`hash-object -w` |
| [3](iterations/iteration-03/exercise/README.md) | `cat-file` |
| [4](iterations/iteration-04/exercise/README.md) | ツリーの読み取りと`ls-tree` |
| [5](iterations/iteration-05/exercise/README.md) | インデックス，`add`，`ls-files` |
| [6](iterations/iteration-06/exercise/README.md) | `write-tree`と`commit-tree` |
| [7](iterations/iteration-07/exercise/README.md) | 参照，`commit`，`branch` |
| [8](iterations/iteration-08/exercise/README.md) | `log` |
| [9](iterations/iteration-09/exercise/README.md) | オブジェクトストアの抽象化と`status` |
| [10](iterations/iteration-10/exercise/README.md) | `diff` |
| [11](iterations/iteration-11/exercise/README.md) | `add`と`status`の並列化 |
