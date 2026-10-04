# Iteration 7：参照，`commit`，`branch`(演習)

検査済みの参照名の型と，片付けを型に任せるロックファイルを作り，`rgit commit`でブランチを進め，`rgit branch`と`rgit rev-parse`でブランチを扱う．

## 進め方

[docs/iteration-07.md](docs/iteration-07.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．RustOwlでムーブを見る．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：テストリストの項目を1つずつ実装する．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：`branch -d`でブランチを消す．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-07.md  演習の手順
design/types.md       型とモジュールの図(Iteration 6の模範解答)
src/                  Iteration 6の模範解答のコード
tests/                Iteration 6の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
