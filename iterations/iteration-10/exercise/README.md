# Iteration 10：`diff`(演習)

2つの列の最短の編集をMyersのアルゴリズムで求め，ファイルの差分をunified形式で表示する`rgit diff`と`rgit diff --cached`を作る．

## 進め方

[docs/iteration-10.md](docs/iteration-10.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：差分のアルゴリズム，ハンク，unified形式，`diff`のサブコマンドを実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：文脈の行数を変える`-U`を加える．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-10.md  演習の手順
design/types.md       型とモジュールの図(Iteration 9の模範解答)
src/                  Iteration 9の模範解答のコード
tests/                Iteration 9の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
