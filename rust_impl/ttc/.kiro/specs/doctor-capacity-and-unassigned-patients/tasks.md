# Implementation Plan

- [x] 1. Extend core data models for capacity and unassigned patients
  - Modify `Patient` struct to support optional current doctor assignment
  - Add capacity field and assigned patient tracking to `Doctor` struct
  - Implement helper methods for capacity calculations and unassigned patient detection
  - _Requirements: 1.1, 2.1, 2.2, 4.3_

- [x] 2. Enhance graph node representation
  - [x] 2.1 Add new `GraphNode` enum variants for dummy doctor and capacity slots
    - Extend existing `GraphNode` enum with `DummyDoctor` and `CapacitySlot` variants
    - Update all pattern matching code to handle new node types
    - _Requirements: 1.2, 2.4_

  - [x] 2.2 Update TTCSCCSolverV2 struct for enhanced node tracking
    - Add fields for capacity slot indexing and dummy doctor tracking
    - Modify `index_to_node` to track node types instead of just patient IDs
    - _Requirements: 1.2, 2.4_

- [x] 3. Implement enhanced input data parsing
  - [x] 3.1 Extend `parse_data_file` function to handle capacity information
    - Add support for optional fifth line containing doctor capacities
    - Handle unassigned patients (current_doctor = 0 or null)
    - Maintain backward compatibility with existing data format
    - _Requirements: 4.1, 4.2, 4.3_

  - [x] 3.2 Update doctor initialization with capacity constraints
    - Set default capacity based on current patient assignments for backward compatibility
    - Validate that current assignments don't exceed specified capacity
    - Initialize assigned patient tracking for each doctor
    - _Requirements: 2.1, 2.2, 4.2_

- [ ] 4. Enhance graph building algorithm
  - [ ] 4.1 Implement dummy doctor node creation logic
    - Create dummy doctor node only when unassigned patients exist
    - Add edges from dummy doctor to all unassigned patients
    - Track dummy doctor index for cycle processing
    - _Requirements: 1.2, 1.3_

  - [ ] 4.2 Implement capacity slot node generation
    - Generate capacity slot nodes for each available doctor position
    - Create edges from capacity slots to dummy doctor
    - Optimize slot creation to prevent excessive graph expansion
    - _Requirements: 2.3, 2.4, 5.1, 5.2_

  - [ ] 4.3 Update `build_adjacency_list` method in TTCSCCSolverV2
    - Integrate dummy doctor and capacity slot creation into existing graph building
    - Maintain existing patient-to-doctor and doctor-to-patient edge logic
    - Add new edge types for capacity management
    - _Requirements: 1.2, 1.3, 2.3, 2.4_

- [ ] 5. Enhance cycle detection and execution
  - [ ] 5.1 Update cycle detection to handle new node types
    - Modify DFS cycle finding to traverse through dummy doctor and capacity slots
    - Ensure cycles involving dummy doctor are valid and executable
    - Handle edge cases where dummy doctor becomes isolated
    - _Requirements: 3.1, 3.2_

  - [ ] 5.2 Implement enhanced cycle execution logic
    - Update cycle execution to handle assignments from unassigned patients to available capacity
    - Update doctor capacity tracking after successful cycle execution
    - Remove filled capacity slots from graph after assignment
    - _Requirements: 3.2, 3.3, 3.4_

- [ ] 6. Update benchmarking and testing infrastructure
  - [ ] 6.1 Extend benchmarking system for capacity-aware scenarios
    - Update `Benchmarker` to handle datasets with unassigned patients
    - Add timing measurements for dummy doctor processing
    - Ensure performance characteristics remain acceptable
    - _Requirements: 5.3, 5.4_

  - [ ]* 6.2 Create test datasets with capacity constraints
    - Generate test data files with unassigned patients and doctor capacities
    - Create edge case scenarios (no unassigned patients, no available capacity)
    - Validate backward compatibility with existing benchmark datasets
    - _Requirements: 4.4, 5.4_

- [ ] 7. Implement validation and error handling
  - [ ] 7.1 Add capacity constraint validation
    - Validate input data for capacity consistency
    - Implement error handling for invalid capacity configurations
    - Add runtime checks to prevent capacity violations during cycle execution
    - _Requirements: 2.5_

  - [ ]* 7.2 Add comprehensive testing for new functionality
    - Write unit tests for capacity calculations and dummy doctor logic
    - Create integration tests for complete cycles involving unassigned patients
    - Test performance impact and memory usage with capacity slots
    - _Requirements: 3.1, 3.2, 5.1, 5.2_

- [ ] 8. Integration and compatibility updates
  - [ ] 8.1 Update existing algorithms (DFS, SCC V1) to support new features
    - Extend DFS algorithm to handle dummy doctor and capacity slots
    - Update SCC V1 solver with same capacity-aware logic
    - Ensure all three algorithms produce consistent results
    - _Requirements: 4.4_

  - [ ] 8.2 Final integration and performance validation
    - Run comprehensive benchmarks comparing all algorithms with new features
    - Validate that existing performance characteristics are maintained
    - Ensure backward compatibility with all existing test cases
    - _Requirements: 4.4, 5.4_