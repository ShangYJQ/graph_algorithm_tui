use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

pub enum EdgeType {
    Single,
    Both,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: i64,
    node: i64,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Graph {
    adj: HashMap<i64, Vec<(i64, i64)>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            adj: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, u: i64, v: i64, w: i64, edge_type: EdgeType) {
        match edge_type {
            EdgeType::Single => {
                self.adj.entry(u).or_insert(Vec::new()).push((v, w));
            }

            EdgeType::Both => {
                self.adj.entry(u).or_insert(Vec::new()).push((v, w));
                self.adj.entry(v).or_insert(Vec::new()).push((u, w));
            }
        }
    }

    pub fn dijkstra(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>, HashMap<i64, i64>, HashMap<i64, i64>) {
        let mut dist: HashMap<i64, i64> = HashMap::new();
        let mut parent: HashMap<i64, i64> = HashMap::new();
        let mut visited_nodes: Vec<i64> = Vec::new();
        let mut visited_edges: Vec<(i64, i64)> = Vec::new();
        let mut processed: HashSet<i64> = HashSet::new();

        let mut pq: BinaryHeap<State> = BinaryHeap::new();

        dist.insert(s, 0);
        pq.push(State { cost: 0, node: s });

        while !pq.is_empty() {
            let u = pq.pop();
            match u {
                Some(u) => {
                    if u.cost > *dist.get(&u.node).unwrap_or(&i64::MAX) {
                        continue;
                    }
                    if !processed.contains(&u.node) {
                        visited_nodes.push(u.node);
                        processed.insert(u.node);
                    }
                    if let Some(v_list) = self.adj.get(&u.node) {
                        for &(v, w) in v_list {
                            let cost = u.cost + w;
                            if cost < *dist.get(&v).unwrap_or(&i64::MAX) {
                                dist.insert(v, cost);
                                parent.insert(v, u.node);
                                pq.push(State { cost, node: v });
                                visited_edges.push((u.node, v));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        (visited_nodes, visited_edges, dist, parent)
    }

    pub fn prim(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>, i64) {
        let mut dist: HashMap<i64, i64> = HashMap::new();
        let mut booked: HashSet<i64> = HashSet::new();
        let mut visited_nodes: Vec<i64> = Vec::new();
        let mut visited_edges: Vec<(i64, i64)> = Vec::new();

        let mut parent: HashMap<i64, i64> = HashMap::new();
        let mut pq: BinaryHeap<State> = BinaryHeap::new();
        let mut total_cost: i64 = 0;

        dist.insert(s, 0);
        pq.push(State { cost: 0, node: s });

        while let Some(State { cost, node: u }) = pq.pop() {
            if booked.contains(&u) {
                continue;
            }
            if cost > *dist.get(&u).unwrap_or(&i64::MAX) {
                continue;
            }
            booked.insert(u);
            visited_nodes.push(u);

            if let Some(&p) = parent.get(&u) {
                visited_edges.push((p, u));
                total_cost += cost;
            }

            if let Some(v_list) = self.adj.get(&u) {
                for &(v, w) in v_list {
                    if !booked.contains(&v) && w < *dist.get(&v).unwrap_or(&i64::MAX) {
                        dist.insert(v, w);
                        parent.insert(v, u);
                        pq.push(State { cost: w, node: v });
                    }
                }
            }
        }

        (visited_nodes, visited_edges, total_cost)
    }

    pub fn bfs(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>) {
        let mut visited: HashSet<i64> = HashSet::new();
        let mut visited_nodes: Vec<i64> = Vec::new();
        let mut visited_edges: Vec<(i64, i64)> = Vec::new();
        let mut q: VecDeque<i64> = VecDeque::new();

        q.push_back(s);
        visited.insert(s);
        visited_nodes.push(s);

        while !q.is_empty() {
            let u = q.pop_front();
            match u {
                Some(u) => {
                    if let Some(v_list) = self.adj.get(&u) {
                        for &(v, _) in v_list {
                            if !visited.contains(&v) {
                                q.push_back(v);
                                visited.insert(v);
                                visited_nodes.push(v);
                                visited_edges.push((u, v));
                            }
                        }
                    }
                }
                _ => println!("error"),
            }
        }

        (visited_nodes, visited_edges)
    }
    pub fn dfs(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>) {
        let mut visited: HashSet<i64> = HashSet::new();
        let mut visited_nodes: Vec<i64> = Vec::new();
        let mut visited_edges: Vec<(i64, i64)> = Vec::new();
        
        self.dfs_helper(s, &mut visited, &mut visited_nodes, &mut visited_edges);
        
        (visited_nodes, visited_edges)
    }

    fn dfs_helper(
        &self,
        curr: i64,
        visited: &mut HashSet<i64>,
        visited_nodes: &mut Vec<i64>,
        visited_edges: &mut Vec<(i64, i64)>,
    ) -> bool {
        visited.insert(curr);
        visited_nodes.push(curr);
        
        if let Some(v_list) = self.adj.get(&curr) {
            for &(v, _) in v_list {
                if !visited.contains(&v) {
                    visited_edges.push((curr, v));
                    if self.dfs_helper(v, visited, visited_nodes, visited_edges) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn nodes(&self) -> Vec<i64> {
        let mut set: HashSet<i64> = HashSet::new();
        for (&u, v_list) in &self.adj {
            set.insert(u);
            for &(v, _) in v_list {
                set.insert(v);
            }
        }
        set.into_iter().collect()
    }

    pub fn edges(&self) -> Vec<(i64, i64, i64)> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for (&u, v_list) in &self.adj {
            for &(v, w) in v_list {
                let key = if u <= v { (u, v) } else { (v, u) };
                if seen.insert(key) {
                    result.push((u, v, w));
                }
            }
        }

        result
    }
}
