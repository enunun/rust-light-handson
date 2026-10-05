# Iteration 9：オブジェクトストアの抽象化と`status`(模範解答)

演習を終えた状態の`rgit`と，テストリストと図の模範解答である．
`rgit init`，`hash-object`，`cat-file`，`ls-tree`，`add`，`ls-files`，`write-tree`，`commit-tree`，`commit`，`rev-parse`，`branch`，`log`，`status`を実行できる．

## 実行の仕方

このディレクトリで`cargo test`を実行すると，単体テストと結合テストが実行される．
別のディレクトリで試すには，別名を作る．

```console
$ alias rgit="cargo run -q --manifest-path $PWD/Cargo.toml --"
$ cd /tmp/demo
$ rgit init
Initialized empty Git repository in /tmp/demo/.git/
```

## ディレクトリの構成

```text
README.md              このファイル
Cargo.toml             パッケージrgit-09-solution(ライブラリの名前はrgit)
TESTLIST.md            テストリストの模範解答
docs/iteration-09.md   演習の各手順の解説
design/types.md        型とモジュールの図の模範解答
src/lib.rs             公開する名前をまとめる
src/main.rs            cli::runを呼び，エラーを表示する
src/cli.rs             引数の解析とサブコマンドの実行，環境変数からの署名
src/commit.rs          署名，commitオブジェクト，ビルダー
src/repo.rs            リポジトリの作成，発見，参照の読み書き，短縮形の解決，add，コミット
src/revision.rs        リビジョンの指定の解決(`~N`を含む)
src/revwalk.rs         コミットのグラフをたどるイテレーター
src/status.rs          HEAD，インデックス，作業ディレクトリの比較
src/store.rs           オブジェクトストアのトレイトと2つの実装
src/error.rs           エラー型
src/index.rs           インデックスの読み書き
src/lockfile.rs        ロックファイル
src/object.rs          オブジェクトの種類，バイト列，ヘッダーの解析
src/oid.rs             オブジェクトIDの型
src/refs.rs            参照名と参照
src/tree.rs            treeのモード，エントリー，解析，書き込み，展開
src/worktree.rs        作業ディレクトリのファイルの収集
benches/object.rs      ベンチマーク(SHA-1の計算とzlibの圧縮)
tests/common/mod.rs    結合テストの補助関数(rgitと本物のgitの実行，作者とコミッターの環境変数)
tests/init.rs          結合テスト(init)
tests/hash_object.rs   結合テスト(hash-object)
tests/cat_file.rs      結合テスト(cat-file)
tests/ls_tree.rs       結合テスト(ls-tree)
tests/add.rs           結合テスト(add，ls-files)
tests/write_tree.rs    結合テスト(write-tree)
tests/commit_tree.rs   結合テスト(commit-tree)
tests/commit.rs        結合テスト(commit，rev-parse)
tests/branch.rs        結合テスト(branch)
tests/log.rs           結合テスト(log)
tests/status.rs        結合テスト(status)
tests/hash_blob.rs     結合テスト(blobのハッシュ)
tests/object_id.rs     結合テスト(オブジェクトID)
```
