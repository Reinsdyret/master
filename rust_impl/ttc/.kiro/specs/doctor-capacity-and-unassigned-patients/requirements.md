# Requirements Document

## Introduction

This feature extends the Top Trading Cycles (TTC) algorithm to handle patients without current doctor assignments and introduces doctor capacity constraints. The system will use a dummy doctor mechanism to represent unassigned patients and available capacity slots, enabling the algorithm to find cycles that include previously unassigned patients.

## Glossary

- **TTC_System**: The Top Trading Cycles algorithm implementation
- **Patient**: An individual seeking medical care assignment
- **Doctor**: A medical provider with limited capacity
- **Dummy_Doctor**: A virtual doctor node representing the pool of unassigned patients
- **Capacity**: The maximum number of patients a doctor can serve
- **Available_Capacity**: The number of open slots a doctor currently has
- **Unassigned_Patient**: A patient without a current doctor assignment
- **Capacity_Slot**: A virtual node representing one available position at a doctor

## Requirements

### Requirement 1

**User Story:** As a healthcare administrator, I want to include unassigned patients in the TTC algorithm, so that they can be matched with available doctor capacity through trading cycles.

#### Acceptance Criteria

1. WHEN the TTC_System processes patient data, THE TTC_System SHALL identify patients with no current doctor assignment
2. THE TTC_System SHALL create a Dummy_Doctor node to represent the pool of unassigned patients
3. THE TTC_System SHALL create edges from the Dummy_Doctor to all Unassigned_Patient nodes
4. THE TTC_System SHALL allow Unassigned_Patient nodes to have preferred doctor assignments

### Requirement 2

**User Story:** As a healthcare administrator, I want doctors to have capacity limits, so that the system respects real-world constraints on patient loads.

#### Acceptance Criteria

1. THE TTC_System SHALL accept capacity information for each Doctor
2. THE TTC_System SHALL calculate Available_Capacity as the difference between total capacity and currently assigned patients
3. WHERE a Doctor has Available_Capacity greater than zero, THE TTC_System SHALL create Capacity_Slot nodes for each available position
4. THE TTC_System SHALL create edges from each Capacity_Slot to the Dummy_Doctor
5. THE TTC_System SHALL ensure no Doctor exceeds their specified capacity during cycle execution

### Requirement 3

**User Story:** As a healthcare administrator, I want the algorithm to find cycles that can assign unassigned patients to available capacity, so that we maximize patient-doctor matching efficiency.

#### Acceptance Criteria

1. THE TTC_System SHALL detect cycles that include paths through the Dummy_Doctor
2. WHEN executing a cycle containing the Dummy_Doctor, THE TTC_System SHALL assign unassigned patients to available doctor capacity
3. THE TTC_System SHALL update doctor capacity counts after cycle execution
4. THE TTC_System SHALL remove filled Capacity_Slot nodes from the graph after assignment

### Requirement 4

**User Story:** As a system developer, I want the enhanced algorithm to maintain compatibility with existing data formats, so that current test cases continue to work.

#### Acceptance Criteria

1. THE TTC_System SHALL accept an optional capacity field in the input data format
2. WHERE no capacity is specified for a Doctor, THE TTC_System SHALL default to the current number of assigned patients as capacity
3. THE TTC_System SHALL support a special value (e.g., 0 or null) to indicate an Unassigned_Patient in the current doctor field
4. THE TTC_System SHALL maintain backward compatibility with existing benchmark datasets

### Requirement 5

**User Story:** As a performance analyst, I want the enhanced algorithm to maintain reasonable performance characteristics, so that large-scale problems remain solvable.

#### Acceptance Criteria

1. THE TTC_System SHALL limit the number of Capacity_Slot nodes to prevent excessive graph expansion
2. THE TTC_System SHALL reuse Capacity_Slot nodes where possible to minimize memory overhead
3. THE TTC_System SHALL provide timing breakdowns that include dummy doctor processing time
4. THE TTC_System SHALL maintain O(n) or better complexity for graph construction where n is the number of active patients