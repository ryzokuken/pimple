"""Generate placeholder Tauri icons (PNG/ICO/ICNS) using only Python stdlib.

These are minimal solid-color placeholders intended for development only. The
`crates/pimple-tauri/icons/` directory is gitignored; without these files
`tauri::generate_context!()` panics at compile time. Run `just icons-dev` or
this script directly after a fresh clone.

A real designed icon set is a v0.2 follow-up task tracked in the v0.2 spec.
"""

import struct
import zlib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT = REPO_ROOT / "crates" / "pimple-tauri" / "icons"


def _chunk(type_: bytes, data: bytes) -> bytes:
    length = struct.pack(">I", len(data))
    crc = struct.pack(">I", zlib.crc32(type_ + data) & 0xFFFFFFFF)
    return length + type_ + data + crc


def make_png(width: int, height: int, rgba: tuple[int, int, int, int] = (107, 70, 193, 255)) -> bytes:
    """Solid-color RGBA PNG. Default is a muted indigo."""
    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = _chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    pixel = struct.pack(">BBBB", *rgba)
    row = b"\x00" + pixel * width  # filter type 0 (None) then RGBA pixels
    idat = _chunk(b"IDAT", zlib.compress(row * height, 9))
    iend = _chunk(b"IEND", b"")
    return sig + ihdr + idat + iend


def make_ico(png_bytes: bytes, width: int, height: int) -> bytes:
    """Single-image ICO wrapping a PNG. 0 in width/height byte means 256."""
    header = struct.pack("<HHH", 0, 1, 1)  # reserved=0, type=1 (icon), count=1
    w_byte = width if width < 256 else 0
    h_byte = height if height < 256 else 0
    entry = struct.pack(
        "<BBBBHHII", w_byte, h_byte, 0, 0, 1, 32, len(png_bytes), 6 + 16
    )
    return header + entry + png_bytes


def make_icns(png128: bytes) -> bytes:
    """Minimal ICNS container with a single 'ic07' (128x128 PNG) entry."""
    entry = b"ic07" + struct.pack(">I", 8 + len(png128)) + png128
    return b"icns" + struct.pack(">I", 8 + len(entry)) + entry


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    png32 = make_png(32, 32)
    png128 = make_png(128, 128)
    png256 = make_png(256, 256)

    (OUT / "32x32.png").write_bytes(png32)
    (OUT / "128x128.png").write_bytes(png128)
    (OUT / "128x128@2x.png").write_bytes(png256)
    (OUT / "icon.ico").write_bytes(make_ico(png32, 32, 32))
    (OUT / "icon.icns").write_bytes(make_icns(png128))

    for p in sorted(OUT.iterdir()):
        print(f"{p.name}\t{p.stat().st_size} bytes")


if __name__ == "__main__":
    main()
