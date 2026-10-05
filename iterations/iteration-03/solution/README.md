# Iteration 3：`cat-file`(模範解答)

演習を終えた状態の`rgit`と，テストリストと図の模範解答である．
`rgit init`，`rgit hash-object [-w]`，`rgit cat-file (-t | -s | -p)`を実行できる．

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
Cargo.toml             パッケージrgit-03-solution(ライブラリの名前はrgit)
TESTLIST.md            テストリストの模範解答
docs/iteration-03.md   演習の各手順の解説
design/types.md        型とモジュールの図の模範解答
src/lib.rs             公開する名前をまとめる
src/main.rs            cli::runを呼び，エラーを表示する
src/cli.rs             引数の解析とサブコマンドの実行
src/repo.rs            リポジトリの作成，発見，オブジェクトの読み書き，短縮形の解決
src/error.rs           エラー型
src/object.rs          オブジェクトの種類，バイト列，ヘッダーの解析
src/oid.rs             オブジェクトIDの型
benches/object.rs      ベンチマーク(SHA-1の計算とzlibの圧縮)
tests/common/mod.rs    結合テストの補助関数(rgitと本物のgitの実行)
tests/init.rs          結合テスト(init)
tests/hash_object.rs   結合テスト(hash-object)
tests/cat_file.rs      結合テスト(cat-file)
tests/hash_blob.rs     結合テスト(blobのハッシュ)
tests/object_id.rs     結合テスト(オブジェクトID)
```
