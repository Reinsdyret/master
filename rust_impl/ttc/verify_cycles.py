#!/usr/bin/env python3
"""
Verify that cycles are valid trading cycles.
For each cycle, check that:
1. Each patient ends up with their preferred doctor
2. The cycle is actually executable (forms a valid trading ring)
"""

import ast
import sys
from typing import List, Dict, Tuple

def parse_data_file(filename: str) -> Tuple[Dict[int, int], Dict[int, int]]:
    """
    Parse the data file to get patient preferences and current assignments.
    Returns: (preferred_doctors, current_doctors) dictionaries with patient_id as key
    """
    with open(filename, 'r') as f:
        lines = f.readlines()

    num_patients, num_doctors = map(int, lines[0].strip().split(','))
    preferred = list(map(int, lines[1].strip().split(',')))
    current = list(map(int, lines[2].strip().split(',')))

    # Create dictionaries (1-indexed patient IDs)
    preferred_doctors = {i+1: preferred[i] for i in range(num_patients)}
    current_doctors = {i+1: current[i] for i in range(num_patients)}

    return preferred_doctors, current_doctors

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

def verify_cycle(cycle: List[int], preferred: Dict[int, int], current: Dict[int, int]) -> Tuple[bool, str]:
    """
    Verify a single cycle is valid.
    In a TTC cycle, each patient i gets the current doctor of patient i+1 (mod cycle length).
    After execution, each patient should have their preferred doctor.
    """
    if len(cycle) < 2:
        # Single patient cycle - patient should already have preferred doctor
        if len(cycle) == 1:
            patient = cycle[0]
            if current[patient] == preferred[patient]:
                return True, f"Valid 1-person cycle (patient {patient} already has preferred doctor)"
            else:
                return False, f"Invalid 1-person cycle: patient {patient} wants doctor {preferred[patient]} but has {current[patient]}"
        return False, "Empty cycle"

    # Check that cycle forms a valid trading ring
    # Each patient should want the doctor that the next patient currently has
    for i in range(len(cycle)):
        current_patient = cycle[i]
        next_patient = cycle[(i + 1) % len(cycle)]

        # After trading, current_patient will get next_patient's current doctor
        new_doctor = current[next_patient]
        wanted_doctor = preferred[current_patient]

        if new_doctor != wanted_doctor:
            return False, f"Invalid cycle at position {i}: patient {current_patient} wants doctor {wanted_doctor} but would get doctor {new_doctor} from patient {next_patient}"

    return True, "Valid cycle"

def verify_all_cycles(cycle_file: str, data_file: str, method_name: str):
    """Verify all cycles in a file."""
    print(f"\n{'='*60}")
    print(f"Verifying {method_name} cycles")
    print(f"{'='*60}")

    preferred, current = parse_data_file(data_file)
    cycles = parse_cycles(cycle_file)

    print(f"Total cycles to verify: {len(cycles)}")

    valid_count = 0
    invalid_count = 0
    invalid_cycles = []

    for idx, cycle in enumerate(cycles):
        is_valid, message = verify_cycle(cycle, preferred, current)

        if is_valid:
            valid_count += 1
        else:
            invalid_count += 1
            invalid_cycles.append((idx, cycle, message))
            print(f"\n❌ Cycle #{idx + 1} INVALID:")
            print(f"   Cycle: {cycle[:10]}{'...' if len(cycle) > 10 else ''}")
            print(f"   Reason: {message}")

    print(f"\n{'='*60}")
    print(f"Results for {method_name}:")
    print(f"  ✅ Valid cycles: {valid_count}/{len(cycles)} ({100*valid_count/len(cycles):.1f}%)")
    print(f"  ❌ Invalid cycles: {invalid_count}/{len(cycles)} ({100*invalid_count/len(cycles):.1f}%)")

    if invalid_count > 0:
        print(f"\nFirst few invalid cycles:")
        for idx, cycle, message in invalid_cycles[:5]:
            print(f"  Cycle #{idx + 1}: {cycle[:5]}... - {message}")

    return valid_count, invalid_count

def main():
    data_file = "data/test_10000_patient_1000_doctors_5_districts.txt"

    # Verify DFS cycles
    dfs_valid, dfs_invalid = verify_all_cycles("dfs_cycles.txt", data_file, "DFS")

    # Verify SCC cycles
    scc_valid, scc_invalid = verify_all_cycles("scc_cycles.txt", data_file, "SCC")

    print(f"\n{'='*60}")
    print("SUMMARY:")
    print(f"{'='*60}")
    print(f"DFS: {dfs_valid} valid, {dfs_invalid} invalid")
    print(f"SCC: {scc_valid} valid, {scc_invalid} invalid")

    if dfs_invalid == 0 and scc_invalid == 0:
        print("\n✅ Both methods produce only valid cycles!")
        print("The different cycles found are due to different traversal strategies,")
        print("but both are correct implementations.")
    elif dfs_invalid > 0 or scc_invalid > 0:
        print("\n❌ Found invalid cycles - there is a bug!")
        return 1

    return 0

if __name__ == "__main__":
    sys.exit(main())