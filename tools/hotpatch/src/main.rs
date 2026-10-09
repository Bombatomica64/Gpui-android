//! Builds Subsecond patches of the lab crate and pushes them to the phone
//! (docs/hot-patching-plan.md, A1). Debug arm64 only.
//!
//!   hotpatch fat [build.sh args]   debug APK whose library records how rustc
//!                                  compiled and linked the lab crate
//!   hotpatch watch                 on each change under src/: recompile the lab
//!                                  crate, link a patch, push it to the phone
//!
//! During `fat` this binary is also cargo's `RUSTC_WORKSPACE_WRAPPER` and the
//! lab's linker. The patch steps follow `dx serve --hotpatch` (Dioxus CLI,
//! packages/cli/src/build/{link,patch}.rs), reduced to ELF/aarch64.

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
            _ => {
                eprintln!("usage: hotpatch fat [build.sh args] | hotpatch watch");
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
    std::fs::create_dir_all(state_dir())?;
    // Make cargo re-run rustc (and so the linker) for the lab crate.
    let lib = root().join("src/lib.rs");
    std::fs::File::options()
        .append(true)
        .open(&lib)?
        .set_modified(SystemTime::now())?;
    let status = Command::new(root().join("build.sh"))
        .arg("android")
        .arg("--debug")
        .args(build_args)
        .env("RUSTC_WORKSPACE_WRAPPER", env::current_exe()?)
        .env("HOTPATCH_MODE", "fat")
        .status()?;
    ensure!(status.success(), "build.sh failed");
    save_base()?;

    // The phone needs no DWARF (patches link against target/hotpatch/base.so,
    // Subsecond finds `main` via the dynamic symbol table): 305 → ~60 MB APK.
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
        if path.file_name().unwrap().to_string_lossy().starts_with("libgpui_mobile.") || path.file_name().unwrap().to_string_lossy().starts_with("libgpui_mobile-") {
            std::fs::remove_file(path)?;
        }
    }
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
    let base_objects: HashMap<String, u64> =
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
    let aslr_reference = match &offline {
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
            &base_objects,
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
    base_objects: &HashMap<String, u64>,
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
    //    Objects identical to the base's are left out: the stubs send calls
    //    into them to the running app, so their statics are kept too.
    //    HOTPATCH_ALL=1 links every object instead, as `dx` does.
    let all = env::var_os("HOTPATCH_ALL").is_some();
    let mut objects = vec![];
    for object in tip_objects(&thin_link) {
        if all || base_objects.get(&cgu_name(&object)) != Some(&hash_file(&object)?) {
            objects.push(object);
        }
    }
    if objects.is_empty() {
        println!("hotpatch: patch {generation}: no code change");
        return Ok(());
    }
    let changed = objects.len();
    if !all {
        objects = stub::with_data_providers(objects, &tip_objects(&thin_link))?;
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
    let table_path = dir.join("patch.json");
    write_json(&table_path, &table)?;
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
            staged.join(format!("patch-{generation}.so")).to_str().unwrap(),
            staged.join("patch.json").to_str().unwrap(),
            &format!("{DEVICE_DIR}/"),
        ])?;
        lap("push", &mut t);
    }

    let total: Duration = times.iter().map(|(_, d)| *d).sum();
    let detail: Vec<String> = times
        .iter()
        .map(|(n, d)| format!("{n} {} ms", d.as_millis()))
        .collect();
    println!(
        "hotpatch: patch {generation}: {changed} changed + {} data objects, {} functions, {:.1} MB, {} ms ({})",
        objects.len() - changed,
        table.map.len(),
        std::fs::metadata(&stripped)?.len() as f64 / 1e6,
        total.as_millis(),
        detail.join(", ")
    );
    Ok(())
}

/// The fat link's arguments that still apply to a patch (as `dx` keeps for lld).
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

