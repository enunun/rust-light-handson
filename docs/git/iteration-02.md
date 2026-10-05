# Iteration 2：`.git`の構成とゆるいオブジェクト

Iteration 2では，リポジトリを作り，blobをオブジェクトデータベースに書き込む．
このノートでは，`.git`ディレクトリの中身と，オブジェクトがファイルとしてどう格納されるかを説明する．

## `.git`ディレクトリ

`git init`は，作業ディレクトリの中に`.git`ディレクトリを作る．Gitのデータはすべてこの中にある．

```console
$ git init -q -b main
$ ls -F .git
HEAD
branches/
config
description
hooks/
info/
objects/
refs/
```

| 名前 | 中身 | `rgit`で扱うIteration |
| --- | --- | --- |
| `objects/` | オブジェクトデータベース | 2から |
| `refs/heads/` | ブランチ．ブランチ名のファイルに，コミットのIDを書く | 7から |
| `HEAD` | 今いるブランチ | 2で作り，7から読む |
| `index` | インデックス(ステージングエリア)．`git add`で作られる | 5から |
| `config` | リポジトリの設定 | 扱わない |
| `hooks/`，`info/`など | フックのひな形，除外の設定など | 扱わない |

`HEAD`の中身は，今いるブランチを指す1行である．

```console
$ cat .git/HEAD
ref: refs/heads/main
```

`git`は，`HEAD`，`objects/`，`refs/`の3つがそろったディレクトリをリポジトリとみなす．
`HEAD`がないと，`objects/`があっても`not a git repository`になる．
`rgit init`は，この3つだけを作る．

`git`のコマンドは，カレントディレクトリから親のディレクトリへ順に`.git`を探す．
作業ディレクトリのどこで実行しても，同じリポジトリを使える．

## ゆるいオブジェクト

`git hash-object -w`は，オブジェクトをデータベースに書き込む．

```console
$ printf 'hello\n' > hello.txt
$ git hash-object -w hello.txt
ce013625030ba8dba906f756967f9e9ca394464a
$ find .git/objects -type f
.git/objects/ce/013625030ba8dba906f756967f9e9ca394464a
```

オブジェクトは，IDの先頭2桁のディレクトリの中に，残りの38桁の名前のファイルとして置かれる．
1つのオブジェクトを1つのファイルにする形式を，ゆるいオブジェクト(loose object)と呼ぶ．
先頭2桁でディレクトリを分けるのは，1つのディレクトリに何万ものファイルが並ばないようにするためである．

オブジェクトが多くなると，`git gc`が多数のオブジェクトを1つのパックファイル(`.git/objects/pack/`)にまとめる．
`rgit`はゆるいオブジェクトだけを読み書きする．

## zlibによる圧縮

ファイルの中身は，Iteration 0で作ったヘッダー付きのバイト列(`blob 6\0hello\n`)を，zlibで圧縮したものである．

```console
$ xxd .git/objects/ce/013625030ba8dba906f756967f9e9ca394464a
00000000: 7801 4bca c94f 5230 63c8 48cd c9c9 e702  x.K..OR0c.H.....
00000010: 001d c504 14                             .....
```

先頭の`78 01`は，zlibの形式を示すヘッダーである．その後ろにDeflateで圧縮したデータが続き，最後の4バイトは検査用のAdler-32である．
IDは圧縮する前のバイト列から計算する．そのため，圧縮の強さが違って中身のバイトが変わっても，IDは変わらない．

### 圧縮の強さと速さ

zlibの圧縮の強さは，0(圧縮しない)から9(最も小さくする)までの段階で選ぶ．強くするほど時間がかかる．
`rgit`が使うflate2の`Compression::default()`は6，`Compression::fast()`は1である．
ヘッダーの2バイト目は強さによって変わる．`rgit`の書いたオブジェクトは`78 9c`で始まる．

```console
$ xxd .git/objects/ce/013625030ba8dba906f756967f9e9ca394464a
00000000: 789c 4bca c94f 5230 63c8 48cd c9c9 e702  x.K..OR0c.H.....
00000010: 001d c504 14                             .....
```

本物の`git`は，ゆるいオブジェクトを強さ1で圧縮する(設定`core.looseCompression`の既定の値)．
ゆるいオブジェクトは，ファイルを`add`するたびに書くものなので，小ささより速さを選んでいる．
長く保存するパックファイルは，`git gc`のときに強さ6で圧縮し直す(設定`core.compression`と`pack.compression`)．

SHA-1の計算と比べると，zlibの圧縮はずっと重い．
1MiBのテキストを測ると，SHA-1は1ms未満，強さ6の圧縮は40ms程度かかる．`hash-object -w`の時間の多くは圧縮である．

書き込んだオブジェクトは`git cat-file`で読める．`-t`は種類，`-p`は内容を表示する．

```console
$ git cat-file -t ce01362
blob
$ git cat-file -p ce01362
hello
```

オブジェクトのファイルは読み取り専用で作られる．
オブジェクトの中身はIDで決まるので，一度書いたら書き換えることはない．
同じIDのファイルがすでにあれば，書き込みを省ける．
