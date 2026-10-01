import sys
from PIL import Image, ImageDraw

def mask_squircle(img_path, out_path):
    img = Image.open(img_path).convert("RGBA")
    w, h = img.size
    
    # Create a mask with a rounded rectangle
    mask = Image.new('L', (w, h), 0)
    draw = ImageDraw.Draw(mask)
    
    # Apple's corner radius is roughly 22.5% of the width
    radius = int(w * 0.225)
    
    draw.rounded_rectangle((0, 0, w, h), radius=radius, fill=255)
    
    # Apply the mask
    result = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    result.paste(img, (0, 0), mask)
    
    result.save(out_path, "PNG")

if __name__ == "__main__":
    if len(sys.argv) > 2:
        mask_squircle(sys.argv[1], sys.argv[2])
