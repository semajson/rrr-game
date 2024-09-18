from PIL import Image
import os

# Load the image
image = Image.open('Overworld.png')

# Set tile size
tile_width, tile_height = 16, 16

# Get the dimensions of the original image
image_width, image_height = image.size

# Create output directory for the tiles
output_dir = 'tiles'
if not os.path.exists(output_dir):
    os.makedirs(output_dir)

# Initialize tile counter
tile_num = 0

# Loop through the original image and extract each tile
for y in range(0, image_height, tile_height):
    for x in range(0, image_width, tile_width):
        # Define the bounding box for the tile
        bbox = (x, y, x + tile_width, y + tile_height)
        tile = image.crop(bbox)

        # Save each tile to the output directory
        tile.save(os.path.join(output_dir, f'tile_{tile_num}.png'))
        tile_num += 1

print(f"Total tiles created: {tile_num}")
