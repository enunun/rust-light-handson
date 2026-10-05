# Iteration 4：ツリーの読み取りと`ls-tree`(演習)

treeオブジェクトの内容を，元のバイト列を借りたままエントリーの列に解析し，`rgit ls-tree`と`rgit cat-file -p`で表示する．

## 進め方

[docs/iteration-04.md](docs/iteration-04.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．RustOwlで借用の範囲を見る．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：`ls-tree -r`でサブディレクトリもたどる．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-04.md  演習の手順
design/types.md       型とモジュールの図(Iteration 3の模範解答)
src/                  Iteration 3の模範解答のコード
benches/              Iteration 3のベンチマーク
tests/                Iteration 3の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
