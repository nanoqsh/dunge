fn main() {
    use std::process::Command;

    let manifest = format!("{dir}/android/Cargo.toml", dir = env!("CARGO_MANIFEST_DIR"));
    let mut cmd = Command::new("cargo");
    cmd.args(["apk", "run", "--manifest-path", &manifest])
        .args((!cfg!(debug_assertions)).then_some("--release"));

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::process::CommandExt;

        let err = cmd.exec();
        eprintln!("{err}");
    }

    #[cfg(not(target_family = "unix"))]
    {
        if let Err(err) = cmd.spawn() {
            eprintln!("{err}");
        }
    }
}
