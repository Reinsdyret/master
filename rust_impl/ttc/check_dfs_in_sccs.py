#!/usr/bin/env python3
"""
Check if DFS cycles are contained within the initial SCCs found by the SCC algorithm.
"""

import ast

def parse_cycles(filename):
    """Parse cycles/SCCs from file."""
    items = []
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if line:
                item = ast.literal_eval(line)
                items.append(set(item))
    return items

def main():
    print("Checking if DFS cycles are within initial SCCs...")
    print("="*60)

    dfs_cycles = parse_cycles("dfs_cycles.txt")
    initial_sccs = parse_cycles("sccs_first_iteration.txt")

    print(f"DFS found {len(dfs_cycles)} cycles")
    print(f"Initial SCCs: {len(initial_sccs)} components")
    print(f"  SCC sizes: min={min(len(s) for s in initial_sccs)}, max={max(len(s) for s in initial_sccs)}, avg={sum(len(s) for s in initial_sccs)/len(initial_sccs):.1f}")

    # Check each DFS cycle
    cycles_fully_in_scc = 0
    cycles_split_across_sccs = 0
    cycles_not_in_sccs = 0

    problematic_cycles = []

    for idx, dfs_cycle in enumerate(dfs_cycles):
        # Check if entire cycle is within one SCC
        found_in_one = False
        for scc in initial_sccs:
            if dfs_cycle.issubset(scc):
                cycles_fully_in_scc += 1
                found_in_one = True
                break

        if not found_in_one:
            # Check if cycle spans multiple SCCs
            patients_in_sccs = set()
            overlapping_sccs = []
            for scc_idx, scc in enumerate(initial_sccs):
                overlap = dfs_cycle & scc
                if overlap:
                    patients_in_sccs |= overlap
                    overlapping_sccs.append((scc_idx, len(overlap), len(scc)))

            if dfs_cycle.issubset(patients_in_sccs):
                cycles_split_across_sccs += 1
                print(f"\n⚠️  DFS cycle #{idx+1} ({len(dfs_cycle)} patients) spans {len(overlapping_sccs)} SCCs")
                for scc_idx, overlap_size, scc_size in overlapping_sccs[:3]:
                    print(f"    SCC #{scc_idx}: {overlap_size}/{scc_size} patients from this cycle")
            else:
                cycles_not_in_sccs += 1
                missing = dfs_cycle - patients_in_sccs
                problematic_cycles.append((idx, len(dfs_cycle), len(missing), sorted(list(dfs_cycle))[:5]))

    print(f"\n{'='*60}")
    print("RESULTS:")
    print(f"  ✅ Cycles fully within one SCC: {cycles_fully_in_scc}/{len(dfs_cycles)}")
    print(f"  ⚠️  Cycles split across multiple SCCs: {cycles_split_across_sccs}/{len(dfs_cycles)}")
    print(f"  ❌ Cycles with patients NOT in any SCC: {cycles_not_in_sccs}/{len(dfs_cycles)}")

    if problematic_cycles:
        print(f"\n❌ PROBLEM: {len(problematic_cycles)} DFS cycles have patients not in ANY initial SCC!")
        print("This means the SCC algorithm is missing some patients that can form cycles.")
        print("\nFirst few problematic cycles:")
        for idx, total, missing, first_patients in problematic_cycles[:5]:
            print(f"  Cycle #{idx+1}: {total} patients, {missing} not in any SCC")
            print(f"    First patients: {first_patients}")

    if cycles_split_across_sccs > 0:
        print(f"\n⚠️  WARNING: {cycles_split_across_sccs} cycles span multiple SCCs!")
        print("This suggests DFS finds cycles that cross SCC boundaries,")
        print("which shouldn't be possible if SCCs are computed correctly.")

if __name__ == "__main__":
    main()