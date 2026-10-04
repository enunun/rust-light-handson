# Iteration 1：オブジェクトIDと短縮形

Iteration 1では，オブジェクトIDを40桁の16進数と20バイトの値の間で変換する型を作る．
このノートでは，Gitがオブジェクトをどう表記し，短縮形をどう扱うかを説明する．

## 20バイトと40桁

オブジェクトIDの正体は，SHA-1の20バイトの値である．
人に見せるときは，1バイトを16進数の2桁にした40桁の文字列で表記する．
Gitの内部では，同じIDを2つの形で使い分ける．

| 形 | 使う場所 |
| --- | --- |
| 40桁の16進数(文字列) | コマンドの入出力，参照のファイル(Iteration 7)，オブジェクトのファイル名(Iteration 2)，commitの中身(Iteration 6) |
| 20バイトの値 | treeの中身(Iteration 4)，インデックス(Iteration 5) |

`rgit`は，内部では常に20バイトの`ObjectId`で持ち，入出力の境界で文字列と変換する．

## 短縮形

40桁は長いので，Gitは先頭の何桁かだけでオブジェクトを指定できる．
`git rev-parse`は，指定したものを40桁のIDに直して表示する．`--short`は逆に，短縮形を表示する．

```console
$ printf 'hello\n' > hello.txt
$ git hash-object -w hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
$ git rev-parse ce01362
ce013625030ba8dba906f756967f9e9ca394464a
$ git rev-parse --short ce013625030ba8dba906f756967f9e9ca394464a
ce01362
```

`git log --oneline`などの表示には，既定で7桁の短縮形を使う．`rgit`も7桁を使う．
本物のGitは，リポジトリのオブジェクトの数が増えると，重ならないように桁数を自動で増やす．

短縮形は4桁から使える．大文字で書いてもよい．3桁以下は受け付けない．

```console
$ git cat-file -p ce01
hello
$ git cat-file -p CE01362
hello
$ git cat-file -p ce0
fatal: Not a valid object name ce0
```

短縮形からIDを探すには，リポジトリのオブジェクトを調べる必要がある．
そのため，短縮形の解決はIteration 3で，オブジェクトを読めるようになってから作る．
Iteration 1の`ObjectId`は，40桁の完全な形だけを受け付ける．
