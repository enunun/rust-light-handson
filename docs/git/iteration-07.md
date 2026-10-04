# Iteration 7：参照，`HEAD`，ブランチ

Iteration 7では，ブランチと`HEAD`を読み書きし，`git commit`のように，コミットを作ってブランチを進める．
このノートでは，参照の仕組み，シンボリック参照，ロックファイルによる更新を説明する．

## 参照

オブジェクトのIDは覚えにくいので，Gitはコミットに名前を付けられる．この名前を参照(ref)と呼ぶ．
参照は，`.git`の下のファイルで，中身はコミットのIDである．

```console
$ ls .git/refs/heads
main
topic
$ cat .git/refs/heads/main
6c049013df700446ee9afd1bdaa0303bf75d842f
```

`refs/heads/`の下の参照がブランチである．`refs/tags/`の下の参照はタグになる(`rgit`は扱わない)．
ブランチ名に`/`を含めると(`feature/login`)，`refs/heads/feature/`というディレクトリの下のファイルになる．

ブランチを作ることは，参照のファイルを1つ書くことでしかない．中身のコミットはそのままである．

```console
$ git branch topic
$ git branch
* main
  topic
$ git rev-parse topic
6c049013df700446ee9afd1bdaa0303bf75d842f
```

## 参照名の規則

参照名はファイルのパスになるので，ファイルシステムで困る名前や，リビジョンの指定と紛らわしい名前は使えない．

- 空の要素(`refs//main`)，`.`で始まる要素，`.lock`で終わる要素を含まない．
- `..`，空白，制御文字，`~^:?*[\`を含まない．`~`や`^`はリビジョンの指定(`HEAD~1`)で使う．

```console
$ git branch 'bad name'
fatal: 'bad name' is not a valid branch name
$ git branch topic
fatal: a branch named 'topic' already exists
```

本物のGitの規則はもう少し多い．`rgit`は，ここに挙げた規則だけを調べる．

## `HEAD`とシンボリック参照

`HEAD`は，今いるブランチを表す参照である．中身はIDではなく，別の参照の名前である．

```console
$ cat .git/HEAD
ref: refs/heads/main
```

`ref: <参照名>`の形の参照をシンボリック参照と呼ぶ．`HEAD`のIDを求めるには，`refs/heads/main`をたどって，その中身を読む．
`git init`の直後は，`HEAD`は`refs/heads/main`を指すが，`refs/heads/main`のファイルはまだない．
コミットのないこのブランチを，生まれていない(unborn)ブランチと呼ぶ．

```console
$ git rev-parse HEAD
fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.
```

`HEAD`がブランチではなくIDを直接持つ状態を，切り離された`HEAD`(detached HEAD)と呼ぶ．
過去のコミットを`git checkout <ID>`で取り出すと，この状態になる．

## `git commit`

`git commit`は，次の順に処理する．

1. インデックスからtreeを書く(`git write-tree`と同じ)．
2. `HEAD`が指すコミットを親にして，commitオブジェクトを書く(`git commit-tree`と同じ)．生まれていないブランチなら親はない．
3. `HEAD`が指すブランチを，新しいコミットに更新する．切り離された`HEAD`なら，`HEAD`そのものを更新する．

```console
$ git commit -m first
[main (root-commit) 6c04901] first
 2 files changed, 2 insertions(+)
 create mode 100644 hello.txt
 create mode 100644 src/main.rs
```

1行目は，ブランチ名，最初のコミットなら`(root-commit)`，短縮したID，メッセージの1行目である．`rgit commit`は，この1行目だけを表示する．
`HEAD`そのものは`ref: refs/heads/main`のまま変わらない．ブランチが進むので，`HEAD`のたどり着くコミットも変わる．

## ロックファイル

参照やインデックスを書き換えている途中で別の`git`が読むと，書きかけの中身を読んでしまう．
2つの`git`が同時に書き換えると，片方の変更が失われる．

Gitは，ファイルを書き換えるとき，次の手順を踏む．

1. `<ファイル名>.lock`を，すでにあれば失敗する方法で作る．作れたら，そのファイルを書き換える権利を得たことになる．
2. 新しい中身を`.lock`のファイルに書く．
3. `.lock`のファイルの名前を元のファイル名に変えて，置き換える．名前の変更は一瞬で起きるので，読む側が読むのは，古い中身と新しい中身の一方だけである．

途中で失敗したら，`.lock`のファイルを消して終える．
`.lock`のファイルが残っていると，ほかの処理は書き換えられない．

```console
$ touch .git/refs/heads/main.lock
$ git commit -q -m second
fatal: cannot lock ref 'HEAD': Unable to create '/tmp/demo/.git/refs/heads/main.lock': File exists.
```

本物の`git`のメッセージには，ほかの`git`が動いていないなら`.lock`のファイルを消すようにという案内が続く．
