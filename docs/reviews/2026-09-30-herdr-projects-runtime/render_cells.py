"""Rasterise exported production Ratatui cells; these are not native captures."""
import json
import re
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).parent / "images"
FONT = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 18)
CELL_W, CELL_H = 11, 22
NAMED = {"Reset": (230, 230, 230), "Black": (0, 0, 0), "White": (255, 255, 255), "Gray": (170, 170, 170), "DarkGray": (85, 85, 85), "Yellow": (170, 85, 0)}

def colour(value, background=False):
    numbers = re.findall(r"\d+", value)
    if value.startswith("Rgb"):
        return tuple(map(int, numbers))
    if value == "Reset" and background:
        return (0, 0, 0)
    if value not in NAMED:
        raise ValueError(f"Unsupported exported colour: {value}")
    return NAMED[value]

for path in sorted(ROOT.glob("*.json")):
    data = json.loads(path.read_text())
    im = Image.new("RGB", (data["width"] * CELL_W, data["height"] * CELL_H))
    draw = ImageDraw.Draw(im)
    for index, cell in enumerate(data["cells"]):
        x = index % data["width"] * CELL_W
        y = index // data["width"] * CELL_H
        fg, bg = colour(cell["fg"]), colour(cell["bg"], True)
        draw.rectangle((x, y, x + CELL_W - 1, y + CELL_H - 1), fill=bg)
        if cell["s"] == "▀":
            draw.rectangle((x, y, x + CELL_W - 1, y + CELL_H // 2 - 1), fill=fg)
        elif cell["s"] != " ":
            draw.text((x, y + 1), cell["s"], fill=fg, font=FONT)
    im.save(path.with_suffix(".png"))
    print(path.with_suffix(".png"))
