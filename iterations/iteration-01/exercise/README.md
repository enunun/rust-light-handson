# Iteration 1：オブジェクトID(演習)

オブジェクトIDを表す型`ObjectId`を作り，`hash_blob`が`ObjectId`を返すようにする．

## 進め方

[docs/iteration-01.md](docs/iteration-01.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．RustOwlを使ってみる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：`ObjectId`の`Debug`の表示を16進数にする．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-01.md  演習の手順
design/types.md       型とモジュールの図(Iteration 0の模範解答)
src/                  Iteration 0の模範解答のコード
tests/hash_blob.rs    Iteration 0の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
