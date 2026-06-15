#!/usr/bin/env python3
"""
District plots for the simulation.

Two things the generic summary/day plotters can't show:

1. District structure  -- from simulation_districts_<ts>.csv
   (district_id, doctor_count, patient_count). Shows the generated, deliberately
   uneven split of doctors and patients across districts.

2. Cross- vs within-district outcomes -- from simulation_summary_<ts>.csv.
   Grouped bars per algorithm comparing cross-district requests against
   within-district ones on:
     - realization rate = resolved_count / added   (the headline: do cross
       requests get served as often, or starve?)
     - average resolved wait
     - P90 resolved wait
     - outstanding (never resolved) count
   The realization rate is a ratio of two stored columns, so it can't be drawn
   by the column-generic plot_summary.py.

Usage:
    python plot_districts.py <districts_csv> [--summary <summary_csv>]
                             [--out-prefix PREFIX] [--no-plot]

Examples:
    python plot_districts.py simulation_results/simulation_districts_20260607_120000.csv
    python plot_districts.py simulation_results/simulation_districts_20260607_120000.csv \
        --summary simulation_results/simulation_summary_20260607_120000.csv
"""

import argparse
import csv
import os
import sys
from datetime import datetime

# 12 distinct colors (matplotlib tab10 + 2 darks), shared with the other plotters.
COLORS = ["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b",
          "#e377c2", "#7f7f7f", "#bcbd22", "#17becf", "#393b79", "#637939"]

# Auto-named plots go here; an explicit --out-prefix is still honored verbatim.
PLOTS_DIR = "plots"
CROSS_COLOR = "#d62728"   # cross-district
WITHIN_COLOR = "#1f77b4"  # within-district

# Cross-vs-within panels: (title, cross_col, within_col, is_ratio)
# is_ratio panels are computed as <prefix>_resolved_count / <prefix>_added.
CROSS_PANELS = [
    ("Realization rate (resolved / added)", "cross", "within", True),
    ("Resolved max wait (d)", "cross_resolved_max", "within_resolved_max", False),
]


def load_csv(path, expect_col):
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        if expect_col not in (reader.fieldnames or []):
            sys.exit(f"{path}: missing '{expect_col}' column -- wrong CSV?")
        rows = list(reader)
    if not rows:
        sys.exit(f"{path} is empty.")
    return rows


def fnum(row, col):
    try:
        return float(row[col])
    except (KeyError, ValueError):
        return 0.0


def realization(row, prefix):
    """resolved_count / added for 'cross' or 'within', guarding divide-by-zero."""
    added = fnum(row, f"{prefix}_added")
    resolved = fnum(row, f"{prefix}_resolved_count")
    return resolved / added if added > 0 else 0.0


def print_district_table(rows):
    total_doc = sum(int(r["doctor_count"]) for r in rows)
    total_pat = sum(int(r["patient_count"]) for r in rows)
    print(f"\nDistrict structure ({len(rows)} districts, "
          f"{total_doc} doctors, {total_pat} patients)")
    print(f"{'district':>8}  {'doctors':>8}  {'patients':>9}  {'pat/doc':>8}")
    print("-" * 40)
    for r in rows:
        doc = int(r["doctor_count"])
        pat = int(r["patient_count"])
        ratio = pat / doc if doc else 0.0
        print(f"{int(r['district_id']):>8}  {doc:>8}  {pat:>9}  {ratio:>8.1f}")
    print("-" * 40)


def print_cross_table(rows):
    print("\nCross- vs within-district outcomes")
    head = (f"{'algorithm':<18}  {'x-real':>7}  {'w-real':>7}  "
            f"{'x-wait':>7}  {'w-wait':>7}  {'x-out':>7}  {'w-out':>7}")
    print(head)
    print("-" * len(head))
    for r in rows:
        print(f"{r['algorithm']:<18}  "
              f"{realization(r, 'cross'):>6.1%}  {realization(r, 'within'):>6.1%}  "
              f"{fnum(r, 'cross_resolved_avg'):>7.1f}  {fnum(r, 'within_resolved_avg'):>7.1f}  "
              f"{int(fnum(r, 'cross_outstanding_count')):>7}  "
              f"{int(fnum(r, 'within_outstanding_count')):>7}")
    print("-" * len(head))


def plot_district_structure(rows, out_path):
    import matplotlib.pyplot as plt

    ids = [int(r["district_id"]) for r in rows]
    docs = [int(r["doctor_count"]) for r in rows]
    pats = [int(r["patient_count"]) for r in rows]
    x = range(len(ids))

    fig, (ax_d, ax_p) = plt.subplots(1, 2, figsize=(13, 4.5))

    bars_d = ax_d.bar(x, docs, color=COLORS[0])
    ax_d.set_title("Doctors per district", fontsize=12)
    ax_d.set_xlabel("District")
    ax_d.set_ylabel("Doctor count")

    bars_p = ax_p.bar(x, pats, color=COLORS[2])
    ax_p.set_title("Patients per district", fontsize=12)
    ax_p.set_xlabel("District")
    ax_p.set_ylabel("Patient count (= total capacity)")

    for ax, bars in ((ax_d, bars_d), (ax_p, bars_p)):
        ax.set_xticks(list(x))
        ax.set_xticklabels(ids)
        ax.grid(True, axis="y", alpha=0.3)
        for b in bars:
            ax.annotate(f"{int(b.get_height())}",
                        (b.get_x() + b.get_width() / 2, b.get_height()),
                        ha="center", va="bottom", fontsize=8,
                        xytext=(0, 2), textcoords="offset points")

    fig.suptitle("District structure", fontsize=13)
    plt.tight_layout()
    plt.savefig(out_path, format="svg")
    print(f"Plot saved to: {out_path}")
    plt.close(fig)


def plot_cross_within(rows,csv_path, out_path):
    import matplotlib.pyplot as plt

    algs = [r["algorithm"] for r in rows]
    x = range(len(algs))
    width = 0.4

    n = len(CROSS_PANELS)
    ncols = 2
    nrows = -(-n // ncols)
    fig, axes = plt.subplots(nrows, ncols, figsize=(7 * ncols, 4.5 * nrows), squeeze=False)

    for idx, (title, cross_key, within_key, is_ratio) in enumerate(CROSS_PANELS):
        ax = axes[idx // ncols][idx % ncols]
        if is_ratio:
            cross_vals = [realization(r, cross_key) for r in rows]
            within_vals = [realization(r, within_key) for r in rows]
        else:
            cross_vals = [fnum(r, cross_key) for r in rows]
            within_vals = [fnum(r, within_key) for r in rows]

        b1 = ax.bar([i - width / 2 for i in x], cross_vals, width,
                    label="cross-district", color=CROSS_COLOR)
        b2 = ax.bar([i + width / 2 for i in x], within_vals, width,
                    label="within-district", color=WITHIN_COLOR)

        ax.set_title(title, fontsize=12)
        ax.set_xticks(list(x))
        ax.set_xticklabels(algs, rotation=30, ha="right", fontsize=9)
        ax.grid(True, axis="y", alpha=0.3)
        if is_ratio:
            ax.set_ylim(0, 1.05)
        for bars in (b1, b2):
            for b in bars:
                v = b.get_height()
                txt = f"{v:.0%}" if is_ratio else (f"{v:.1f}" if v < 100 else f"{int(v)}")
                ax.annotate(txt, (b.get_x() + b.get_width() / 2, v),
                            ha="center", va="bottom", fontsize=7,
                            xytext=(0, 2), textcoords="offset points")
        ax.legend(fontsize=9)

    for j in range(n, nrows * ncols):
        axes[j // ncols][j % ncols].axis("off")

    fig.suptitle("Cross- vs within-district request outcomes", fontsize=13)
    plt.tight_layout()
    plt.savefig(out_path, format="svg")
    print(f"Plot saved to: {out_path}")
    plt.close(fig)


def main():
    parser = argparse.ArgumentParser(description="District structure and cross-district outcome plots.")
    parser.add_argument("districts_csv", help="Path to simulation_districts_<ts>.csv")
    parser.add_argument("--summary", default=None,
                        help="Path to simulation_summary_<ts>.csv for cross-vs-within panels")
    parser.add_argument("--out-prefix", default=None,
                        help="Output PNG prefix (default: auto-named with timestamp)")
    parser.add_argument("--no-plot", action="store_true", help="Print tables only, no PNGs")
    args = parser.parse_args()

    if not os.path.isfile(args.districts_csv):
        sys.exit(f"File not found: {args.districts_csv}")

    district_rows = load_csv(args.districts_csv, "district_id")
    print_district_table(district_rows)

    summary_rows = None
    if args.summary:
        if not os.path.isfile(args.summary):
            sys.exit(f"File not found: {args.summary}")
        summary_rows = load_csv(args.summary, "algorithm")
        print_cross_table(summary_rows)

    if args.no_plot:
        return

    try:
        import matplotlib  # noqa: F401
    except ImportError:
        sys.exit("matplotlib not installed. Run: pip install matplotlib")

    if args.out_prefix:
        prefix = args.out_prefix
    else:
        os.makedirs(PLOTS_DIR, exist_ok=True)
        prefix = os.path.join(PLOTS_DIR, f"district_plot_{datetime.now().strftime('%Y%m%d_%H%M%S')}")
    plot_district_structure(district_rows, f"{prefix}_structure.svg")
    if summary_rows:
        plot_cross_within(summary_rows, args.summary, f"{prefix}_cross_within.svg")


if __name__ == "__main__":
    main()
