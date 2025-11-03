import random
from tqdm import tqdm

def generate_ttc_test_data(num_patients, num_doctors, filename):
    # Generate random preferability scores for each doctor (higher = more preferred)
    doctor_preferability = [random.uniform(0.1, 10.0) for _ in range(num_doctors)]
    
    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")
        
        preferred = []
        current = []
        priorities = []
        
        # Use tqdm to show progress on patients
        for patient in tqdm(range(num_patients), desc="Generating patients (original)"):
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


def generate_chain_district_test_data(num_patients, num_doctors, num_districts, filename):
    """
    Generate test data where districts form a directed chain: 1→2→3→...
    Patients in district i only want doctors in district i+1 (or same district).
    This creates a pathological case for DFS with no cycles between districts.
    """
    # Equal-sized districts
    district_sizes = [1.0 / num_districts] * num_districts

    # Assign patients and doctors evenly to districts
    patient_districts = []
    doctor_districts = []

    patients_per_district = num_patients // num_districts
    doctors_per_district = num_doctors // num_districts

    for district in range(num_districts):
        for _ in range(patients_per_district):
            patient_districts.append(district)
        for _ in range(doctors_per_district):
            doctor_districts.append(district)

    # Handle remainder
    for _ in range(num_patients - len(patient_districts)):
        patient_districts.append(num_districts - 1)
    for _ in range(num_doctors - len(doctor_districts)):
        doctor_districts.append(num_districts - 1)

    # Shuffle to mix up assignment
    random.shuffle(patient_districts)
    random.shuffle(doctor_districts)

    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")

        preferred = []
        current = []

        # Generate preferences with chain structure
        for patient in tqdm(range(num_patients), desc="Generating chain-structured patients"):
            patient_district = patient_districts[patient]

            # 70% chance: prefer doctor in NEXT district (creating the chain)
            # 30% chance: prefer doctor in SAME district (local cycles possible)
            if patient_district < num_districts - 1 and random.random() < 0.7:
                target_district = patient_district + 1
            else:
                target_district = patient_district

            # Find doctors in target district
            target_doctors = [
                doc for doc in range(num_doctors)
                if doctor_districts[doc] == target_district
            ]

            if target_doctors:
                preferred_doctor_idx = random.choice(target_doctors)
                preferred.append(str(preferred_doctor_idx + 1))
            else:
                preferred.append(str(random.randint(1, num_doctors)))

            # Current doctor is always in same district
            same_district_doctors = [
                doc for doc in range(num_doctors)
                if doctor_districts[doc] == patient_district
            ]
            if same_district_doctors:
                current_doctor_idx = random.choice(same_district_doctors)
                current.append(str(current_doctor_idx + 1))
            else:
                current.append(str(random.randint(1, num_doctors)))

        # Randomize priorities
        patient_priorities = list(range(1, num_patients + 1))
        random.shuffle(patient_priorities)

        f.write(",".join(preferred) + "\n")
        f.write(",".join(current) + "\n")
        f.write(",".join(map(str, patient_priorities)) + "\n")


def generate_district_based_test_data(num_patients, num_doctors, num_no_doctor_patients, num_districts, cross_district_prob, filename):
    # Create districts with varying sizes
    district_sizes = []
    remaining_capacity = 1.0
    for i in range(num_districts - 1):
        size = random.uniform(0.1, remaining_capacity * 0.6)
        district_sizes.append(size)
        remaining_capacity -= size
    district_sizes.append(remaining_capacity)

    # Assign patients and doctors to districts
    patient_districts = []
    doctor_districts = []
    doctor_num_patients = [0] * num_doctors

    # Assign patients to districts
    for patient in tqdm(range(num_patients), desc="Assigning patients to districts"):
        district = random.choices(range(num_districts), weights=district_sizes)[0]
        patient_districts.append(district)

    # Assign doctors to districts
    for doctor in tqdm(range(num_doctors), desc="Assigning doctors to districts"):
        district = random.choices(range(num_districts), weights=district_sizes)[0]
        doctor_districts.append(district)
    
    doctor_preferability = [1 / num_doctors] * num_doctors
    
    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")
        
        preferred = []
        current = []
        priorities = []
        
        # Randomly select which patients will have no doctor
        patients_without_doctor = set(random.sample(range(num_patients), num_no_doctor_patients))

        # Patients loop with progress bar
        for patient in tqdm(range(num_patients), desc="Generating patients (district-based)"):
            patient_district = patient_districts[patient]

            if random.random() < cross_district_prob:
                available_doctors = list(range(num_doctors))
            else:
                available_doctors = [
                    doc for doc in range(num_doctors)
                    if doctor_districts[doc] == patient_district
                ]
                if not available_doctors:
                    available_doctors = list(range(num_doctors))

            weights = [doctor_preferability[doc] for doc in available_doctors]
            preferred_doctor_idx = random.choices(available_doctors, weights=weights)[0]
            preferred.append(str(preferred_doctor_idx + 1))

            # Assign no doctor (0) if this patient is in the no-doctor set
            if patient in patients_without_doctor:
                current.append("0")
            else:
                current_doctor_idx = str(random.randint(1, num_doctors))

                if cross_district_prob == 0.0:
                    same_district_doctors = [
                        doc for doc in range(num_doctors)
                        if doctor_districts[doc] == patient_district
                    ]
                    if same_district_doctors:
                        current_doctor_idx = random.choice(same_district_doctors) + 1

                current.append(str(current_doctor_idx))
                doctor_num_patients[int(current_doctor_idx) - 1] += 1

        # Calculate doctor capacities
        # Each doctor gets current patients + random extra capacity (0-5)
        doctor_capacities = []
        for i in range(num_doctors):
            current_patients = doctor_num_patients[i]
            extra = random.randint(0, 5)
            capacity = current_patients + extra
            doctor_capacities.append(capacity)

        
        doctor_num_patients_str = [str(num) for num in doctor_num_patients]

        doctor_capacities_str = [str(cap) for cap in doctor_capacities]

        patient_priorities = list(range(1, num_patients + 1))
        random.shuffle(patient_priorities)

        f.write(",".join(preferred) + "\n")
        f.write(",".join(current) + "\n")
        f.write(",".join(map(str, patient_priorities)) + "\n")
        f.write(",".join(doctor_num_patients_str) + "\n")
        f.write(",".join(doctor_capacities_str) + "\n")
        print("done")


if __name__ == "__main__":
    print("Choose generator type:")
    print("1. Original generator")
    print("2. District-based generator")
    print("3. Chain district generator (1→2→3, pathological for DFS)")
    choice = input("Enter choice (1, 2, or 3): ").strip()

    patients = int(input("How many Patients?: "))
    doctors = int(input("How many Doctors?: "))
    no_doctor_patients = int(input("How many patients with no doctor?: "))

    if choice == "3":
        districts = int(input("How many Districts?: "))
        filename = f"data/test_{patients}_patient_{doctors}_doctors_{districts}_districts_chain.txt"
        generate_chain_district_test_data(patients, doctors, districts, filename)
        print(f"Generated chain district test file: {filename}")
    elif choice == "2":
        districts = int(input("How many Districts?: "))
        cross_district_prob = float(input("Cross district probability (0.0-1.0): "))
        filename = f"data/test_{patients}_patient_{doctors}_doctors_{districts}_districts_{cross_district_prob}_prob.txt"
        generate_district_based_test_data(patients, doctors, no_doctor_patients, districts, cross_district_prob, filename)
        print(f"Generated district-based test file: {filename}")
    else:
        filename = f"data/test_{patients}_patient_{doctors}_doctors.txt"
        generate_ttc_test_data(patients, doctors, filename)
        print(f"Generated original test file: {filename}")


# import random
# from tqdm import tqdm

# def generate_ttc_test_data(num_patients, num_doctors, filename):
#     # Generate random preferability scores for each doctor (higher = more preferred)
#     doctor_preferability = [random.uniform(0.1, 10.0) for _ in range(num_doctors)]
    
#     with open(filename, 'w') as f:
#         f.write(f"{num_patients},{num_doctors}\n")
        
#         preferred = []
#         current = []
#         priorities = []
        
#         for patient in range(num_patients):
#             # Choose preferred doctor based on preferability weights
#             preferred_doctor = random.choices(
#                 range(1, num_doctors + 1), 
#                 weights=doctor_preferability
#             )[0]
#             preferred.append(str(preferred_doctor))
            
#             # Current doctor is still random (independent of preference)
#             current.append(str(random.randint(1, num_doctors)))
            
#         patient_priorities = list(range(1, num_patients + 1))
#         random.shuffle(patient_priorities)
#         priorities.append(",".join(map(str, patient_priorities)))
        
#         f.write(",".join(preferred) + "\n")
#         f.write(",".join(current) + "\n")
#         f.write(";".join(priorities) + "\n")


# def generate_district_based_test_data(num_patients, num_doctors, num_districts, filename):
#     # Create districts with varying sizes
#     district_sizes = []
#     remaining_capacity = 1.0
#     for i in range(num_districts - 1):
#         size = random.uniform(0.1, remaining_capacity * 0.6)
#         district_sizes.append(size)
#         remaining_capacity -= size
#     district_sizes.append(remaining_capacity)
    
#     # Assign patients and doctors to districts
#     patient_districts = []
#     doctor_districts = []
    
#     # Assign patients to districts based on district sizes
#     for patient in range(num_patients):
#         district = random.choices(range(num_districts), weights=district_sizes)[0]
#         patient_districts.append(district)
    
#     # Assign doctors to districts based on district sizes
#     for doctor in range(num_doctors):
#         district = random.choices(range(num_districts), weights=district_sizes)[0]
#         doctor_districts.append(district)
    
#     # Generate base preferability scores for each doctor
#     #doctor_preferability = [random.uniform(0.1, 10.0) for _ in range(num_doctors)]
#     doctor_preferability = [1 / num_doctors] * num_doctors
    
#     # Cross-district preference probability (rare occurrence)
#     cross_district_prob = 0.0
    
#     with open(filename, 'w') as f:
#         f.write(f"{num_patients},{num_doctors}\n")
        
#         preferred = []
#         current = []
#         priorities = []
        
#         for patient in range(num_patients):
#             patient_district = patient_districts[patient]
            
#             # Decide if patient prefers within district or crosses to another district
#             if random.random() < cross_district_prob:
#                 # Rare cross-district preference
#                 available_doctors = list(range(num_doctors))
#             else:
#                 # Prefer doctors in same district
#                 available_doctors = [
#                     doc for doc in range(num_doctors) 
#                     if doctor_districts[doc] == patient_district
#                 ]
                
#                 # If no doctors in same district, fall back to all doctors
#                 if not available_doctors:
#                     available_doctors = list(range(num_doctors))
            
#             # Create weights for available doctors
#             weights = [doctor_preferability[doc] for doc in available_doctors]
            
#             # Choose preferred doctor
#             preferred_doctor_idx = random.choices(available_doctors, weights=weights)[0]
#             preferred.append(str(preferred_doctor_idx + 1))  # 1-indexed
            
#             # Current doctor assignment - respect district boundaries when cross_district_prob is 0
#             if cross_district_prob == 0.0:
#                 # Only assign doctors from same district
#                 same_district_doctors = [
#                     doc for doc in range(num_doctors)
#                     if doctor_districts[doc] == patient_district
#                 ]
#                 if same_district_doctors:
#                     current_doctor_idx = random.choice(same_district_doctors)
#                     current.append(str(current_doctor_idx + 1))  # 1-indexed
#                 else:
#                     # Fallback if no doctors in same district
#                     current.append(str(random.randint(1, num_doctors)))
#             else:
#                 # Original behavior - any doctor can be current doctor
#                 current.append(str(random.randint(1, num_doctors)))
            
#         patient_priorities = list(range(1, num_patients + 1))
#         random.shuffle(patient_priorities)
#         priorities.append(",".join(map(str, patient_priorities)))
        
#         f.write(",".join(preferred) + "\n")
#         f.write(",".join(current) + "\n")
#         f.write(";".join(priorities) + "\n")


# if __name__ == "__main__":
#     print("Choose generator type:")
#     print("1. Original generator")
#     print("2. District-based generator")
#     choice = input("Enter choice (1 or 2): ").strip()
    
#     patients = int(input("How many Patients?: "))
#     doctors = int(input("How many Doctors?: "))
    
#     if choice == "2":
#         districts = int(input("How many Districts?: "))
#         filename = f"data/test_{patients}_patient_{doctors}_doctors_{districts}_districts.txt"
#         generate_district_based_test_data(patients, doctors, districts, filename)
#         print(f"Generated district-based test file: {filename}")
#     else:
#         filename = f"data/test_{patients}_patient_{doctors}_doctors.txt"
#         generate_ttc_test_data(patients, doctors, filename)
#         print(f"Generated original test file: {filename}")

