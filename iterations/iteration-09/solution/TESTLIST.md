# テストリスト

## 単体テスト

### store

- [x] `MemoryObjectStore`は，書いたオブジェクトを読め，IDの先頭で探せ，ないオブジェクトは`ObjectNotFound`になる
- [x] `LooseObjectStore`も，同じ振る舞いをする(同じ確認の関数を`&mut dyn ObjectStore`で使う)
- [x] `LooseObjectStore`は，zlibで圧縮した内容を`ce/013625…`に書く(`Repository::write_blob`のテストを移す)
- [x] `find`は，先頭が同じ2つのオブジェクトをIDの順に返す

### tree

- [x] `write_tree`は，`MemoryObjectStore`にサブディレクトリを入れ子のtreeにして書く(`Repository::write_tree`のテストを移す)
- [x] 空のインデックスの`write_tree`は，空のtreeになる(同上)
- [x] `flatten_tree`は，入れ子のtreeを，パスとモードとIDの表にする

### revwalk

- [x] 既存の4つのテストを，`MemoryObjectStore`で書く形に変える

### status

- [x] コミットしたままの作業ディレクトリは，何も表示しない
- [x] コミットの前に`add`したファイルは`A  hello.txt`になる
- [x] 変えたファイルはYが`M`になり，`add`するとXが`M`になり，さらに変えると`MM`になる
- [x] 消したファイルはYが`D`になり，インデックスからも除くとXが`D`になる
- [x] 追跡していないファイルは，変更のあるファイルのあとにパスの順で`??`になる
- [x] モードだけが違っても変更になる

### repo

- [x] `read_object`と`write_blob`のテストを`store`に移し，残りのテストは`objects_mut()`で書く形に変える

## 結合テスト

### status

- [x] コミットしたままのリポジトリの`status`は空で，`git status --porcelain -uall`と一致する
- [x] 最初のコミットの前の`status`が`A  hello.txt`と`?? todo.txt`を表示し，`git`と一致する
- [x] 変更，削除，追加，モードの変更，追跡していないファイルを含む`status`が，`git`と一致する
- [x] 消したファイルを`add .`したあとの`status`が`D  d.txt`になり，`git`と一致する
