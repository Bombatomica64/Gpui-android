//! Builds Subsecond patches of the lab crate and pushes them to the phone
//! (docs/hot-patching-plan.md, A1). Debug arm64 only.
//!
//!   hotpatch fat [cargo args]      arm64 debug APK whose library records how
//!                                  rustc compiled and linked the lab crate
//!   hotpatch watch                 on each change under src/: recompile the lab
//!                                  crate, link a patch, push it to the phone
//!   hotpatch reload [cargo args]   rebuild, push the library and restart the
//!                                  app on the same screen (A2, no reinstall)
//!
//! During `fat` this binary is also cargo's `RUSTC_WORKSPACE_WRAPPER` and the
//! lab's linker. The patch steps follow `dx serve --hotpatch` (Dioxus CLI,
//! packages/cli/src/build/{link,patch}.rs, MIT OR Apache-2.0), reduced to
//! ELF/aarch64; credits in ../README.md.

mod stub;

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{Duration, Instant, SystemTime},
};

use anyhow::{Context, bail, ensure};
use serde::{Deserialize, Serialize};

const TIP: &str = "gpui_mobile_lab";
const PACKAGE: &str = "dev.gpui.mobile.lab";
const DEVICE_DIR: &str = "/data/local/tmp/gpui-hot";
const DEVICE_DEV_DIR: &str = "/data/local/tmp/gpui-dev";
const LOG_TAG: &str = "GPUI_MOBILE_LAB";
/// Cargo's jobserver file descriptors don't exist outside cargo.
const JOBSERVER_VARS: [&str; 3] = ["CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS"];

/// The lab crate's rustc invocation, recorded during `fat` and replayed per patch.
#[derive(Serialize, Deserialize)]
struct Rustc {
    rustc: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: PathBuf,
    linker: String,
}

/// What `src/hot.rs` reads: the jump table from the app's own code, plus one
/// from each earlier patch, which the app rebases to where it loaded that patch.
#[derive(Serialize)]
struct Patch {
    table: subsecond_types::JumpTable,
    earlier: Vec<EarlierMap>,
}

#[derive(Serialize)]
struct EarlierMap {
    lib: String,
    map: Vec<(u64, u64)>,
}

struct Earlier {
    lib: String,
    hot: HashMap<String, u64>,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = if env::var_os("HOTPATCH_SHIM").is_some() {
        link_shim(&args)
    } else if args.first().is_some_and(|a| a.ends_with("rustc")) {
        wrap_rustc(&args)
    } else {
        match args.first().map(String::as_str) {
            Some("fat") => fat(&args[1..]),
            Some("watch") => watch(),
            Some("save-base") => save_base(),
            Some("reload") => reload(&args[1..]),
            _ => {
                eprintln!("usage: hotpatch fat [cargo args] | hotpatch watch | hotpatch reload [cargo args]");
                return ExitCode::from(2);
            }
        }
    };
    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("hotpatch: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn state_dir() -> PathBuf {
    root().join("target/hotpatch")
}

fn ndk_bin(tool: &str) -> anyhow::Result<PathBuf> {
    let sdk = env::var("ANDROID_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env::var("HOME").unwrap()).join("Android/Sdk"));
    let ndk = match env::var("ANDROID_NDK_HOME") {
        Ok(ndk) => PathBuf::from(ndk),
        Err(_) => {
            let mut all: Vec<_> = std::fs::read_dir(sdk.join("ndk"))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .collect();
            all.sort();
            all.pop().context("no NDK under $ANDROID_HOME/ndk")?
        }
    };
    Ok(ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin").join(tool))
}

/// `fat`: the normal debug build, with this binary wrapping rustc for the lab crate.
fn fat(build_args: &[String]) -> anyhow::Result<ExitCode> {
    build_library(build_args)?;
    // Gradle repackages in place and would keep the old library's bytes as dead space.
    let built = root().join("android/app/build/outputs/apk/debug/app-debug.apk");
    _ = std::fs::remove_file(&built);
    let status = Command::new(root().join("android/gradlew"))
        .args(["--no-daemon", "-q", "assembleDebug"])
        .current_dir(root().join("android"))
        .status()?;
    ensure!(status.success(), "gradle failed");
    let version = std::fs::read_to_string(root().join("Cargo.toml"))?
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"').map(String::from))
        .context("no version in Cargo.toml")?;
    let apk = root().join(format!("dist/gpui-mobile-lab-{version}-debug.apk"));
    std::fs::copy(&built, &apk)?;
    println!("hotpatch: {} ({} MB)", apk.display(), std::fs::metadata(&apk)?.len() / 1_000_000);
    Ok(ExitCode::SUCCESS)
}

/// Builds the lab library recording its rustc/link commands (the base for
/// `watch`), and leaves a copy without DWARF in jniLibs. Returns that copy.
fn build_library(build_args: &[String]) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(state_dir())?;
    // Make cargo re-run rustc (and so the linker) for the lab crate.
    let lib = root().join("src/lib.rs");
    std::fs::File::options()
        .append(true)
        .open(&lib)?
        .set_modified(SystemTime::now())?;
    // build.sh's steps, but Gradle only sees the stripped library (the
    // unstripped one is ~290 MB, copied several times into Gradle's outputs).
    let jni_libs = root().join("android/app/src/main/jniLibs");
    _ = std::fs::remove_dir_all(&jni_libs);
    let status = Command::new("cargo")
        .args(["ndk", "-t", "arm64-v8a", "--platform", "26", "-o"])
        .arg(&jni_libs)
        .arg("build")
        .args(build_args)
        .current_dir(root())
        .env("RUSTC_WORKSPACE_WRAPPER", env::current_exe()?)
        .env("HOTPATCH_MODE", "fat")
        .status()?;
    ensure!(status.success(), "cargo ndk failed");
    save_base()?;

    // The phone needs no DWARF (patches link against target/hotpatch/base.so,
    // Subsecond finds `main` via the dynamic symbol table): a 94 MB APK, not 305.
    // Only non-loaded sections go, so addresses are unchanged.
    let packaged = root().join("android/app/src/main/jniLibs/arm64-v8a/libgpui_mobile_lab.so");
    let status = Command::new(ndk_bin("llvm-strip")?)
        .arg("--strip-debug")
        .arg(&packaged)
        .status()?;
    ensure!(status.success(), "llvm-strip failed");
    // gpui-mobile also builds a cdylib; the lab links it statically (as #31).
    for entry in std::fs::read_dir(packaged.parent().unwrap())? {
        let path = entry?.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name.starts_with("libgpui_mobile.") || name.starts_with("libgpui_mobile-") {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(packaged)
}

/// A2: run a new build without reinstalling. Pushes the stripped library to
/// the app's files/dev/ (LabActivity loads it in debuggable builds) and
/// restarts the app on the screen that was open. Run inside `phone session`.
fn reload(build_args: &[String]) -> anyhow::Result<ExitCode> {
    let mut times = vec![];
    let mut t = Instant::now();
    let mut lap = |name: &'static str, t: &mut Instant| {
        times.push((name, t.elapsed()));
        *t = Instant::now();
    };
    let lib = build_library(build_args)?;
    lap("build+strip", &mut t);

    let log = adb(&["shell", &format!("logcat -d -s {LOG_TAG}:I | grep -E 'open screen: |close screen: ' | tail -1")])?;
    let screen = log.split("open screen: ").nth(1).map(|s| s.trim().to_string());
    // The staged copy stays on the phone as the base of the next delta;
    // `pushed` is the host's copy of it.
    let staged = format!("{DEVICE_DEV_DIR}/libgpui_mobile_lab.so");
    let pushed = state_dir().join("device-lib.so");
    let sent = match push_delta(&lib, &pushed, &staged) {
        Ok(bytes) => format!("{:.1} MB delta", bytes as f64 / 1e6),
        Err(err) => {
            println!("hotpatch: full push ({err:#})");
            _ = std::fs::remove_file(&pushed);
            adb(&["shell", &format!("rm -rf {DEVICE_DEV_DIR} && mkdir -p {DEVICE_DEV_DIR} && chmod 755 {DEVICE_DEV_DIR}")])?;
            push_chunked(&lib, &staged)?;
            "full library".to_string()
        }
    };
    std::fs::copy(&lib, &pushed)?;
    // Loaded code must be read-only for apps targeting Android 14.
    adb(&[
        "shell",
        &format!(
            "run-as {PACKAGE} sh -c 'mkdir -p files/dev && rm -f files/dev/libgpui_mobile_lab.so \
             && cp {staged} files/dev/ && chmod 444 files/dev/libgpui_mobile_lab.so'"
        ),
    ])?;
    lap("push", &mut t);

    adb(&["shell", &format!("am force-stop {PACKAGE}")])?;
    let mut start = format!("am start -W -n {PACKAGE}/.LabActivity");
    if let Some(screen) = &screen {
        // One string for the device shell: quote once (titles have spaces).
        start.push_str(&format!(" --es screen '{}'", screen.replace('\'', "'\\''")));
    }
    let out = adb(&["shell", &start])?;
    ensure!(out.contains("Status: ok"), "am start failed: {out}");
    lap("restart", &mut t);

    let total: Duration = times.iter().map(|(_, d)| *d).sum();
    let detail: Vec<String> = times.iter().map(|(n, d)| format!("{n} {} ms", d.as_millis())).collect();
    println!(
        "hotpatch: reloaded {:.0} MB ({sent}){} in {} ms ({})",
        std::fs::metadata(&lib)?.len() as f64 / 1e6,
        screen.map(|s| format!(" into '{s}'")).unwrap_or_default(),
        total.as_millis(),
        detail.join(", ")
    );
    Ok(ExitCode::SUCCESS)
}

/// Sends only a zstd `--patch-from` delta against the library staged by the
/// last reload (~1.4 MB instead of ~21 MB compressed), which `unpatch`
/// applies on the phone. Errs, leaving the staged copy alone, when there is
/// no base or it isn't the one the delta was made from (zstd's checksum).
fn push_delta(lib: &Path, pushed: &Path, staged: &str) -> anyhow::Result<u64> {
    ensure!(pushed.exists(), "no library staged on the phone yet");
    let delta = state_dir().join("lib.delta.zst");
    let status = Command::new("zstd")
        .args(["-q", "-f", "-3", "--patch-from"])
        .arg(pushed)
        .arg(lib)
        .arg("-o")
        .arg(&delta)
        .status()?;
    ensure!(status.success(), "zstd --patch-from failed");
    let bytes = std::fs::metadata(&delta)?.len();
    let unpatch = unpatch_binary()?;
    let result = (|| {
        // --sync skips unpatch when the phone's copy is current.
        adb(&["push", "--sync", unpatch.to_str().unwrap(), &format!("{DEVICE_DEV_DIR}/unpatch")])?;
        adb(&["push", delta.to_str().unwrap(), &format!("{DEVICE_DEV_DIR}/lib.delta.zst")])?;
        let d = DEVICE_DEV_DIR;
        adb(&[
            "shell",
            &format!(
                "test -f {staged} && chmod 755 {d}/unpatch && {d}/unpatch {staged} {d}/lib.delta.zst {d}/new.so \
                 && mv {d}/new.so {staged}; status=$?; rm -f {d}/lib.delta.zst; exit $status"
            ),
        ])
    })();
    _ = std::fs::remove_file(&delta);
    result?;
    Ok(bytes)
}

/// The phone has no zstd binary: builds tools/hotpatch/unpatch for it.
fn unpatch_binary() -> anyhow::Result<PathBuf> {
    let dir = root().join("tools/hotpatch/unpatch");
    let status = Command::new("cargo")
        .args(["ndk", "-t", "arm64-v8a", "--platform", "26", "build", "-q"])
        .current_dir(&dir)
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .status()?;
    ensure!(status.success(), "building unpatch failed");
    Ok(dir.join("target/aarch64-linux-android/debug/unpatch"))
}

/// `adb push` in 16 MB parts: one call for a large library can outlast
/// `phone adb`'s 120 s limit through the tunnel. Libraries compress ~4x
/// with zstd, which adb does not use unless asked.
fn push_chunked(local: &Path, remote: &str) -> anyhow::Result<()> {
    let bytes = std::fs::read(local)?;
    let dir = state_dir().join("chunks");
    _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let mut parts = vec![];
    for (i, chunk) in bytes.chunks(16 << 20).enumerate() {
        let part = dir.join(format!("part{i:03}"));
        std::fs::write(&part, chunk)?;
        adb(&["push", "-z", "zstd", part.to_str().unwrap(), &format!("{remote}.part{i:03}")])?;
        parts.push(format!("{remote}.part{i:03}"));
    }
    adb(&["shell", &format!("cat {} > {remote} && rm {}", parts.join(" "), parts.join(" "))])?;
    _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

/// Keeps the fat library and its objects' hashes for `watch`.
fn save_base() -> anyhow::Result<ExitCode> {
    let fat_link: Vec<String> = read_json(&state_dir().join("link-fat.json"))?;
    let base = link_output(&fat_link)?;
    std::fs::copy(&base, state_dir().join("base.so"))?;
    // Patches only carry the objects that differ from these.
    let hashes: Vec<(String, u64)> = tip_objects(&fat_link)
        .into_iter()
        .map(|o| Ok((cgu_name(&o), hash_file(&o)?)))
        .collect::<anyhow::Result<_>>()?;
    write_json(&state_dir().join("base-objects.json"), &hashes)?;
    remove_stale_objects(&tip_objects(&fat_link))?;
    println!("hotpatch: base library {}", base.display());
    Ok(ExitCode::SUCCESS)
}

/// cargo calls `<wrapper> <rustc> <args...>` for workspace crates.
fn wrap_rustc(args: &[String]) -> anyhow::Result<ExitCode> {
    let (rustc, rest) = (&args[0], &args[1..]);
    let is_tip = rest.windows(2).any(|w| w[0] == "--crate-name" && w[1] == TIP)
        && rest.iter().any(|a| a == "cdylib");
    if !is_tip || env::var("HOTPATCH_MODE").as_deref() != Ok("fat") {
        return run_status(Command::new(rustc).args(rest));
    }
    let linker = rest
        .windows(2)
        .find_map(|w| (w[0] == "-C").then(|| w[1].strip_prefix("linker=")).flatten())
        .or_else(|| rest.iter().find_map(|a| a.strip_prefix("-Clinker=")))
        .context("no -C linker= in the lab's rustc args")?
        .to_string();
    let mut args = rest.to_vec();
    args.extend(
        [
            // Keep the object files (patches relink them) and every function,
            // so patches can call anything the lab crate defines.
            "-Csave-temps=true",
            "-Clink-dead-code",
            "-Clink-arg=-Wl,--no-gc-sections",
            // Crate-local ThinLTO is most of an incremental rebuild (~5 of
            // ~6 s); without it a one-line change recompiles in ~1.2 s.
            "-Clto=off",
        ]
        .map(String::from),
    );
    args.push(format!("-Clinker={}", env::current_exe()?.display()));
    let recorded = Rustc {
        rustc: rustc.clone(),
        args: args.clone(),
        env: env::vars().collect(),
        cwd: env::current_dir()?,
        linker: linker.clone(),
    };
    write_json(&state_dir().join("rustc.json"), &recorded)?;
    run_status(
        Command::new(rustc)
            .args(&args)
            .env("HOTPATCH_SHIM", "1")
            .env("HOTPATCH_REAL_LINKER", &linker),
    )
}

/// rustc's linker for the lab: records the arguments, then links (fat) or
/// stops (thin, the watcher links the patch itself).
fn link_shim(args: &[String]) -> anyhow::Result<ExitCode> {
    let mut expanded = vec![];
    for arg in args {
        match arg.strip_prefix('@') {
            Some(file) => expanded.extend(std::fs::read_to_string(file)?.lines().map(String::from)),
            None => expanded.push(arg.clone()),
        }
    }
    let mode = env::var("HOTPATCH_MODE").unwrap_or_else(|_| "fat".into());
    write_json(&state_dir().join(format!("link-{mode}.json")), &expanded)?;
    if mode == "thin" {
        return Ok(ExitCode::SUCCESS);
    }
    run_status(Command::new(env::var("HOTPATCH_REAL_LINKER")?).args(args))
}

fn watch() -> anyhow::Result<ExitCode> {
    let dir = state_dir();
    let rustc: Rustc = read_json(&dir.join("rustc.json")).context("run `hotpatch fat` first")?;
    let fat_link: Vec<String> = read_json(&dir.join("link-fat.json"))?;
    let mut last_objects: HashMap<String, u64> =
        read_json::<Vec<(String, u64)>>(&dir.join("base-objects.json"))?
            .into_iter()
            .collect();

    let t = Instant::now();
    let cache = stub::BaseSymbols::load(&dir.join("base.so"))?;
    println!(
        "hotpatch: read {} base symbols in {} ms",
        cache.symbols.len(),
        t.elapsed().as_millis()
    );

    // HOTPATCH_OFFLINE=<hex aslr_reference>: build patches without a phone.
    let offline = env::var("HOTPATCH_OFFLINE").ok();
    // HOTPATCH_ASLR=<hex>: the app's value, when its log line has rotated out.
    let aslr_reference = match offline.clone().or_else(|| env::var("HOTPATCH_ASLR").ok()) {
        Some(hex) => u64::from_str_radix(hex.trim_start_matches("0x"), 16)?,
        None => device_aslr_reference()?,
    };
    println!("hotpatch: app aslr_reference {aslr_reference:#x}");
    if offline.is_none() {
        adb(&["shell", &format!("mkdir -p {DEVICE_DIR} && chmod 755 {DEVICE_DIR}")])?;
    }

    let src = root().join("src");
    let mut seen = newest_mtime(&src)?;
    let mut generation = 0;
    let mut earlier = vec![];
    println!("hotpatch: watching {}", src.display());
    loop {
        std::thread::sleep(Duration::from_millis(50));
        let now = newest_mtime(&src)?;
        if now <= seen {
            continue;
        }
        seen = now;
        generation += 1;
        let result = patch_once(
            &rustc,
            &fat_link,
            &mut last_objects,
            &mut earlier,
            &cache,
            aslr_reference,
            generation,
        );
        if let Err(err) = result {
            eprintln!("hotpatch: patch {generation} failed: {err:#}");
        }
    }
}

fn patch_once(
    rustc: &Rustc,
    fat_link: &[String],
    last_objects: &mut HashMap<String, u64>,
    earlier: &mut Vec<Earlier>,
    cache: &stub::BaseSymbols,
    aslr_reference: u64,
    generation: u32,
) -> anyhow::Result<()> {
    let dir = state_dir();
    let mut times = vec![];
    let mut t = Instant::now();
    let mut lap = |name: &'static str, t: &mut Instant| {
        times.push((name, t.elapsed()));
        *t = Instant::now();
    };

    // 1. Recompile the lab crate with the recorded command (incremental).
    let out = Command::new(&rustc.rustc)
        .args(&rustc.args)
        .env_clear()
        .envs(rustc.env.iter().filter(|(k, _)| !JOBSERVER_VARS.contains(&k.as_str())).cloned())
        .env("HOTPATCH_SHIM", "1")
        .env("HOTPATCH_MODE", "thin")
        .env("HOTPATCH_REAL_LINKER", &rustc.linker)
        .current_dir(&rustc.cwd)
        .output()?;
    if !out.status.success() {
        bail!("rustc failed:\n{}", String::from_utf8_lossy(&out.stderr));
    }
    lap("rustc", &mut t);

    // 2. Link the lab's fresh objects, plus stubs that jump into the running
    //    app for everything else, into a shared library.
    let thin_link: Vec<String> = read_json(&dir.join("link-thin.json"))?;
    //    Every lab object goes in, as with `dx`: code left in the app would
    //    call the app's old copies of changed functions directly.
    let objects = tip_objects(&thin_link);
    let mut changed = 0;
    for object in &objects {
        let hash = hash_file(object)?;
        if last_objects.insert(cgu_name(object), hash) != Some(hash) {
            changed += 1;
        }
    }
    if changed == 0 {
        println!("hotpatch: patch {generation}: no code change");
        return Ok(());
    }
    let stub_path = dir.join("stub.o");
    std::fs::write(&stub_path, stub::undefined_symbol_stub(cache, &objects, aslr_reference)?)?;
    lap("stub", &mut t);

    let patch = dir.join(format!("patch-{generation}.so"));
    let out = Command::new(&rustc.linker)
        .args(&objects)
        .arg(&stub_path)
        .args(thin_link_args(fat_link))
        .arg("-o")
        .arg(&patch)
        .env_clear()
        .envs(rustc.env.iter().cloned())
        .output()?;
    if !out.status.success() {
        bail!("link failed:\n{}", String::from_utf8_lossy(&out.stderr));
    }
    remove_stale_objects(&objects)?;
    lap("link", &mut t);

    // 3. Map old function addresses to new ones; ship a library without DWARF.
    let device_lib = format!("{DEVICE_DIR}/patch-{generation}.so");
    let mut table = stub::jump_table(&patch, cache)?;
    table.lib = PathBuf::from(&device_lib);
    let stripped = dir.join(format!("patch-{generation}.stripped.so"));
    let status = Command::new(ndk_bin("llvm-strip")?)
        .arg("--strip-debug")
        .arg(&patch)
        .arg("-o")
        .arg(&stripped)
        .status()?;
    ensure!(status.success(), "llvm-strip failed");
    // Views created by an earlier patch's code (a screen opened after a
    // patch) call that patch's hot closures: map those to the new ones too.
    let hot = stub::hot_closures(&patch)?;
    let earlier_maps = earlier
        .iter()
        .map(|e| EarlierMap {
            lib: e.lib.clone(),
            map: e.hot.iter().filter_map(|(name, old)| Some((*old, *hot.get(name)?))).collect(),
        })
        .collect();
    earlier.push(Earlier { lib: device_lib.clone(), hot });
    let table_path = dir.join("patch.json");
    write_json(&table_path, &Patch { table: table.clone(), earlier: earlier_maps })?;
    lap("table+strip", &mut t);

    // 4. Push library and table.
    if env::var_os("HOTPATCH_OFFLINE").is_none() {
        // One adb call (each costs ~0.5 s through the tunnel); files go in
        // argument order, so the table lands after the library.
        let staged = dir.join("push");
        _ = std::fs::remove_dir_all(&staged);
        std::fs::create_dir_all(&staged)?;
        std::fs::copy(&stripped, staged.join(format!("patch-{generation}.so")))?;
        std::fs::copy(&table_path, staged.join("patch.json"))?;
        adb(&[
            "push",
            "-z",
            "zstd",
            staged.join(format!("patch-{generation}.so")).to_str().unwrap(),
            staged.join("patch.json").to_str().unwrap(),
            &format!("{DEVICE_DIR}/"),
        ])?;
        lap("push", &mut t);
    }

    let size = std::fs::metadata(&stripped)?.len();
    _ = std::fs::remove_file(&patch);
    _ = std::fs::remove_file(&stripped);

    let total: Duration = times.iter().map(|(_, d)| *d).sum();
    let detail: Vec<String> = times
        .iter()
        .map(|(n, d)| format!("{n} {} ms", d.as_millis()))
        .collect();
    println!(
        "hotpatch: patch {generation}: {changed}/{} objects changed, {} hot closures, {:.1} MB, {} ms ({})",
        objects.len(),
        table.map.len(),
        size as f64 / 1e6,
        total.as_millis(),
        detail.join(", ")
    );
    Ok(())
}

/// The fat link's arguments that still apply to a patch: the Dioxus CLI's
/// `thin_link_args` for lld (packages/cli/src/build/link.rs).
fn thin_link_args(fat: &[String]) -> Vec<String> {
    let mut out: Vec<String> = [
        "-shared",
        "-Wl,--eh-frame-hdr",
        "-Wl,-z,noexecstack",
        "-Wl,-z,relro,-z,now",
        "-nodefaultlibs",
        "-Wl,-Bdynamic",
    ]
    .map(String::from)
    .to_vec();
    for (i, arg) in fat.iter().enumerate() {
        if arg == "-L" || arg == "-target" {
            out.push(arg.clone());
            out.push(fat[i + 1].clone());
        } else if arg.starts_with("-l")
            || arg.starts_with("-m")
            || arg.starts_with("-L")
            || arg.starts_with("-Wl,--target=")
            || arg.starts_with("-fuse-ld")
            || arg.starts_with("-B")
            || arg.contains("-ld-path")
        {
            out.push(arg.clone());
        }
    }
    out
}

/// -Csave-temps keeps each compilation's objects under new names (~65 MB):
/// delete all but `current`.
fn remove_stale_objects(current: &[PathBuf]) -> anyhow::Result<()> {
    let Some(dir) = current.first().and_then(|o| o.parent()) else {
        return Ok(());
    };
    for path in std::fs::read_dir(dir)? {
        let path = path?.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name.starts_with(&format!("{TIP}.")) && name.ends_with(".rcgu.o") && !current.contains(&path) {
            _ = std::fs::remove_file(&path);
        }
    }
    Ok(())
}

fn tip_objects(link_args: &[String]) -> Vec<PathBuf> {
    link_args
        .iter()
        .filter(|a| a.ends_with(".rcgu.o"))
        .map(PathBuf::from)
        .collect()
}

/// `<crate>.<cgu>.<session>.rcgu.o` without the session part, which rustc
/// changes on every compilation.
fn cgu_name(object: &Path) -> String {
    let name = object.file_name().unwrap().to_string_lossy();
    name.split('.').take(2).collect::<Vec<_>>().join(".")
}

fn hash_file(path: &Path) -> anyhow::Result<u64> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::fs::read(path)
        .with_context(|| format!("reading {}", path.display()))?
        .hash(&mut hasher);
    Ok(hasher.finish())
}

fn link_output(args: &[String]) -> anyhow::Result<PathBuf> {
    let i = args.iter().position(|a| a == "-o").context("no -o in link args")?;
    Ok(PathBuf::from(&args[i + 1]))
}

/// Reads the running app's `subsecond::aslr_reference()` from its log line.
fn device_aslr_reference() -> anyhow::Result<u64> {
    let pid = adb(&["shell", &format!("pidof {PACKAGE}")])?;
    let pid = pid.trim();
    ensure!(!pid.is_empty(), "{PACKAGE} is not running");
    let log = adb(&["shell", &format!("logcat -d -s {LOG_TAG}:I | grep 'hot: aslr_reference'")])?;
    let line = log
        .lines()
        .rev()
        .find(|l| l.ends_with(&format!("pid={pid}")))
        .with_context(|| format!("no aslr_reference line for pid {pid} in logcat"))?;
    let hex = line
        .split("aslr_reference=0x")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .context("malformed aslr_reference line")?;
    Ok(u64::from_str_radix(hex, 16)?)
}

/// adb through the shared `phone` lock (~/.local/bin/phone); `watch` must
/// run inside `phone session`.
fn adb(args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("phone").arg("adb").args(args).output()?;
    ensure!(
        out.status.success(),
        "phone adb {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn newest_mtime(dir: &Path) -> anyhow::Result<SystemTime> {
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let mtime = if entry.file_type()?.is_dir() {
            newest_mtime(&path)?
        } else {
            entry.metadata()?.modified()?
        };
        newest = newest.max(mtime);
    }
    Ok(newest)
}

fn run_status(cmd: &mut Command) -> anyhow::Result<ExitCode> {
    let status = cmd.status()?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> anyhow::Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    Ok(std::fs::write(path, serde_json::to_vec(value)?)?)
}

