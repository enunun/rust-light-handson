# Iteration 3：オブジェクトのヘッダーと短縮形の解決

Iteration 3では，ゆるいオブジェクトを読み，種類，大きさ，内容を表示する`cat-file`を作る．
このノートでは，オブジェクトを読む手順と，短縮形のIDからオブジェクトを探す手順を説明する．

## オブジェクトを読む手順

Iteration 2で書いたオブジェクトのファイルは，`<種類> <大きさ>\0<内容>`をzlibで圧縮したものである．
読むときは，書くときと逆の順に戻す．

1. IDからファイルのパス(`.git/objects/ce/013625…`)を作り，開く．ファイルがなければ，そのオブジェクトはない．
2. zlibで展開する．
3. 最初のNUL(`\0`)までをヘッダーとし，空白で種類と大きさに分ける．
4. NULの後ろを内容とし，その長さがヘッダーの大きさと一致することを確かめる．

種類は`blob`，`tree`，`commit`，`tag`の4つである．`rgit`は`tag`以外の3つを扱う．

## `git cat-file`

`git cat-file`は，オブジェクトの種類(`-t`)，大きさ(`-s`)，内容(`-p`)を表示する．

```console
$ git cat-file -t ce01362
blob
$ git cat-file -s ce01362
6
$ git cat-file -p ce01362
hello
```

大きさは，ヘッダーに書かれた内容のバイト数である．圧縮したファイルの大きさではない．
`-t`，`-s`，`-p`は，どれか1つだけを指定する．

```console
$ git cat-file -t -p ce01362
error: -p is incompatible with -t
```

`-p`は，blobとcommitでは内容をそのまま表示する．treeの内容はバイナリーなので，`-p`は読みやすい形に整えて表示する(Iteration 4)．

## 短縮形の解決

短縮形のIDからオブジェクトを探すには，先頭2桁のディレクトリの中から，残りの桁で始まる名前のファイルを探す．
見つかったファイルが1つなら，それが指すオブジェクトである．

見つからなければ`Not a valid object name`になる．2つ以上見つかれば，どれを指すか決められない．

```console
$ printf '195\n' > a.txt
$ printf '389\n' > b.txt
$ git hash-object -w a.txt b.txt
6bb2f98fb0227744dff2c9023c2a8d53cc721588
6bb2f4ee89f3ff56785055f588c560ce557d0655
$ git cat-file -t 6bb2
error: short object ID 6bb2 is ambiguous
hint: The candidates are:
hint:   6bb2f4e blob
hint:   6bb2f98 blob
fatal: Not a valid object name 6bb2
$ git cat-file -t 6bb2f9
blob
```

この2つのblobは，先頭の4桁が偶然そろっている．5桁目まで指定すれば1つに決まる．
`rgit`は，曖昧なときに`short object ID 6bb2 is ambiguous`のエラーにする．候補の一覧は表示しない．

先頭2桁でディレクトリを分けているので，探すのは1つのディレクトリの中だけで済む．
短縮形が4桁以上なのは，2桁のディレクトリ名に加えて，ファイル名を少なくとも2桁で絞り込むためである．
