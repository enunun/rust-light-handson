# Iteration 7：参照，`commit`，`branch`

このIterationでは，参照とブランチを読み書きし，`rgit commit`でコミットを作ってブランチを進める．
参照名を検査済みの型で表し，ファイルの安全な置き換えを，片付けを型に任せるロックファイルで行う．

## 7-1 準備

このディレクトリ(`iterations/iteration-07/exercise`)に移動し，`cargo test`で，Iteration 6から引き継いだテストがすべて通ることを確かめる．

## 7-2 文法と概念

次の2つのノートを読む．

- [Rustのノート](../../../../docs/rust/iteration-07.md)：検査済みの値だけを持つ型，`Drop`とRAII，値を消費するメソッド，RustOwlでムーブを見る，ファイルの排他的な作成と置き換え，`as_deref`
- [Gitのノート](../../../../docs/git/iteration-07.md)：参照，参照名の規則，`HEAD`とシンボリック参照，`git commit`の手順，ロックファイル

読み終えたら，`src/lib.rs`の末尾に`#[cfg(test)] mod practice`を作り，次の課題を確かめる．

1. 偶数だけを持つ`struct Even(u32)`に`TryFrom<u32>`を実装し，奇数は`Err(String)`にする．
2. 作るときに`TempDir`の中にファイルを書き，`Drop`でそのファイルを消す`struct Marker`を作る．内側の`{ }`の中で作り，`{ }`を出たあとにファイルが消えていることを確かめる．
3. `self`を受け取るメソッド`use_once`を持つ`struct Ticket`を作る．`use_once`を2回呼ぶと，どんなエラーになるか．
4. `OpenOptions`の`create_new(true)`で同じファイルを2回開き，2回目の`ErrorKind`を確かめる．
5. `Option<String>`の`as_deref().unwrap_or("HEAD")`を，`Some`と`None`で確かめる．

確かめ終えたら，`mod practice`を消す．

## 7-3 テストリスト

### 要件

- 参照名を表す型`RefName`を作る．`HEAD`か，`refs/`で始まる名前だけを受け付ける．
  - 空の要素，`.`で始まる要素，`..`，空白，`~^:?*[\`，末尾の`/`と`.lock`を含む名前はエラーにする．
- 参照はファイル`.git/<参照名>`に書く．中身は40桁のIDか，`ref: <参照名>`(シンボリック参照)である．
- 参照とインデックスを更新するときは`<ファイル名>.lock`を作って書き込み，名前を変えて置き換える．
  - `.lock`がすでにあればエラーにする．途中で失敗したら`.lock`を消す．
- `rgit commit -m <message>`は，インデックスから`write-tree`をし，`HEAD`が指すコミットを親にしてcommitを作り，`HEAD`が指すブランチを更新する．
  - 出力は`[<ブランチ名> <短縮ID>] <メッセージの1行目>`で，最初のコミットでは`(root-commit)`を付ける．
- `rgit rev-parse <rev>`は，`HEAD`，ブランチ名，40桁または短縮形のIDを，40桁のIDにして出力する．
- `rgit branch`はブランチの一覧を，今のブランチに`*`を付けて出力する．`rgit branch <name> [<rev>]`はブランチを作る．
  - 不正なブランチ名は`'<name>' is not a valid branch name`，すでにあるブランチは`a branch named '<name>' already exists`のエラーにする．
- `cat-file`，`ls-tree`，`commit-tree`のオブジェクトの指定にも，`rev-parse`と同じ形を使える．

### 使用例

```console
$ rgit commit -m first
[main (root-commit) 6c04901] first
$ rgit branch topic
$ rgit branch
* main
  topic
$ rgit rev-parse topic
6c049013df700446ee9afd1bdaa0303bf75d842f
```

### 作るもの

| モジュール | 作るもの |
| --- | --- |
| `refs` | `pub struct RefName(String)`，`impl TryFrom<&str>`，`RefName::branch`，`pub enum Ref { Direct(ObjectId), Symbolic(RefName) }` |
| `lockfile` | `pub struct LockFile`，`LockFile::acquire`，`LockFile::write_all`，`LockFile::commit(self)`，`impl Drop` |
| `revision` | `pub fn resolve(repo: &Repository, rev: &str) -> Result<ObjectId, Error>` |
| `repo` | 参照を読み書きするメソッド(`read_ref`，`final_ref_name`，`resolve_ref`，`update_ref`，`branches`)，`Repository::commit` |
| `index` | `Index::save`は`LockFile`で書く(リファクタリング) |
| `cli` | `commit`，`rev-parse`，`branch`のサブコマンド |

### 書くときに考えること

- 参照名の規則は，1つの規則につき1つの不正な名前を並べると，どの規則の検査を忘れたかが分かる．
- `LockFile`の振る舞いは，置き換える場合，置き換えずに捨てる場合，`.lock`がすでにある場合に分けられる．
- `HEAD`には，ブランチを指す場合，まだコミットのないブランチを指す場合がある．どちらも確かめる．
- 結合テストでは，同じ操作を本物の`git`でした別のリポジトリと，コミットのIDを比べられる．作者とコミッターの環境変数は，補助関数がそろえる．

## 7-4 図

`design/types.md`を，このIterationの終わりの状態に更新する．

- 新しいモジュール`refs`，`lockfile`，`revision`の名前空間を加える．`Ref`の列挙子は何を持つか．
- `LockFile`の`commit`は`self`を受け取る．`+commit(self) Result`のように書くと区別できる．
- `Repository`に加わるメソッドと，`Repository`と`Index`から`LockFile`への依存を描く．

## 7-5 テスト駆動の実装

### 実装のヒント

- `RefName`のフィールドは非公開にし，`try_from`と，ブランチ名から作る`branch`だけで作らせる．
- `LockFile::acquire`は，`<path>.lock`を`create_new(true)`で開く．`ErrorKind::AlreadyExists`なら`Locked`のエラーにする．
- `LockFile`は，置き換えたかどうかを`bool`のフィールドで持ち，`Drop`では置き換えていなければ`.lock`を消す．
- `Ref::parse`は，`ref:`と空白で始まれば`Symbolic`，そうでなければ40桁のIDとして読む．ファイルの末尾の改行は`trim_end`で除く．
- `HEAD`のように参照が参照を指す場合，`Direct`か，ファイルのない参照にたどり着くまでくり返したどる．循環した参照で止まらないように，たどる回数に上限を付ける．
- `commit`では，`HEAD`のたどり着いた参照名を更新する．`HEAD`そのものを書き換えると，ブランチではなく`HEAD`が進んでしまう．
- `branches`は，Iteration 5の`list_files`を`refs/heads`に使うと，`feature/login`のような名前も集められる．
- `rev-parse`は，参照名として読めて，その参照があればそのIDを，なければ`resolve_prefix`の結果を返す．

## 7-6 振り返り

1. 自分の`TESTLIST.md`を[模範解答のテストリスト](../../solution/TESTLIST.md)と比べる．
2. `update_ref`が`&str`の参照名を受け取る設計と比べる．検査はどこで何回行われるか．検査を忘れたときに何が起きるか．
3. `LockFile`の`Drop`がない場合，`update_ref`が途中の`?`で返ると，何が残るか．それを防ぐには，ほかの言語では何を書くか．
4. `commit`が`&mut self`を受け取る設計と比べて，何が防げるか．
5. 図と実装を見比べ，違うところがあれば図を直す．

## 7-7 発展課題

`git branch -d <name>`は，ブランチを消し，`Deleted branch <name> (was <短縮ID>).`と出力する．今のブランチは消せない．
`rgit branch -d <name>`を作る．

```console
$ rgit branch -d topic
Deleted branch topic (was 6c04901).
```

これまでと同じく，テストリスト，図，実装の順に進める．
