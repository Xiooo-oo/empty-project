"""Copy the supplied cat art and make clean, transparent pixel-art sprites."""

from collections import deque
import os
from pathlib import Path
import shutil

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(os.environ.get("CAT_DESKPET_SOURCE", r"D:\迁移文件夹\文件夹老大\桌宠图片"))
MAPPING = {
    "sleep.png": "6f59a595daca4f91a30b4da4ace96e84.jpeg~tplv-a9rns2rl98-ds_wm_1_5_marc_b_3_dk_RG91YmFvMDIxTk5BTkEwMjYwODA5Mk5O.png",
    "idle.png": "3670856f95d448e29404611aa18d314c.jpeg~tplv-a9rns2rl98-ds_wm_1_5_marc_b_3_dk_RG91YmFvMDIxTk5BMDAxMjYwODA5Mk5O.png",
    "walk-1.png": "5506271892e242568057ddf97eb70772.jpeg~tplv-a9rns2rl98-ds_wm_1_5_marc_b_3_dk_RG91YmFvMDIxTk5BMDAxMjYwODA5Mk5O.png",
    "walk-2.png": "d779fbf1ddfc49aa8cc3cc7321b6d9dc.jpeg~tplv-a9rns2rl98-ds_wm_1_5_marc_b_3_dk_RG91YmFvMDIxTk5BMDAxMjYwODA5Mk5O.png",
    "walk-3.png": "eda771e4f8b84dce8f4150995c96698a.jpeg~tplv-a9rns2rl98-ds_wm_1_5_marc_b_3_dk_RG91YmFvMDIxTk5BMDAxMjYwODA5Mk5O.png",
}


def cutout(image: Image.Image) -> Image.Image:
    """Flood-fill near-white background from the edges, preserving white fur."""
    image = image.convert("RGBA")
    w, h = image.size
    px = image.load()
    seen = bytearray(w * h)
    queue = deque()

    def is_background(x: int, y: int) -> bool:
        r, g, b, _ = px[x, y]
        return min(r, g, b) >= 218 and max(r, g, b) - min(r, g, b) <= 34

    for x in range(w):
        for y in (0, h - 1):
            if is_background(x, y):
                idx = y * w + x
                if not seen[idx]:
                    seen[idx] = 1
                    queue.append((x, y))
    for y in range(h):
        for x in (0, w - 1):
            if is_background(x, y):
                idx = y * w + x
                if not seen[idx]:
                    seen[idx] = 1
                    queue.append((x, y))

    while queue:
        x, y = queue.popleft()
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if 0 <= nx < w and 0 <= ny < h:
                idx = ny * w + nx
                if not seen[idx] and is_background(nx, ny):
                    seen[idx] = 1
                    queue.append((nx, ny))

    for y in range(h):
        for x in range(w):
            if seen[y * w + x]:
                r, g, b, _ = px[x, y]
                px[x, y] = (r, g, b, 0)

    bounds = image.getchannel("A").getbbox()
    assert bounds, "No cat pixels found in source image"
    return image.crop(bounds)


def main() -> None:
    output = ROOT / "public" / "cats"
    originals = ROOT / "assets" / "source"
    docs = ROOT / "docs"
    output.mkdir(parents=True, exist_ok=True)
    originals.mkdir(parents=True, exist_ok=True)
    docs.mkdir(parents=True, exist_ok=True)

    sprites: dict[str, Image.Image] = {}
    for target, filename in MAPPING.items():
        source = originals / filename
        if not source.is_file():
            source = SOURCE / filename
        if not source.is_file():
            raise FileNotFoundError(source)
        if source.resolve() != (originals / filename).resolve():
            shutil.copy2(source, originals / filename)
        sprite = cutout(Image.open(source))
        # Keep a shared canvas so all poses sit at the same UI position.
        sprite.thumbnail((320, 320), Image.Resampling.NEAREST)
        canvas = Image.new("RGBA", (320, 320), (0, 0, 0, 0))
        canvas.alpha_composite(sprite, ((320 - sprite.width) // 2, (320 - sprite.height) // 2))
        canvas.save(output / target, optimize=True)
        sprites[target] = canvas

    # A short animated preview also doubles as the README demo image.
    cards = []
    font = next((ImageFont.truetype(path, 15) for path in (
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ) if Path(path).is_file()), ImageFont.load_default())
    labels = [("idle.png", "待机"), ("walk-1.png", "行走"), ("walk-2.png", "行走"),
              ("walk-3.png", "行走"), ("sleep.png", "休息")]
    for name, label in labels:
        card = Image.new("RGBA", (220, 220), (250, 245, 233, 255))
        draw = ImageDraw.Draw(card)
        draw.rounded_rectangle((4, 4, 216, 216), radius=16, fill=(255, 252, 243, 255), outline=(71, 56, 68, 255), width=3)
        cat = sprites[name].resize((184, 184), Image.Resampling.NEAREST)
        card.alpha_composite(cat, (18, 10))
        draw.rounded_rectangle((70, 184, 150, 212), radius=8, fill=(247, 237, 218, 255))
        draw.text((110, 198), label, anchor="mm", font=font, fill=(56, 44, 55, 255))
        cards.append(card.convert("P", palette=Image.Palette.ADAPTIVE))
    cards[0].save(docs / "demo.gif", save_all=True, append_images=cards[1:], duration=[900, 220, 220, 220, 900], loop=0, optimize=True)

    icons = ROOT / "src-tauri" / "icons"
    icons.mkdir(parents=True, exist_ok=True)
    icon = Image.new("RGBA", (256, 256), (250, 245, 233, 255))
    draw = ImageDraw.Draw(icon)
    draw.rounded_rectangle((4, 4, 252, 252), radius=56, fill=(250, 245, 233, 255), outline=(71, 56, 68, 255), width=9)
    mascot = sprites["idle.png"].resize((220, 220), Image.Resampling.NEAREST)
    icon.alpha_composite(mascot, (18, 12))
    icon.resize((32, 32), Image.Resampling.NEAREST).save(icons / "32x32.png")
    icon.resize((128, 128), Image.Resampling.NEAREST).save(icons / "128x128.png")
    icon.save(icons / "icon.ico", sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print(f"Prepared {len(sprites)} transparent sprites and docs/demo.gif")


if __name__ == "__main__":
    main()
