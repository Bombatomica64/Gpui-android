#!/usr/bin/env python3
"""Break an unstripped Android .so down by section, crate and crate family.

    tools/size-report.py <lib.so> <llvm-bin-dir> <out-dir>

Writes report.md (also printed) plus sections.txt, symbols.txt and crates.tsv
to <out-dir>. Symbols are attributed like cargo-bloat: to the first non-std
crate named at a path start, so `drop_in_place<wgpu_core::X>` counts as
wgpu_core. Sizes are bytes in the stripped library.
"""
import collections
import re
import subprocess
import sys
from pathlib import Path

STD = {"core", "alloc", "std", "std_detect", "panic_abort", "compiler_builtins", "rustc_demangle"}

# Crate (lib name) prefix -> family. First match wins.
FAMILIES = [
    ("naga (shader compiler)", ["naga", "spirv", "codespan_reporting", "hexf_parse", "bit_set", "bit_vec", "unicode_ident"]),
    ("wgpu + Vulkan/GL", ["wgpu", "ash", "glow", "khronos_egl", "gpu_alloc", "gpu_descriptor", "gpu_allocator",
                          "renderdoc", "range_alloc", "libloading", "profiling", "raw_window_handle"]),
    ("fonts + text shaping", ["cosmic_text", "swash", "skrifa", "read_fonts", "font_types", "zeno", "yazi", "fontdb",
                              "rustybuzz", "harfrust", "harfbuzz", "ttf_parser", "unicode_bidi", "unicode_linebreak",
                              "unicode_script", "unicode_segmentation", "unicode_properties", "sys_locale",
                              "self_cell", "font_kit", "core_maths"]),
    ("images + SVG", ["image", "png", "zune", "jpeg_decoder", "fdeflate", "miniz_oxide", "flate2", "crc32fast",
                      "adler", "tiny_skia", "resvg", "usvg", "roxmltree", "svgtypes", "simplecss", "kurbo",
                      "imagesize", "strict_num", "gif", "image_webp", "lyon", "data_url", "xmlwriter",
                      "arrayref", "bytemuck", "float_cmp", "weezl", "color_quant"]),
    ("accessibility (AccessKit)", ["accesskit"]),
    ("gpui-kit components", ["gpui_component", "gpui_kit", "gpui_base", "gpui_fps", "gpui_macros_kit",
                             "rust_i18n", "markdown", "pulldown_cmark", "tree_sitter", "syntect", "lsp_types",
                             "ropey", "sum_tree_kit", "smol_str", "notify", "chart"]),
    ("lab app", ["gpui_mobile_lab"]),
    ("gpui-mobile + Android glue", ["gpui_mobile", "jni", "ndk", "android_activity", "android_logger",
                                    "android_properties", "cesu8", "combine"]),
    ("gpui", ["gpui", "taffy", "collections", "sum_tree", "refineable", "util", "scheduler", "etagere",
              "slotmap", "seahash"]),
    ("general-purpose crates", ["smallvec", "async_task", "futures", "parking_lot", "lock_api", "flume", "backtrace", "addr2line", "gimli", "object", "rustc_demangle", "ctor", "inventory",
              "itertools", "num", "half", "euclid", "rand", "getrandom", "uuid", "chrono", "regex",
              "aho_corasick", "memchr", "serde", "toml", "hashbrown", "indexmap", "log", "anyhow",
              "thiserror", "strum", "derive_more", "bitflags", "once_cell", "arrayvec", "url", "idna",
              "percent_encoding", "http", "bytes", "schemars", "sha2", "digest", "base64", "ryu",
              "itoa", "time", "postage", "pin_project", "event_listener", "async", "blocking",
              "polling", "smol", "crossbeam", "foldhash", "rustc_hash", "ahash", "twox_hash", "dashmap",
              "semver", "unicode_width", "image_buffer", "stacksafe", "dyn_clone", "waker_fn", "tracing"]),
]

TOKEN = re.compile(r"(?:^|[<\s(&\[*,;{])([A-Za-z_][A-Za-z0-9_]*)::")


def crate_of(name: str) -> str:
    tokens = TOKEN.findall(name)
    if not tokens:
        return "[no crate: C, compiler or anonymous]"
    for t in tokens:
        if t not in STD:
            return t
    return "std (core/alloc/std)"


def family_of(crate: str) -> str:
    if crate.startswith("std ") or crate.startswith("["):
        return crate
    for fam, prefixes in FAMILIES:
        if any(crate == p or crate.startswith(p + "_") or (len(p) > 4 and crate.startswith(p)) for p in prefixes):
            return fam
    return "other crates"


def mb(n: int) -> str:
    return f"{n / 1e6:.2f}"


def main() -> None:
    lib, bindir, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    out.mkdir(parents=True, exist_ok=True)

    sections_raw = subprocess.run([bindir / "llvm-readelf", "-S", "-W", lib], check=True,
                                  capture_output=True, text=True).stdout
    (out / "sections.txt").write_text(sections_raw)
    sections = []
    for line in sections_raw.splitlines():
        m = re.match(r"\s*\[\s*\d+\]\s+(\S+)\s+\S+\s+[0-9a-f]+\s+[0-9a-f]+\s+([0-9a-f]+)\s+\S+\s+(\S*)", line)
        if m and "A" in m.group(3):
            sections.append((m.group(1), int(m.group(2), 16)))
    sections.sort(key=lambda s: -s[1])

    nm = subprocess.run([bindir / "llvm-nm", "-S", "-C", "--size-sort", "--defined-only", lib], check=True,
                        capture_output=True, text=True).stdout
    (out / "symbols.txt").write_text(nm)
    code = collections.Counter()
    data = collections.Counter()
    biggest_data = []
    seen = set()
    for line in nm.splitlines():
        parts = line.split(" ", 3)
        if len(parts) < 4:
            continue
        addr, size, kind, name = parts
        size = int(size, 16)
        # Aliases (same address) would be counted twice.
        if (addr, size) in seen or size == 0:
            continue
        seen.add((addr, size))
        crate = crate_of(name)
        if kind in "tTwW":
            code[crate] += size
        else:
            data[crate] += size
            biggest_data.append((size, name))

    crates = sorted(set(code) | set(data), key=lambda c: -(code[c] + data[c]))
    with open(out / "crates.tsv", "w") as f:
        f.write("crate\tfamily\tcode\tdata\n")
        for c in crates:
            f.write(f"{c}\t{family_of(c)}\t{code[c]}\t{data[c]}\n")

    fam = collections.Counter()
    for c in crates:
        fam[family_of(c)] += code[c] + data[c]
    total_syms = sum(fam.values())
    alloc_total = sum(s for _, s in sections)

    lines = [f"Allocated sections: **{mb(alloc_total)} MB**; named symbols cover {mb(total_syms)} MB "
             f"(the rest is anonymous constants, unwind tables, relocations and dynamic symbols).", ""]
    lines += ["| Section | MB |", "|---|---:|"]
    lines += [f"| `{n}` | {mb(s)} |" for n, s in sections if s >= 50_000]
    lines += ["", "| Family | MB | % of named |", "|---|---:|---:|"]
    lines += [f"| {f} | {mb(s)} | {100 * s / total_syms:.1f} |" for f, s in fam.most_common()]
    lines += ["", "| Crate | Family | Code MB | Data MB |", "|---|---|---:|---:|"]
    lines += [f"| `{c}` | {family_of(c)} | {mb(code[c])} | {mb(data[c])} |" for c in crates[:40]]
    lines += ["", "Largest data symbols:", "", "| KB | Symbol |", "|---:|---|"]
    for size, name in sorted(biggest_data, reverse=True)[:15]:
        lines.append(f"| {size // 1000} | `{name[:110]}` |")
    report = "\n".join(lines) + "\n"
    (out / "report.md").write_text(report)
    print(report)


if __name__ == "__main__":
    main()
