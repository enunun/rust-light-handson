# Iteration 0：blobのハッシュの計算(模範解答)

演習を終えた状態の`rgit`と，テストリストと図の模範解答である．
バイト列からGitのblobのIDを計算する`hash_blob`を持つ．

## 実行の仕方

このディレクトリで`cargo test`を実行すると，単体テストと結合テストが実行される．
`cargo run`は，標準入力のハッシュを出力する．

```console
$ printf 'hello\n' | cargo run -q
ce013625030ba8dba906f756967f9e9ca394464a
```

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit-00-solution(ライブラリの名前はrgit)
TESTLIST.md           テストリストの模範解答
docs/iteration-00.md  演習の各手順の解説
design/types.md       型とモジュールの図の模範解答
src/lib.rs            公開する名前をまとめる
src/object.rs         blobのハッシュの計算と，その単体テスト
src/main.rs           標準入力のハッシュを出力する
tests/hash_blob.rs    結合テスト
```
