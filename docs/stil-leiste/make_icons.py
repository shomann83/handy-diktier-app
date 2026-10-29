"""Generate TypeTalk icons from the 3-t-s hexagon mark.

Usage (from the repo root, Python with Pillow):
    py docs/stil-leiste/make_icons.py <path-to-3-t-s-logo.png>

Writes:
    src-tauri/icons/typetalk-1024.png   source for `bun tauri icon`
    src-tauri/resources/*.png           tray icons (64x64) for all states
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
BLACK = (0, 0, 0, 255)
LIGHT_GREY = (206, 211, 220)  # #CED3DC, the hexagon outline in the logo
DARK_GREY = (60, 60, 60)
YELLOW = (255, 255, 0, 255)
TURQUOISE = (53, 214, 221, 255)

MARK_BOX = (0, 0, 641, 740)  # hexagon mark inside the horizontal logo


def load_mark(logo_path: str) -> Image.Image:
    return Image.open(logo_path).convert("RGBA").crop(MARK_BOX)


def recolor_grey(mark: Image.Image, rgb) -> Image.Image:
    """Swap the light-grey parts (outline, third arrow) for another colour."""
    out = mark.copy()
    px = out.load()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = px[x, y]
            if a and abs(r - LIGHT_GREY[0]) < 40 and abs(g - LIGHT_GREY[1]) < 40 and abs(b - LIGHT_GREY[2]) < 40:
                px[x, y] = (*rgb, a)
    return out


def fit(mark: Image.Image, size: int, fill: float) -> Image.Image:
    """Scale the mark to `fill` of the square and centre it on transparency."""
    h = int(size * fill)
    w = int(mark.width * h / mark.height)
    scaled = mark.resize((w, h), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.alpha_composite(scaled, ((size - w) // 2, (size - h) // 2))
    return canvas


def app_icon(mark: Image.Image) -> Image.Image:
    size = 1024
    icon = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(icon).rounded_rectangle(
        (0, 0, size - 1, size - 1), radius=int(size * 0.22), fill=BLACK
    )
    icon.alpha_composite(fit(mark, size, 0.74))
    return icon


def with_dot(icon: Image.Image, colour) -> Image.Image:
    """Mark recording/transcribing with a filled dot in the hexagon centre."""
    out = icon.copy()
    s = out.width
    r = s * 0.13
    cx, cy = s / 2, s * 0.53
    ImageDraw.Draw(out).ellipse((cx - r, cy - r, cx + r, cy + r), fill=colour)
    return out


def main(logo_path: str) -> None:
    mark = load_mark(logo_path)
    app_icon(mark).save(ROOT / "src-tauri/icons/typetalk-1024.png")

    res = ROOT / "src-tauri/resources"
    light = fit(mark, 64, 0.94)  # for dark taskbars
    dark = fit(recolor_grey(mark, DARK_GREY), 64, 0.94)  # for light taskbars
    for suffix, base in (("", light), ("_dark", dark)):
        base.save(res / f"tray_idle{suffix}.png")
        base.save(res / f"tray_idle_warning{suffix}.png")
        with_dot(base, YELLOW).save(res / f"tray_recording{suffix}.png")
        with_dot(base, TURQUOISE).save(res / f"tray_transcribing{suffix}.png")
    # "Colored" theme (Linux) and the warning badge source
    light.save(res / "handy.png")
    light.save(res / "handy_warning.png")
    with_dot(light, YELLOW).save(res / "recording.png")
    with_dot(light, TURQUOISE).save(res / "transcribing.png")


if __name__ == "__main__":
    main(sys.argv[1])
