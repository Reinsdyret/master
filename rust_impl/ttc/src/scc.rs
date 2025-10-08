#[derive(Debug, Clone)]
pub struct TarjanSCC {
    disc: Vec<i32>,
    low: Vec<i32>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    components: Vec<Vec<usize>>,
}

impl TarjanSCC {
    pub fn new() -> Self {
        TarjanSCC {
            disc: Vec::new(),
            low: Vec::new(),
            on_stack: Vec::new(),
            stack: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn find_sccs(&mut self, graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let n = graph.len();
        self.ensure_capacity(n);
        self.reset_state(n);

        let mut next_index = 1;
        for v in 0..n {
            if self.disc[v] == -1 {
                self.dfs(v, graph, &mut next_index);
            }
        }

        std::mem::take(&mut self.components)
    }

    fn ensure_capacity(&mut self, n: usize) {
        if self.disc.len() < n {
            self.disc.resize(n, -1);
            self.low.resize(n, 0);
            self.on_stack.resize(n, false);
        }
    }

    fn reset_state(&mut self, n: usize) {
        for i in 0..n {
            self.disc[i] = -1;
            self.low[i] = 0;
            self.on_stack[i] = false;
        }
        self.stack.clear();
        self.components.clear();
    }

    fn dfs(&mut self, u: usize, graph: &[Vec<usize>], next_index: &mut i32) {
        self.disc[u] = *next_index;
        self.low[u] = *next_index;
        *next_index += 1;
        self.stack.push(u);
        self.on_stack[u] = true;

        for &v in &graph[u] {
            if v >= graph.len() {
                continue;
            }
            if self.disc[v] == -1 {
                self.dfs(v, graph, next_index);
                self.low[u] = self.low[u].min(self.low[v]);
            } else if self.on_stack[v] {
                self.low[u] = self.low[u].min(self.disc[v]);
            }
        }

        if self.disc[u] == self.low[u] {
            let mut component = Vec::new();
            loop {
                let v = self.stack.pop().expect("stack underflow in Tarjan DFS");
                self.on_stack[v] = false;
                component.push(v);
                if v == u {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}
