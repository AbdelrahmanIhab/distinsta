#!/usr/bin/env python3
"""
Simple script to create a test image for the distributed system.
"""

try:
    from PIL import Image, ImageDraw, ImageFont
    import sys

    # Create a 800x600 image with a blue background
    img = Image.new('RGB', (800, 600), color='blue')
    draw = ImageDraw.Draw(img)

    # Draw some text on it
    text = "Test Image for Distributed System"
    # Use default font
    draw.text((200, 280), text, fill='white')

    # Draw a rectangle
    draw.rectangle([150, 150, 650, 450], outline='yellow', width=3)

    # Save the image
    img.save('test_image.png')
    print("Test image created successfully: test_image.png")

except ImportError:
    print("PIL/Pillow not installed. Creating a simple alternate method...")
    print("You can manually add any PNG image as test_image.png")
    sys.exit(1)
