# Iteration 6：`write-tree`と`commit-tree`(演習)

インデックスから入れ子のtreeオブジェクトを書く`rgit write-tree`と，型状態パターンのビルダーでcommitオブジェクトを組み立てる`rgit commit-tree`を作る．

## 進め方

[docs/iteration-06.md](docs/iteration-06.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：`ls-tree`にコミットを指定できるようにする．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-06.md  演習の手順
design/types.md       型とモジュールの図(Iteration 5の模範解答)
src/                  Iteration 5の模範解答のコード
tests/                Iteration 5の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
