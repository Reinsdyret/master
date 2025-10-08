#include <iostream>
#include <fstream>
#include <sstream>
#include <vector>
#include <stack>
#include <algorithm>
#include <chrono>
#include <string>

using namespace std;

struct Patient {
    int id;
    int preferred_doctor;
    int current_doctor;
    int priority;
    bool wants_to_switch;
};

int n, foundat = 1;
vector<vector<int>> graph, scc;
vector<int> disc, low;
vector<bool> onstack;

void tarjan(int u) {
    static stack<int> st;

    disc[u] = low[u] = foundat++;
    st.push(u);
    onstack[u] = true;

    for (auto i : graph[u]) {
        if (disc[i] == -1) {
            tarjan(i);
            low[u] = min(low[u], low[i]);
        }
        else if (onstack[i])
            low[u] = min(low[u], disc[i]);
    }

    if (disc[u] == low[u]) {
        vector<int> scctem;
        while (1) {
            int v = st.top();
            st.pop();
            onstack[v] = false;
            scctem.push_back(v);
            if (u == v)
                break;
        }
        scc.push_back(scctem);
    }
}

vector<vector<int>> find_comps() {
    // Initialize
    fill(onstack.begin(), onstack.end(), false);
    fill(disc.begin(), disc.end(), -1);
    fill(low.begin(), low.end(), 0);
    scc.clear();
    foundat = 1;

    for (int v = 0; v < n; ++v) {
        if (disc[v] == -1) {
            tarjan(v);
        }
    }

    return scc;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        cerr << "Usage: " << argv[0] << " <data_file>" << endl;
        return 1;
    }

    string filename = argv[1];
    ifstream file(filename);

    if (!file.is_open()) {
        cerr << "Error opening file: " << filename << endl;
        return 1;
    }

    // Parse first line: num_patients,num_doctors
    string line;
    getline(file, line);
    stringstream ss(line);
    int num_patients, num_doctors;
    char comma;
    ss >> num_patients >> comma >> num_doctors;

    cout << "Loading data: " << num_patients << " patients, " << num_doctors << " doctors" << endl;

    // Parse preferred doctors
    getline(file, line);
    vector<int> preferred(num_patients + 1);
    ss.clear();
    ss.str(line);
    for (int i = 1; i <= num_patients; i++) {
        ss >> preferred[i];
        if (i < num_patients) ss >> comma;
    }

    // Parse current doctors
    getline(file, line);
    vector<int> current(num_patients + 1);
    ss.clear();
    ss.str(line);
    for (int i = 1; i <= num_patients; i++) {
        ss >> current[i];
        if (i < num_patients) ss >> comma;
    }

    // Parse priorities
    getline(file, line);
    vector<int> priorities(num_patients + 1);
    ss.clear();
    ss.str(line);
    for (int i = 1; i <= num_patients; i++) {
        ss >> priorities[i];
        if (i < num_patients) ss >> comma;
    }

    file.close();

    // Create patients
    vector<Patient> patients(num_patients + 1);
    for (int i = 1; i <= num_patients; i++) {
        patients[i].id = i;
        patients[i].preferred_doctor = preferred[i];
        patients[i].current_doctor = current[i];
        patients[i].priority = priorities[i];
        patients[i].wants_to_switch = (preferred[i] != current[i]);
    }

    cout << "Building patient-to-patient graph..." << endl;

    // Build adjacency list: patient -> patient (who has preferred doctor)
    // Map from doctor to patients who currently have that doctor
    vector<vector<int>> doctor_to_patients(num_doctors + 1);
    for (int i = 1; i <= num_patients; i++) {
        if (patients[i].wants_to_switch) {
            doctor_to_patients[patients[i].current_doctor].push_back(i);
        }
    }

    // Build graph: patient i -> patient j if i wants j's doctor
    n = num_patients + 1;
    graph.resize(n);
    disc.resize(n);
    low.resize(n);
    onstack.resize(n);

    int edge_count = 0;
    int active_patients = 0;

    for (int i = 1; i <= num_patients; i++) {
        if (!patients[i].wants_to_switch) continue;

        active_patients++;
        int wanted_doctor = patients[i].preferred_doctor;

        // Add edges to all patients who have the wanted doctor
        for (int j : doctor_to_patients[wanted_doctor]) {
            if (i != j && patients[j].wants_to_switch) {
                graph[i].push_back(j);
                edge_count++;
            }
        }
    }

    cout << "Graph built: " << active_patients << " active patients, " << edge_count << " edges" << endl;

    // Find SCCs using Tarjan's algorithm
    cout << "Finding SCCs..." << endl;
    auto start = chrono::high_resolution_clock::now();

    vector<vector<int>> components = find_comps();

    auto end = chrono::high_resolution_clock::now();
    auto duration = chrono::duration_cast<chrono::milliseconds>(end - start);

    // Count components with size > 1
    int large_comps = 0;
    int largest_comp = 0;
    for (const auto& component : components) {
        if (component.size() > 1) {
            large_comps++;
        }
        if ((int)component.size() > largest_comp) {
            largest_comp = component.size();
        }
    }

    cout << "\n=== Results ===" << endl;
    cout << "Total SCCs: " << components.size() << endl;
    cout << "SCCs with size > 1: " << large_comps << endl;
    cout << "Largest SCC size: " << largest_comp << endl;
    cout << "Time: " << duration.count() << " ms" << endl;

    return 0;
}
