fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=msvcprt");
        cc::Build::new()
            .file("src/compat_stl.c")
            .compile("compat_stl");
    }
}
