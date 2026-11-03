"""Draws the UMLCraft icon (two class boxes joined by a generalization arrow) and writes every
size and format the installers use: PNG 16-1024, umlcraft.ico (Windows) and umlcraft.icns (macOS).
Run: python3 packaging/icons/make-icons.py"""
import os
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
S = 1024


def draw(size=S):
    k = size / S
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    r = lambda *v: [round(x * k) for x in v]
    # tile with a vertical gradient
    tile = Image.new("RGBA", (size, size))
    td = ImageDraw.Draw(tile)
    for y in range(size):
        t = y / max(size - 1, 1)
        c = tuple(round(a + (b - a) * t) for a, b in zip((0x3a, 0x6f, 0xc4), (0x1b, 0x2f, 0x5a)))
        td.line([(0, y), (size, y)], fill=c + (255,))
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle(r(64, 64, 960, 960), radius=round(190 * k), fill=255)
    img.paste(tile, (0, 0), mask)
    w = max(1, round(26 * k))
    white, gold = (255, 255, 255, 255), (0xf2, 0xb8, 0x2e, 255)

    def box(x, y, bw, bh, header):
        d.rounded_rectangle(r(x + 18, y + 22, x + bw + 18, y + bh + 22), radius=round(18 * k), fill=(0, 0, 0, 60))
        d.rounded_rectangle(r(x, y, x + bw, y + bh), radius=round(18 * k), fill=white)
        d.rounded_rectangle(r(x, y, x + bw, y + header), radius=round(18 * k), fill=gold)
        d.rectangle(r(x, y + header - 20, x + bw, y + header), fill=gold)
        for i in range(2):
            ly = y + header + 52 + i * 62
            d.rounded_rectangle(r(x + 44, ly, x + bw - 70 - i * 60, ly + 22), radius=round(11 * k), fill=(0x9a, 0xa7, 0xb8, 255))

    box(300, 170, 424, 280, 100)  # parent
    box(170, 620, 300, 250, 86)  # child left
    box(560, 620, 300, 250, 86)  # child right
    # generalization: lines up to a hollow triangle under the parent
    d.line(r(320, 620, 320, 572, 704, 572, 704, 620), fill=white, width=w, joint="curve")
    d.line(r(512, 572, 512, 548), fill=white, width=w)
    # hollow triangle: a white one with a smaller tile-coloured one inside
    d.polygon(r(512, 462, 446, 556, 578, 556), fill=white)
    d.polygon(r(512, 500, 474, 540, 550, 540), fill=(0x2a, 0x4c, 0x8c, 255))
    return img


big = draw(S)
for n in [16, 24, 32, 48, 64, 128, 256, 512, 1024]:
    (big if n == S else draw(n) if n <= 64 else big.resize((n, n), Image.LANCZOS)).save(os.path.join(HERE, f"umlcraft-{n}.png"))
big.save(os.path.join(HERE, "umlcraft.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
big.save(os.path.join(HERE, "umlcraft.icns"))
print("icons written to", HERE)
