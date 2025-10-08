#!/usr/bin/env python3
"""
Check which patients are fulfilled in each method and compare by priority.
"""

import ast
import sys
from typing import List, Set, Dict

def parse_data_file(filename: str):
    """Parse the data file to get priorities."""
    with open(filename, 'r') as f:
        lines = f.readlines()

    num_patients = int(lines[0].strip().split(',')[0])
    priorities = list(map(int, lines[3].strip().split(',')))

    # Create dictionary (1-indexed patient IDs)
    patient_priorities = {i+1: priorities[i] for i in range(num_patients)}

    return patient_priorities

def parse_cycles(filename: str) -> List[List[int]]:
    """Parse cycles from output file."""
    cycles = []
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if line:
                cycle = ast.literal_eval(line)
                cycles.append(cycle)
    return cycles

def get_fulfilled_patients(cycles: List[List[int]]) -> Set[int]:
    """Get set of all patients in cycles."""
    fulfilled = set()
    for cycle in cycles:
        fulfilled.update(cycle)
    return fulfilled

def analyze_differences(dfs_file: str, scc_file: str, data_file: str):
    """Analyze which patients are fulfilled in each method."""

    priorities = parse_data_file(data_file)

    dfs_cycles = parse_cycles(dfs_file)
    scc_cycles = parse_cycles(scc_file)

    dfs_fulfilled = get_fulfilled_patients(dfs_cycles)
    scc_fulfilled = get_fulfilled_patients(scc_cycles)

    print(f"DFS fulfilled patients: {len(dfs_fulfilled)}")
    print(f"SCC fulfilled patients: {len(scc_fulfilled)}")
    print(f"Difference: {len(scc_fulfilled) - len(dfs_fulfilled)}")

    # Find patients only in DFS
    only_dfs = dfs_fulfilled - scc_fulfilled
    # Find patients only in SCC
    only_scc = scc_fulfilled - dfs_fulfilled

    print(f"\n{'='*60}")
    print(f"Patients ONLY fulfilled in DFS: {len(only_dfs)}")
    if only_dfs:
        # Sort by priority
        only_dfs_sorted = sorted(only_dfs, key=lambda p: priorities[p])
        print(f"  Lowest priority: {only_dfs_sorted[0]} (priority {priorities[only_dfs_sorted[0]]})")
        print(f"  Highest priority: {only_dfs_sorted[-1]} (priority {priorities[only_dfs_sorted[-1]]})")
        print(f"  Average priority: {sum(priorities[p] for p in only_dfs) / len(only_dfs):.1f}")
        print(f"  First 20 patients (by priority): {only_dfs_sorted[:20]}")

    print(f"\n{'='*60}")
    print(f"Patients ONLY fulfilled in SCC: {len(only_scc)}")
    if only_scc:
        # Sort by priority
        only_scc_sorted = sorted(only_scc, key=lambda p: priorities[p])
        print(f"  Lowest priority: {only_scc_sorted[0]} (priority {priorities[only_scc_sorted[0]]})")
        print(f"  Highest priority: {only_scc_sorted[-1]} (priority {priorities[only_scc_sorted[-1]]})")
        print(f"  Average priority: {sum(priorities[p] for p in only_scc) / len(only_scc):.1f}")
        print(f"  First 20 patients (by priority): {only_scc_sorted[:20]}")

    # Check if lower priority patients are missing in SCC
    print(f"\n{'='*60}")
    print("PRIORITY ANALYSIS:")
    print(f"{'='*60}")

    if only_dfs:
        low_prio_dfs = [p for p in only_dfs if priorities[p] <= 1000]
        print(f"DFS-only patients with priority <= 1000: {len(low_prio_dfs)}")
        if low_prio_dfs:
            low_prio_dfs_sorted = sorted(low_prio_dfs, key=lambda p: priorities[p])
            print(f"  These patients: {low_prio_dfs_sorted[:10]}")

    if only_scc:
        low_prio_scc = [p for p in only_scc if priorities[p] <= 1000]
        print(f"SCC-only patients with priority <= 1000: {len(low_prio_scc)}")
        if low_prio_scc:
            low_prio_scc_sorted = sorted(low_prio_scc, key=lambda p: priorities[p])
            print(f"  These patients: {low_prio_scc_sorted[:10]}")

    # Statistical comparison
    print(f"\n{'='*60}")
    print("OVERALL PRIORITY STATISTICS:")
    print(f"{'='*60}")

    dfs_priorities = [priorities[p] for p in dfs_fulfilled]
    scc_priorities = [priorities[p] for p in scc_fulfilled]

    print(f"DFS fulfilled patients:")
    print(f"  Average priority: {sum(dfs_priorities) / len(dfs_priorities):.1f}")
    print(f"  Median priority: {sorted(dfs_priorities)[len(dfs_priorities)//2]}")
    print(f"  Min priority: {min(dfs_priorities)}")
    print(f"  Max priority: {max(dfs_priorities)}")

    print(f"\nSCC fulfilled patients:")
    print(f"  Average priority: {sum(scc_priorities) / len(scc_priorities):.1f}")
    print(f"  Median priority: {sorted(scc_priorities)[len(scc_priorities)//2]}")
    print(f"  Min priority: {min(scc_priorities)}")
    print(f"  Max priority: {max(scc_priorities)}")

    # Check if SCC is missing low-priority patients that DFS got
    if only_dfs:
        print(f"\n{'='*60}")
        print("⚠️  WARNING: DFS fulfilled some patients that SCC did not!")
        print("This suggests SCC might not be processing in strict priority order")
        print("or is making suboptimal choices.")

def main():
    data_file = "data/test_10000_patient_1000_doctors_5_districts.txt"
    analyze_differences("dfs_cycles.txt", "scc_cycles.txt", data_file)

if __name__ == "__main__":
    main()