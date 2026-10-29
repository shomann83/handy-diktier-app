"""Generate the TypeTalk icons (3-t-s CI): hexagonal speech bubble in light
grey, a yellow text cursor ("Type") and turquoise sound waves ("Talk").

Usage (from the repo root, Python with Pillow):
    py docs/stil-leiste/make_icons.py

Writes:
    src-tauri/icons/typetalk-1024.png   source for `bun tauri icon`
    src-tauri/resources/*.png           tray icons (64x64) for all states
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
BLACK = (0, 0, 0, 255)
LIGHT_GREY = (206, 211, 220, 255)  # #CED3DC
DARK_GREY = (60, 60, 60, 255)
YELLOW = (255, 255, 0, 255)  # #FFFF00
TURQUOISE = (53, 214, 221, 255)  # #35D6DD

SUPERSAMPLE = 4

# Motif in a 120x120 design grid (same as the approved sketch "C").
BUBBLE = [(60, 18), (96, 38), (96, 76), (60, 96), (48, 89), (34, 100), (36, 82), (24, 76), (24, 38)]
CURSOR = ((48, 42), (48, 72))
WAVES = [((60, 50), (64, 57), (60, 64)), ((70, 44), (78, 57), (70, 70))]  # quadratic beziers


def quad(p0, p1, p2, steps=24):
    return [
        (
            (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t**2 * p2[0],
            (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t**2 * p2[1],
        )
        for t in (i / steps for i in range(steps + 1))
    ]


def round_line(draw, points, width, colour):
    draw.line(points, fill=colour, width=width, joint="curve")
    r = width / 2
    for x, y in (points[0], points[-1]):
        draw.ellipse((x - r, y - r, x + r, y + r), fill=colour)


def motif(size: int, frame, inset: float, background: bool, dot=None) -> Image.Image:
    """Draw the motif at `size` px; `inset` shrinks the 120-grid toward the centre."""
    big = size * SUPERSAMPLE
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    if background:
        draw.rounded_rectangle((0, 0, big - 1, big - 1), radius=int(big * 0.22), fill=BLACK)

    scale = big / 120 * inset
    offset = big * (1 - inset) / 2

    def pt(p):
        return (offset + p[0] * scale, offset + p[1] * scale)

    stroke = lambda w: max(1, int(w * scale))  # noqa: E731
    draw.line([pt(p) for p in BUBBLE + BUBBLE[:2]], fill=frame, width=stroke(5), joint="curve")
    round_line(draw, [pt(p) for p in CURSOR], stroke(6), YELLOW)
    for wave in WAVES:
        round_line(draw, [pt(p) for p in quad(*wave)], stroke(5), TURQUOISE)
    if dot:
        r = big * 0.14
        cx, cy = big - r * 1.05, big - r * 1.05
        draw.ellipse((cx - r, cy - r, cx + r, cy + r), fill=dot)
    return img.resize((size, size), Image.LANCZOS)


def main() -> None:
    motif(1024, LIGHT_GREY, 1.0, background=True).save(ROOT / "src-tauri/icons/typetalk-1024.png")

    res = ROOT / "src-tauri/resources"
    # Tray: no background, motif fills the square; frame grey for dark
    # taskbars, dark grey ("_dark" files) for light taskbars.
    for suffix, frame in (("", LIGHT_GREY), ("_dark", DARK_GREY)):
        idle = motif(64, frame, 1.18, background=False)
        idle.save(res / f"tray_idle{suffix}.png")
        idle.save(res / f"tray_idle_warning{suffix}.png")
        motif(64, frame, 1.18, False, YELLOW).save(res / f"tray_recording{suffix}.png")
        motif(64, frame, 1.18, False, TURQUOISE).save(res / f"tray_transcribing{suffix}.png")
    # "Colored" theme (Linux) and the warning badge source
    colored = motif(64, LIGHT_GREY, 1.18, background=False)
    colored.save(res / "handy.png")
    colored.save(res / "handy_warning.png")
    motif(64, LIGHT_GREY, 1.18, False, YELLOW).save(res / "recording.png")
    motif(64, LIGHT_GREY, 1.18, False, TURQUOISE).save(res / "transcribing.png")


if __name__ == "__main__":
    main()
