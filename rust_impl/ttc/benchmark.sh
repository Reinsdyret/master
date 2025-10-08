#!/bin/bash

echo "Running 10 benchmark iterations..."
echo ""

for i in {1..10}; do
    echo -n "Run $i: "
    cargo run --release 2>&1 | grep "PERFORMANCE COMPARISON" -A 4 | grep "time:" | awk '{print $2, $3, $4}'
done

echo ""
echo "Calculating statistics..."
