import random

def generate_ttc_test_data_no_overlap(num_patients, num_doctors, filename):
    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")
        
        preferred = []
        for patient in range(num_patients):
            preferred.append(str(random.randint(1, num_doctors)))
        
        current = list(range(1, min(num_patients, num_doctors) + 1))
        if num_patients > num_doctors:
            current.extend([random.randint(1, num_doctors) for _ in range(num_patients - num_doctors)])
        random.shuffle(current)
        current = [str(doc) for doc in current]
        
        f.write(",".join(preferred) + "\n")
        f.write(",".join(current) + "\n")

if __name__ == "__main__":
    generate_ttc_test_data_no_overlap(5, 5, "test_5_patient_5_doctors_no_overlap.txt")
    generate_ttc_test_data_no_overlap(6, 4, "test_6_patient_4_doctors_no_overlap.txt")
    generate_ttc_test_data_no_overlap(8, 8, "test_8_patient_8_doctors_no_overlap.txt")
    print("Generated no-overlap test files")