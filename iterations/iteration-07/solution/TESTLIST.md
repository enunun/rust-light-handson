# テストリスト

## 単体テスト

### refs

- [x] `HEAD`と，`refs/`で始まる名前(`/`を含むものも)は，参照名になる
- [x] 規則に反する名前(`refs/`で始まらない，空の要素，`.`で始まる要素，`..`，空白，`~`，`.lock`で終わる)は，エラーになる
- [x] ブランチ名`topic`から`refs/heads/topic`を作り，ブランチ名を取り出せる．不正なブランチ名は`InvalidBranchName`になる
- [x] 40桁のIDと改行は`Direct`の参照に，`ref: refs/heads/main`は`Symbolic`の参照になる

### lockfile

- [x] `commit`は，`.lock`に書いた中身で元のファイルを置き換え，`.lock`を消す．`commit`までは元のファイルは変わらない
- [x] `commit`せずに捨てると，`.lock`を消し，元のファイルを変えない
- [x] `.lock`がすでにあれば，`Locked`のエラーになる
- [x] 親のディレクトリがなければ作る

### repo

- [x] 作ったばかりのリポジトリの`HEAD`は`refs/heads/main`を指し，そのブランチにはまだコミットがない
- [x] ブランチを更新すると，ファイルにIDと改行を書き，`HEAD`からたどれる
- [x] ブランチ名を名前の順に返す(`feature/login`，`main`，`topic`)

### revision

- [x] `HEAD`，ブランチ名，`refs/heads/main`から，ブランチが指すIDを返す
- [x] 短縮したIDから，オブジェクトのIDを返す
- [x] コミットのない`HEAD`は`ObjectNotFound`になる

### index

- [x] インデックスを`LockFile`で書く(既存のテストが通ったままである)

## 結合テスト

### commit

- [x] 最初のコミットは`[main (root-commit) 6c04901] first`を出力し，`refs/heads/main`を更新する
- [x] 2つのコミットのIDが，同じ操作をした本物の`git commit`と一致し，`git log`でたどれる
- [x] `rev-parse`は，`HEAD`，ブランチ名，短縮したIDを40桁のIDにする
- [x] `cat-file -p HEAD`が，`git cat-file -p HEAD`と一致する
- [x] ブランチの`.lock`があると，`Unable to create '…': File exists.`のエラーになり，ブランチは進まない
- [x] コミットのない`HEAD`の`rev-parse`は，`Not a valid object name HEAD`のエラーになる

### branch

- [x] `branch`は，ブランチを名前の順に並べ，今のブランチに`*`を付ける．`git branch`と一致する
- [x] `branch topic`で作ったブランチは，`HEAD`と同じコミットを指す
- [x] `branch old <rev>`は，指定したコミットを指すブランチを作る
- [x] `/`を含む名前のブランチを作れる
- [x] 不正なブランチ名は`'bad name' is not a valid branch name`のエラーになる
- [x] すでにあるブランチは`a branch named 'topic' already exists`のエラーになる
