# テストリスト

## 単体テスト

### store

- [x] `check_store`を`&dyn ObjectStore`で使う形に変える(`write`が`&self`になる)
- [x] `LooseObjectStore`に8つのスレッドが同じオブジェクトを同時に書いても，すべて同じIDを返し，読め，一時ファイルが残らない
- [x] `MemoryObjectStore`を4つのスレッドで共有して書くと，4つのオブジェクトが入る

### parallel

- [x] 結果は，スレッドの数(1，2，3，8)によらず要素と同じ順に並ぶ
- [x] 空の要素の結果は空になる
- [x] スレッドの数が0なら，1つのスレッドで処理する
- [x] 4つのスレッドを指定すると，複数のスレッドで処理される
- [x] `available_jobs`は1以上である

### repo，revision，tree，revwalk，status

- [x] `&mut`で書き込んでいたテストを，`&`で書き込む形に変える．`add`と`status`にはスレッドの数を渡す

## 結合テスト

### add

- [x] 200個のファイルを`add --jobs 1`，`4`，`16`したインデックスが，本物の`git add`のインデックスと一致し，`git fsck`が成功する

### status

- [x] 50個の追跡していないファイルを含む`status --jobs 1`，`3`，`8`が，`git status --porcelain -uall`と一致する
