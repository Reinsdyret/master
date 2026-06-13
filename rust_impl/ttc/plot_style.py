#!/usr/bin/env python3
"""
Shared algorithm styling for every simulation plot.

Keeping one name -> color map here (instead of a per-file COLORS list indexed by
position) guarantees an algorithm draws in the *same* color in every plot,
regardless of how many algorithms a given CSV contains or what order they appear
in. Import `color_for` / `label_for` from this module instead of indexing a list.
"""

# Canonical display name -> fixed color. Colors match the original tab10 order so
# existing plots are visually unchanged after the switch to name-based lookup.
_PALETTE = {
    "Greedy DFS":                           "#1f77b4",  # blue
    "Huitfeldt TTC":                        "#ff7f0e",  # orange
    "Cycle Cancelling for cardinality":     "#2ca02c",  # green
    "Cycle Cancelling for strict priority": "#d62728",  # red
    "Cycle Cancelling for utility":         "#9467bd",  # purple
    # Utility exponential variants — fixed colors so 1.01..1.9 stay consistent
    # across summary and simulation plots.
    "Util Exp 1.01":                        "#8c564b",  # brown
    "Util Exp 1.05":                        "#e377c2",  # pink
    "Util Exp 1.1":                         "#7f7f7f",  # gray
    "Util Exp 1.5":                         "#bcbd22",  # olive
    "Util Exp 1.9":                         "#17becf",  # cyan
}

# Old CSV label -> new canonical label. Lets pre-rename CSVs map to the same
# color and show the new name in legends without re-running the simulation.
_ALIASES = {
    "Exact Cardinality": "Cycle Cancelling for cardinality",
    "Exact Priority":    "Cycle Cancelling for strict priority",
    "Util Linear":       "Cycle Cancelling for utility",
}

# Per-algorithm marker + linestyle so lines stay distinguishable without relying
# on color alone (colorblind-safe). Keyed by canonical name like _PALETTE.
_MARKERS = {
    "Greedy DFS":                           "o",  # circle
    "Huitfeldt TTC":                        "s",  # square
    "Cycle Cancelling for cardinality":     "^",  # triangle up
    "Cycle Cancelling for strict priority": "D",  # diamond
    "Cycle Cancelling for utility":         "v",  # triangle down
    "Util Exp 1.01":                        "P",  # plus (filled)
    "Util Exp 1.05":                        "X",  # x (filled)
    "Util Exp 1.1":                         "*",  # star
    "Util Exp 1.5":                         "p",  # pentagon
    "Util Exp 1.9":                         "h",  # hexagon
}

_LINESTYLES = {
    "Greedy DFS":                           "-",
    "Huitfeldt TTC":                        "--",
    "Cycle Cancelling for cardinality":     "-.",
    "Cycle Cancelling for strict priority": ":",
    "Cycle Cancelling for utility":         (0, (5, 1)),       # dense dash
    "Util Exp 1.01":                        (0, (3, 1, 1, 1)), # dash-dot-dot
    "Util Exp 1.05":                        (0, (1, 1)),       # dotted
    "Util Exp 1.1":                         (0, (5, 2)),
    "Util Exp 1.5":                         (0, (3, 2, 1, 2)),
    "Util Exp 1.9":                         (0, (7, 1, 2, 1)),
}

# Deterministic fallback cycles for unlisted algorithms.
_MARKER_FALLBACK = ["o", "s", "^", "D", "v", "P", "X", "*", "p", "h"]
_LINESTYLE_FALLBACK = ["-", "--", "-.", ":", (0, (5, 1)), (0, (3, 1, 1, 1))]

# Deterministic fallback for any algorithm not listed above (stable run-to-run).
_FALLBACK = ["#393b79", "#637939", "#9b59b6", "#f1c40f", "#1abc9c", "#e74c3c"]


def label_for(name):
    """Canonical display name (maps old labels to the renamed ones)."""
    return _ALIASES.get(name, name)


def color_for(name):
    """Fixed color for an algorithm, keyed by name rather than plot order."""
    name = label_for(name)
    if name in _PALETTE:
        return _PALETTE[name]
    idx = sum(ord(c) for c in name) % len(_FALLBACK)
    return _FALLBACK[idx]


def marker_for(name):
    """Fixed point marker for an algorithm (distinguishes lines without color)."""
    name = label_for(name)
    if name in _MARKERS:
        return _MARKERS[name]
    idx = sum(ord(c) for c in name) % len(_MARKER_FALLBACK)
    return _MARKER_FALLBACK[idx]


def linestyle_for(name):
    """Fixed line dash pattern for an algorithm (colorblind-safe distinction)."""
    name = label_for(name)
    if name in _LINESTYLES:
        return _LINESTYLES[name]
    idx = sum(ord(c) for c in name) % len(_LINESTYLE_FALLBACK)
    return _LINESTYLE_FALLBACK[idx]
