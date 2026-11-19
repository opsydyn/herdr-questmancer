"""Build the side-by-side art audit from the checkout's actual assets."""
import json
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageOps


REVIEW = Path(__file__).resolve().parent
REPO = REVIEW.parents[3]
GROUND = "#1d202a"
PANEL = "#282c37"
PAPER = "#e6cf9a"
INK = "#f2e4c9"
MUTED = "#b9b4ab"


def heading(draw, at, text, size=24, fill=INK):
    draw.text(at, text, font=ImageFont.load_default(size), fill=fill)


def buffer_image(data):
    result = Image.new("RGBA", tuple(data["size"]))
    result.putdata([(*pixel, 255) if pixel else (0, 0, 0, 0) for pixel in data["pixels"]])
    return result


def paste(canvas, image, x, y, size):
    frame = image.resize(size, Image.Resampling.NEAREST).convert("RGBA")
    canvas.paste(frame, (x, y), frame)


def master(entry, temporary):
    return Image.open(temporary / entry["native"]).convert("RGBA")


def lineup(entries, temporary):
    panel_width, panel_height = 378, 207
    canvas = Image.new("RGB", (1560, 913), GROUND)
    draw = ImageDraw.Draw(canvas)
    heading(draw, (20, 14), "Questmancer · Character lineup", 30)
    heading(draw, (20, 49), "Native images share a canvas. Scene and roster sprites share a pixel scale and ground line.", 17, MUTED)
    columns = [
        (8, 99, 132, "native", "Native PNG"),
        (118, 72, 96, "fallback", "Card fallback"),
        (204, 48, 72, "world", "Scene"),
        (260, 24, 36, "roster", "Roster"),
        (298, 72, 96, "picker", "Picker"),
    ]
    for index, entry in enumerate(entries):
        x, y = 18 + (index % 4) * 384, 79 + (index // 4) * 208
        draw.rectangle((x, y, x + panel_width, y + panel_height - 6), fill=PANEL)
        heading(draw, (x + 10, y + 8), entry["label"], 19)
        for col, w, h, key, name in columns:
            bottom = y + 167
            draw.line((x + col, bottom + 1, x + col + w - 1, bottom + 1), fill=MUTED)
            draw.text((x + col + w // 2, bottom + 10), name,
                      font=ImageFont.load_default(12), fill=MUTED, anchor="mt")
            if key not in entry:
                continue
            im = master(entry, temporary) if key == "native" else buffer_image(entry[key])
            if key in ("native", "fallback"):
                draw.rectangle((x + col, bottom - h, x + col + w - 1, bottom - 1), fill=PAPER)
            paste(canvas, im, x + col, bottom - h, (w, h))
    canvas.save(REVIEW / "01-lineup.png", optimize=True)


def framing(entries, temporary):
    canvas = Image.new("RGB", (1324, 510), GROUND)
    draw = ImageDraw.Draw(canvas)
    heading(draw, (22, 15), "Native portrait framing study", 28)
    heading(draw, (22, 52), "Same PNG pixels, compared on a common ground line.", 16, MUTED)
    stats = {}
    for i, name in enumerate(("Wizard", "Mage", "Artificer", "Rat Royalty", "Sorcerer", "Bard")):
        entry = next(item for item in entries if item["label"] == name)
        original = master(entry, temporary)
        box = original.getchannel("A").point(lambda a: 255 if a >= 128 else 0).getbbox()
        crop_width, crop_height = box[2] - box[0], box[3] - box[1]
        stats[name] = [original.size, box, round(crop_height / original.height * 100, 1)]
        x = 53 + i * 212
        label = f"{name} · reference" if name in ("Wizard", "Mage") else name
        heading(draw, (x, 87), label, 20)
        for row in range(2):
            y = 122 + row * 175
            draw.rectangle((x, y, x + 119, y + 159), fill=PAPER)
            if row == 0:
                paste(canvas, original, x, y, (120, 160))
            else:
                aligned_portrait(canvas, draw, original, box, x, y)
        heading(draw, (x, 468), f"{crop_height / original.height:.0%} height · {crop_width / original.width:.0%} width", 14, MUTED)
    heading(draw, (596, 56), "Top  Current canvas    Bottom  Framing study", 13, MUTED)
    canvas.save(REVIEW / "02-framing.png", optimize=True)
    print(json.dumps(stats, indent=2))


def aligned_portrait(canvas, draw, original, box, x, y):
    clipped = original.crop(box)
    scaled = ImageOps.contain(clipped, (110, 135), Image.Resampling.NEAREST)
    paste(canvas, scaled, x + (120 - scaled.width) // 2, y + 146 - scaled.height, scaled.size)
    draw.line((x, y + 147, x + 119, y + 147), fill="#93723e")


def native_style(entries, temporary):
    portraits = {entry["label"]: master(entry, temporary) for entry in entries}
    rows = [
        ("Artificer", "Rat Royalty", "Sorcerer", "Bard"),
        ("Runewright", "Testmender", "Cleric", "Paladin"),
        ("Druid", "Ranger", "Rogue", "Pathseeker"),
        ("Barbarian", "Librarian"),
    ]
    canvas = Image.new("RGB", (900, 910), GROUND)
    draw = ImageDraw.Draw(canvas)
    heading(draw, (20, 15), "Native portraits · Wizard and Mage reference", 27)
    heading(draw, (20, 51), "The references repeat in every row. Figures fit the same card bounds and share a ground line.", 16, MUTED)
    for row, subjects in enumerate(rows):
        y = 79 + row * 203
        for column, name in enumerate(("Wizard", "Mage", *subjects)):
            x = 20 + column * 145 + (15 if column >= 2 else 0)
            original = portraits[name]
            box = original.getchannel("A").point(lambda a: 255 if a >= 128 else 0).getbbox()
            draw.rectangle((x, y, x + 119, y + 159), fill=PAPER)
            aligned_portrait(canvas, draw, original, box, x, y)
            fill = "#87c7c0" if column < 2 else INK
            heading(draw, (x, y + 168), name, 17, fill)
    canvas.save(REVIEW / "04-native-style.png", optimize=True)


def terminal_image(snapshot):
    width, height = snapshot["size"]
    result = Image.new("RGB", (width * 7, height * 14), GROUND)
    draw = ImageDraw.Draw(result)
    font = ImageFont.load_default(12)
    for face in ("DejaVuSansMono.ttf", "/System/Library/Fonts/Monaco.ttf"):
        try:
            font = ImageFont.truetype(face, 11)
            break
        except OSError:
            continue
    for i, (glyph, fg, bg) in enumerate(snapshot["cells"]):
        x, y = (i % width) * 7, (i // width) * 14
        if bg:
            draw.rectangle((x, y, x + 6, y + 13), fill=tuple(bg))
        if glyph == "▀":
            draw.rectangle((x, y, x + 6, y + 6), fill=tuple(fg))
        elif glyph in "┌┐└┘─│" and fg:
            colour = tuple(fg)
            if glyph in "┌└─":
                draw.line((x + 3, y + 7, x + 6, y + 7), fill=colour)
            if glyph in "┐┘─":
                draw.line((x, y + 7, x + 3, y + 7), fill=colour)
            if glyph in "└┘│":
                draw.line((x + 3, y, x + 3, y + 7), fill=colour)
            if glyph in "┌┐│":
                draw.line((x + 3, y + 7, x + 3, y + 13), fill=colour)
        elif glyph.strip() and fg:
            draw.text((x, y), glyph, font=font, fill=tuple(fg))
    return result


def in_context(evidence):
    canvas = Image.new("RGB", (1020, 690), GROUND)
    draw = ImageDraw.Draw(canvas)
    heading(draw, (20, 12), "The party in context", 28)
    for x, key, title in ((20, "hall", "Guild Hall"), (520, "delve", "Delve")):
        heading(draw, (x, 52), title, 18)
        paste(canvas, buffer_image(evidence[key]), x, 79, (480, 270))
    for x, y, snapshot, label in (
        (20, 397, evidence["inbox"][0], "Wide Inbox with magical scroll"),
        (636, 397, evidence["inbox"][1], "Compact Inbox"),
    ):
        heading(draw, (x, y - 25), label, 18)
        canvas.paste(terminal_image(snapshot), (x, y))
    canvas.save(REVIEW / "03-in-context.png", optimize=True)


def main():
    with tempfile.TemporaryDirectory(prefix="questmancer-lineup-") as directory:
        temporary = Path(directory)
        build = subprocess.check_output(
            ["cargo", "build", "--locked", "--lib", "--features", "storybook", "--message-format=json"],
            cwd=REPO, text=True,
        )
        libs = {}
        for message in map(json.loads, build.splitlines()):
            if message.get("reason") == "compiler-artifact" and message["target"]["name"] in {"questmancer", "ratatui", "serde_json"}:
                libs.update({message["target"]["name"]: path for path in message["filenames"] if path.endswith(".rlib")})
        assert len(libs) == 3, "Missing reviewed library artifacts"
        dep_dir = Path(libs["ratatui"]).parent
        cmd = ["rustc", "--edition=2024", "-D", "warnings", str(REVIEW / "export.rs"), "-L", f"dependency={dep_dir}", "-o", str(temporary / "export")]
        for name, filename in libs.items():
            cmd.extend(["--extern", f"{name}={filename}"])
        subprocess.run(cmd, cwd=REPO, check=True)
        subprocess.run([temporary / "export", temporary], cwd=REPO, check=True)
        evidence = json.loads((temporary / "evidence.json").read_text())
        assert len(evidence["lineup"]) == 16
        lineup(evidence["lineup"], temporary)
        framing(evidence["lineup"], temporary)
        in_context(evidence)
        native_style(evidence["lineup"], temporary)


if __name__ == "__main__":
    main()
