import struct
import math
import os

def create_bmp_raw(width, height, pixel_generator):
    """
    Creates a 24-bit uncompressed BMP file.
    pixel_generator(x, y) returns (R, G, B) where x is 0..width-1, y is 0..height-1.
    Note: BMP pixels are written bottom-to-top, left-to-right in BGR format.
    Row bytes must be padded to multiples of 4.
    """
    row_size = (width * 3 + 3) & ~3
    pixel_data_size = row_size * height
    file_size = 54 + pixel_data_size
    
    # File Header (14 bytes)
    header = struct.pack('<2sIHHI', b'BM', file_size, 0, 0, 54)
    
    # DIB Header (40 bytes)
    dib = struct.pack('<IiiHHIIiiII', 40, width, height, 1, 24, 0, pixel_data_size, 2835, 2835, 0, 0)
    
    # Write pixel data (bottom-to-top)
    pixels = bytearray()
    for y in range(height):
        row = bytearray()
        for x in range(width):
            r, g, b = pixel_generator(x, y)
            row.extend(struct.pack('BBB', int(b), int(g), int(r)))
        # Pad row to 4-byte boundary
        padding = row_size - len(row)
        row.extend(b'\x00' * padding)
        pixels.extend(row)
        
    return header + dib + pixels

def generate_assets():
    icons_dir = os.path.join("src-tauri", "icons")
    os.makedirs(icons_dir, exist_ok=True)
    
    # --- 1. Sidebar Image (164 x 314) ---
    # Dark Resolve-style gradient with an Indigo/Violet accent band and camera logo
    def sidebar_pixels(x, y):
        # Normalize coordinates
        nx = x / 164.0
        ny = y / 314.0
        
        # Base background: dark grey/black gradient
        bg_r = 11 + ny * 15
        bg_g = 12 + ny * 12
        bg_b = 20 + ny * 10
        
        # Glowing vertical/diagonal stripe of indigo (#6366f1 -> 99, 102, 241)
        # Center of stripe varies with height to create a diagonal
        stripe_center = 0.2 + 0.6 * ny
        dist_to_stripe = abs(nx - stripe_center)
        glow = math.exp(- (dist_to_stripe * 8) ** 2)
        
        r = bg_r * (1 - glow) + 99 * glow
        g = bg_g * (1 - glow) + 102 * glow
        b = bg_b * (1 - glow) + 241 * glow
        
        # Simple Camera Icon drawn in the middle (between y=120 and y=180, x=42 and x=122)
        # Camera body box
        if 130 <= y <= 165 and 52 <= x <= 112:
            # Inside the body: draw a lens circle
            cx, cy = 82, 147
            dx, dy = x - cx, y - cy
            dist = math.sqrt(dx*dx + dy*dy)
            if 14 <= dist <= 16: # Outer ring
                r, g, b = 255, 255, 255
            elif dist <= 12: # Lens inner
                # Lens glass reflection
                if dx > 0 and dy > 0:
                    r, g, b = 139, 92, 246 # purple reflection
                else:
                    r, g, b = 30, 30, 35
            else:
                r, g, b = 255, 255, 255
        # Camera flash/shutter button top
        elif 165 < y <= 172 and 62 <= x <= 74:
            r, g, b = 239, 68, 68 # red button
        elif 165 < y <= 172 and 80 <= x <= 102:
            r, g, b = 255, 255, 255 # lens mount/housing top
            
        return min(255, max(0, r)), min(255, max(0, g)), min(255, max(0, b))

    # --- 2. Header Image (150 x 57) ---
    # Dark grey background with purple glow on the right
    def header_pixels(x, y):
        nx = x / 150.0
        ny = y / 57.0
        
        # Background: dark metallic gray
        bg_r = 18 + ny * 6
        bg_g = 18 + ny * 6
        bg_b = 20 + ny * 8
        
        # Glow in the top right corner (#a855f7 -> 168, 85, 247)
        dist_to_corner = math.sqrt((nx - 1.0)**2 + (ny - 1.0)**2)
        glow = math.exp(- (dist_to_corner * 2.5) ** 2) * 0.4
        
        r = bg_r * (1 - glow) + 168 * glow
        g = bg_g * (1 - glow) + 85 * glow
        b = bg_b * (1 - glow) + 247 * glow
        
        # Miniature Camera Icon on the left
        if 18 <= y <= 38 and 15 <= x <= 45:
            cx, cy = 30, 28
            dx, dy = x - cx, y - cy
            dist = math.sqrt(dx*dx + dy*dy)
            if 6 <= dist <= 8:
                r, g, b = 255, 255, 255
            elif dist <= 5:
                r, g, b = 99, 102, 241
            else:
                r, g, b = 255, 255, 255
        elif 38 < y <= 42 and 22 <= x <= 28:
            r, g, b = 239, 68, 68
        elif 38 < y <= 42 and 32 <= x <= 40:
            r, g, b = 255, 255, 255
            
        return min(255, max(0, r)), min(255, max(0, g)), min(255, max(0, b))
        
    sidebar_data = create_bmp_raw(164, 314, sidebar_pixels)
    header_data = create_bmp_raw(150, 57, header_pixels)
    
    with open(os.path.join(icons_dir, "nsis-sidebar.bmp"), "wb") as f:
        f.write(sidebar_data)
    with open(os.path.join(icons_dir, "nsis-header.bmp"), "wb") as f:
        f.write(header_data)
        
    print("NSIS sidebar and header BMP images generated successfully!")

if __name__ == "__main__":
    generate_assets()
