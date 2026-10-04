# テストリスト

## 単体テスト

### object

- [x] `ObjectKind::Commit`は`commit`と表示し，`tree`から`ObjectKind::Tree`を作る
- [x] 知らない種類`tag`は，`CorruptObject`になる
- [x] `encode`は，種類と内容から`blob 6\0hello\n`を作る(`blob_bytes`のテストを移す)
- [x] `parse_header`は，`blob 6\0hello\n`を種類`Blob`と内容`hello\n`に分ける
- [x] NULのないバイト列は，`CorruptObject("missing header")`になる
- [x] 大きさが内容と合わないバイト列は，`CorruptObject("size mismatch")`になる

### repo

- [x] `read_object`は，`write_blob`で書いたblobの種類と内容を返す
- [x] `read_object`は，ないオブジェクトでは，そのIDを含む`ObjectNotFound`になる
- [x] `resolve_prefix`は，4桁，大文字の7桁，40桁のどれからも，ただ1つのオブジェクトを見つける
- [x] `resolve_prefix`は，3桁では`ObjectNotFound`になる
- [x] `resolve_prefix`は，一致するオブジェクトがなければ`ObjectNotFound`になる
- [x] `resolve_prefix`は，2つのオブジェクトに一致すれば`AmbiguousObject`になり，6桁なら1つに決まる

## 結合テスト

### cat_file

- [x] 本物の`git`が書いたblobの種類`blob`を表示する(`-t`)
- [x] blobの大きさ`6`を表示する(`-s`)
- [x] blobの内容`hello\n`を表示する(`-p`)
- [x] 本物の`git`が作ったcommitの種類，内容，大きさを，`git cat-file`と同じに表示する
- [x] ないオブジェクトは`Not a valid object name abcd`のエラーになる
- [x] 曖昧な短縮形は`short object ID 6bb2 is ambiguous`のエラーになる
- [x] `-t`と`-p`を同時に指定すると，引数の誤りになる
