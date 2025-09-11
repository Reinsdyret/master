import random

def generate_ttc_test_data(num_patients, num_doctors, filename):
    # Generate random preferability scores for each doctor (higher = more preferred)
    doctor_preferability = [random.uniform(0.1, 10.0) for _ in range(num_doctors)]
    
    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")
        
        preferred = []
        current = []
        priorities = []
        
        for patient in range(num_patients):
            # Choose preferred doctor based on preferability weights
            preferred_doctor = random.choices(
                range(1, num_doctors + 1), 
                weights=doctor_preferability
            )[0]
            preferred.append(str(preferred_doctor))
            
            # Current doctor is still random (independent of preference)
            current.append(str(random.randint(1, num_doctors)))
            
        patient_priorities = list(range(1, num_patients + 1))
        random.shuffle(patient_priorities)
        priorities.append(",".join(map(str, patient_priorities)))
        
        f.write(",".join(preferred) + "\n")
        f.write(",".join(current) + "\n")
        f.write(";".join(priorities) + "\n")


if __name__ == "__main__":
    patients = int(input("How many Patients?: \n"))
    doctors = int(input("How many Doctors?: \n"))
    generate_ttc_test_data(patients, doctors, f"data/test_{patients}_patient_{doctors}_doctors.txt")
    print("Generated test files")

