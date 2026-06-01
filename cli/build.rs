use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

const LIB_PFF_TAG: &str = "20231205";

enum LibPffConfigureMode {
    Native,
    HostMingw32BuildLinuxGnu,
    HostLinuxMuslBuildLinuxGnu,
}

fn download_libpff(libpff_dir: &Path) {
    let libpff_repo = "https://github.com/libyal/libpff";
    // The release tarball vendors the libyal dependencies at pinned versions and
    // ships a generated `configure`, so the build never fetches sources at build time.
    let archive_url =
        format!("{libpff_repo}/releases/download/{LIB_PFF_TAG}/libpff-alpha-{LIB_PFF_TAG}.tar.gz");
    let tarball_path = format!("libpff-{LIB_PFF_TAG}.tar.gz");

    let status = Command::new("curl")
        .args(["--fail", "-L", &archive_url, "-o", &tarball_path])
        .status()
        .expect("Could not get curl status");

    if !status.success() {
        println!("cargo:error=Failed to download libpff release");
        exit(1);
    }

    let status = Command::new("tar")
        .args(["-xzf", &tarball_path])
        .status()
        .expect("Could not get tar status");

    if !status.success() {
        println!("cargo:error=Failed to extract libpff");
        exit(1);
    }

    fs::rename(format!("libpff-{LIB_PFF_TAG}"), libpff_dir).expect("Could not move lib pff dir");
    fs::remove_file(tarball_path).expect("Could not delete tarball path");
}

fn build_libpff(libpff_dir: &Path, configure_mode: LibPffConfigureMode) {
    env::set_current_dir(libpff_dir).expect("Could not cd into libpff dir");

    let configure_status = match configure_mode {
        LibPffConfigureMode::Native => Command::new("./configure")
            .arg("--disable-shared")
            .status()
            .expect("Could not get configure status (native)"),
        LibPffConfigureMode::HostMingw32BuildLinuxGnu => Command::new("./configure")
            .arg("--host=x86_64-w64-mingw32")
            .arg("--build=x86_64-pc-linux-gnu")
            .arg("--disable-shared")
            .status()
            .expect("Could not get configure status (HostMingw32BuildLinuxGnu)"),
        LibPffConfigureMode::HostLinuxMuslBuildLinuxGnu => Command::new("./configure")
            .arg("--host=x86_64-unknown-linux-musl")
            .arg("--build=x86_64-unknown-linux-gnu")
            .arg("--disable-shared")
            .env("CC", "musl-gcc")
            .status()
            .expect("Could not get configure status (HostLinuxMuslBuildLinuxGnu)"),
    };

    if !configure_status.success() {
        println!("cargo:error=Could not run configure");
        exit(1);
    }

    let make_status = Command::new("make")
        .status()
        .expect("Could not get make status");
    if !make_status.success() {
        println!("cargo:error=Could not run make");
        exit(1);
    }
}

fn libpff_built(deps_dir: &Path) -> bool {
    let libs_path = get_libs_dir(deps_dir);
    libs_path.join("libpff.a").exists()
}

fn download_and_build_libpff() {
    let deps_dir = get_deps_dir();
    let libpff_dir = deps_dir.join(format!("libpff-{LIB_PFF_TAG}"));

    if !libpff_dir.exists() {
        download_libpff(&libpff_dir);
    }

    // Need to check if built here
    if !libpff_built(&deps_dir) {
        let configure_mode = get_lib_pff_configure_mode();
        build_libpff(&libpff_dir, configure_mode);
    }
}

fn get_lib_pff_configure_mode() -> LibPffConfigureMode {
    let target = std::env::var("TARGET").expect("Could not get target");

    if cfg!(target_os = "linux") && target == "x86_64-pc-windows-gnu" {
        LibPffConfigureMode::HostMingw32BuildLinuxGnu
    } else if target == "x86_64-unknown-linux-musl" {
        LibPffConfigureMode::HostLinuxMuslBuildLinuxGnu
    } else {
        LibPffConfigureMode::Native
    }
}

fn get_deps_dir() -> PathBuf {
    let out_dir_path = env::var("OUT_DIR").expect("Could not get output dir path");
    let out_dir = Path::new(&out_dir_path);
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Could not get profile dir");

    let deps_dir = profile_dir
        .join("deps")
        .canonicalize()
        .expect("Could not canonicalize deps dir");

    if !deps_dir.exists() {
        panic!("Deps dir does not exist")
    }

    deps_dir
}

fn get_libs_dir(deps_dir: &Path) -> PathBuf {
    deps_dir
        .join(format!("libpff-{LIB_PFF_TAG}"))
        .join("libpff")
        .join(".libs")
}

fn main() {
    download_and_build_libpff();

    println!(
        "cargo:rustc-link-search=native={}",
        get_libs_dir(&get_deps_dir()).to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static=pff");
    println!("cargo:rustc-link-lib=static=z");
}
