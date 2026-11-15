"""Draws social.jpg, the preview image of links to the page in chats and social media
(1200 x 630): the icon, the name and what OpenECraft is beside a screenshot of the
documentation. Run it again when the screenshot or the icon changes:

    python website/social.py        (needs Pillow)

The font is Ubuntu Light of egui (epaint_default_fonts in the registry of cargo), the
font of the editor; another one is given as the first argument.
"""
import glob
import os
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

here = Path(__file__).resolve().parent
fonts = sorted(glob.glob(os.path.expanduser("~/.cargo/registry/src/*/epaint_default_fonts-*/fonts/Ubuntu-Light.ttf")))
font_file = sys.argv[1] if len(sys.argv) > 1 else (fonts[-1] if fonts else None)
if not font_file:
    sys.exit("no font: give a TrueType file as argument")

W, H, SCALE = 1200, 630, 2
BG, BG2 = (0x07, 0x0B, 0x14), (0x0C, 0x13, 0x22)
TEXT, MUTED, CYAN = (0xE8, 0xEE, 0xF9), (0x9A, 0xA9, 0xC4), (0x3A, 0xD6, 0xFF)
TOP, BOTTOM = (0x2C, 0x72, 0xCC), (0x15, 0x45, 0x86)
WHITE, AMBER, LINE = (0xFF, 0xFF, 0xFF), (0xF5, 0xB8, 0x2E), (0x2A, 0x3A, 0x58)


def px(value):
    return round(value * SCALE)


def font(size):
    return ImageFont.truetype(font_file, px(size))


def gradient(size, top, bottom):
    image = Image.new("RGB", size)
    draw = ImageDraw.Draw(image)
    for y in range(size[1]):
        t = y / max(1, size[1] - 1)
        draw.line([(0, y), (size[0], y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(top, bottom)))
    return image


def icon(side):
    """The drawing of assets/icon.svg (256 units) as a tile with round corners."""
    unit = side / 256
    tile = gradient((side, side), TOP, BOTTOM)
    draw = ImageDraw.Draw(tile)
    width = 19.2 * unit
    for start, end, colour in [((128, 35.84), (128, 97.28), WHITE), ((128, 97.28), (99.84, 97.28), WHITE), ((128, 158.72), (128, 220.16), WHITE), ((128, 158.72), (79.36, 84.48), AMBER)]:
        points = [(x * unit, y * unit) for x, y in (start, end)]
        draw.line(points, fill=colour, width=round(width))
        for x, y in points:
            draw.ellipse([x - width / 2, y - width / 2, x + width / 2, y + width / 2], fill=colour)
    mask = Image.new("L", (side, side), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, side - 1, side - 1], radius=round(51.2 * unit), fill=255)
    return tile, mask


image = gradient((px(W), px(H)), BG2, BG)
draw = ImageDraw.Draw(image)

# The screenshot, running out of the picture on the right.
shot = Image.open(here.parent / "docs" / "images" / "panel-3d.png").convert("RGB")
height = px(470)
shot = shot.resize((round(shot.width * height / shot.height), height), Image.LANCZOS)
left, top = px(560), px(80)
mask = Image.new("L", shot.size, 0)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, shot.width - 1, shot.height - 1], radius=px(14), fill=255)
image.paste(shot, (left, top), mask)
draw.rounded_rectangle([left, top, left + shot.width - 1, top + shot.height - 1], radius=px(14), outline=LINE, width=px(2))

# Icon, name and what it is.
tile, tile_mask = icon(px(84))
image.paste(tile, (px(64), px(150)), tile_mask)
draw.text((px(64), px(262)), "OpenECraft", font=font(68), fill=TEXT)
draw.text((px(66), px(352)), "Open source electrical CAE", font=font(31), fill=CYAN)
draw.text((px(66), px(410)), "Schematics · Reports", font=font(25), fill=MUTED)
draw.text((px(66), px(444)), "Panel layout in 3D", font=font(25), fill=MUTED)

image.resize((W, H), Image.LANCZOS).save(here / "social.jpg", quality=84, optimize=True, progressive=False)
