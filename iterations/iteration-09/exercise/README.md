# Iteration 9：オブジェクトストアの抽象化と`status`(演習)

オブジェクトの読み書きをトレイト`ObjectStore`にまとめ，ディスクとメモリーの2つの実装を作る．そのうえで，HEAD，インデックス，作業ディレクトリを比べる`rgit status`を作る．

## 進め方

[docs/iteration-09.md](docs/iteration-09.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：トレイトへのリファクタリングと，`status`を実装し，速度を測る．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：書き込みの回数を数えるオブジェクトストアを作る．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-09.md  演習の手順
design/types.md       型とモジュールの図(Iteration 8の模範解答)
src/                  Iteration 8の模範解答のコード
benches/              Iteration 8のベンチマーク
tests/                Iteration 8の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
