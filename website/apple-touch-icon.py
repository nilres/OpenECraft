"""Draws apple-touch-icon.png, the icon of the page on the home screen of an iPhone or
iPad: the drawing of assets/icon.svg as an opaque square of 180 x 180 pixels (iOS rounds
the corners itself and shows transparent ones black). Run it again when the icon changes:

    python website/apple-touch-icon.py        (needs Pillow)
"""
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 180
SCALE = 4  # drawn larger and reduced: smooth edges
TOP, BOTTOM = (0x2C, 0x72, 0xCC), (0x15, 0x45, 0x86)
WHITE, AMBER = (0xFF, 0xFF, 0xFF), (0xF5, 0xB8, 0x2E)
WIDTH = 19.2
# The lines of assets/icon.svg, in its 256 units.
LINES = [
    ((128, 35.84), (128, 97.28), WHITE),
    ((128, 97.28), (99.84, 97.28), WHITE),
    ((128, 158.72), (128, 220.16), WHITE),
    ((128, 158.72), (79.36, 84.48), AMBER),
]

side = SIZE * SCALE
unit = side / 256
image = Image.new("RGB", (side, side))
draw = ImageDraw.Draw(image)
for y in range(side):
    t = y / (side - 1)
    draw.line([(0, y), (side, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(TOP, BOTTOM)))
radius = WIDTH * unit / 2
for start, end, colour in LINES:
    points = [(x * unit, y * unit) for x, y in (start, end)]
    draw.line(points, fill=colour, width=round(WIDTH * unit))
    for x, y in points:
        draw.ellipse([x - radius, y - radius, x + radius, y + radius], fill=colour)
image.resize((SIZE, SIZE), Image.LANCZOS).save(Path(__file__).with_name("apple-touch-icon.png"), optimize=True)
