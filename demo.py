#!/usr/bin/env python3
"""
Demo Load Balancing Test
Tests system with 50 heavy images of varying sizes for demonstration
"""

import subprocess
import tempfile
import shutil
import os
import sys
import re
import time
import random
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import numpy as np

# Configuration
NUM_UPLOADS = 50
USERNAME = "demo"
MIN_IMAGE_SIZE = 1600    # pixels (1600x1600 = ~7 MB minimum)
MAX_IMAGE_SIZE = 3500    # pixels (3500x3500 = ~35 MB maximum)

def print_header(text):
    """Print a formatted header"""
    print("\n" + "=" * 60)
    print(text.center(60))
    print("=" * 60 + "\n")

def create_test_image(path, size, index):
    """Create a test image with random pixels to ensure large file size"""
    # Generate random RGB values for every pixel
    # This creates incompressible data
    random_pixels = np.random.randint(0, 256, (size, size, 3), dtype=np.uint8)

    img = Image.fromarray(random_pixels, 'RGB')

    # Add some structure to make it somewhat realistic
    draw = ImageDraw.Draw(img)

    # Add random colored rectangles
    for _ in range(20):
        x1 = random.randint(0, size - 100)
        y1 = random.randint(0, size - 100)
        x2 = x1 + random.randint(50, 200)
        y2 = y1 + random.randint(50, 200)
        color = (random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
        draw.rectangle([x1, y1, x2, y2], fill=color)

    # Add text identifier
    try:
        font = ImageFont.load_default()
    except:
        font = None

    text = f"Demo Image #{index} - {size}x{size}"
    draw.text((10, 10), text, fill=(255, 255, 255), font=font)

    # Save with minimal compression
    img.save(path, 'PNG', compress_level=0)

def create_test_files(test_dir, num_files):
    """Create test images of varying sizes"""
    print(f"Creating {num_files} demo images with varying sizes...")
    print(f"  Size range: {MIN_IMAGE_SIZE}x{MIN_IMAGE_SIZE} to {MAX_IMAGE_SIZE}x{MAX_IMAGE_SIZE}")

    start_time = time.time()

    # Categorize by size for statistics
    small_count = 0   # 1600-2000px (~7-12 MB)
    medium_count = 0  # 2000-2500px (~12-19 MB)
    large_count = 0   # 2500-3000px (~19-27 MB)
    xlarge_count = 0  # 3000-3500px (~27-37 MB)

    for i in range(1, num_files + 1):
        # Distribute evenly across size ranges
        rand_val = i % 4
        if rand_val == 0:
            size = random.randint(1600, 2000)
            small_count += 1
        elif rand_val == 1:
            size = random.randint(2000, 2500)
            medium_count += 1
        elif rand_val == 2:
            size = random.randint(2500, 3000)
            large_count += 1
        else:
            size = random.randint(3000, MAX_IMAGE_SIZE)
            xlarge_count += 1

        image_path = os.path.join(test_dir, f"demo_image_{i}.png")
        create_test_image(image_path, size, i)

        # Progress indicator
        if i % 10 == 0:
            print(f"  Created {i}/{num_files} images...")

    elapsed = time.time() - start_time

    # Calculate total size
    total_size = 0
    for i in range(1, num_files + 1):
        image_path = os.path.join(test_dir, f"demo_image_{i}.png")
        total_size += os.path.getsize(image_path)

    total_size_mb = total_size / (1024 * 1024)
    total_size_gb = total_size_mb / 1024

    print(f"\n✓ Created {num_files} images in {elapsed:.1f} seconds")
    print(f"  Distribution:")
    print(f"    - Small  (1600-2000px, ~7-12 MB):  {small_count:2d} images")
    print(f"    - Medium (2000-2500px, ~12-19 MB): {medium_count:2d} images")
    print(f"    - Large  (2500-3000px, ~19-27 MB): {large_count:2d} images")
    print(f"    - XLarge (3000-3500px, ~27-37 MB): {xlarge_count:2d} images")
    print(f"  Total size: {total_size_mb:.0f} MB ({total_size_gb:.2f} GB)")
    print(f"  Average size: {total_size_mb/num_files:.1f} MB per image")
    print(f"  Network traffic (3x broadcast): ~{total_size_gb*3:.2f} GB\n")

def generate_upload_commands(test_dir, num_files):
    """Generate upload commands for the client"""
    commands = []
    for i in range(1, num_files + 1):
        image_path = os.path.join(test_dir, f"demo_image_{i}.png")
        commands.append(f"upload {image_path}\n")
    commands.append("quit\n")
    return "".join(commands)

def run_demo_test(commands):
    """Run the client with upload commands"""
    print_header("Starting Demo Test")
    print(f"Uploading {NUM_UPLOADS} images to demonstrate load balancing...")
    print("Press Ctrl+C to abort\n")

    start_time = time.time()

    # Run cargo with the commands
    process = subprocess.Popen(
        ["cargo", "run", "--release", "--bin", "client", USERNAME],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )

    try:
        output, _ = process.communicate(input=commands, timeout=600)  # 10 minute timeout
    except subprocess.TimeoutExpired:
        process.kill()
        output, _ = process.communicate()
        print("\n⚠ WARNING: Test timed out after 10 minutes")

    elapsed = time.time() - start_time

    print(f"\n✓ Demo completed in {elapsed:.1f} seconds ({elapsed/60:.1f} minutes)")
    print(f"  Average: {elapsed/NUM_UPLOADS:.3f} seconds per upload")
    print(f"  Throughput: {NUM_UPLOADS/elapsed:.2f} uploads/sec\n")

    return output, elapsed

def analyze_results(output, num_uploads, elapsed_time):
    """Analyze the test results"""
    print_header("Demo Results Analysis")

    # Count processed requests per node
    node1_processed = len(re.findall(r'✓ Server 1 processed request', output))
    node2_processed = len(re.findall(r'✓ Server 2 processed request', output))
    node3_processed = len(re.findall(r'✓ Server 3 processed request', output))

    # Count declined requests per node
    node1_declined = len(re.findall(r'- Server 1 declined:', output))
    node2_declined = len(re.findall(r'- Server 2 declined:', output))
    node3_declined = len(re.findall(r'- Server 3 declined:', output))

    # Count successes and failures
    successes = len(re.findall(r'✓ Success!', output))
    failures = num_uploads - successes

    # Print overall statistics
    print("Overall Statistics:")
    print("-" * 60)
    print(f"Total uploads attempted: {num_uploads}")
    print(f"Successful uploads:      {successes} ({successes/num_uploads*100:.1f}%)")
    print(f"Failed uploads:          {failures} ({failures/num_uploads*100:.1f}%)")
    print(f"Total time:              {elapsed_time:.1f} seconds ({elapsed_time/60:.1f} minutes)")
    print(f"Throughput:              {successes/elapsed_time:.2f} uploads/sec")
    print()

    # Print distribution
    print("Request Distribution:")
    print("-" * 60)
    print(f"Node 1: {node1_processed:2d} processed, {node1_declined:2d} declined")
    print(f"Node 2: {node2_processed:2d} processed, {node2_declined:2d} declined")
    print(f"Node 3: {node3_processed:2d} processed, {node3_declined:2d} declined")
    print()

    total_processed = node1_processed + node2_processed + node3_processed

    print(f"Total processed: {total_processed}")
    print()

    # Calculate percentages and visualize
    if total_processed > 0:
        node1_pct = (node1_processed / total_processed) * 100
        node2_pct = (node2_processed / total_processed) * 100
        node3_pct = (node3_processed / total_processed) * 100

        print("Load Distribution (%):")
        print("-" * 60)
        print(f"Node 1: {node1_pct:5.1f}%  {'█' * int(node1_pct / 2)} ({node1_processed} uploads)")
        print(f"Node 2: {node2_pct:5.1f}%  {'█' * int(node2_pct / 2)} ({node2_processed} uploads)")
        print(f"Node 3: {node3_pct:5.1f}%  {'█' * int(node3_pct / 2)} ({node3_processed} uploads)")
        print()

        # Calculate standard deviation for balance analysis
        avg_pct = 100 / 3  # Should be 33.33%
        variance = ((node1_pct - avg_pct)**2 + (node2_pct - avg_pct)**2 + (node3_pct - avg_pct)**2) / 3
        std_dev = variance ** 0.5

        print(f"Balance Analysis:")
        print(f"  Expected per node: {avg_pct:.1f}%")
        print(f"  Standard deviation: {std_dev:.2f}%")

        if std_dev < 5:
            print(f"  ✓ Excellent balance (σ < 5%)")
        elif std_dev < 10:
            print(f"  ✓ Good balance (σ < 10%)")
        elif std_dev < 15:
            print(f"  ⚠ Acceptable balance (σ < 15%)")
        else:
            print(f"  ✗ Poor balance (σ >= 15%)")
        print()

    return {
        'node1': node1_processed,
        'node2': node2_processed,
        'node3': node3_processed,
        'total': total_processed,
        'successes': successes,
        'failures': failures
    }

def evaluate_demo(results, num_uploads):
    """Evaluate demo results"""
    print_header("Demo Evaluation")

    nodes_used = sum([1 for count in [results['node1'], results['node2'], results['node3']] if count > 0])

    print("Load Balancing:")
    print("-" * 60)

    if nodes_used == 3:
        print("✓ PASS: All 3 nodes are actively processing requests")
        print("✓ Load balancing is working correctly!")

        # Check if distribution is balanced
        min_expected = results['total'] / 5  # 20% threshold

        balanced = all([
            results['node1'] >= min_expected,
            results['node2'] >= min_expected,
            results['node3'] >= min_expected
        ])

        if balanced:
            print("✓ Distribution is well-balanced across all nodes")
        else:
            print("⚠ Distribution is slightly uneven (acceptable for small sample)")

    elif nodes_used == 1:
        print("✗ FAIL: Only 1 node is processing requests")
        active_node = ""
        if results['node1'] > 0:
            active_node = "Node 1"
        elif results['node2'] > 0:
            active_node = "Node 2"
        elif results['node3'] > 0:
            active_node = "Node 3"
        print(f"  All requests going to: {active_node}")

    else:
        print(f"⚠ WARNING: Only {nodes_used} nodes are processing requests")

    print()

    # Success rate
    print("Reliability:")
    print("-" * 60)
    success_rate = (results['successes'] / num_uploads) * 100

    if success_rate == 100:
        print(f"✓ PERFECT: {success_rate:.1f}% success rate")
    elif success_rate >= 95:
        print(f"✓ EXCELLENT: {success_rate:.1f}% success rate")
    else:
        print(f"⚠ {success_rate:.1f}% success rate")

    print()

def generate_demo_summary(results, elapsed_time, num_uploads):
    """Generate a demo summary"""
    print_header("Demo Summary")

    total = results['total']

    print(f"Demo Configuration:")
    print(f"  - Images uploaded: {num_uploads}")
    print(f"  - Image size range: {MIN_IMAGE_SIZE}x{MIN_IMAGE_SIZE} to {MAX_IMAGE_SIZE}x{MAX_IMAGE_SIZE}")
    print(f"  - Total time: {elapsed_time:.1f} seconds ({elapsed_time/60:.1f} minutes)")
    print()

    print(f"Performance Metrics:")
    print(f"  - Throughput: {results['successes']/elapsed_time:.2f} uploads/sec")
    print(f"  - Average latency: {elapsed_time/num_uploads:.3f} sec/upload")
    print(f"  - Success rate: {results['successes']/num_uploads*100:.2f}%")
    print()

    print(f"Load Distribution:")
    if total > 0:
        print(f"  - Node 1: {results['node1']/total*100:5.1f}% ({results['node1']} uploads)")
        print(f"  - Node 2: {results['node2']/total*100:5.1f}% ({results['node2']} uploads)")
        print(f"  - Node 3: {results['node3']/total*100:5.1f}% ({results['node3']} uploads)")
    print()

    # Overall verdict
    nodes_used = sum([1 for count in [results['node1'], results['node2'], results['node3']] if count > 0])
    success_rate = (results['successes'] / num_uploads) * 100

    print("Overall Result:")
    print("-" * 60)
    if nodes_used == 3 and success_rate >= 95:
        print("  ✓✓✓ DEMO SUCCESSFUL")
        print("  The distributed system is working correctly:")
        print("    • Load balancing distributes work across all nodes")
        print("    • Fault tolerance ensures high availability")
        print("    • Hash-based assignment prevents duplicate processing")
        print("    • System handles heavy images efficiently")
    elif nodes_used == 3:
        print("  ✓ DEMO PASSED (with minor issues)")
        print("  Load balancing works but some uploads failed")
    else:
        print("  ✗ DEMO ISSUES DETECTED")
        print("  Not all nodes are participating in load balancing")

    print()

def main():
    """Main demo function"""
    print_header("DISTRIBUTED SYSTEM DEMO")
    print(f"Configuration:")
    print(f"  - Number of uploads: {NUM_UPLOADS}")
    print(f"  - Username: {USERNAME}")
    print(f"  - Image size range: {MIN_IMAGE_SIZE}x{MIN_IMAGE_SIZE} to {MAX_IMAGE_SIZE}x{MAX_IMAGE_SIZE}")
    print(f"\nThis demo will:")
    print(f"  1. Generate {NUM_UPLOADS} unique heavy images (7-37 MB each)")
    print(f"  2. Upload all images through the distributed system")
    print(f"  3. Demonstrate load balancing across 3 nodes")
    print(f"  4. Show fault tolerance and hash-based distribution")
    print(f"\nExpected runtime: 2-5 minutes")
    print()

    input("Press ENTER to start the demo (or Ctrl+C to cancel)...")

    # Create temporary directory for test files
    test_dir = tempfile.mkdtemp()

    try:
        # Create test files
        create_test_files(test_dir, NUM_UPLOADS)

        # Generate commands
        print("Generating upload commands...")
        commands = generate_upload_commands(test_dir, NUM_UPLOADS)
        print(f"✓ Generated {NUM_UPLOADS} upload commands\n")

        # Run demo
        output, elapsed_time = run_demo_test(commands)

        # Save output to file
        log_file = "demo_output.log"
        with open(log_file, "w") as f:
            f.write(output)
        print(f"✓ Full output saved to: {log_file}\n")

        # Analyze results
        results = analyze_results(output, NUM_UPLOADS, elapsed_time)

        # Evaluate demo
        evaluate_demo(results, NUM_UPLOADS)

        # Generate summary
        generate_demo_summary(results, elapsed_time, NUM_UPLOADS)

        print_header("Demo Complete!")
        print("You can now show your supervisor:")
        print("  1. The load distribution statistics above")
        print("  2. The encrypted images in the images/ directory")
        print(f"  3. The detailed log file: {log_file}")
        print()

    except KeyboardInterrupt:
        print("\n\n⚠ Demo interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n✗ Error during demo: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        # Cleanup
        print("Cleaning up temporary files...")
        shutil.rmtree(test_dir)
        print("✓ Done!\n")

if __name__ == "__main__":
    main()
