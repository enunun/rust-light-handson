# テストリスト

## 単体テスト

### object

- [x] `blob_bytes`は，`hello\n`から`blob 6\0hello\n`を作る

### repo

- [x] `init`は，`.git/objects`，`.git/refs/heads`と，`ref: refs/heads/main`を書いた`.git/HEAD`を作る
- [x] `discover`は，2階層下のディレクトリから，親のリポジトリを見つける
- [x] `discover`は，リポジトリの外では`NotARepository`になる
- [x] `write_blob`は，`hello\n`をzlibで圧縮して`.git/objects/ce/013625…`に書き，そのIDを返す

## 結合テスト

### init

- [x] `init`は，`Initialized empty Git repository in <.gitの絶対パス>/`を出力する
- [x] `init project`は，`project/.git`を作る
- [x] 本物の`git`が，`rgit init`で作ったリポジトリを認める(`git rev-parse --git-dir`が`.git`を出力する)

### hash_object

- [x] `-w`なしの`hash-object`は，リポジトリの外でもIDを出力する
- [x] `-w`なしの`hash-object`は，オブジェクトを書き込まない
- [x] `-w`付きで書いたオブジェクトを，本物の`git cat-file`で読める
- [x] `-w`付きの`hash-object`は，サブディレクトリから親のリポジトリを見つける
- [x] リポジトリの外での`-w`付きの`hash-object`は，`not a git repository …`のエラーになる
