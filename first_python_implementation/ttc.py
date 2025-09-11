# TTC algorithm
# Each person has a preference list [x,y]. X is their preferred new doctor while y is their current doctor.
# Then in the graph all people with preference are nodes and all doctors are nodes
# For each person with preference create edge between that person and the doctor they prefer 
# also create an edge from the doctor they currently have and the person.
# Then find cycles in the graph, i dont know exactly what cycles and how. Maybe taking a random subset and then largest. Or just a random one
# Let all people get their preferred doctor in the cycle and remove the people and doctors from the graph or remove all edges incident to them (way to do this is to update all nodes pointing to them because all people should go down in the prefence list)

def ttc(graph):
    cycles = []
    visited = set()
    
    for node in graph:
        if node not in visited:
            dfs(graph, node, visited, [], cycles)
    
    return cycles


def dfs(graph, current_node, visited, path, cycles):
    if current_node in path:
        cycle_start = path.index(current_node)
        cycle = path[cycle_start:] + [current_node]
        cycles.append(cycle)
        return
    
    if current_node in visited:
        return
    
    visited.add(current_node)
    path.append(current_node)
    
    if current_node in graph:
        for neighbor in graph[current_node]:
            dfs(graph, neighbor, visited, path, cycles)
    
    path.pop()


graph = {}
num_patients = int(input("How many patients?: \n"))
num_doctors = int(input("How many doctors?: \n"))

with open(f"data/test_{num_patients}_patient_{num_doctors}_doctors.txt", 'r') as f:
    num_patients, num_doctors = map(int, f.readline().strip().split(','))
    
    for i in range(num_patients): 
        graph[i] = []
    for i in range(num_doctors): 
        graph[f"doc_{i+1}"] = []
    
    preferred = f.readline().strip().split(',')
    current = f.readline().strip().split(',')
    
    for patient in range(num_patients):
        if preferred[patient] != "NA":
            preferred_doctor = f"doc_{preferred[patient]}"
            graph[patient].append(preferred_doctor)
        
        if current[patient] != "NA":
            current_doctor = f"doc_{current[patient]}"
            graph[current_doctor].append(patient)


cycles = ttc(graph)
print(graph)
print(f"{len(cycles)} CYCLES: ")
for c in cycles: print(c)



