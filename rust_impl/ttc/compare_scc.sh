#!/bin/bash

# Comparison script for C++ vs Rust SCC implementations

if [ $# -lt 1 ]; then
    echo "Usage: $0 <data_file>"
    exit 1
fi

DATA_FILE=$1

echo "=========================================="
echo "SCC Algorithm Comparison"
echo "=========================================="
echo "Data file: $DATA_FILE"
echo ""

echo "--- C++ Implementation ---"
./scc_benchmark "$DATA_FILE"
echo ""

echo "--- Rust Implementation (Tarjan SCC from petgraph) ---"
ulimit -s 65536 && ./target/release/scc_benchmark "$DATA_FILE"
echo ""

echo "=========================================="
echo "Comparison complete"
echo "=========================================="
