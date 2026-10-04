# Iteration 8：`log`(演習)

コミットのグラフを時刻の新しい順にたどるイテレーター`RevWalk`を作り，`rgit log`で履歴を表示する．`HEAD~1`のような指定も受け付ける．

## 進め方

[docs/iteration-08.md](docs/iteration-08.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：`log --first-parent`を作る．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-08.md  演習の手順
design/types.md       型とモジュールの図(Iteration 7の模範解答)
src/                  Iteration 7の模範解答のコード
tests/                Iteration 7の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
