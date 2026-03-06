#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont, ImageFilter
from pathlib import Path
import sys

ROOT = Path(__file__).parent
bird_path = ROOT / 'chant_bird.png'
if not bird_path.exists():
    print('chant_bird.png not found in', ROOT)
    sys.exit(1)

out = ROOT

def load_font(preferred=('DejaVuSans-Bold.ttf','Arial.ttf')):
    for name in preferred:
        try:
            return ImageFont.truetype(name, 72)
        except Exception:
            continue
    return ImageFont.load_default()

font = load_font()

W,H = 900,420
canvas = Image.new('RGBA',(W,H),(0,0,0,0))

# open and resize bird to fit left card area
bird = Image.open(bird_path).convert('RGBA')
card_w,card_h = 536,396
bird = bird.resize((card_w,card_h), Image.LANCZOS)

# create rounded mask
mask = Image.new('L',(card_w,card_h),0)
draw = ImageDraw.Draw(mask)
radius = 20
draw.rounded_rectangle([0,0,card_w,card_h], radius=radius, fill=255)

# drop shadow
shadow = Image.new('RGBA',(card_w,card_h),(0,0,0,0))
shdraw = ImageDraw.Draw(shadow)
shdraw.rectangle([0,0,card_w,card_h], fill=(0,0,0,180))
shadow = shadow.filter(ImageFilter.GaussianBlur(8))
canvas.paste(shadow,(12,18), shadow)

# paste bird with mask
canvas.paste(bird, (12,12), mask)

# white highlight overlay (soft radial)
highlight = Image.new('RGBA',(card_w,card_h),(255,255,255,0))
hd = ImageDraw.Draw(highlight)
for i,alpha in enumerate(range(90,0,-6)):
    bbox = [card_w*0.1 - i*6, card_h*0.05 - i*6, card_w*0.9 + i*6, card_h*0.9 + i*6]
    hd.ellipse(bbox, fill=(255,255,255,alpha))
highlight = highlight.filter(ImageFilter.GaussianBlur(14))
canvas.paste(highlight,(12,12), highlight)

# subtle vignette
vign = Image.new('RGBA',(card_w,card_h),(0,0,0,0))
vd = ImageDraw.Draw(vign)
vd.ellipse([-card_w*0.2,-card_h*0.2, card_w*1.2, card_h*1.2], fill=(0,0,0,40))
vign = vign.filter(ImageFilter.GaussianBlur(30))
canvas.paste(vign,(12,12), vign)

# draw wordmark text on right
draw = ImageDraw.Draw(canvas)
text = 'chant'
font_large = load_font(('DejaVuSans-Bold.ttf',))
font_large_size = 78
try:
    font_large = ImageFont.truetype('DejaVuSans-Bold.ttf', font_large_size)
except Exception:
    font_large = ImageFont.load_default()

tx,ty = 600,170
# layered stroke
draw.text((tx,ty),(text), font=font_large, fill=(255,255,255,255))
draw.text((tx,ty),(text), font=font_large, fill=(15,33,48,255))

# tagline
tag = 'simple • lightweight • friendly'
try:
    tagfont = ImageFont.truetype('DejaVuSans.ttf', 14)
except Exception:
    tagfont = ImageFont.load_default()
draw.text((tx,ty+74), tag, font=tagfont, fill=(74,91,106,255))

canvas.save(out / 'chant_900x420.png')
print('WROTE', out / 'chant_900x420.png')

# helper to scale and pad to square
def scale_to_square(im, size, bg=(0,0,0,0)):
    im_ratio = im.width / im.height
    if im_ratio > 1:
        new_w = size
        new_h = int(size / im_ratio)
    else:
        new_h = size
        new_w = int(size * im_ratio)
    im2 = im.resize((new_w,new_h), Image.LANCZOS)
    outimg = Image.new('RGBA',(size,size), bg)
    outimg.paste(im2, ((size-new_w)//2, (size-new_h)//2), im2)
    return outimg

base = canvas
scale_to_square(base, 64).save(out / 'chant_favicon.png')
scale_to_square(base, 256).save(out / 'chant_256.png')
scale_to_square(base, 1024).save(out / 'chant_1024.png')
print('WROTE PNG sizes: favicon, 256, 1024')
