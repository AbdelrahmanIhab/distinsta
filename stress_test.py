#!/usr/bin/env python3
"""
Load Balancing Stress Test
Tests that requests are distributed across all server nodes
"""

import subprocess
import tempfile
import shutil
import os
import sys
import re
from pathlib import Path

# Configuration
NUM_UPLOADS = 30
USERNAME = "testuser"
TEST_IMAGE = "test_image.png"

def print_header(text):
    """Print a formatted header"""
    print("\n" + "=" * 50)
    print(text)
    print("=" * 50 + "\n")

def create_test_files(test_dir, num_files):
    """Create multiple copies of test image with different names"""
    if not os.path.exists(TEST_IMAGE):
        print(f"Error: {TEST_IMAGE} not found!")
        print("Please ensure test_image.png exists in the current directory")
        sys.exit(1)

    print(f"Creating {num_files} test files in: {test_dir}")

    for i in range(1, num_files + 1):
        shutil.copy(TEST_IMAGE, os.path.join(test_dir, f"image_{i}.png"))

    print(f"✓ Created {num_files} test images\n")

def generate_upload_commands(test_dir, num_files):
    """Generate upload commands for the client"""
    commands = []
    for i in range(1, num_files + 1):
        image_path = os.path.join(test_dir, f"image_{i}.png")
        commands.append(f"upload {image_path}\n")
    commands.append("quit\n")
    return "".join(commands)

def run_stress_test(commands):
    """Run the client with upload commands"""
    print_header("Starting Uploads")

    # Run cargo with the commands
    process = subprocess.Popen(
        ["cargo", "run", "--bin", "client", USERNAME],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )

    output, _ = process.communicate(input=commands)

    return output

def analyze_results(output, num_uploads):
    """Analyze the test results"""
    print_header("Analyzing Results")

    # Count processed requests per node
    node1_processed = len(re.findall(r'Server 1 processed request', output))
    node2_processed = len(re.findall(r'Server 2 processed request', output))
    node3_processed = len(re.findall(r'Server 3 processed request', output))

    # Count declined requests per node
    node1_declined = len(re.findall(r'Server 1 declined:', output))
    node2_declined = len(re.findall(r'Server 2 declined:', output))
    node3_declined = len(re.findall(r'Server 3 declined:', output))

    # Print distribution
    print("Upload Distribution:")
    print("-" * 50)
    print(f"Node 1: {node1_processed} processed, {node1_declined} declined")
    print(f"Node 2: {node2_processed} processed, {node2_declined} declined")
    print(f"Node 3: {node3_processed} processed, {node3_declined} declined")
    print()

    total_processed = node1_processed + node2_processed + node3_processed
    print(f"Total processed: {total_processed} out of {num_uploads} requests\n")

    # Calculate percentages
    if total_processed > 0:
        node1_pct = (node1_processed / total_processed) * 100
        node2_pct = (node2_processed / total_processed) * 100
        node3_pct = (node3_processed / total_processed) * 100

        print("Load Distribution (%):")
        print("-" * 50)
        print(f"Node 1: {node1_pct:5.1f}%  {'█' * int(node1_pct / 2)}")
        print(f"Node 2: {node2_pct:5.1f}%  {'█' * int(node2_pct / 2)}")
        print(f"Node 3: {node3_pct:5.1f}%  {'█' * int(node3_pct / 2)}")
        print()

    return {
        'node1': node1_processed,
        'node2': node2_processed,
        'node3': node3_processed,
        'total': total_processed
    }

def evaluate_load_balancing(results, num_uploads):
    """Evaluate if load balancing is working correctly"""
    print_header("Test Results")

    nodes_used = sum([1 for count in [results['node1'], results['node2'], results['node3']] if count > 0])

    if nodes_used == 3:
        print("✓ PASS: All 3 nodes are processing requests")
        print("✓ Load balancing is working!\n")

        # Check if distribution is balanced (at least 20% per node)
        min_expected = results['total'] / 5  # 20% threshold

        balanced = all([
            results['node1'] >= min_expected,
            results['node2'] >= min_expected,
            results['node3'] >= min_expected
        ])

        if balanced:
            print("✓ Distribution is well-balanced (>20% per node)")

            # Check for very even distribution (<40% per node)
            max_expected = results['total'] * 0.4
            very_balanced = all([
                results['node1'] <= max_expected,
                results['node2'] <= max_expected,
                results['node3'] <= max_expected
            ])

            if very_balanced:
                print("✓ Distribution is excellently balanced (<40% per node)")
        else:
            print("⚠ Distribution is uneven (some nodes <20%)")
            print("  This might be due to hash distribution variance")

    elif nodes_used == 1:
        print("✗ FAIL: Only 1 node is processing requests")
        print("✗ Load balancing is NOT working!")

        active_node = ""
        if results['node1'] > 0:
            active_node = "Node 1"
        elif results['node2'] > 0:
            active_node = "Node 2"
        elif results['node3'] > 0:
            active_node = "Node 3"

        print(f"  All requests going to: {active_node}")
        print("\nPossible issues:")
        print("  - Hash function not distributing properly")
        print("  - get_alive_nodes() returning only one node")
        print("  - Network connectivity issues")

    else:
        print(f"⚠ WARNING: Only {nodes_used} nodes are processing requests")
        print("  Check if all servers are running")

        if results['node1'] == 0:
            print("  - Node 1 is not processing any requests")
        if results['node2'] == 0:
            print("  - Node 2 is not processing any requests")
        if results['node3'] == 0:
            print("  - Node 3 is not processing any requests")

    print()

def main():
    """Main stress test function"""
    print_header("Load Balancing Stress Test")
    print(f"Configuration:")
    print(f"  - Number of uploads: {NUM_UPLOADS}")
    print(f"  - Username: {USERNAME}")
    print(f"  - Test image: {TEST_IMAGE}")
    print()

    # Create temporary directory for test files
    test_dir = tempfile.mkdtemp()

    try:
        # Create test files
        create_test_files(test_dir, NUM_UPLOADS)

        # Generate commands
        print("Generating upload commands...")
        commands = generate_upload_commands(test_dir, NUM_UPLOADS)
        print(f"✓ Generated {NUM_UPLOADS} upload commands\n")

        # Run stress test
        output = run_stress_test(commands)

        # Save output to file for debugging
        with open("stress_test_output.log", "w") as f:
            f.write(output)
        print("✓ Full output saved to: stress_test_output.log\n")

        # Analyze results
        results = analyze_results(output, NUM_UPLOADS)

        # Evaluate load balancing
        evaluate_load_balancing(results, NUM_UPLOADS)

    finally:
        # Cleanup
        print("Cleaning up temporary files...")
        shutil.rmtree(test_dir)
        print("✓ Done!\n")

if __name__ == "__main__":
    main()
