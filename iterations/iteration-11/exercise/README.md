# Iteration 11：`add`と`status`の並列化(演習)

オブジェクトストアを複数のスレッドから書き込める形に変え，`rgit add`と`rgit status`のファイルの処理を複数のスレッドで行う．

## 進め方

[docs/iteration-11.md](docs/iteration-11.md)の手順に従って，次の順に進める．

1. 準備：引き継いだテストが通ることを確かめる．
2. 文法と概念：ノートを読み，小さな課題で確かめる．
3. テストリスト：`TESTLIST.md`に，確かめる振る舞いを書き出す．
4. 図：`design/types.md`を更新する．
5. テスト駆動の実装：オブジェクトストアの書き込みを`&self`にし，並列に処理する関数と`--jobs`を実装し，スレッドの数ごとの速さを測る．
6. 振り返り：模範解答と比べ，図と実装を見比べる．
7. 発展課題：エラーが起きたら残りの処理をやめる関数と，インデックスに記録したファイルの状態でハッシュの計算を省く`status`を作る．

## ディレクトリの構成

```text
README.md             このファイル
Cargo.toml            パッケージrgit
TESTLIST.md           テストリスト(見出しだけのひな形)
docs/iteration-11.md  演習の手順
design/types.md       型とモジュールの図(Iteration 10の模範解答)
src/                  Iteration 10の模範解答のコード
benches/              Iteration 10のベンチマーク
tests/                Iteration 10の結合テスト
```

模範解答は[../solution/](../solution/README.md)にある．
