use std::collections::{HashMap, HashSet, VecDeque};
use crate::{AssignmentState, CycleStats, ResultWithStats};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum Agent {
    Pat(usize),
    Doc(usize),
}

fn find_cycle_h(
    starting: Agent,
    available: &HashSet<Agent>,
    prefs: &HashMap<Agent, Vec<Agent>>,
    prefs_avail: &mut HashMap<Agent, Vec<Agent>>,
) -> Vec<Agent> {
    let mut chain = vec![starting];
    loop {
        let last = *chain.last().unwrap();
        let avail: Vec<Agent> = prefs[&last].iter()
            .filter(|a| available.contains(*a))
            .copied()
            .collect();
        let top = avail[0];
        prefs_avail.insert(last, avail);
        if chain.contains(&top) {
            break;
        }
        chain.push(top);
    }

    let mut cycle = vec![*chain.last().unwrap()];
    loop {
        let last = *cycle.last().unwrap();
        let top = prefs_avail[&last][0];
        if cycle[0] == top {
            break;
        }
        cycle.push(top);
    }
    cycle
}

fn run_ttc_inner(
    panels_orig: &HashMap<usize, HashSet<usize>>,
    mut preferences: HashMap<usize, Vec<usize>>,
    priority_list_input: Vec<usize>,
) -> (HashSet<usize>, usize, Vec<usize>) {

    let waiters: HashSet<usize> = preferences.keys().copied().collect();

    let mut panels_ttc: HashMap<usize, HashSet<usize>> = panels_orig.iter()
        .map(|(&g, p)| (g, p.iter().filter(|&&pat| waiters.contains(&pat)).copied().collect()))
        .collect();

    // First pass: remove zero-capacity doctors and stuck patients
    let mut gps_zero: HashSet<usize> = panels_ttc.iter()
        .filter(|(_, p)| p.is_empty())
        .map(|(&g, _)| g)
        .collect();
    for &g in &gps_zero {
        panels_ttc.remove(&g);
    }
    for pref in preferences.values_mut() {
        pref.retain(|g| !gps_zero.contains(g));
    }

    let mut stuck: HashSet<usize> = preferences.iter()
        .filter(|(_, p)| p.len() <= 1)
        .map(|(&pat, _)| pat)
        .collect();
    for &pat in &stuck {
        preferences.remove(&pat);
    }
    for p in panels_ttc.values_mut() {
        p.retain(|pat| !stuck.contains(pat));
    }

    // Repeat until no more zero-capacity doctors
    loop {
        gps_zero = panels_ttc.iter()
            .filter(|(_, p)| p.is_empty())
            .map(|(&g, _)| g)
            .collect();
        if gps_zero.is_empty() {
            break;
        }
        for &g in &gps_zero {
            panels_ttc.remove(&g);
        }
        for pref in preferences.values_mut() {
            pref.retain(|g| !gps_zero.contains(g));
        }

        stuck = preferences.iter()
            .filter(|(_, p)| p.len() <= 1)
            .map(|(&pat, _)| pat)
            .collect();
        for &pat in &stuck {
            preferences.remove(&pat);
        }
        for p in panels_ttc.values_mut() {
            p.retain(|pat| !stuck.contains(pat));
        }
    }

    let mut pats_assigned: HashSet<usize> = HashSet::new();
    let mut cycles_found = 0usize;
    let mut cycle_lengths: Vec<usize> = Vec::new();

    if panels_ttc.is_empty() {
        return (pats_assigned, cycles_found, cycle_lengths);
    }

    let patients: HashSet<usize> = panels_ttc.values()
        .flat_map(|p| p.iter().copied())
        .collect();
    let gps: HashSet<usize> = panels_ttc.keys().copied().collect();

    let mut capacity: HashMap<usize, usize> = gps.iter()
        .map(|&g| (g, panels_ttc[&g].len()))
        .collect();

    // priority_list filtered to active patients
    let priority_list_active: Vec<usize> = priority_list_input.iter()
        .filter(|&&p| patients.contains(&p))
        .copied()
        .collect();

    // Build prefs dict (patients)
    let mut prefs: HashMap<Agent, Vec<Agent>> = HashMap::new();
    for (&pat, pref_list) in &preferences {
        if patients.contains(&pat) {
            let filtered: Vec<Agent> = pref_list.iter()
                .filter(|&&g| gps.contains(&g))
                .map(|&g| Agent::Doc(g))
                .collect();
            if !filtered.is_empty() {
                prefs.insert(Agent::Pat(pat), filtered);
            }
        }
    }

    // Build prefs dict (doctors): sorted current patients + waitlisted by priority
    for (&gp, panel) in &panels_ttc {
        let mut curr_pats: Vec<usize> = panel.iter().copied().collect();
        curr_pats.sort();

        let wl_pats: Vec<usize> = priority_list_active.iter()
            .filter(|&&p| {
                preferences.get(&p)
                    .map_or(false, |pref| pref.contains(&gp))
            })
            .copied()
            .collect();

        let doc_prefs: Vec<Agent> = curr_pats.iter().map(|&p| Agent::Pat(p))
            .chain(wl_pats.iter().map(|&p| Agent::Pat(p)))
            .collect();
        prefs.insert(Agent::Doc(gp), doc_prefs);
    }

    let mut available: HashSet<Agent> = patients.iter().map(|&p| Agent::Pat(p))
        .chain(gps.iter().map(|&g| Agent::Doc(g)))
        .collect();

    let mut priority_q: VecDeque<usize> = priority_list_active.into_iter().collect();
    if priority_q.is_empty() {
        return (pats_assigned, cycles_found, cycle_lengths);
    }
    let mut starting = priority_q.pop_front().unwrap();

    while !available.is_empty() {
        // Advance past unavailable starting patients
        while !available.contains(&Agent::Pat(starting)) {
            match priority_q.pop_front() {
                Some(p) => starting = p,
                None => return (pats_assigned, cycles_found, cycle_lengths),
            }
        }

        let patient_prefs = match prefs.get(&Agent::Pat(starting)) {
            Some(p) if !p.is_empty() => p.clone(),
            _ => {
                available.remove(&Agent::Pat(starting));
                continue;
            }
        };

        // Check if stuck: all non-last preferences unavailable
        let all_but_last_unavail = patient_prefs[..patient_prefs.len() - 1]
            .iter()
            .all(|a| !available.contains(a));

        if all_but_last_unavail {
            let gp_curr = match patient_prefs.last().unwrap() {
                Agent::Doc(g) => *g,
                _ => panic!("last pref must be a doctor"),
            };
            let cap = capacity.get_mut(&gp_curr).unwrap();
            *cap -= 1;
            if *cap == 0 {
                available.remove(&Agent::Doc(gp_curr));
            }
            available.remove(&Agent::Pat(starting));
        } else {
            let mut prefs_avail: HashMap<Agent, Vec<Agent>> = HashMap::new();
            let cycle = find_cycle_h(Agent::Pat(starting), &available, &prefs, &mut prefs_avail);

            let num_docs_in_cycle = cycle.iter()
                .filter(|a| matches!(a, Agent::Doc(_)))
                .count();

            for j in 0..cycle.len() {
                if let Agent::Doc(gp) = cycle[j] {
                    let cap = capacity.get_mut(&gp).unwrap();
                    *cap -= 1;
                    if *cap == 0 {
                        available.remove(&Agent::Doc(gp));
                    }
                    let j_prev = if j == 0 { cycle.len() - 1 } else { j - 1 };
                    available.remove(&cycle[j_prev]);

                    if cycle.len() > 2 {
                        let j_next = if j == cycle.len() - 1 { 0 } else { j + 1 };
                        let pat_prev = match cycle[j_prev] {
                            Agent::Pat(p) => p,
                            _ => panic!("expected patient"),
                        };
                        let _pat_next = match cycle[j_next] {
                            Agent::Pat(p) => p,
                            _ => panic!("expected patient"),
                        };

                        pats_assigned.insert(pat_prev);
                        preferences.remove(&pat_prev);
                    }
                }
            }

            if cycle.len() > 2 {
                cycles_found += 1;
                cycle_lengths.push(num_docs_in_cycle);
            }
        }
    }

    (pats_assigned, cycles_found, cycle_lengths)
}

pub fn huitfeldt_ttc(state: &mut AssignmentState) -> ResultWithStats {
    // Build panels: doc_id -> set of pat_ids (skip unassigned and dummy patients)
    let mut panels: HashMap<usize, HashSet<usize>> = HashMap::new();
    for patient in &state.patients {
        if patient.is_dummy {
            continue;
        }
        if let Some(curr) = patient.current_doctor {
            if curr == 0 {
                continue;
            }
            panels.entry(curr).or_default().insert(patient.id);
        }
    }

    // Build preferences and priority_list for patients wanting to switch
    let mut wants_switch: Vec<(usize, usize)> = Vec::new();
    for patient in &state.patients {
        if patient.is_dummy {
            continue;
        }
        let curr = match patient.current_doctor {
            Some(d) if d > 0 => d,
            _ => continue,
        };
        if patient.preferred_doctor == curr {
            continue;
        }
        wants_switch.push((patient.priority, patient.id));
    }
    wants_switch.sort();

    let priority_list: Vec<usize> = wants_switch.iter().map(|&(_, id)| id).collect();
    let mut preferences: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(_, pat_id) in &wants_switch {
        let p = &state.patients[pat_id - 1];
        preferences.insert(pat_id, vec![p.preferred_doctor, p.current_doctor.unwrap()]);
    }

    let (reassigned, cycles_found, cycle_lengths) =
        run_ttc_inner(&panels, preferences, priority_list);

    let solution: HashSet<usize> = reassigned.iter()
        .filter_map(|&id| state.get_patient(id).map(|p| p.priority))
        .collect();

    let patients_reassigned = reassigned.len();
    let mut cycle_stats = CycleStats::new();
    for len in cycle_lengths {
        cycle_stats.record_cycle(len);
    }

    for &pat_id in &reassigned {
        state.resolve_patient(pat_id);
    }

    ResultWithStats {
        solution,
        cycles_found,
        patients_reassigned,
        patients_pruned: 0,
        remaining_capacity: state.get_total_availability(),
        cycle_stats,
        initial_unsatisfied: 0,
        final_unsatisfied: 0,
        initial_unassigned: 0,
        final_unassigned: 0,
        total_capacity: 0,
        initial_capacity_used: 0,
    }
}
