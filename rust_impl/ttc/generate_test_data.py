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


def generate_district_based_test_data(num_patients, num_doctors, num_districts, filename):
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
    
    # Assign patients to districts
    for patient in tqdm(range(num_patients), desc="Assigning patients to districts"):
        district = random.choices(range(num_districts), weights=district_sizes)[0]
        patient_districts.append(district)
    
    # Assign doctors to districts
    for doctor in tqdm(range(num_doctors), desc="Assigning doctors to districts"):
        district = random.choices(range(num_districts), weights=district_sizes)[0]
        doctor_districts.append(district)
    
    doctor_preferability = [1 / num_doctors] * num_doctors
    cross_district_prob = float(input("Cross district prob in %: "))
    
    with open(filename, 'w') as f:
        f.write(f"{num_patients},{num_doctors}\n")
        
        preferred = []
        current = []
        priorities = []
        
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
            
            if cross_district_prob == 0.0:
                same_district_doctors = [
                    doc for doc in range(num_doctors)
                    if doctor_districts[doc] == patient_district
                ]
                if same_district_doctors:
                    current_doctor_idx = random.choice(same_district_doctors)
                    current.append(str(current_doctor_idx + 1))
                else:
                    current.append(str(random.randint(1, num_doctors)))
            else:
                current.append(str(random.randint(1, num_doctors)))
            
        patient_priorities = list(range(1, num_patients + 1))
        random.shuffle(patient_priorities)
        priorities.append(",".join(map(str, patient_priorities)))
        
        f.write(",".join(preferred) + "\n")
        f.write(",".join(current) + "\n")
        f.write(";".join(priorities) + "\n")


if __name__ == "__main__":
    print("Choose generator type:")
    print("1. Original generator")
    print("2. District-based generator")
    choice = input("Enter choice (1 or 2): ").strip()
    
    patients = int(input("How many Patients?: "))
    doctors = int(input("How many Doctors?: "))
    
    if choice == "2":
        districts = int(input("How many Districts?: "))
        filename = f"data/test_{patients}_patient_{doctors}_doctors_{districts}_districts.txt"
        generate_district_based_test_data(patients, doctors, districts, filename)
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

