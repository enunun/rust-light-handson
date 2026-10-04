# Iteration 2：`init`と`hash-object -w`(演習)

コマンドラインのプログラム`rgit`を作り，`rgit init`でリポジトリを作り，`rgit hash-object -w`でblobをオブジェクトデータベースに書き込む．

## 進め方

[docs/iteration-02.md](docs/iteration-02.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめ，依存を加える．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：作り直した`init`のメッセージを変える．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-02.md  演習の手順
design/types.md       型とモジュールの図(Iteration 1の模範解答)
src/                  Iteration 1の模範解答のコード
tests/                Iteration 1の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
