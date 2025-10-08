#!/usr/bin/env python3
"""
Script to compare cycle outputs from DFS and SCC methods.
This script reads the cycles from dfs_cycles.txt and scc_cycles.txt
and checks if they contain the same cycles (regardless of order).
"""

import ast
import sys
from typing import List, Set, Tuple

def parse_cycle_file(filename: str) -> List[List[int]]:
    """Parse a cycle file and return list of cycles."""
    cycles = []
    try:
        with open(filename, 'r') as f:
            for line_num, line in enumerate(f, 1):
                line = line.strip()
                if not line:
                    continue
                try:
                    # Parse the cycle (format: [1, 2, 3, 4])
                    cycle = ast.literal_eval(line)
                    if isinstance(cycle, list):
                        cycles.append(cycle)
                    else:
                        print(f"Warning: Invalid cycle format on line {line_num} of {filename}: {line}")
                except (ValueError, SyntaxError) as e:
                    print(f"Warning: Could not parse line {line_num} of {filename}: {line} ({e})")
    except FileNotFoundError:
        print(f"Error: File {filename} not found!")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading {filename}: {e}")
        sys.exit(1)

    return cycles

def normalize_cycle(cycle: List[int]) -> Tuple[int, ...]:
    """
    Normalize a cycle by rotating it so the smallest element comes first.
    This allows us to compare cycles regardless of their starting point.
    """
    if not cycle:
        return tuple()

    min_idx = cycle.index(min(cycle))
    normalized = cycle[min_idx:] + cycle[:min_idx]
    return tuple(normalized)

def cycles_to_set(cycles: List[List[int]]) -> Tuple[Set[Tuple[int, ...]], Set[Tuple[int, ...]]]:
    """Convert list of cycles to set of normalized cycles for comparison."""
    normalized_cycles = set()
    raw_cycles = set()

    for cycle in cycles:
        if len(cycle) > 0:
            normalized = normalize_cycle(cycle)
            normalized_cycles.add(normalized)
            raw_cycles.add(tuple(sorted(cycle)))  # Also keep sorted version for additional comparison

    return normalized_cycles, raw_cycles

def compare_cycles(dfs_cycles: List[List[int]], scc_cycles: List[List[int]]) -> bool:
    """Compare two lists of cycles and return True if they're equivalent."""

    print(f"DFS found {len(dfs_cycles)} cycles")
    print(f"SCC found {len(scc_cycles)} cycles")

    if len(dfs_cycles) != len(scc_cycles):
        print(f"❌ Different number of cycles: DFS={len(dfs_cycles)}, SCC={len(scc_cycles)}")
        # return False

    # Normalize cycles for comparison
    dfs_normalized, dfs_raw = cycles_to_set(dfs_cycles)
    scc_normalized, scc_raw = cycles_to_set(scc_cycles)

    print(f"DFS unique normalized cycles: {len(dfs_normalized)}")
    print(f"SCC unique normalized cycles: {len(scc_normalized)}")

    # Check if normalized cycles are the same
    if dfs_normalized == scc_normalized:
        print("✅ Cycles match perfectly (normalized comparison)!")
        return True

    # If normalized comparison fails, try raw sorted comparison
    if dfs_raw == scc_raw:
        print("✅ Cycles match (sorted comparison)!")
        return True

    # Find differences
    only_in_dfs = dfs_normalized - scc_normalized
    only_in_scc = scc_normalized - dfs_normalized

    print("❌ Cycles do not match!")

    if only_in_dfs:
        print(f"Cycles only in DFS ({len(only_in_dfs)}):")
        for cycle in sorted(only_in_dfs)[:5]:  # Show first 5
            print(f"  {list(cycle)}")
        if len(only_in_dfs) > 5:
            print(f"  ... and {len(only_in_dfs) - 5} more")

    if only_in_scc:
        print(f"Cycles only in SCC ({len(only_in_scc)}):")
        for cycle in sorted(only_in_scc)[:5]:  # Show first 5
            print(f"  {list(cycle)}")
        if len(only_in_scc) > 5:
            print(f"  ... and {len(only_in_scc) - 5} more")

    return False

def analyze_cycle_stats(cycles: List[List[int]], method_name: str):
    """Print statistics about the cycles."""
    if not cycles:
        print(f"{method_name}: No cycles found")
        return

    cycle_lengths = [len(cycle) for cycle in cycles]
    total_patients = sum(cycle_lengths)

    print(f"\n{method_name} Statistics:")
    print(f"  Total cycles: {len(cycles)}")
    print(f"  Total patients in cycles: {total_patients}")
    print(f"  Average cycle length: {total_patients / len(cycles):.2f}")
    print(f"  Cycle length range: {min(cycle_lengths)} - {max(cycle_lengths)}")

    # Show cycle length distribution
    from collections import Counter
    length_dist = Counter(cycle_lengths)
    print(f"  Cycle length distribution:")
    for length in sorted(length_dist.keys())[:10]:  # Show first 10
        print(f"    Length {length}: {length_dist[length]} cycles")
    if len(length_dist) > 10:
        print(f"    ... and {len(length_dist) - 10} more lengths")

def main():
    """Main function to compare cycle files."""
    print("🔍 Comparing cycles from DFS and SCC methods...")
    print("=" * 60)

    # Parse both files
    dfs_cycles = parse_cycle_file("dfs_cycles.txt")
    scc_cycles = parse_cycle_file("scc_cycles.txt")

    # Analyze statistics
    analyze_cycle_stats(dfs_cycles, "DFS")
    analyze_cycle_stats(scc_cycles, "SCC")

    print("\n" + "=" * 60)
    print("🔍 Cycle Comparison Results:")
    print("=" * 60)

    # Compare cycles
    cycles_match = compare_cycles(dfs_cycles, scc_cycles)

    print("\n" + "=" * 60)
    if cycles_match:
        print("✅ CONCLUSION: Both methods found the same cycles!")
        print("   The DFS and SCC implementations are equivalent.")
    else:
        print("❌ CONCLUSION: The methods found different cycles!")
        print("   There may be a bug in one of the implementations.")
    print("=" * 60)

    return 0 if cycles_match else 1

if __name__ == "__main__":
    sys.exit(main())