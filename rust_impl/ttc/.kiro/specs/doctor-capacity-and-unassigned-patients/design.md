# Design Document

## Overview

This design extends the TTC algorithm to handle doctor capacity constraints and unassigned patients through a dummy doctor mechanism. The approach creates virtual nodes to represent available capacity and uses a special dummy doctor to pool unassigned patients, enabling cycles that can match unassigned patients with available capacity.

## Architecture

### Graph Structure Enhancement

The enhanced graph will contain four types of nodes:

1. **Patient Nodes**: Existing patient entities (both assigned and unassigned)
2. **Doctor Nodes**: Existing doctor entities with capacity tracking
3. **Dummy Doctor Node**: Single virtual doctor representing the unassigned patient pool
4. **Capacity Slot Nodes**: Virtual nodes representing individual available positions at doctors

### Node Relationships

```
Patient (assigned) → Preferred Doctor
Patient (unassigned) → Preferred Doctor
Doctor → Assigned Patients
Capacity Slot → Dummy Doctor
Dummy Doctor → Unassigned Patients
```

## Components and Interfaces

### Enhanced Data Models

#### Doctor Enhancement
```rust
pub struct Doctor {
    pub id: usize,
    pub capacity: usize,                    // NEW: Maximum patient capacity
    pub switching_patients: Vec<Patient>,   // Existing
    pub assigned_patients: Vec<usize>,      // NEW: All currently assigned patient IDs
}

impl Doctor {
    pub fn available_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.assigned_patients.len())
    }
    
    pub fn has_available_capacity(&self) -> bool {
        self.available_capacity() > 0
    }
}
```

#### Patient Enhancement
```rust
pub struct Patient {
    pub id: usize,
    pub priority: usize,
    pub preferred_doctor: usize,
    pub current_doctor: Option<usize>,      // CHANGED: Now optional for unassigned patients
    pub wants_to_switch: bool,
    pub is_stuck: bool,
}

impl Patient {
    pub fn is_unassigned(&self) -> bool {
        self.current_doctor.is_none()
    }
}
```

#### New Node Types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GraphNode {
    Patient(usize),
    Doctor(usize),
    DummyDoctor,                           // NEW: Single dummy doctor
    CapacitySlot { doctor_id: usize, slot_id: usize }, // NEW: Available capacity
}
```

### Enhanced Graph Building

#### TTCSCCSolverV2 Modifications

```rust
pub struct TTCSCCSolverV2 {
    stats: SCCStats,
    tarjan: TarjanSCC,
    graph: Vec<Vec<usize>>,
    index_to_node: Vec<Option<GraphNode>>,  // CHANGED: Track node types
    patient_index: FxHashMap<usize, usize>,
    doctor_index: FxHashMap<usize, usize>,
    capacity_slot_index: FxHashMap<(usize, usize), usize>, // NEW: (doctor_id, slot_id) -> index
    dummy_doctor_index: Option<usize>,      // NEW: Index of dummy doctor
    used_indices: Vec<usize>,
}
```

## Data Models

### Input Format Enhancement

Extend the existing CSV format to support capacity:

```
num_patients,num_doctors
preferred_doctor_1,preferred_doctor_2,...
current_doctor_1,current_doctor_2,...     # 0 or null indicates unassigned
priority_1,priority_2,...
capacity_1,capacity_2,...                 # NEW: Optional fifth line for doctor capacities
```

### Capacity Slot Management

Each doctor with available capacity will generate capacity slot nodes:
- Doctor with capacity 5 and 3 assigned patients → 2 capacity slot nodes
- Capacity slots are numbered: `CapacitySlot { doctor_id: 1, slot_id: 0 }`, `CapacitySlot { doctor_id: 1, slot_id: 1 }`

## Error Handling

### Capacity Validation
- Validate that current assignments don't exceed doctor capacity
- Handle edge cases where capacity is less than current assignments
- Provide clear error messages for invalid capacity configurations

### Dummy Doctor Edge Cases
- Handle scenarios with no unassigned patients (dummy doctor not needed)
- Handle scenarios with no available capacity (dummy doctor becomes isolated)
- Ensure dummy doctor doesn't participate in invalid cycles

## Testing Strategy

### Unit Tests
- Test capacity calculation logic
- Test dummy doctor node creation
- Test capacity slot generation
- Test graph building with mixed assigned/unassigned patients

### Integration Tests
- Test complete cycles involving unassigned patients
- Test capacity constraint enforcement
- Test backward compatibility with existing datasets

### Performance Tests
- Benchmark graph size increase with capacity slots
- Measure impact on SCC finding performance
- Compare memory usage with and without capacity features

## Implementation Phases

### Phase 1: Data Model Extensions
1. Extend `Patient` struct to support optional current doctor
2. Extend `Doctor` struct to include capacity and assigned patient tracking
3. Add new `GraphNode` variants for dummy doctor and capacity slots
4. Update parsing logic to handle new input format

### Phase 2: Graph Building Enhancement
1. Modify `build_adjacency_list` to create dummy doctor node
2. Add capacity slot node generation
3. Implement edges from capacity slots to dummy doctor
4. Implement edges from dummy doctor to unassigned patients

### Phase 3: Cycle Detection and Execution
1. Update cycle detection to handle new node types
2. Modify cycle execution to handle capacity assignments
3. Update capacity tracking after cycle execution
4. Remove filled capacity slots from graph

### Phase 4: Integration and Testing
1. Update benchmarking to support new features
2. Create test datasets with unassigned patients and capacity constraints
3. Validate algorithm correctness with new scenarios
4. Performance testing and optimization

## Backward Compatibility

- Existing datasets without capacity information will work unchanged
- Default capacity equals current patient count for existing doctors
- Current doctor field of 0 or empty string indicates unassigned patient
- All existing algorithms (DFS, SCC V1) can be extended with same pattern