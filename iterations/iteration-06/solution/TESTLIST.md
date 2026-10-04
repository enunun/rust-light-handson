# テストリスト

## 単体テスト

### tree

- [x] 名前のバイト順に並べる(`a.txt`は`b.txt`より前)
- [x] ディレクトリは名前の後ろに`/`があるものとして並べる(`a-b`，`a.txt`，ディレクトリ`a`の順．ファイル`a`は`a.txt`より前)
- [x] `tree_bytes`は，エントリーを並べ替えて内容にし，`parse_tree`で読み戻せる

### commit

- [x] 署名は`Alice <alice@example.com> 1767225600 +0900`の形で表示し，西のずれは`-0130`と表示する
- [x] 署名の文字列を読み，表示し直すと元に戻る
- [x] タイムゾーンは符号と4桁の数字でなければ読めない
- [x] ビルダーは，呼ぶ順によらず，指定したtree，親(順番どおり)，メッセージでコミットを作る
- [x] commitの内容は，`tree`，`author`，`committer`の行，空行，メッセージからなる
- [x] commitの内容を読み，直列化し直すと元に戻る(親と複数行のメッセージを含む)
- [x] `tree`の行のないcommitは読めない

### repo

- [x] `write_tree`は，サブディレクトリを入れ子のtreeにして書く
- [x] 空のインデックスの`write_tree`は，空のtree`4b825dc…`になる

## 結合テスト

### write_tree

- [x] `add .`のあとの`write-tree`が，`git write-tree`と一致する(`a-b`，`a.txt`，`a/b`を含む)
- [x] `rgit`が書いたtreeを，`git ls-tree -r`で読める
- [x] `write-tree`は，作業ディレクトリではなくインデックスの中身を書く

### commit_tree

- [x] 同じ作者とコミッターの環境変数で，`commit-tree`のIDが`git commit-tree`と一致する
- [x] `-p`で親を指定した`commit-tree`のIDが`git commit-tree`と一致し，`git log`でたどれる
- [x] 環境変数`GIT_AUTHOR_NAME`がなければ，`environment variable GIT_AUTHOR_NAME is not set`のエラーになる
- [x] ないtreeを指定すると`Not a valid object name abcd`のエラーになる

### 既存のテスト

- [x] `cli::run`に環境変数を渡すように，結合テストの補助関数を変える．エラーを確かめるテストは，補助関数`rgit_error`を使う形に変える
