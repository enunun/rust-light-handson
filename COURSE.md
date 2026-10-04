# コース計画

このファイルは，教材を作る人とエージェントのための計画書である．
受講者向けの内容は`README.md`と[docs/ROADMAP.md](docs/ROADMAP.md)にある．
各Iterationで作る機能と学ぶことはロードマップだけに書き，このファイルには決定事項と規約を書く．

## 受講者と目標

- 受講者は，Rust以外の言語(Python，JavaScript，Go，Javaなど)で実務の経験があり，単体テストを書いたことがある．Rustは初めてである．
- 受講者の多くはオブジェクト指向言語に慣れている．Rustならではの設計(ニュータイプ，代数的データ型，型状態，トレイト，所有権を使うAPI，`Send`と`Sync`)を，ほかの言語での書き方と比べながら学ぶ．振り返りには，型の設計を比べる問いを入れる．
- 修了すると，受講者は次のことができる．
  - 所有権，借用，ライフタイムを理解し，コンパイラーのエラーを読んで直せる．
  - 不正な値を作れない型を，ニュータイプ，`enum`，型状態パターンで設計できる．
  - トレイトとジェネリクスで実装を差し替えられる設計にし，テストで使い分けられる．
  - `std::thread::scope`，`Mutex`，チャネルで処理を並列にし，`Send`と`Sync`のエラーを読める．
  - Gitのオブジェクト，ツリー，インデックス，コミット，参照の仕組みと，`status`と`diff`が何を比べているかを説明できる．
- 教材は日本語で書く．文体は常体(である調)で，句読点は「，」と「．」を使う．`pnpm lint`(textlintとmarkdownlint)を通す．
- 規模は12回のIteration(0〜11)で，1回あたり60〜120分とする．1つのIterationで扱うRustの話題は，1つか2つのまとまりに絞る．

## 題材

Gitの互換実装`rgit`を育てる．
完成形の使用例，対応するGitの範囲，各Iterationの内容は[docs/ROADMAP.md](docs/ROADMAP.md)にある．

Gitとの互換の方針は次のとおりである．

- オブジェクトと参照の形式は本物のGitと同じにする．`rgit`が書いたリポジトリを`git`で読め，`git`が書いたリポジトリ(ゆるいオブジェクトだけのもの)を`rgit`で読める．
- コマンドの名前，引数，出力の形式は，本物のGitの同じコマンドに合わせる．出力を簡単にしたコマンド(`commit`，`log`)は，ロードマップにその形式を書く．
- 扱う範囲は，ゆるいオブジェクト，インデックスの版2，ブランチとシンボリック参照である．パックファイル，`packed-refs`，マージ，シンボリックリンクは対象外とする．

## 設計ドキュメント

各パッケージの`design/types.md`の1枚だけとする．記法はMarkdownの中のMermaidの`classDiagram`である．

- モジュールを`namespace`で描き，その中に，モジュールが定義する`struct`，`enum`，`trait`，`type`を`class`で描く．
- モジュールの公開関数は，名前空間の中のクラス`<モジュール名>_mod`(ステレオタイプ`<<module>>`)に書く．Mermaidは同じ名前の名前空間とクラスを描画できないため，この名前にする．
- 標準のトレイトの実装は，クラスの中に`+impl Display`のように書く．外部のクレートと標準ライブラリの型は描かない．
- 関係は，所有`*--`，参照`-->`，実装`..|>`，依存`..>`で描く．
- `scripts/check-design.mjs`で，名前空間とモジュール，クラスと型，`<<module>>`のクラスの関数と公開関数の名前を照合する．
- 書き方の詳細と例は`docs/design.md`にある．

`types.md`はIteration 0で作り，すべてのIterationで更新する．

## 開発環境

### ツール

- Rust 1.98.1をmiseで入れる(`mise.toml`)．エディションは2024とする．
- テストは`cargo test`，整形は`cargo fmt`，リントは`cargo clippy -- -D warnings`を使う．
- 本物の`git`を，結合テストでの照合と，教材の出力の取得に使う．`git`と，バイト列を表示する`xxd`をDockerfileで入れる．
- RustOwl(所有権とライフタイムの可視化)を，miseで入れる．VS Codeの拡張機能`cordx56.rustowl-vscode`を`devcontainer.json`に書く．Iteration 1，4，7のノートで使い方を説明する．
- VS Codeの拡張機能は，ほかに`rust-lang.rust-analyzer`，`vadimcn.vscode-lldb`(デバッガー)，`tamasfe.even-better-toml`，`bierner.markdown-mermaid`を入れる．
- Mermaidの構文検査と設計の照合には，Node(`mermaid`と`jsdom`)を使う．
- 使うクレートは次のとおりである．Iterationごとの追加は，ロードマップの「受講者が行うツール操作」に書く．

| クレート | 用途 | 始まり |
| --- | --- | --- |
| `sha1` | SHA-1 | Iteration 0 |
| `clap`(`derive`) | コマンドラインの解析 | Iteration 2 |
| `flate2` | zlibの圧縮と展開 | Iteration 2 |
| `thiserror` | エラー型 | Iteration 2 |
| `tempfile`(開発用) | テストの一時ディレクトリ | Iteration 2 |

### リポジトリの構成

```text
COURSE.md                          コース計画(教材を作る人向け)
README.md                          コースの概要とIterationの一覧(受講者向け)
Cargo.toml                         模範解答のためのCargoワークスペース
docs/ROADMAP.md                    各Iterationの要件，学ぶこと，図の更新
docs/tdd.md                        テスト駆動開発とテストリストの書き方
docs/design.md                     型とモジュールの図の書き方
docs/rust/README.md                Rustのノートの目次
docs/rust/iteration-NN.md          Iteration NNで初めて使うRustの文法と概念
docs/git/README.md                 Gitのノートの目次
docs/git/iteration-NN.md           Iteration NNで初めて扱うGitの仕組み
scripts/check-mermaid.mjs          Mermaidの構文検査
scripts/check-design.mjs           図と実装の照合
scripts/test-exercises.sh          演習パッケージのビルドとテスト
iterations/iteration-NN/
  exercise/                        受講者が作業する場所
  solution/                        演習を終えた状態と模範解答
```

Iteration番号は，ディレクトリ名とファイル名に2桁(`iteration-00`)で書き，本文に`Iteration 0`と書く．

### パッケージ

- 模範解答のパッケージ名は`rgit-NN-solution`とし，`[lib] name = "rgit"`を書く．ルートの`Cargo.toml`が`members = ["iterations/*/solution"]`でワークスペースに含める．
- 演習のパッケージ名は`rgit`とする．演習はワークスペースに含めない単独のパッケージで，受講者は`exercise/`の中で`cargo`を実行する．
  - Cargoの`exclude`はglobを使えないため，Iterationを作るたびに，ルートの`Cargo.toml`の`exclude`に`iterations/iteration-NN/exercise`を加える．
- 演習の`Cargo.toml`は，1つ前の模範解答の`Cargo.toml`からパッケージ名だけを変えたものとする．そのため模範解答の`Cargo.toml`はワークスペースから設定を継承せず，依存の版を直接書く．
- バイナリ名はパッケージ名になるため，演習と模範解答で異なる．結合テストはバイナリを起動せず，`rgit::cli::run`を呼ぶ．`main.rs`は`cli::run`を呼んでエラーを表示するだけにする．
- 結合テストは，`tempfile::TempDir`にリポジトリを作り，`std::process::Command`で本物の`git`も実行して結果を比べる．`git`には`GIT_CONFIG_NOSYSTEM=1`と`HOME`を一時ディレクトリにした環境を渡し，利用者の設定の影響を受けないようにする．

### コマンド

| 目的 | 受講者(`exercise/`の中) | 教材を作る人(リポジトリのルート) |
| --- | --- | --- |
| ビルド | `cargo build` | `cargo build -p rgit-NN-solution` |
| テスト | `cargo test` | `cargo test -p rgit-NN-solution` |
| 単体テストだけ | `cargo test --lib` | `cargo test -p rgit-NN-solution --lib` |
| 実行 | `cargo run -- 引数` | `cargo run -p rgit-NN-solution -- 引数` |
| 整形とリント | `cargo fmt`，`cargo clippy` | `mise run fmt`，`mise run lint` |
| リポジトリ全体の検査 | なし | `mise run check` |

使用例のように，別のディレクトリで`rgit`を試すときは，`cargo install --path .`で`rgit`をインストールする(Iteration 2で示す)．
RustにはREPLがないため，ノートの例は`cargo test`で動く小さなテストか，`examples/`で試せるプログラムとして書く．

`mise run check`は次を実行する．

- `cargo fmt --all --check`と`cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`(すべての模範解答)
- `scripts/test-exercises.sh`(`Cargo.toml`のあるすべての演習をビルドしてテストする)
- `pnpm lint`(textlint，markdownlint，Mermaidの構文検査，設計の照合)

### 受講者が行うツール操作

完全なコマンドは，初めて使うIterationの演習の手順で示す．それ以降は，することだけを書く．

| Iteration | 操作 |
| --- | --- |
| 0 | `cargo init --lib --name rgit`，`cargo add`，`cargo build`，`cargo test`，`cargo test --lib`，`cargo test フィルター`，`cargo fmt`，`cargo clippy`，`src/main.rs`の追加と`cargo run` |
| 1 | RustOwlの表示(カーソルを変数に合わせる) |
| 2 | `cargo add`の`--features`と`--dev`，`cargo run -- 引数`，`cargo test --test 名前`，`cargo install --path .` |
| 5 | `xxd`によるバイト列の表示 |

### ノート

- Rustの文法と概念は`docs/rust/iteration-NN.md`，Gitの仕組みは`docs/git/iteration-NN.md`に書く．
- それぞれの`README.md`に目次を置き，Iterationを作るたびに更新する．
- Gitのノートは，本物の`git`の配管コマンド(`git cat-file`，`git hash-object`，`git ls-tree`など)で`.git`の中を観察する手順を含める．
- 例は実際に動かした結果を載せる．

## テスト

- 単体テストは，各モジュールのファイルの末尾の`#[cfg(test)] mod tests`に書く．
- 結合テストは`tests/`にコマンドごとのファイルで書く(`tests/hash_object.rs`など)．共通の補助関数は`tests/common/mod.rs`に置く．ファイル名にIteration番号を入れない．
- テストリストは各パッケージの`TESTLIST.md`に書く．見出しは「単体テスト」と「結合テスト」とし，項目はチェックボックスにする．

## Iteration 0の演習の形

Iteration 0の`exercise/`には，Cargoのパッケージがない．受講者が`cargo init`で作る．

```text
iterations/iteration-00/exercise/
  README.md
  TESTLIST.md            見出しだけのひな形
  docs/iteration-00.md   演習の手順
  design/
    types.md             見出しと，描くものを説明するコメント
```

受講者は`cargo init --lib --name rgit`で`Cargo.toml`と`src/lib.rs`を作り，`src/object.rs`と`src/main.rs`を自分で加える．

## 落とし穴

- コンテナにリンカー(`cc`)がないと，`cargo test`が`linker 'cc' not found`で失敗する．Dockerfileで`gcc`と`libc6-dev`を入れる．
- Cargoのワークスペースの`exclude`はglobを受け付けない．演習ディレクトリを1つずつ書く．
- 演習はどれもパッケージ名が`rgit`で版も同じなので，1つのターゲットディレクトリを共有すると，結合テストが別の演習のライブラリにリンクされる．演習ごとにターゲットディレクトリを分ける．
- Gitはゆるいオブジェクトのファイルを読み取り専用(`0444`)で作る．テストでオブジェクトを壊すときは，先に書き込みの権限を付けるか，ファイルを消す．
- Mermaidの`classDiagram`で，名前空間とクラスに同じ名前を付けると，構文検査は通るが描画が終わらなくなる．モジュールの公開関数を書くクラスは`<モジュール名>_mod`とし，`scripts/check-design.mjs`で重なりを検出する．
- RustOwlは，`rustowl toolchain install`で解析用のツールチェーンを入れるまで動かない(`cargo not found`の警告のあと`Failed to create analyzer`になる)．Dockerfileで`mise install`のあとに実行する．
- pnpm 12は，公開から1日たっていない版をロックファイルに入れると`ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION`で失敗する．ロックファイルを作り直すときは，`pnpm-lock.yaml`を消して`pnpm install`を実行する．
- 教材に載せるCargoの出力のパスは，Dev Containerと同じ`/workspaces/...`にする．Cargoはシンボリックリンクを解決した実際のパスを表示するので，リポジトリを`/workspaces`にマウントした環境で実行して出力を取る．
