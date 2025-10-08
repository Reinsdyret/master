#!/usr/bin/env python3

import statistics
import sys

# Read data from benchmark_results.txt
def read_benchmark_file(filename="benchmark_results.txt"):
    data = {}
    try:
        with open(filename, 'r') as f:
            for line in f:
                line = line.strip()
                if line.startswith('#') or not line:
                    continue
                if '=' in line:
                    name, values = line.split('=', 1)
                    name = name.strip()
                    data[name] = eval(values.strip())
        return data['dfs_pruning'], data['scc_v1'], data['scc_v2_optimized']
    except FileNotFoundError:
        print(f"❌ Error: {filename} not found!")
        print("Please run the Rust benchmark first: cargo run --release")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error reading {filename}: {e}")
        sys.exit(1)

pruning, scc_v1, scc_v2  = read_benchmark_file()
print(f"✅ Loaded {len(pruning)} benchmark samples from benchmark_results.txt\n")

def analyze(name, data):
    mean = statistics.mean(data)
    stdev = statistics.stdev(data)
    min_val = min(data)
    max_val = max(data)
    median = statistics.median(data)
    cv = (stdev / mean) * 100  # Coefficient of variation

    print(f"\n{name}:")
    print(f"  Mean:   {mean:.2f}ms")
    print(f"  Median: {median:.2f}ms")
    print(f"  StdDev: {stdev:.2f}ms ({cv:.1f}% variance)")
    print(f"  Range:  {min_val:.2f}ms - {max_val:.2f}ms")
    return mean

print("="*50)
print(f"BENCHMARK ANALYSIS ({len(pruning)} runs)")
print("="*50)

pruning_mean = analyze("DFS Pruning", pruning)
tarjan_mean = analyze("SCC Tarjan (Optimized)", scc_v1)
scc_v2_mean = analyze("Scc v2", scc_v2)

print("\n" + "="*50)
print("RELATIVE PERFORMANCE:")
print("="*50)
print(f"Tarjan vs Pruning: {pruning_mean/tarjan_mean:.2f}x faster")

print("\n" + "="*50)
print("STABILITY (lower is more consistent):")
print("="*50)
print(f"Pruning:  {(statistics.stdev(pruning)/statistics.mean(pruning))*100:.1f}% variance")
print(f"Tarjan:   {(statistics.stdev(tarjan)/statistics.mean(tarjan))*100:.1f}% variance")
