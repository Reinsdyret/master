#!/usr/bin/env python3
"""
Parse a TTC data file, enumerate all simple cycles using NetworkX,
then solve a maximum-weight vertex-disjoint cycle packing ILP with PuLP.
"""

import sys
import time
from collections import defaultdict

import networkx as nx
import pulp


def parse_data_file(filepath):
    with open(filepath, "r") as f:
        lines = f.read().strip().split("\n")

    num_patients, num_doctors = map(int, lines[0].split(","))
    preferred_doctors = list(map(int, lines[1].split(",")))

    current_doctors = []
    for s in lines[2].split(","):
        s = s.strip()
        current_doctors.append(int(s) if s else None)

    priorities = list(map(int, lines[3].split(",")))

    capacities = None
    if len(lines) >= 6:
        capacities = list(map(int, lines[5].split(",")))

    patients = []
    for i in range(num_patients):
        pid = i + 1
        pref = preferred_doctors[i]
        curr = current_doctors[i]
        wants_to_switch = (curr is None) or (pref != curr)
        patients.append(
            {
                "id": pid,
                "is_dummy": False,
                "priority": priorities[i],
                "preferred_doctor": pref,
                "current_doctor": curr,
                "wants_to_switch": wants_to_switch,
            }
        )

    doctors = {}

    # Dummy doctor 0 for unassigned patients
    doctors[0] = {
        "id": 0,
        "is_dummy": True,
        "capacity": sum(1 for d in current_doctors if d == 0),
        "assigned_patients": [p["id"] for p in patients if p["current_doctor"] == 0],
    }

    next_dummy_id = num_patients + 1

    for d in range(1, num_doctors + 1):
        if capacities:
            cap = capacities[d - 1]
        else:
            cap = sum(1 for p in patients if p["current_doctor"] == d)

        assigned = [p["id"] for p in patients if p["current_doctor"] == d]
        available = max(0, cap - len(assigned))

        for _ in range(available):
            patients.append(
                {
                    "id": next_dummy_id,
                    "is_dummy": True,
                    "priority": float("inf"),
                    "preferred_doctor": 0,
                    "current_doctor": d,
                    "wants_to_switch": True,
                }
            )
            assigned.append(next_dummy_id)
            next_dummy_id += 1

        doctors[d] = {
            "id": d,
            "is_dummy": False,
            "capacity": cap,
            "assigned_patients": assigned,
        }

    return patients, doctors


def build_graph(patients):
    """
    Directed graph: edge from A to B iff
      A wants to switch AND A.preferred_doctor == B.current_doctor.

    In the TTC setting each patient points to all patients currently at
    their preferred doctor — those are the people whose slot they could
    "trade into".
    """
    G = nx.DiGraph()

    patients_at_doctor = defaultdict(list)
    for p in patients:
        if p["current_doctor"] is not None:
            patients_at_doctor[p["current_doctor"]].append(p["id"])

    edge_count = 0
    for p in patients:
        if not p["wants_to_switch"]:
            continue
        pref = p["preferred_doctor"]
        for target_id in patients_at_doctor.get(pref, []):
            if target_id != p["id"]:
                G.add_edge(p["id"], target_id)
                edge_count += 1

    print(f"  Built {edge_count} edges")
    return G


def solve_ilp(cycles, patients):
    """
    Maximum vertex-disjoint cycle packing ILP.

    Maximise the total number of non-dummy vertices covered by the
    selected cycles, subject to every vertex appearing in at most one
    chosen cycle.

        max  sum_c  w_c * x_c
        s.t. sum_{c : v in c}  x_c  <= 1   for every vertex v
             x_c in {0, 1}
    """
    if not cycles:
        print("No cycles to optimise.")
        return []

    dummy_ids = {p["id"] for p in patients if p["is_dummy"]}

    # weight = number of non-dummy vertices in the cycle
    weights = []
    for c in cycles:
        w = sum(1 for v in c if v not in dummy_ids)
        weights.append(w)

    prob = pulp.LpProblem("MaxVertexDisjointCycles", pulp.LpMaximize)

    x = [pulp.LpVariable(f"x_{i}", cat=pulp.LpBinary) for i in range(len(cycles))]

    # objective
    prob += pulp.lpSum(weights[i] * x[i] for i in range(len(cycles)))

    # vertex-disjoint constraints
    vertex_to_cycles = defaultdict(list)
    for i, c in enumerate(cycles):
        for v in c:
            vertex_to_cycles[v].append(i)

    for v, cycle_indices in vertex_to_cycles.items():
        prob += pulp.lpSum(x[i] for i in cycle_indices) <= 1

    print("  Solving ILP...")
    t0 = time.time()
    prob.solve(pulp.PULP_CBC_CMD(msg=1))
    elapsed = time.time() - t0
    print(f"  Solver finished in {elapsed:.2f}s  —  status: {pulp.LpStatus[prob.status]}")

    selected = []
    total_vertices = 0
    for i in range(len(cycles)):
        if pulp.value(x[i]) > 0.5:
            selected.append(cycles[i])
            total_vertices += weights[i]

    print(f"  Selected {len(selected)} cycles covering {total_vertices} non-dummy vertices")
    return selected


def main():
    if len(sys.argv) < 2:
        print("Usage: python find_cycles.py <data_file> [output_file] [--max-length N]")
        sys.exit(1)

    filepath = sys.argv[1]
    output_file = None
    max_length = None

    args = sys.argv[2:]
    i = 0
    while i < len(args):
        if args[i] == "--max-length" and i + 1 < len(args):
            max_length = int(args[i + 1])
            i += 2
        else:
            output_file = args[i]
            i += 1

    t_start = time.time()
    print(f"Parsing {filepath}...")
    patients, doctors = parse_data_file(filepath)
    print(f"  Parsed in {time.time() - t_start:.2f}s")
    switching = sum(1 for p in patients if p["wants_to_switch"])
    print(f"  {len(patients)} patients (incl. dummies), {len(doctors)} doctors")
    print(f"  {switching} patients want to switch")

    print("Building directed graph...")
    G = build_graph(patients)
    print(f"  {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

    print("Finding all simple cycles...")
    t0 = time.time()
    if max_length is not None:
        print(f"  (bounded to length <= {max_length})")
        cycles = list(nx.simple_cycles(G, length_bound=max_length))
    else:
        cycles = list(nx.simple_cycles(G))
    elapsed = time.time() - t0
    print(f"  Found {len(cycles)} cycles in {elapsed:.2f}s")

    # --- ILP: pick vertex-disjoint cycles maximising total vertices ---
    selected = solve_ilp(cycles, patients)

    # Close cycles for output: [A, B, C] -> [A, B, C, A]
    formatted = [c + [c[0]] for c in selected]

    print(f"\n=== Result: {len(formatted)} cycles ===")
    for cycle in formatted[:30]:
        print("  " + " -> ".join(map(str, cycle)))
    if len(formatted) > 30:
        print(f"  ... ({len(formatted) - 30} more)")

    if output_file:
        with open(output_file, "w") as f:
            for cycle in formatted:
                f.write(",".join(map(str, cycle)) + "\n")
        print(f"\nWrote {len(formatted)} cycles to {output_file}")


if __name__ == "__main__":
    main()
