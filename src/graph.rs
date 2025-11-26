use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

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

    pub fn add_edge(&mut self, u: i64, v: i64, w: i64) {
        self.adj.entry(u).or_insert(Vec::new()).push((v, w));
    }

    pub fn dijkstra(&self, s: i64) {
        let mut dist: HashMap<i64, i64> = HashMap::new();

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
                    if let Some(v_list) = self.adj.get(&u.node) {
                        for &(v, w) in v_list {
                            let cost = u.cost + w;
                            if cost < *dist.get(&v).unwrap_or(&i64::MAX) {
                                dist.insert(v, cost);
                                pq.push(State { cost, node: v });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        println!("dijkstra:");
        for (v, w) in dist {
            println!("to: {} dist: {}", v, w)
        }
    }

    pub fn prim(&self, s: i64) {
        let mut dist: HashMap<i64, i64> = HashMap::new();
        let mut booked: HashSet<i64> = HashSet::new();

        let mut parent: HashMap<i64, i64> = HashMap::new();
        let mut pq: BinaryHeap<State> = BinaryHeap::new();

        let mut total_cost: i64 = 0i64;
        dist.insert(s, 0);
        pq.push(State { cost: 0, node: s });

        println!("prim:");
        while let Some(State { cost, node: u }) = pq.pop() {
            if booked.contains(&u) {
                continue;
            }
            if cost > *dist.get(&u).unwrap_or(&i64::MAX) {
                continue;
            }
            booked.insert(u);

            if let Some(&p) = parent.get(&u) {
                println!("{}->{} weight: {}", p, u, cost);
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
        println!("total cost: {}", total_cost);
    }

    pub fn bfs(&self, s: i64) {
        let mut visited: HashSet<i64> = HashSet::new();
        let mut q: VecDeque<i64> = VecDeque::new();

        q.push_back(s);
        visited.insert(s);

        print!("bfs:\n {} ", s);
        while !q.is_empty() {
            let u = q.pop_front();
            match u {
                Some(u) => {
                    if let Some(v_list) = self.adj.get(&u) {
                        for &(v, _) in v_list {
                            if !visited.contains(&v) {
                                q.push_back(v);
                                visited.insert(v);
                                print!(" {} ", v);
                            }
                        }
                    }
                }
                _ => println!("error"),
            }
        }
        println!()
    }
    pub fn dfs(&mut self, s: i64) {
        let mut visited: HashSet<i64> = HashSet::new();
        print!("dfs遍历顺序:\n {} ", s);
        self.dfs_helper(s, &mut visited);
        println!()
    }

    fn dfs_helper(&self, curr: i64, visited: &mut HashSet<i64>) -> bool {
        visited.insert(curr);
        if let Some(v_list) = self.adj.get(&curr) {
            for &(v, _) in v_list {
                if !visited.contains(&v) {
                    print!(" {} ", v);
                    if self.dfs_helper(v, visited) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
