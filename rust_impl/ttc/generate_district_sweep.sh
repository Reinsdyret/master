#!/bin/bash

# Generate 100 test data files with varying district counts
# Fixed parameters: 200000 patients, 5000 doctors, 0.0 cross_prob
# Variable: districts from 1 to 1000 (logarithmic spacing)

python3 << 'EOF'
import sys
sys.path.insert(0, '.')
from generate_test_data import generate_district_based_test_data
import numpy as np

PATIENTS = 200000
DOCTORS = 5000
CROSS_PROB = 0.0

# Generate 100 district values with logarithmic spacing from 1 to 1000
districts = np.logspace(0, 3, 100, dtype=int)  # 10^0 to 10^3 = 1 to 1000
districts = sorted(set(districts))  # Remove duplicates and sort

for d in districts:
    filename = f"data/test_{PATIENTS}_patient_{DOCTORS}_doctors_{d}_districts_{CROSS_PROB}_prob.txt"
    print(f'Generating test data: {d} districts -> {filename}')
    generate_district_based_test_data(PATIENTS, DOCTORS, d, CROSS_PROB, filename)

print(f"\nGenerated {len(districts)} test files in data/ directory")
EOF
