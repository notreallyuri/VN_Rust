import argparse
import math
from pathlib import Path

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

W, H = 1280, 720
CW, CH = 600, 900
SS = 2

RNG = np.random.default_rng(1894)


def rgb(hex_color):
    hex_color = hex_color.lstrip("#")
    return tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))


def vertical_gradient(size, top, bottom):
    w, h = size
    t = np.linspace(0.0, 1.0, h)[:, None, None]
    top = np.array(top, dtype=np.float32)[None, None, :]
    bottom = np.array(bottom, dtype=np.float32)[None, None, :]
    pixels = top * (1 - t) + bottom * t
    return np.repeat(pixels, w, axis=1)


def to_image(pixels):
    return Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8), "RGB")


def glow(pixels, center, radius, color, strength=1.0):
    h, w, _ = pixels.shape
    y, x = np.mgrid[0:h, 0:w]
    d = np.sqrt((x - center[0]) ** 2 + (y - center[1]) ** 2) / radius
    falloff = np.clip(1.0 - d, 0.0, 1.0) ** 2 * strength
    pixels += falloff[:, :, None] * np.array(color, dtype=np.float32)[None, None, :]
    return pixels


def vignette(pixels, strength=0.6):
    h, w, _ = pixels.shape
    y, x = np.mgrid[0:h, 0:w]
    d = np.sqrt(((x - w / 2) / (w / 2)) ** 2 + ((y - h / 2) / (h / 2)) ** 2) / math.sqrt(2)
    pixels *= (1.0 - strength * d**1.8)[:, :, None]
    return pixels


def grade(pixels, tint, amount=0.12):
    return pixels * (1 - amount) + np.array(tint, dtype=np.float32)[None, None, :] * amount * (
        pixels / 255.0
    ) * 2.2


class Painting:
    def __init__(self, top, bottom):
        self.base = to_image(vertical_gradient((W, H), rgb(top), rgb(bottom)))
        self.layers = []

    def shapes(self, blur, draw_fn, opacity=1.0):
        layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        draw_fn(ImageDraw.Draw(layer))
        if blur:
            layer = layer.filter(ImageFilter.GaussianBlur(blur))
        if opacity < 1.0:
            alpha = layer.getchannel("A").point(lambda a: int(a * opacity))
            layer.putalpha(alpha)
        self.layers.append(layer)

    def finish(self, glows=(), tint="#000000", tint_amount=0.0, vignette_strength=0.65):
        image = self.base.convert("RGBA")
        for layer in self.layers:
            image = Image.alpha_composite(image, layer)
        pixels = np.asarray(image.convert("RGB"), dtype=np.float32)
        for center, radius, color, strength in glows:
            pixels = glow(pixels, center, radius, rgb(color), strength)
        if tint_amount:
            pixels = grade(pixels, rgb(tint), tint_amount)
        pixels = vignette(pixels, vignette_strength)
        return to_image(pixels).filter(ImageFilter.GaussianBlur(0.6))


def window(d, x, y, w, h, frame, glass, bars=True):
    d.rectangle([x - 8, y - 8, x + w + 8, y + h + 8], fill=frame)
    d.rectangle([x, y, x + w, y + h], fill=glass)
    if bars:
        d.line([x + w / 2, y, x + w / 2, y + h], fill=frame, width=6)
        d.line([x, y + h / 2, x + w, y + h / 2], fill=frame, width=6)


def bookshelf(d, x, y, w, h, wood, spines):
    d.rectangle([x, y, x + w, y + h], fill=wood)
    rows = 5
    for r in range(rows):
        ry = y + 12 + r * (h - 24) / rows
        rh = (h - 24) / rows - 10
        cx = x + 10
        while cx < x + w - 14:
            bw = int(RNG.integers(8, 18))
            bh = rh * float(RNG.uniform(0.7, 1.0))
            color = spines[int(RNG.integers(0, len(spines)))]
            d.rectangle([cx, ry + rh - bh, cx + bw, ry + rh], fill=color)
            cx += bw + 2


def candle(d, x, y, scale=1.0):
    d.rectangle([x - 5 * scale, y, x + 5 * scale, y + 40 * scale], fill=rgb("#d9cdb4"))
    d.ellipse([x - 6 * scale, y - 20 * scale, x + 6 * scale, y - 2 * scale], fill=rgb("#ffd98a"))


def rain(d, color, count=260, length=38):
    for _ in range(count):
        x = float(RNG.uniform(0, W))
        y = float(RNG.uniform(0, H))
        d.line([x, y, x - 8, y + length], fill=color, width=1)


def title():
    p = Painting("#0b0a10", "#1b1622")
    p.shapes(30, lambda d: d.ellipse([440, 60, 840, 460], fill=rgb("#3a2f3f")))
    p.shapes(
        2,
        lambda d: [
            d.polygon([(0, 720), (0, 520), (220, 470), (420, 540), (640, 480), (860, 545), (1080, 470), (1280, 520), (1280, 720)], fill=rgb("#08070b")),
            d.rectangle([590, 330, 690, 560], fill=rgb("#0a090e")),
            d.polygon([(575, 335), (640, 250), (705, 335)], fill=rgb("#0a090e")),
            d.rectangle([628, 395, 652, 440], fill=rgb("#e8c98a")),
        ],
    )
    return p.finish(
        glows=[((640, 418), 140, "#b98a4a", 0.5), ((640, 260), 420, "#5b4a6a", 0.35)],
        vignette_strength=0.75,
    )


def archive_office():
    p = Painting("#1d1a17", "#0f0d0c")
    spines = [rgb(c) for c in ("#4a2e25", "#3b3a2a", "#2d3342", "#5a4a32", "#3f2a2f")]
    p.shapes(1.5, lambda d: [bookshelf(d, x, 90, 230, 520, rgb("#231a14"), spines) for x in (30, 280, 990)])
    p.shapes(
        1.5,
        lambda d: [
            window(d, 580, 110, 320, 280, rgb("#1a1511"), rgb("#3c4658")),
            d.rectangle([0, 600, 1280, 720], fill=rgb("#17120e")),
            d.rectangle([520, 520, 1000, 560], fill=rgb("#3a2a1d")),
            d.rectangle([540, 560, 560, 700], fill=rgb("#2a1e15")),
            d.rectangle([960, 560, 980, 700], fill=rgb("#2a1e15")),
            d.rectangle([600, 500, 700, 522], fill=rgb("#c9b48d")),
            d.rectangle([720, 492, 800, 522], fill=rgb("#6b4e32")),
            candle(d, 880, 468),
        ],
    )
    return p.finish(
        glows=[((880, 460), 260, "#e0a655", 0.55), ((740, 250), 360, "#56637a", 0.25)],
        tint="#c79a5b",
        tint_amount=0.1,
    )


def von_lucis_study():
    p = Painting("#1a1216", "#0c0809")
    p.shapes(
        2,
        lambda d: [
            d.rectangle([0, 580, 1280, 720], fill=rgb("#140c0c")),
            d.rectangle([160, 360, 460, 600], fill=rgb("#2a1a16")),
            d.rectangle([200, 430, 420, 600], fill=rgb("#0a0606")),
            d.rectangle([140, 340, 480, 370], fill=rgb("#3a2620")),
            bookshelf(d, 900, 80, 300, 520, rgb("#1e1310"), [rgb("#4d2b25"), rgb("#2f2a22"), rgb("#44362a")]),
            d.rectangle([560, 480, 820, 520], fill=rgb("#35231b")),
            d.rectangle([600, 460, 700, 480], fill=rgb("#d9cfbf")),
        ],
    )
    p.shapes(10, lambda d: d.ellipse([230, 470, 390, 590], fill=rgb("#ff7a2a")), opacity=0.9)
    return p.finish(
        glows=[((310, 540), 320, "#ff8a3a", 0.75), ((650, 470), 180, "#e0b070", 0.25)],
        tint="#a84a2a",
        tint_amount=0.12,
    )


def von_lucis_bedroom():
    p = Painting("#141a26", "#0a0c12")
    p.shapes(
        2,
        lambda d: [
            window(d, 820, 120, 260, 340, rgb("#10131b"), rgb("#6d7c95")),
            d.rectangle([0, 600, 1280, 720], fill=rgb("#0c0e14")),
            d.rectangle([120, 430, 640, 600], fill=rgb("#2b3142")),
            d.rectangle([120, 330, 170, 600], fill=rgb("#1c2130")),
            d.rectangle([140, 420, 640, 470], fill=rgb("#c8c6c0")),
        ],
    )
    p.shapes(40, lambda d: d.polygon([(820, 460), (1080, 460), (900, 720), (500, 720)], fill=rgb("#8a9bb8")), opacity=0.35)
    return p.finish(glows=[((950, 290), 380, "#9fb2d4", 0.4)], tint="#6a86b8", tint_amount=0.1)


def von_lucis_hall():
    p = Painting("#1a1512", "#0b0908")
    p.shapes(
        2,
        lambda d: [
            d.polygon([(0, 0), (420, 160), (420, 560), (0, 720)], fill=rgb("#211a15")),
            d.polygon([(1280, 0), (860, 160), (860, 560), (1280, 720)], fill=rgb("#211a15")),
            d.rectangle([420, 160, 860, 560], fill=rgb("#120e0c")),
            d.polygon([(0, 720), (420, 560), (860, 560), (1280, 720)], fill=rgb("#17110d")),
            d.rectangle([590, 250, 690, 560], fill=rgb("#2b2019")),
            [candle(d, x, y, s) for x, y, s in ((150, 300, 1.4), (300, 280, 1.1), (1130, 300, 1.4))],
        ],
    )
    return p.finish(
        glows=[((150, 280), 220, "#f0b060", 0.55), ((300, 262), 170, "#f0b060", 0.45), ((1130, 280), 220, "#f0b060", 0.55)],
        tint="#d19a55",
        tint_amount=0.08,
    )


def santa_ilde_door():
    p = Painting("#0d1119", "#06080c")
    p.shapes(
        2,
        lambda d: [
            d.rectangle([260, 120, 1020, 720], fill=rgb("#1b1e25")),
            d.polygon([(220, 130), (640, 20), (1060, 130)], fill=rgb("#14161c")),
            d.rectangle([560, 360, 720, 720], fill=rgb("#2a2019")),
            d.rectangle([574, 374, 706, 720], fill=rgb("#3a2b20")),
            window(d, 330, 260, 120, 160, rgb("#14161c"), rgb("#caa15f")),
            window(d, 830, 260, 120, 160, rgb("#14161c"), rgb("#2c3342")),
            d.rectangle([0, 660, 1280, 720], fill=rgb("#0a0b0e")),
        ],
    )
    p.shapes(0, lambda d: rain(d, (150, 165, 190, 90)))
    return p.finish(
        glows=[((390, 340), 200, "#e0a860", 0.45), ((640, 350), 90, "#ffcf8a", 0.35)],
        tint="#5a6f96",
        tint_amount=0.1,
        vignette_strength=0.8,
    )


def river_road():
    p = Painting("#10151c", "#0a0c10")
    p.shapes(20, lambda d: d.ellipse([940, 60, 1060, 180], fill=rgb("#c9d2dc")), opacity=0.8)
    p.shapes(
        2,
        lambda d: [
            d.polygon([(0, 430), (1280, 380), (1280, 720), (0, 720)], fill=rgb("#0c1015")),
            d.polygon([(0, 520), (1280, 470), (1280, 560), (0, 610)], fill=rgb("#27313d")),
            d.polygon([(420, 720), (560, 440), (640, 440), (820, 720)], fill=rgb("#1c1a18")),
            d.rectangle([980, 340, 1280, 380], fill=rgb("#15181d")),
            [d.polygon([(x, 440), (x + 30, 250 + (x % 7) * 10), (x + 60, 440)], fill=rgb("#07090c")) for x in range(0, 380, 55)],
        ],
    )
    return p.finish(glows=[((1000, 120), 300, "#8ea4bd", 0.35)], tint="#4a6a8a", tint_amount=0.1, vignette_strength=0.8)


def santa_ilde_office():
    p = Painting("#1c1814", "#0d0b09")
    spines = [rgb(c) for c in ("#3d2c22", "#2e3029", "#3a3326")]
    p.shapes(
        1.5,
        lambda d: [
            bookshelf(d, 60, 110, 260, 480, rgb("#1f1812"), spines),
            window(d, 760, 120, 300, 260, rgb("#17130f"), rgb("#9a8b6c")),
            d.rectangle([0, 600, 1280, 720], fill=rgb("#15110d")),
            d.rectangle([420, 500, 900, 540], fill=rgb("#3a2b1f")),
            d.rectangle([520, 482, 600, 502], fill=rgb("#bba98a")),
            d.ellipse([760, 470, 800, 500], fill=rgb("#2a1f17")),
        ],
    )
    return p.finish(glows=[((910, 250), 380, "#d8b67a", 0.4)], tint="#c29a60", tint_amount=0.1)


def field_post():
    p = Painting("#161512", "#0b0a09")
    p.shapes(
        2,
        lambda d: [
            d.rectangle([0, 580, 1280, 720], fill=rgb("#12100d")),
            d.rectangle([300, 450, 980, 490], fill=rgb("#2f271d")),
            [d.rectangle([330 + i * 70, 420 - (i % 3) * 6, 390 + i * 70, 450], fill=rgb("#cdbf9f")) for i in range(8)],
            d.rectangle([1050, 150, 1200, 560], fill=rgb("#1b1814")),
            [d.rectangle([1065, 170 + i * 45, 1185, 205 + i * 45], fill=rgb("#26221c")) for i in range(8)],
            candle(d, 360, 380),
        ],
    )
    p.shapes(60, lambda d: d.ellipse([200, 200, 700, 520], fill=rgb("#000000")), opacity=0.5)
    return p.finish(glows=[((360, 370), 300, "#e0a050", 0.6)], tint="#b08850", tint_amount=0.1, vignette_strength=0.85)


def santa_ilde_courtyard():
    p = Painting("#3a2a2f", "#1b1418")
    p.shapes(
        2,
        lambda d: [
            d.rectangle([0, 200, 380, 720], fill=rgb("#221a1c")),
            d.rectangle([900, 180, 1280, 720], fill=rgb("#221a1c")),
            [d.rectangle([40 + i * 110, 280, 100 + i * 110, 400], fill=rgb("#c89868")) for i in range(3)],
            [d.rectangle([940 + i * 110, 260, 1000 + i * 110, 380], fill=rgb("#16111a")) for i in range(3)],
            d.polygon([(0, 720), (380, 560), (900, 560), (1280, 720)], fill=rgb("#2a2126")),
            d.polygon([(628, 560), (636, 380), (644, 380), (656, 560)], fill=rgb("#161216")),
            [d.ellipse([x - r, y - r * 0.8, x + r, y + r * 0.8], fill=rgb("#1b1519")) for x, y, r in ((600, 330, 70), (680, 320, 80), (640, 270, 75), (560, 380, 50), (720, 375, 55))],
        ],
    )
    return p.finish(glows=[((640, 120), 520, "#d9884a", 0.35), ((150, 340), 260, "#e0a060", 0.3)], tint="#d07a4a", tint_amount=0.12)


def nursery():
    p = Painting("#1b1f26", "#0d0f13")
    p.shapes(
        2,
        lambda d: [
            window(d, 520, 90, 240, 300, rgb("#12151b"), rgb("#7c8aa0")),
            d.rectangle([0, 600, 1280, 720], fill=rgb("#101216")),
            [
                (
                    d.rectangle([x, 440, x + 200, 560], fill=rgb("#2c3240")),
                    d.rectangle([x, 420, x + 12, 600], fill=rgb("#20242e")),
                    d.rectangle([x + 188, 420, x + 200, 600], fill=rgb("#20242e")),
                )
                for x in (80, 380, 880)
            ],
            d.ellipse([940, 470, 990, 520], fill=rgb("#b8864a")),
            d.rectangle([1000, 480, 1040, 520], fill=rgb("#7a3b3b")),
        ],
    )
    p.shapes(30, lambda d: d.polygon([(520, 390), (760, 390), (860, 720), (420, 720)], fill=rgb("#9aa8bd")), opacity=0.25)
    return p.finish(glows=[((640, 240), 360, "#a6b4c8", 0.35), ((990, 500), 120, "#e9c27a", 0.4)], tint="#7a8fb0", tint_amount=0.08)


BACKGROUNDS = {
    "title": title,
    "archive_office": archive_office,
    "von_lucis_study": von_lucis_study,
    "von_lucis_bedroom": von_lucis_bedroom,
    "von_lucis_hall": von_lucis_hall,
    "santa_ilde_door": santa_ilde_door,
    "river_road": river_road,
    "field_post": field_post,
    "santa_ilde_office": santa_ilde_office,
    "santa_ilde_courtyard": santa_ilde_courtyard,
    "nursery": nursery,
}


def bust_masks(pose):
    w, h = CW * SS, CH * SS
    body = Image.new("L", (w, h), 0)
    hair = Image.new("L", (w, h), 0)
    d = ImageDraw.Draw(body)
    hd = ImageDraw.Draw(hair)

    scale = pose.get("scale", 1.0)
    cx = w * (0.5 + pose.get("shift", 0.0))
    bottom = h

    def s(v):
        return v * SS * scale

    head_ry = s(128) * pose.get("head", 1.0)
    head_rx = head_ry * 0.8
    neck_y = bottom - s(400)
    head_cy = neck_y - head_ry * 0.72 + s(pose.get("lower", 0))
    head_cx = cx + s(pose.get("lean", 0))
    shoulder = s(235) * pose.get("shoulders", 1.0)

    d.polygon(
        [
            (cx - s(58), neck_y - s(30)),
            (cx + s(58), neck_y - s(30)),
            (cx + s(74), neck_y + s(50)),
            (cx + shoulder * 0.75, neck_y + s(88)),
            (cx + shoulder, neck_y + s(150)),
            (cx + shoulder + s(30), bottom),
            (cx - shoulder - s(30), bottom),
            (cx - shoulder, neck_y + s(150)),
            (cx - shoulder * 0.75, neck_y + s(88)),
            (cx - s(74), neck_y + s(50)),
        ],
        fill=255,
    )
    d.ellipse([cx - shoulder, neck_y + s(70), cx + shoulder, neck_y + s(230)], fill=255)

    feature = pose.get("feature")
    if feature == "long_hair":
        hd.ellipse([head_cx - head_rx * 1.12, head_cy - head_ry * 1.08, head_cx + head_rx * 1.12, head_cy + head_ry * 0.5], fill=255)
        hd.polygon(
            [
                (head_cx - head_rx * 1.12, head_cy - head_ry * 0.1),
                (head_cx + head_rx * 1.12, head_cy - head_ry * 0.1),
                (head_cx + head_rx * 1.3, neck_y + s(150)),
                (head_cx + head_rx * 0.55, neck_y + s(170)),
                (head_cx - head_rx * 0.55, neck_y + s(170)),
                (head_cx - head_rx * 1.3, neck_y + s(150)),
            ],
            fill=255,
        )
    if feature == "short_hair":
        hd.ellipse([head_cx - head_rx * 1.06, head_cy - head_ry * 1.08, head_cx + head_rx * 1.06, head_cy - head_ry * 0.05], fill=255)
    if feature in ("veil", "hood"):
        top = head_cy - head_ry * (1.4 if feature == "hood" else 1.15)
        hd.polygon(
            [
                (head_cx, top - (s(50) if feature == "hood" else 0)),
                (head_cx + head_rx * 1.45, head_cy - head_ry * 0.2),
                (head_cx + shoulder * 0.95, neck_y + s(160)),
                (head_cx - shoulder * 0.95, neck_y + s(160)),
                (head_cx - head_rx * 1.45, head_cy - head_ry * 0.2),
            ],
            fill=255,
        )
        hd.ellipse([head_cx - head_rx * 1.2, top - s(10), head_cx + head_rx * 1.2, head_cy + head_ry * 0.4], fill=255)
    if feature in ("hat", "top_hat"):
        brim_y = head_cy - head_ry * 0.55
        hd.ellipse([head_cx - head_rx * 1.6, brim_y - s(16), head_cx + head_rx * 1.6, brim_y + s(16)], fill=255)
        crown = s(125) if feature == "top_hat" else s(72)
        hd.rounded_rectangle([head_cx - head_rx * 0.98, brim_y - crown, head_cx + head_rx * 0.98, brim_y], radius=s(18), fill=255)

    face = Image.new("L", (w, h), 0)
    ImageDraw.Draw(face).ellipse([head_cx - head_rx, head_cy - head_ry, head_cx + head_rx, head_cy + head_ry], fill=255)
    if feature == "hood":
        face = ImageChops.multiply(face, Image.new("L", (w, h), 90))
    body = ImageChops.lighter(body, face)
    return body, hair, face, (head_cx, head_cy, head_rx, head_ry), neck_y, cx


def shade(mask, top, bottom):
    w, h = mask.size
    fill = vertical_gradient((w, h), top, bottom)
    image = Image.fromarray(np.clip(fill, 0, 255).astype(np.uint8), "RGB").convert("RGBA")
    image.putalpha(mask)
    return image


def rim(mask, dx, color, strength):
    shifted = ImageChops.offset(mask, int(dx * SS), int(3 * SS))
    edge = ImageChops.subtract(mask, shifted).filter(ImageFilter.GaussianBlur(2.5 * SS))
    layer = Image.new("RGBA", mask.size, rgb(color) + (0,))
    layer.putalpha(edge.point(lambda a: int(min(255, a * strength))))
    return layer


def portrait(color, pose):
    w, h = CW * SS, CH * SS
    body, hair, face, (hx, hy, hrx, hry), neck_y, cx = bust_masks(pose)

    base = np.array(rgb(color), dtype=np.float32)
    mood = pose.get("mood", 1.0)
    image = shade(body, base * 0.72 * mood, base * 0.3 * mood)

    face_light = pose.get("face_light", 1.0)
    skin = np.clip(base * 0.55 + 70, 0, 255) * mood
    face_layer = shade(face, skin * (0.8 + 0.2 * face_light), skin * 0.7)
    face_layer.putalpha(face.filter(ImageFilter.GaussianBlur(1.5 * SS)).point(lambda a: int(a * min(1.0, 0.75 * face_light))))

    hair_color = np.array(rgb(pose.get("hair", "#2a2230")), dtype=np.float32) * mood
    hair_layer = shade(hair, hair_color * 1.3, hair_color * 0.7)

    if pose.get("feature") in ("long_hair", "veil"):
        image.alpha_composite(hair_layer)
        image.alpha_composite(face_layer)
    else:
        image.alpha_composite(face_layer)
        image.alpha_composite(hair_layer)

    silhouette = ImageChops.lighter(body, hair)
    image.alpha_composite(rim(silhouette, -9, pose.get("rim", "#e8dcc4"), pose.get("rim_strength", 1.0) * 1.2))
    image.alpha_composite(rim(silhouette, 9, pose.get("back_rim", "#8c9ac4"), 0.7))

    detail = ImageDraw.Draw(image)
    accent = rgb(pose.get("accent", "#d8d2c4"))
    feature = pose.get("feature")
    if feature == "cap":
        detail.ellipse([hx - hrx * 0.9, hy - hry * 1.08, hx + hrx * 0.9, hy - hry * 0.62], fill=accent + (240,))
    if feature == "veil":
        detail.arc([hx - hrx * 1.08, hy - hry * 1.08, hx + hrx * 1.08, hy + hry * 1.08], 195, 345, fill=accent + (230,), width=int(18 * SS))
    if pose.get("collar"):
        detail.polygon([(cx - 70 * SS, neck_y + 30 * SS), (cx, neck_y + 120 * SS), (cx + 70 * SS, neck_y + 30 * SS), (cx, neck_y + 60 * SS)], fill=accent + (220,))
    if pose.get("tie"):
        detail.polygon([(cx - 16 * SS, neck_y + 60 * SS), (cx + 16 * SS, neck_y + 60 * SS), (cx, neck_y + 210 * SS)], fill=rgb(pose.get("tie_color", "#5a2a2a")) + (230,))
    if pose.get("glasses"):
        for side in (-1, 1):
            gx = hx + side * hrx * 0.42
            gy = hy - hry * 0.05
            detail.ellipse([gx - 24 * SS, gy - 17 * SS, gx + 24 * SS, gy + 17 * SS], outline=accent + (215,), width=int(4 * SS))
        detail.line([hx - hrx * 0.42 + 24 * SS, hy - hry * 0.05, hx + hrx * 0.42 - 24 * SS, hy - hry * 0.05], fill=accent + (215,), width=int(4 * SS))

    if pose.get("ghost"):
        faint = image.copy()
        faint.putalpha(faint.getchannel("A").point(lambda a: int(a * 0.35)))
        doubled = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        doubled.alpha_composite(faint, (int(-38 * SS), int(-6 * SS)))
        doubled.alpha_composite(faint, (int(34 * SS), int(8 * SS)))
        body_copy = image.copy()
        body_copy.putalpha(body_copy.getchannel("A").point(lambda a: int(a * 0.6)))
        doubled.alpha_composite(body_copy)
        image = doubled.filter(ImageFilter.GaussianBlur(2.5 * SS))

    return image.resize((CW, CH), Image.LANCZOS)


PEOPLE = {
    "registrar": ("#8c95a8", {"collar": True, "shoulders": 1.1, "rim": "#c9d4e8", "hair": "#3a3a44"}, {"neutral": {}, "stern": {"mood": 0.8, "rim": "#9fb0cc", "lower": 8}}),
    "mary": ("#d8b98a", {"feature": "long_hair", "shoulders": 0.86, "head": 0.95, "hair": "#5a3b26"}, {
        "tired": {"lower": 26, "lean": -10, "mood": 0.85, "rim_strength": 0.6},
        "afraid": {"scale": 0.95, "rim": "#a9c0e8", "mood": 0.8, "shift": 0.02},
        "resolved": {"rim": "#ffd49a", "rim_strength": 1.2, "face_light": 1.4},
    }),
    "hugo": ("#9fb2d0", {"collar": True, "tie": True, "shoulders": 1.08}, {
        "neutral": {},
        "tired": {"lower": 30, "lean": 12, "mood": 0.8, "rim_strength": 0.6},
    }),
    "adelaide": ("#c7a7c4", {"feature": "cap", "shoulders": 0.92, "accent": "#ece6da"}, {
        "neutral": {},
        "afraid": {"scale": 0.95, "rim": "#a9c0e8", "mood": 0.8},
    }),
    "clara": ("#a9c7a0", {"feature": "veil", "shoulders": 0.95, "accent": "#ece6da", "hair": "#1c2420"}, {
        "neutral": {},
        "guarded": {"lower": 14, "mood": 0.85, "rim": "#d0d8e0", "face_light": 0.7},
        "unveiled": {"feature": "cap", "rim": "#ffd49a", "rim_strength": 1.1, "face_light": 1.3},
    }),
    "francis": ("#cfa987", {"shoulders": 1.0, "tie": True, "accent": "#6a4a3a"}, {"neutral": {}}),
    "moriarty": ("#c98276", {"glasses": True, "shoulders": 1.02, "accent": "#e6dccb"}, {
        "neutral": {},
        "older": {"lower": 22, "lean": 14, "mood": 0.75, "rim": "#d8d8d8", "rim_strength": 0.7},
        "wary": {"mood": 0.85, "rim": "#e6a070", "lower": 6},
    }),
    "house_envoy": ("#9a9aa0", {"feature": "top_hat", "collar": True, "shoulders": 1.1, "rim": "#b8b8c8", "hair": "#16161a"}, {"neutral": {}}),
    "man_in_black": ("#6f6f7c", {"feature": "hat", "shoulders": 1.12, "mood": 0.8, "hair": "#101014"}, {"neutral": {}}),
    "shadow": ("#7d7299", {"feature": "hood", "shoulders": 0.98, "rim": "#b4a8e0", "face_light": 0.2, "hair": "#2a2438"}, {"neutral": {}}),
    "shadow_third": ("#7d7299", {"feature": "hood", "shoulders": 0.98, "rim": "#b4a8e0", "face_light": 0.2, "hair": "#2a2438", "ghost": True}, {"neutral": {}}),
    "guest": ("#8a8a8a", {"shoulders": 1.0, "ghost": True, "rim": "#cfcfcf"}, {"neutral": {}}),
    "gabriel": ("#e8c79a", {"scale": 0.68, "head": 1.12, "shoulders": 0.8, "rim": "#fff0c8", "feature": "short_hair", "hair": "#6a4a2a"}, {
        "neutral": {},
        "curious": {"lean": 16, "lower": -6, "face_light": 1.4, "rim_strength": 1.1},
    }),
}


def ui_panel(fill, rule, size=96, inset=6):
    k = 4
    big = size * k
    noise = RNG.normal(0.0, 5.0, (big, big, 1))
    pixels = np.ones((big, big, 3), dtype=np.float32) * np.array(fill, dtype=np.float32) + noise
    image = Image.fromarray(np.clip(pixels, 0, 255).astype(np.uint8), "RGB").convert("RGBA")
    d = ImageDraw.Draw(image)
    a = inset * k
    d.rectangle([a, a, big - a - 1, big - a - 1], outline=rule + (255,), width=2 * k)
    for cx, cy in [(a, a), (big - a, a), (a, big - a), (big - a, big - a)]:
        r = 5 * k
        d.polygon([(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)], fill=rule + (255,))
    return image.resize((size, size), Image.LANCZOS)


def ui_icon(draw_icon, size=64):
    k = 4
    image = Image.new("RGBA", (size * k, size * k), (0, 0, 0, 0))
    draw_icon(ImageDraw.Draw(image), size * k)
    return image.resize((size, size), Image.LANCZOS)


def icon_evidence(d, s):
    w = s // 12
    d.ellipse([s * 0.14, s * 0.12, s * 0.66, s * 0.64], outline="white", width=w)
    d.line([(s * 0.58, s * 0.56), (s * 0.86, s * 0.84)], fill="white", width=int(w * 1.6))


def icon_case_file(d, s):
    w = s // 14
    d.polygon(
        [(s * 0.1, s * 0.24), (s * 0.4, s * 0.24), (s * 0.48, s * 0.32), (s * 0.9, s * 0.32),
         (s * 0.9, s * 0.8), (s * 0.1, s * 0.8)],
        outline="white", width=w,
    )
    d.line([(s * 0.1, s * 0.42), (s * 0.9, s * 0.42)], fill="white", width=w)


def icon_save(d, s):
    w = s // 14
    d.polygon(
        [(s * 0.26, s * 0.1), (s * 0.74, s * 0.1), (s * 0.74, s * 0.9), (s * 0.5, s * 0.68),
         (s * 0.26, s * 0.9)],
        outline="white", width=w,
    )


def icon_log(d, s):
    w = s // 14
    d.rectangle([s * 0.2, s * 0.1, s * 0.8, s * 0.9], outline="white", width=w)
    for y in (0.32, 0.5, 0.68):
        d.line([(s * 0.32, s * y), (s * 0.68, s * y)], fill="white", width=w)


def icon_auto(d, s):
    d.polygon([(s * 0.3, s * 0.18), (s * 0.3, s * 0.82), (s * 0.82, s * 0.5)], fill="white")


def icon_menu(d, s):
    w = s // 11
    for y in (0.28, 0.5, 0.72):
        d.line([(s * 0.16, s * y), (s * 0.84, s * y)], fill="white", width=w)


UI = {
    "choice": lambda: ui_panel(rgb("#1c1820"), rgb("#6e6456")),
    "choice_hover": lambda: ui_panel(rgb("#2a2330"), rgb("#ddc9a4")),
    "icon_evidence": lambda: ui_icon(icon_evidence),
    "icon_case_file": lambda: ui_icon(icon_case_file),
    "icon_save": lambda: ui_icon(icon_save),
    "icon_menu": lambda: ui_icon(icon_menu),
    "icon_log": lambda: ui_icon(icon_log),
    "icon_auto": lambda: ui_icon(icon_auto),
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--assets", default=str(Path(__file__).resolve().parent.parent / "assets"))
    parser.add_argument("--only", nargs="*")
    args = parser.parse_args()
    assets = Path(args.assets)

    for name, paint in BACKGROUNDS.items():
        if args.only and name not in args.only:
            continue
        path = assets / "backgrounds" / f"{name}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        paint().save(path, optimize=True)
        print(f"wrote {path}")

    for person, (color, base, expressions) in PEOPLE.items():
        if args.only and person not in args.only:
            continue
        for expression, overrides in expressions.items():
            path = assets / "characters" / person / f"{expression}.png"
            path.parent.mkdir(parents=True, exist_ok=True)
            portrait(color, {**base, **overrides}).save(path, optimize=True)
            print(f"wrote {path}")

    for name, paint in UI.items():
        if args.only and name not in args.only and "ui" not in args.only:
            continue
        path = assets / "ui" / f"{name}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        paint().save(path, optimize=True)
        print(f"wrote {path}")


if __name__ == "__main__":
    main()
