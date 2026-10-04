# テストリスト

## 単体テスト

### tree

- [x] treeの中の表記`100644`，`100755`，`120000`，`40000`，`160000`から，それぞれのモードを作る
- [x] 知らない表記`100600`は，`CorruptObject("unknown mode")`になる
- [x] ディレクトリのモードは`040000`と表示し，種類は`tree`である
- [x] 空のtreeは，エントリーを持たない
- [x] 2つのエントリー(ファイル`hello.txt`とディレクトリ`src`)を，順に読む
- [x] IDが20バイトに足りないエントリーは，`CorruptObject("invalid tree entry")`になる
- [x] 名前がUTF-8でないエントリーは，エラーになる
- [x] エントリーは`040000 tree <ID>\tsrc`の形で表示する

## 結合テスト

### ls_tree

- [x] 4つのモードのエントリーを持つtreeの`ls-tree`が，`git ls-tree`と一致する
- [x] treeの`cat-file -p`が，`git cat-file -p`と一致する
- [x] サブディレクトリのtreeを，短縮形で指定して表示する
- [x] blobの`ls-tree`は，`not a tree object`のエラーになる
