#!/usr/bin/env python3
"""
Plot benchmark results from the comprehensive benchmark run.
"""

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
from pathlib import Path

def parse_benchmark_file(filename):
    """Parse the benchmark results file."""
    with open(filename, 'r') as f:
        lines = f.readlines()

    # Find the detailed data section
    detailed_start = None
    for i, line in enumerate(lines):
        if line.strip() == '[detailed_data]':
            detailed_start = i + 1
            break

    if detailed_start is None:
        raise ValueError("Could not find [detailed_data] section")

    # Read CSV data
    csv_lines = lines[detailed_start:]

    # Parse CSV
    import io
    df = pd.read_csv(io.StringIO(''.join(csv_lines)))

    return df

def plot_total_time_comparison(df, output_dir='plots'):
    """Plot total execution time comparison across algorithms."""
    Path(output_dir).mkdir(exist_ok=True)

    # Group by file and compute means
    summary = df.groupby(['file_name', 'num_patients']).agg({
        'dfs_total_ms': 'mean',
        'scc_v1_total_ms': 'mean',
        'scc_v2_total_ms': 'mean'
    }).reset_index()

    summary = summary.sort_values('num_patients')

    plt.figure(figsize=(12, 6))
    x = np.arange(len(summary))
    width = 0.25

    plt.bar(x - width, summary['dfs_total_ms'], width, label='DFS', alpha=0.8)
    plt.bar(x, summary['scc_v1_total_ms'], width, label='SCC V1', alpha=0.8)
    plt.bar(x + width, summary['scc_v2_total_ms'], width, label='SCC V2', alpha=0.8)

    plt.xlabel('Dataset (by number of patients)', fontsize=12)
    plt.ylabel('Execution Time (ms)', fontsize=12)
    plt.title('Algorithm Performance Comparison: Total Execution Time', fontsize=14, fontweight='bold')
    plt.xticks(x, summary['num_patients'], rotation=45)
    plt.legend()
    plt.grid(axis='y', alpha=0.3)
    plt.tight_layout()
    plt.savefig(f'{output_dir}/total_time_comparison.png', dpi=300)
    print(f"Saved: {output_dir}/total_time_comparison.png")
    plt.close()

def plot_scaling_analysis(df, output_dir='plots'):
    """Plot how algorithms scale with problem size."""
    Path(output_dir).mkdir(exist_ok=True)

    summary = df.groupby(['file_name', 'num_patients']).agg({
        'dfs_total_ms': 'mean',
        'scc_v1_total_ms': 'mean',
        'scc_v2_total_ms': 'mean'
    }).reset_index()

    summary = summary.sort_values('num_patients')

    plt.figure(figsize=(12, 6))
    plt.plot(summary['num_patients'], summary['dfs_total_ms'],
             marker='o', linewidth=2, markersize=8, label='DFS')
    plt.plot(summary['num_patients'], summary['scc_v1_total_ms'],
             marker='s', linewidth=2, markersize=8, label='SCC V1')
    plt.plot(summary['num_patients'], summary['scc_v2_total_ms'],
             marker='^', linewidth=2, markersize=8, label='SCC V2')

    plt.xlabel('Number of Patients', fontsize=12)
    plt.ylabel('Execution Time (ms)', fontsize=12)
    plt.title('Algorithm Scaling with Problem Size', fontsize=14, fontweight='bold')
    plt.legend(fontsize=11)
    plt.grid(True, alpha=0.3)
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scaling_analysis.png', dpi=300)
    print(f"Saved: {output_dir}/scaling_analysis.png")
    plt.close()

def plot_scc_timing_breakdown(df, output_dir='plots'):
    """Plot timing breakdown for SCC algorithms."""
    Path(output_dir).mkdir(exist_ok=True)

    summary = df.groupby(['file_name', 'num_patients']).agg({
        'scc_v2_graph_ms': 'mean',
        'scc_v2_scc_ms': 'mean',
        'scc_v2_cycle_ms': 'mean',
        'scc_v2_exec_ms': 'mean'
    }).reset_index()

    summary = summary.sort_values('num_patients')

    plt.figure(figsize=(12, 6))
    x = np.arange(len(summary))

    plt.bar(x, summary['scc_v2_graph_ms'], label='Graph Building', alpha=0.8)
    plt.bar(x, summary['scc_v2_scc_ms'], bottom=summary['scc_v2_graph_ms'],
            label='SCC Finding', alpha=0.8)
    plt.bar(x, summary['scc_v2_cycle_ms'],
            bottom=summary['scc_v2_graph_ms'] + summary['scc_v2_scc_ms'],
            label='Cycle Finding', alpha=0.8)
    plt.bar(x, summary['scc_v2_exec_ms'],
            bottom=summary['scc_v2_graph_ms'] + summary['scc_v2_scc_ms'] + summary['scc_v2_cycle_ms'],
            label='Cycle Execution', alpha=0.8)

    plt.xlabel('Dataset (by number of patients)', fontsize=12)
    plt.ylabel('Time (ms)', fontsize=12)
    plt.title('SCC V2 Algorithm: Timing Breakdown', fontsize=14, fontweight='bold')
    plt.xticks(x, summary['num_patients'], rotation=45)
    plt.legend()
    plt.grid(axis='y', alpha=0.3)
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scc_v2_breakdown.png', dpi=300)
    print(f"Saved: {output_dir}/scc_v2_breakdown.png")
    plt.close()

    # Same for SCC V1
    summary_v1 = df.groupby(['file_name', 'num_patients']).agg({
        'scc_v1_graph_ms': 'mean',
        'scc_v1_scc_ms': 'mean',
        'scc_v1_cycle_ms': 'mean',
        'scc_v1_exec_ms': 'mean'
    }).reset_index()

    summary_v1 = summary_v1.sort_values('num_patients')

    plt.figure(figsize=(12, 6))
    x = np.arange(len(summary_v1))

    plt.bar(x, summary_v1['scc_v1_graph_ms'], label='Graph Building', alpha=0.8)
    plt.bar(x, summary_v1['scc_v1_scc_ms'], bottom=summary_v1['scc_v1_graph_ms'],
            label='SCC Finding', alpha=0.8)
    plt.bar(x, summary_v1['scc_v1_cycle_ms'],
            bottom=summary_v1['scc_v1_graph_ms'] + summary_v1['scc_v1_scc_ms'],
            label='Cycle Finding', alpha=0.8)
    plt.bar(x, summary_v1['scc_v1_exec_ms'],
            bottom=summary_v1['scc_v1_graph_ms'] + summary_v1['scc_v1_scc_ms'] + summary_v1['scc_v1_cycle_ms'],
            label='Cycle Execution', alpha=0.8)

    plt.xlabel('Dataset (by number of patients)', fontsize=12)
    plt.ylabel('Time (ms)', fontsize=12)
    plt.title('SCC V1 Algorithm: Timing Breakdown', fontsize=14, fontweight='bold')
    plt.xticks(x, summary_v1['num_patients'], rotation=45)
    plt.legend()
    plt.grid(axis='y', alpha=0.3)
    plt.tight_layout()
    plt.savefig(f'{output_dir}/scc_v1_breakdown.png', dpi=300)
    print(f"Saved: {output_dir}/scc_v1_breakdown.png")
    plt.close()

def plot_speedup_comparison(df, output_dir='plots'):
    """Plot speedup of SCC algorithms compared to DFS."""
    Path(output_dir).mkdir(exist_ok=True)

    summary = df.groupby(['file_name', 'num_patients']).agg({
        'dfs_total_ms': 'mean',
        'scc_v1_total_ms': 'mean',
        'scc_v2_total_ms': 'mean'
    }).reset_index()

    summary = summary.sort_values('num_patients')

    # Calculate speedup
    summary['scc_v1_speedup'] = summary['dfs_total_ms'] / summary['scc_v1_total_ms']
    summary['scc_v2_speedup'] = summary['dfs_total_ms'] / summary['scc_v2_total_ms']

    plt.figure(figsize=(12, 6))
    x = np.arange(len(summary))
    width = 0.35

    plt.bar(x - width/2, summary['scc_v1_speedup'], width, label='SCC V1 vs DFS', alpha=0.8)
    plt.bar(x + width/2, summary['scc_v2_speedup'], width, label='SCC V2 vs DFS', alpha=0.8)

    plt.axhline(y=1.0, color='r', linestyle='--', alpha=0.5, label='Break-even (1x)')

    plt.xlabel('Dataset (by number of patients)', fontsize=12)
    plt.ylabel('Speedup Factor', fontsize=12)
    plt.title('Speedup of SCC Algorithms vs DFS', fontsize=14, fontweight='bold')
    plt.xticks(x, summary['num_patients'], rotation=45)
    plt.legend()
    plt.grid(axis='y', alpha=0.3)
    plt.tight_layout()
    plt.savefig(f'{output_dir}/speedup_comparison.png', dpi=300)
    print(f"Saved: {output_dir}/speedup_comparison.png")
    plt.close()

def plot_variability(df, output_dir='plots'):
    """Plot variability across runs using box plots."""
    Path(output_dir).mkdir(exist_ok=True)

    # Select a few interesting datasets
    datasets = df.groupby('num_patients')['file_name'].first().reset_index()
    selected = datasets.sort_values('num_patients').iloc[::2]  # Every other dataset

    fig, axes = plt.subplots(1, 3, figsize=(16, 5))

    for idx, (ax, algo, col) in enumerate(zip(axes,
                                                ['DFS', 'SCC V1', 'SCC V2'],
                                                ['dfs_total_ms', 'scc_v1_total_ms', 'scc_v2_total_ms'])):
        data_to_plot = []
        labels = []

        for _, row in selected.iterrows():
            file_data = df[df['file_name'] == row['file_name']][col]
            data_to_plot.append(file_data)
            labels.append(f"{row['num_patients']}")

        bp = ax.boxplot(data_to_plot, labels=labels, patch_artist=True)

        for patch in bp['boxes']:
            patch.set_facecolor('lightblue')
            patch.set_alpha(0.7)

        ax.set_xlabel('Number of Patients', fontsize=11)
        ax.set_ylabel('Execution Time (ms)', fontsize=11)
        ax.set_title(f'{algo} Algorithm', fontsize=12, fontweight='bold')
        ax.grid(axis='y', alpha=0.3)
        plt.setp(ax.xaxis.get_majorticklabels(), rotation=45)

    plt.suptitle('Execution Time Variability Across 10 Runs', fontsize=14, fontweight='bold')
    plt.tight_layout()
    plt.savefig(f'{output_dir}/variability_analysis.png', dpi=300)
    print(f"Saved: {output_dir}/variability_analysis.png")
    plt.close()

def extract_districts_from_filename(filename):
    """Extract number of districts from filename."""
    import re
    match = re.search(r'_(\d+)_districts', filename)
    if match:
        return int(match.group(1))
    return 0  # No districts specified

def plot_district_impact(df, output_dir='plots'):
    """Plot how district count affects performance for same-size datasets."""
    Path(output_dir).mkdir(exist_ok=True)

    # Add district column
    df['districts'] = df['file_name'].apply(extract_districts_from_filename)

    # Group by patient/doctor counts to find datasets with multiple district variants
    grouped = df.groupby(['num_patients', 'num_doctors'])

    for (num_patients, num_doctors), group in grouped:
        if group['districts'].nunique() < 2:
            continue  # Skip if no district variation

        summary = group.groupby('districts').agg({
            'dfs_total_ms': 'mean',
            'scc_v1_total_ms': 'mean',
            'scc_v2_total_ms': 'mean'
        }).reset_index()

        summary = summary.sort_values('districts')

        if len(summary) < 2:
            continue

        plt.figure(figsize=(12, 6))

        plt.plot(summary['districts'], summary['dfs_total_ms'],
                marker='o', linewidth=2, markersize=10, label='DFS')
        plt.plot(summary['districts'], summary['scc_v1_total_ms'],
                marker='s', linewidth=2, markersize=10, label='SCC V1')
        plt.plot(summary['districts'], summary['scc_v2_total_ms'],
                marker='^', linewidth=2, markersize=10, label='SCC V2')

        plt.xlabel('Number of Districts', fontsize=12)
        plt.ylabel('Execution Time (ms)', fontsize=12)
        plt.title(f'Impact of District Count on Performance\n({num_patients:,} patients, {num_doctors:,} doctors)',
                 fontsize=14, fontweight='bold')
        plt.legend(fontsize=11)
        plt.grid(True, alpha=0.3)
        plt.tight_layout()

        filename = f'{output_dir}/district_impact_{num_patients}p_{num_doctors}d.png'
        plt.savefig(filename, dpi=300)
        print(f"Saved: {filename}")
        plt.close()

def plot_district_speedup(df, output_dir='plots'):
    """Plot speedup vs DFS as districts change."""
    Path(output_dir).mkdir(exist_ok=True)

    # Add district column
    df['districts'] = df['file_name'].apply(extract_districts_from_filename)

    # Group by patient/doctor counts
    grouped = df.groupby(['num_patients', 'num_doctors'])

    for (num_patients, num_doctors), group in grouped:
        if group['districts'].nunique() < 2:
            continue

        summary = group.groupby('districts').agg({
            'dfs_total_ms': 'mean',
            'scc_v1_total_ms': 'mean',
            'scc_v2_total_ms': 'mean'
        }).reset_index()

        summary = summary.sort_values('districts')

        if len(summary) < 2:
            continue

        summary['scc_v1_speedup'] = summary['dfs_total_ms'] / summary['scc_v1_total_ms']
        summary['scc_v2_speedup'] = summary['dfs_total_ms'] / summary['scc_v2_total_ms']

        plt.figure(figsize=(12, 6))

        plt.plot(summary['districts'], summary['scc_v1_speedup'],
                marker='s', linewidth=2, markersize=10, label='SCC V1 vs DFS')
        plt.plot(summary['districts'], summary['scc_v2_speedup'],
                marker='^', linewidth=2, markersize=10, label='SCC V2 vs DFS')

        plt.axhline(y=1.0, color='r', linestyle='--', alpha=0.5, label='Break-even (1x)')

        plt.xlabel('Number of Districts', fontsize=12)
        plt.ylabel('Speedup Factor', fontsize=12)
        plt.title(f'Speedup vs DFS by District Count\n({num_patients:,} patients, {num_doctors:,} doctors)',
                 fontsize=14, fontweight='bold')
        plt.legend(fontsize=11)
        plt.grid(True, alpha=0.3)
        plt.tight_layout()

        filename = f'{output_dir}/district_speedup_{num_patients}p_{num_doctors}d.png'
        plt.savefig(filename, dpi=300)
        print(f"Saved: {filename}")
        plt.close()

def generate_summary_table(df, output_dir='plots'):
    """Generate a summary table."""
    summary = df.groupby(['file_name', 'num_patients', 'num_doctors']).agg({
        'dfs_total_ms': ['mean', 'std'],
        'scc_v1_total_ms': ['mean', 'std'],
        'scc_v2_total_ms': ['mean', 'std'],
        'cycles_found': 'mean',
        'patients_reassigned': 'mean'
    }).reset_index()

    summary = summary.sort_values('num_patients')

    # Save to CSV
    summary.to_csv(f'{output_dir}/summary_table.csv', index=False)
    print(f"Saved: {output_dir}/summary_table.csv")

    return summary

def main():
    """Main function to generate all plots."""
    benchmark_file = 'benchmark_results_comprehensive.txt'
    output_dir = 'plots'

    if not Path(benchmark_file).exists():
        print(f"Error: {benchmark_file} not found!")
        print("Please run the benchmark first: cargo run --release")
        return

    print(f"Reading benchmark data from {benchmark_file}...")
    df = parse_benchmark_file(benchmark_file)

    print(f"\nDataset summary:")
    print(f"  Total runs: {len(df)}")
    print(f"  Files benchmarked: {df['file_name'].nunique()}")
    print(f"  Runs per file: {len(df) // df['file_name'].nunique()}")

    print(f"\nGenerating plots in '{output_dir}/' directory...")

    plot_total_time_comparison(df, output_dir)
    plot_scaling_analysis(df, output_dir)
    plot_scc_timing_breakdown(df, output_dir)
    plot_speedup_comparison(df, output_dir)
    plot_variability(df, output_dir)
    plot_district_impact(df, output_dir)
    plot_district_speedup(df, output_dir)
    generate_summary_table(df, output_dir)

    print("\n✅ All plots generated successfully!")
    print(f"\nYou can find all plots in the '{output_dir}/' directory:")
    print("  - total_time_comparison.png: Bar chart comparing all algorithms")
    print("  - scaling_analysis.png: Line plot showing how algorithms scale")
    print("  - scc_v1_breakdown.png: Stacked bar chart of SCC V1 timing phases")
    print("  - scc_v2_breakdown.png: Stacked bar chart of SCC V2 timing phases")
    print("  - speedup_comparison.png: Speedup factors vs DFS")
    print("  - variability_analysis.png: Box plots showing run-to-run variability")
    print("  - district_impact_*.png: How district count affects performance")
    print("  - district_speedup_*.png: How district count affects speedup vs DFS")
    print("  - summary_table.csv: Detailed summary statistics")

if __name__ == '__main__':
    main()
