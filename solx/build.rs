//!
//! The build script for `slang`.
//!

fn main() {
    println!("cargo:rerun-if-changed=allocator_windows.cpp");

    // Debug builds keep the system allocator, as `ALLOCATOR` in `src/slang.rs` does.
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Always set by Cargo");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").expect("Always set by Cargo");
    match (target_os.as_str(), target_env.as_str()) {
        ("linux", "gnu") => link_mimalloc(),
        ("windows", _) => {
            link_mimalloc();
            link_windows_new_delete();
        }
        _ => {}
    }
}

///
/// Links mimalloc ahead of the C and C++ runtime archives of the static release binary, which
/// would otherwise link their own allocators as well. Libraries of this crate are linked right
/// after its objects, ahead of every dependency, and whole, since the references to them come
/// later.
///
fn link_mimalloc() {
    println!("cargo:rustc-link-lib=static:+whole-archive=mimalloc");
}

///
/// Links the `operator new` and `operator delete` of `allocator_windows.cpp`, whole for the same
/// reason as mimalloc.
///
fn link_windows_new_delete() {
    let mimalloc_include =
        std::env::var("DEP_MIMALLOC_INCLUDE").expect("Always set by mimalloc-sys");
    cc::Build::new()
        .cpp(true)
        .file("allocator_windows.cpp")
        .include(mimalloc_include)
        .link_lib_modifier("+whole-archive")
        .compile("allocator_windows");
}
