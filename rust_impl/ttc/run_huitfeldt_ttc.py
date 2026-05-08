"""
Run one iteration of the Huitfeldt et al. TTC on one of our datasets.

Usage:
    python3 run_huitfeldt_ttc.py [data_file]

If no file is given, uses the small 50-patient test dataset.
"""

import sys
import os
import time

# ---------------------------------------------------------------------------
# TTC logic -- inlined from Huitfeldt et al. f_ttc_module.py / f_simulation_utils.py
# ---------------------------------------------------------------------------

def current_panel(PAT_ID, panels):
    for gp, panel in panels.items():
        if PAT_ID in panel:
            return gp
    raise RuntimeError(f"Patient {PAT_ID} not found in any panel")


def find_cycle(starting_patient, available_agents, prefs):
    prefs_avail = {}
    chain = [starting_patient]
    while True:
        last_agent = chain[-1]
        prefs_avail[last_agent] = [a for a in prefs[last_agent] if a in available_agents]
        top_choice = prefs_avail[last_agent][0]
        if top_choice in chain:
            break
        else:
            chain.append(top_choice)

    cycle = [chain[-1]]
    while True:
        last_agent = cycle[-1]
        top_choice = prefs_avail[last_agent][0]
        if cycle[0] == top_choice:
            break
        else:
            cycle.append(top_choice)

    assert cycle[0] == prefs_avail[cycle[-1]][0]
    return cycle


def run_ttc(panels, preferences, priority_list):
    assert set(priority_list) == set(preferences.keys())
    for pat in preferences:
        assert preferences[pat][-1] == current_panel(pat, panels), \
            f"Patient {pat}: current GP not ranked last (last={preferences[pat][-1]}, panel={current_panel(pat, panels)})"

    panels_TTC      = {'g{0}'.format(g): {'p{0}'.format(p) for p in panel}    for g, panel    in panels.items()}
    preferences_TTC = {'p{0}'.format(p): ['g{0}'.format(g) for g in preflist] for p, preflist in preferences.items()}
    priority_list_TTC = ['p{0}'.format(p) for p in priority_list]

    pop_size = sum(len(panels_TTC[gp]) for gp in panels_TTC)

    waiters = set(priority_list_TTC)
    panels_TTC = {gp: panel & waiters for gp, panel in panels_TTC.items()}

    gps_zero_capacity = {gp for gp, panel in panels_TTC.items() if len(panel) == 0}
    for gp in gps_zero_capacity:
        del panels_TTC[gp]
    preferences_TTC = {pat: [gp for gp in preflist if gp not in gps_zero_capacity]
                       for pat, preflist in preferences_TTC.items()}

    waiters_for_zero_cap = {pat for pat in preferences_TTC if len(preferences_TTC[pat]) == 1}
    for pat in waiters_for_zero_cap:
        del preferences_TTC[pat]
    panels_TTC = {gp: panel - waiters_for_zero_cap for gp, panel in panels_TTC.items()}

    while len(gps_zero_capacity) > 0:
        gps_zero_capacity = {g for g, panel in panels_TTC.items() if len(panel) == 0}
        for gp in gps_zero_capacity:
            del panels_TTC[gp]
        preferences_TTC = {pat: [gp for gp in preflist if gp not in gps_zero_capacity]
                           for pat, preflist in preferences_TTC.items()}

        waiters_for_zero_cap = {pat for pat in preferences_TTC if len(preferences_TTC[pat]) == 1}
        for pat in waiters_for_zero_cap:
            del preferences_TTC[pat]
        panels_TTC = {gp: panel - waiters_for_zero_cap for gp, panel in panels_TTC.items()}

    pats_assigned_through_ttc = set()

    if len(panels_TTC) > 0:
        patients = set().union(*panels_TTC.values())
        gps      = set(panels_TTC.keys())

        capacity = {g: len(panels_TTC[g]) for g in gps}
        assert min(capacity.values()) > 0

        prefs = dict(preferences_TTC)
        priority_list_TTC = [p for p in priority_list_TTC if p in patients]

        for gp, panel in panels_TTC.items():
            curr_pats = sorted(list(panel))
            wl_pats   = [pat for pat in priority_list_TTC if gp in prefs[pat]]
            prefs[gp] = curr_pats + wl_pats

        available_agents  = patients | gps
        starting_patient  = priority_list_TTC.pop(0)

        while len(available_agents) > 0:
            while starting_patient not in available_agents:
                starting_patient = priority_list_TTC.pop(0)

            if set(prefs[starting_patient][:-1]).isdisjoint(available_agents):
                gp_curr = prefs[starting_patient][-1]
                capacity[gp_curr] -= 1
                if capacity[gp_curr] == 0:
                    available_agents.remove(gp_curr)
                available_agents.remove(starting_patient)
            else:
                cycle = find_cycle(starting_patient, available_agents, prefs)

                for j in range(len(cycle)):
                    agent = cycle[j]
                    if agent in gps:
                        capacity[agent] -= 1
                        if capacity[agent] == 0:
                            available_agents.remove(agent)
                        available_agents.remove(cycle[j - 1])

                        if len(cycle) > 2:
                            gp       = int(agent[1:])
                            pat_prev = int(cycle[j - 1][1:])
                            if j == len(cycle) - 1:
                                pat_next = int(cycle[0][1:])
                            else:
                                pat_next = int(cycle[j + 1][1:])

                            panels[gp].add(pat_prev)
                            panels[gp].remove(pat_next)

                            del preferences[pat_prev]
                            priority_list.remove(pat_prev)
                            pats_assigned_through_ttc.add(pat_prev)

            assert min(capacity.values()) >= 0

    assert pop_size == sum(len(panels[g]) for g in panels)
    assert len(waiters) - len(pats_assigned_through_ttc) == len(priority_list)
    assert len(waiters) - len(pats_assigned_through_ttc) == len(preferences)
    assert set(priority_list) == set(preferences.keys())

    return pats_assigned_through_ttc


# ---------------------------------------------------------------------------
# Data loader for our format
# ---------------------------------------------------------------------------

def load_dataset(path):
    with open(path) as f:
        lines = [l.strip() for l in f if l.strip()]

    num_patients, num_doctors = map(int, lines[0].split(','))
    preferred = list(map(int, lines[1].split(',')))   # preferred[i] = preferred doctor of patient i+1
    current   = list(map(int, lines[2].split(',')))   # current[i]   = current  doctor of patient i+1 (0=unassigned)
    priority  = list(map(int, lines[3].split(',')))   # priority[i]  = priority value of patient i+1 (lower=higher)

    assert len(preferred) == num_patients
    assert len(current)   == num_patients
    assert len(priority)  == num_patients

    return num_patients, num_doctors, preferred, current, priority


def build_ttc_inputs(num_patients, preferred, current, priority):
    """
    Build panels, preferences, priority_list for run_ttc.

    Only patients with current_doctor > 0 go into panels.
    Only patients who want to switch (preferred != current, current > 0) go into preferences/priority_list.
    Preference list per patient: [preferred_doctor, current_doctor]  (current last, as required).
    """
    # panels: gp_id (int) -> set of pat_ids (int, 1-indexed)
    panels = {}
    for i in range(num_patients):
        pat_id  = i + 1
        curr_gp = current[i]
        if curr_gp == 0:
            continue  # unassigned patients skipped
        panels.setdefault(curr_gp, set()).add(pat_id)

    # preferences & priority_list: only patients who want to switch
    wants_switch = []
    for i in range(num_patients):
        pat_id   = i + 1
        pref_gp  = preferred[i]
        curr_gp  = current[i]
        if curr_gp == 0:
            continue
        if pref_gp == curr_gp:
            continue
        wants_switch.append((priority[i], pat_id))

    wants_switch.sort()  # ascending priority value = highest priority first
    priority_list = [pat_id for _, pat_id in wants_switch]

    preferences = {}
    for _, pat_id in wants_switch:
        i = pat_id - 1
        preferences[pat_id] = [preferred[i], current[i]]

    return panels, preferences, priority_list


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    data_dir = os.path.join(os.path.dirname(__file__), 'data')
    default  = os.path.join(data_dir, 'test_50_patient_5_doctors_2_districts_0.1_prob.txt')
    path     = sys.argv[1] if len(sys.argv) > 1 else default

    print(f"Dataset : {os.path.basename(path)}")

    num_patients, num_doctors, preferred, current, priority = load_dataset(path)
    print(f"Patients: {num_patients}  Doctors: {num_doctors}")

    panels, preferences, priority_list = build_ttc_inputs(
        num_patients, preferred, current, priority
    )

    assigned_patients  = sum(len(p) for p in panels.values())
    unassigned_patients = num_patients - assigned_patients
    wants_switch_count  = len(priority_list)

    print(f"Assigned patients : {assigned_patients}")
    print(f"Unassigned (skip) : {unassigned_patients}")
    print(f"Want to switch    : {wants_switch_count}")
    print(f"Doctors with patients: {len(panels)}")
    print()

    # Keep copies for before/after comparison
    panels_before = {g: set(p) for g, p in panels.items()}

    t0 = time.perf_counter()
    reassigned = run_ttc(panels, preferences, priority_list)
    elapsed_ms = (time.perf_counter() - t0) * 1000

    print(f"Reassigned via TTC: {len(reassigned)}")
    print(f"Time               : {elapsed_ms:.1f} ms")
    if wants_switch_count > 0:
        rate = len(reassigned) / wants_switch_count * 100
        print(f"Satisfaction rate  : {rate:.1f}%  ({len(reassigned)}/{wants_switch_count})")

    # Show a few example trades
    if reassigned:
        print("\nExample reassignments (patient -> new doctor):")
        shown = 0
        for gp_id, panel_after in panels.items():
            new_arrivals = panel_after - panels_before[gp_id]
            for pat_id in new_arrivals:
                old_gp = next(g for g, p in panels_before.items() if pat_id in p)
                print(f"  Patient {pat_id:>6}: Doctor {old_gp} -> Doctor {gp_id}")
                shown += 1
                if shown >= 10:
                    break
            if shown >= 10:
                break
        if len(reassigned) > 10:
            print(f"  ... ({len(reassigned) - 10} more)")


if __name__ == '__main__':
    main()
