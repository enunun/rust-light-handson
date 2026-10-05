use std::path::Path;
use std::process::Command;

/// `rgit`を`dir`で実行し，標準出力に書いた文字列を返す．
pub fn rgit(dir: &Path, args: &[&str]) -> String {
    let mut out = Vec::new();
    rgit::cli::run(args, dir, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// 本物の`git`を`dir`で実行し，標準出力を返す．利用者の設定は読まない．
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
