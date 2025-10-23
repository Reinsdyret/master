use ttc::parse_data_file;

fn main() {
    // Test parsing with capacity information
    match parse_data_file("test_capacity_parsing.txt") {
        Ok((patients, doctors)) => {
            println!("Successfully parsed {} patients and {} doctors", patients.len(), doctors.len());
            
            // Check patients
            for patient in &patients {
                println!("Patient {}: preferred={}, current={:?}, unassigned={}", 
                    patient.id, patient.preferred_doctor, patient.current_doctor, patient.is_unassigned());
            }
            
            // Check doctors
            for doctor in &doctors {
                println!("Doctor {}: capacity={}, assigned={:?}, available={}", 
                    doctor.id, doctor.capacity, doctor.assigned_patients, doctor.available_capacity());
            }
        }
        Err(e) => {
            println!("Error parsing file: {}", e);
        }
    }
}