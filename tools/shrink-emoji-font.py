#!/usr/bin/env -S uv run --quiet
# /// script
# requires-python = ">=3.11"
# dependencies = ["fonttools==4.60.1", "pillow==11.3.0", "imagequant==1.1.4", "zopflipy==1.11"]
# ///
"""Downscale the bitmap strike of a CBDT colour font (docs/binary-size.md).

    tools/shrink-emoji-font.py IN.ttf OUT.ttf [--ppem 72] [--colors 64] [--jobs 4]

Every glyph stays, so skin tones, flags and ZWJ sequences keep working; only the
resolution of the PNGs changes. Advances live in hmtx and do not change, so text
lays out exactly as before.
"""

import argparse
import io
import sys
from concurrent.futures import ProcessPoolExecutor

import imagequant
import zopfli
from fontTools.ttLib import TTFont
from PIL import Image


def shrink_png(job: tuple[bytes, tuple[int, int], int]) -> bytes:
    data, size, colors = job
    image = Image.open(io.BytesIO(data)).convert("RGBA").resize(size, Image.Resampling.LANCZOS)
    # Like Noto's own build (pngquant + zopfli): Pillow's quantizer and zlib lose
    # all of the saving from the smaller bitmaps.
    image = imagequant.quantize_pil_image(image, dithering_level=0.0, max_colors=colors)
    out = io.BytesIO()
    image.save(out, "PNG")
    return zopfli.ZopfliPNG(iterations=5, lossy_transparent=True).optimize(out.getvalue())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("input")
    parser.add_argument("output")
    parser.add_argument("--ppem", type=int, default=72)
    parser.add_argument("--colors", type=int, default=64)
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()

    font = TTFont(args.input)
    strikes = font["CBLC"].strikes
    if len(strikes) != 1:
        sys.exit(f"expected one strike, found {len(strikes)}")
    table = strikes[0].bitmapSizeTable
    if table.ppemX != table.ppemY or args.ppem >= table.ppemX:
        sys.exit(f"--ppem must be below the font's {table.ppemX}")
    scale = args.ppem / table.ppemX

    def scaled(value: int) -> int:
        return round(value * scale)

    glyphs = list(font["CBDT"].strikeData[0].values())
    jobs = []
    for glyph in glyphs:
        metrics = glyph.metrics
        size = (scaled(metrics.width), scaled(metrics.height))
        jobs.append((glyph.imageData, size, args.colors))
        metrics.width, metrics.height = size
        metrics.BearingX, metrics.BearingY = scaled(metrics.BearingX), scaled(metrics.BearingY)
        metrics.Advance = scaled(metrics.Advance)
    with ProcessPoolExecutor(args.jobs) as pool:
        for glyph, png in zip(glyphs, pool.map(shrink_png, jobs, chunksize=32)):
            glyph.imageData = png

    table.ppemX = table.ppemY = args.ppem
    for line in (table.hori, table.vert):
        line.ascender, line.descender = scaled(line.ascender), scaled(line.descender)
        line.widthMax = scaled(line.widthMax)

    font.save(args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
