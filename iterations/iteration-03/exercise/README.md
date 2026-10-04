# Iteration 3：`cat-file`(演習)

オブジェクトを読んでヘッダーを解析し，`rgit cat-file`で種類，大きさ，内容を表示する．短縮形のIDも受け付ける．

## 進め方

[docs/iteration-03.md](docs/iteration-03.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：曖昧な短縮形の候補を表示する．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-03.md  演習の手順
design/types.md       型とモジュールの図(Iteration 2の模範解答)
src/                  Iteration 2の模範解答のコード
tests/                Iteration 2の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
