# Iteration 1：オブジェクトID(模範解答)

演習を終えた状態の`rgit`と，テストリストと図の模範解答である．
オブジェクトIDの型`ObjectId`を持ち，`hash_blob`は`ObjectId`を返す．

## 実行の仕方

このディレクトリで`cargo test`を実行すると，単体テストと結合テストが実行される．

```console
$ printf 'hello\n' | cargo run -q
ce013625030ba8dba906f756967f9e9ca394464a
```

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit-01-solution(ライブラリの名前はrgit)
TESTLIST.md           テストリストの模範解答
docs/iteration-01.md  演習の各手順の解説
design/types.md       型とモジュールの図の模範解答
src/lib.rs            公開する名前をまとめる
src/oid.rs            オブジェクトIDの型と，その単体テスト
src/object.rs         blobのハッシュの計算と，その単体テスト
src/main.rs           標準入力のハッシュを出力する
tests/hash_blob.rs    結合テスト(blobのハッシュ)
tests/object_id.rs    結合テスト(オブジェクトID)
```
