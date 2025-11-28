# 图算法可视化 TUI

一个基于Rust和Ratatui的终端图算法可视化工具，支持DFS、BFS、Prim和Dijkstra算法的动画演示。

## 项目概述

本项目实现了一个交互式的终端用户界面（TUI），能够实时展示图算法的执行过程。核心特性包括：
- 力导向图布局（使用force_graph库）
- 逐步动画展示算法访问过程
- 节点和边的颜色高亮显示
- 实时访问日志记录
- 算法结果统计

## 核心架构

### 1. 数据结构设计

```rust
struct App {
    // 图数据
    data_graph: Graph,                    // 逻辑图结构（邻接表）
    graph: ForceGraph<i64, i64>,          // 可视化图结构（力导向布局）
    
    // 可视化状态
    visited_nodes: HashSet<i64>,          // 当前已访问的节点（用于渲染）
    visited_edges: HashSet<(i64, i64)>,   // 当前已访问的边（用于渲染）
    
    // 动画控制
    animation_nodes: Vec<i64>,             // 完整的节点访问序列
    animation_edges: Vec<(i64, i64)>,      // 完整的边访问序列
    animation_index: usize,                // 当前动画进度索引
    animation_timer: f64,                  // 动画计时器
    animation_step_is_edge: bool,          // 当前步骤：true=边，false=节点
    
    // 算法信息
    current_algorithm: String,             // 当前运行的算法名称
    visit_log: Vec<String>,                // 访问日志
    
    // 算法结果
    prim_total_cost: i64,                  // Prim算法的MST总权重
    dijkstra_dist: HashMap<i64, i64>,      // Dijkstra的最短距离
    dijkstra_parent: HashMap<i64, i64>,    // Dijkstra的父节点信息
}
```

**关键设计思想：**
- `visited_nodes`和`visited_edges`是**渲染状态**，只包含当前应该高亮显示的节点/边
- `animation_nodes`和`animation_edges`是**完整序列**，存储算法的完整访问顺序
- 通过`animation_index`控制逐步将元素从完整序列添加到渲染状态

## 染色控制机制详解

### 核心原理

染色控制采用**双状态分离**的设计：
1. **完整序列**：算法执行后立即获得所有访问顺序
2. **渲染状态**：动画过程中逐步添加元素到渲染集合

### 实现步骤

#### 步骤1：算法执行获取完整序列

当用户选择一个算法时，例如DFS：

```rust
fn run_dfs(&mut self) {
    // 设置算法名称
    self.current_algorithm = "DFS".to_string();
    self.visit_log.clear();
    
    // 清空上次的渲染状态
    self.visited_nodes.clear();
    self.visited_edges.clear();
    
    // 执行算法，获取完整的访问序列
    let (nodes, edges) = self.data_graph.dfs(1);
    self.animation_nodes = nodes;    // [1, 2, 3, 4, ...]
    self.animation_edges = edges;    // [(1,2), (2,3), ...]
    
    // 立即显示第一个节点（起始节点）
    if !self.animation_nodes.is_empty() {
        self.visited_nodes.insert(self.animation_nodes[0]);
        self.visit_log.push(format!("访问节点: {}", self.animation_nodes[0]));
    }
    
    // 初始化动画状态
    self.animation_index = 0;           // 从第0个元素开始
    self.animation_timer = 0.0;         // 重置计时器
    self.animation_step_is_edge = true; // 下一步是显示边
}
```

**关键点：**
- 第一个节点**立即**添加到`visited_nodes`（所以用户选择算法后立即看到起始节点变黄）
- `animation_index = 0`从第0个开始，但第0个节点已经显示，所以后续动画从边开始

#### 步骤2：算法返回访问序列

图算法修改为返回访问序列，以DFS为例：

```rust
pub fn dfs(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>) {
    let mut visited: HashSet<i64> = HashSet::new();
    let mut visited_nodes: Vec<i64> = Vec::new();  // 记录节点访问顺序
    let mut visited_edges: Vec<(i64, i64)> = Vec::new();  // 记录边访问顺序
    
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
    visited_nodes.push(curr);  // 记录访问节点
    
    if let Some(v_list) = self.adj.get(&curr) {
        for &(v, _) in v_list {
            if !visited.contains(&v) {
                visited_edges.push((curr, v));  // 记录访问边
                if self.dfs_helper(v, visited, visited_nodes, visited_edges) {
                    return true;
                }
            }
        }
    }
    false
}
```

**返回的序列示例：**
- `visited_nodes = [1, 2, 5, 6, 3, 4]`
- `visited_edges = [(1,2), (2,5), (5,6), (1,3), (3,4)]`

#### 步骤3：动画循环控制

主循环中每帧调用`update_animation()`：

```rust
pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
    // ... 初始化代码 ...
    
    while !self.exit {
        self.handle_events()?;      // 处理用户输入
        self.update_animation();     // 更新动画状态 ⭐
        
        // 更新物理模拟
        self.graph.update(self.dt as f32);
        
        // 渲染
        terminal.draw(|frame| self.draw(frame))?;
    }
    Ok(())
}
```

#### 步骤4：动画状态更新（核心逻辑）

这是染色控制的**核心**：

```rust
fn update_animation(&mut self) {
    // 如果没有动画序列，直接返回
    if self.animation_nodes.is_empty() {
        return;
    }
    
    // 检查动画是否完成
    let nodes_done = self.animation_index >= self.animation_nodes.len();
    let edges_done = self.animation_index >= self.animation_edges.len();
    
    if nodes_done && (edges_done || !self.animation_step_is_edge) {
        return;  // 动画已完成
    }
    
    // 累加时间
    self.animation_timer += self.dt;  // dt = 0.005秒
    
    // 每0.2秒执行一次动画步骤
    if self.animation_timer >= 0.2 {
        self.animation_timer = 0.0;  // 重置计时器
        
        if self.animation_step_is_edge {
            // ===== 当前步骤：显示边 =====
            if self.animation_index < self.animation_edges.len() {
                let edge = self.animation_edges[self.animation_index];
                // 将边添加到渲染集合（变黄）
                self.visited_edges.insert(edge);
                self.visit_log.push(format!("访问边: {} -> {}", edge.0, edge.1));
            }
            self.animation_index += 1;  // 移动到下一个节点
            self.animation_step_is_edge = false;  // 下一步显示节点
            
        } else {
            // ===== 当前步骤：显示节点 =====
            if self.animation_index < self.animation_nodes.len() {
                let node = self.animation_nodes[self.animation_index];
                // 将节点添加到渲染集合（变黄）
                self.visited_nodes.insert(node);
                self.visit_log.push(format!("访问节点: {}", node));
            }
            
            // 决定下一步
            if self.animation_index < self.animation_edges.len() {
                self.animation_step_is_edge = true;  // 下一步显示边
            } else {
                self.animation_index += 1;  // 没有边了，跳到下一个节点
            }
        }
    }
}
```

**动画时序示例：**

假设序列为：
- nodes: `[1, 2, 3]`
- edges: `[(1,2), (2,3)]`

| 时间 | index | step_is_edge | 操作 | visited_nodes | visited_edges |
|------|-------|--------------|------|---------------|---------------|
| t=0s | 0 | true | 初始化，显示节点1 | {1} | {} |
| t=0.2s | 0 | true→false | 添加边(1,2) | {1} | {(1,2)} |
| t=0.4s | 1 | false→true | 添加节点2 | {1,2} | {(1,2)} |
| t=0.6s | 1 | true→false | 添加边(2,3) | {1,2} | {(1,2), (2,3)} |
| t=0.8s | 2 | false | 添加节点3 | {1,2,3} | {(1,2), (2,3)} |
| t=1.0s | 3 | - | 动画完成 | {1,2,3} | {(1,2), (2,3)} |

**关键设计：**
- 节点和边**交替**显示（不会同时变黄）
- 每0.2秒一个步骤，动画流畅
- `animation_step_is_edge`标志控制当前显示节点还是边

#### 步骤5：渲染染色

渲染函数根据`visited_nodes`和`visited_edges`决定颜色：

```rust
fn render_ctx(&self, ctx: &mut Context) {
    // 绘制边
    self.graph.visit_edges(|node1, node2, edge_data| {
        let u = node1.data.user_data;
        let v = node2.data.user_data;
        
        // 检查边是否被访问（双向检查）
        let is_visited =
            self.visited_edges.contains(&(u, v)) || 
            self.visited_edges.contains(&(v, u));
        
        let x1 = node1.x() as f64;
        let y1 = node1.y() as f64;
        let x2 = node2.x() as f64;
        let y2 = node2.y() as f64;
        
        // 根据访问状态决定颜色
        ctx.draw(&CanvaLine {
            x1, y1, x2, y2,
            color: if is_visited {
                Color::Yellow  // 已访问：黄色
            } else {
                Color::LightBlue  // 未访问：浅蓝色
            },
        });
        
        // 在边的中点显示权重
        let mid_x = (x1 + x2) / 2.0;
        let mid_y = (y1 + y2) / 2.0;
        ctx.print(mid_x, mid_y, format!("{}", edge_data.user_data).white());
    });
    
    // 绘制节点
    self.graph.visit_nodes(|node| {
        let node_id = node.data.user_data;
        let is_visited = self.visited_nodes.contains(&node_id);
        
        // 根据访问状态决定颜色
        ctx.draw(&Circle {
            x: node.x() as f64,
            y: node.y() as f64,
            radius: self.r,
            color: if is_visited {
                Color::Yellow     // 已访问：黄色
            } else {
                Color::LightBlue  // 未访问：浅蓝色
            },
        });
        
        // 显示节点ID
        ctx.print(
            node.x() as f64,
            node.y() as f64,
            format!("{}", node.data.user_data).yellow(),
        );
    });
}
```

**染色决策：**
```rust
// 伪代码
if node_id in visited_nodes:
    color = Yellow
else:
    color = LightBlue
```

### 染色控制总结

**核心机制：**
```
算法执行 → 完整序列 → 动画控制器 → 逐步添加 → 渲染集合 → 颜色渲染
   ↓           ↓           ↓            ↓          ↓          ↓
  DFS      [1,2,3,4]    计时器0.2s   insert(1)  {1}      黄色
                        animation_index++  insert(2)  {1,2}    黄色
                        ...               ...       ...      ...
```

**时序保证：**
1. 用户选择算法 → 第一个节点立即黄色（0秒）
2. 等待0.2秒 → 第一条边变黄色
3. 等待0.2秒 → 第二个节点变黄色
4. 等待0.2秒 → 第二条边变黄色
5. ... 依此类推

**状态分离：**
- `animation_nodes/edges`：不可变的完整序列（算法结果）
- `visited_nodes/edges`：可变的渲染状态（动画过程）
- 通过`animation_index`建立两者的映射关系

## 算法结果展示

### Prim算法

Prim算法额外返回最小生成树的总权重：

```rust
pub fn prim(&self, s: i64) -> (Vec<i64>, Vec<(i64, i64)>, i64) {
    // ... Prim算法实现 ...
    let mut total_cost: i64 = 0;
    
    // 每次添加边到MST时累加权重
    if let Some(&p) = parent.get(&u) {
        visited_edges.push((p, u));
        total_cost += cost;  // 累加权重
    }
    
    (visited_nodes, visited_edges, total_cost)
}
```

### Dijkstra算法

Dijkstra算法返回最短距离和父节点信息，用于重建路径：

```rust
pub fn dijkstra(&self, s: i64) 
    -> (Vec<i64>, Vec<(i64, i64)>, HashMap<i64, i64>, HashMap<i64, i64>) {
    let mut dist: HashMap<i64, i64> = HashMap::new();
    let mut parent: HashMap<i64, i64> = HashMap::new();
    
    // ... Dijkstra算法实现 ...
    
    // 记录父节点用于路径重建
    if cost < *dist.get(&v).unwrap_or(&i64::MAX) {
        dist.insert(v, cost);
        parent.insert(v, u.node);  // 记录父节点
        pq.push(State { cost, node: v });
        visited_edges.push((u.node, v));
    }
    
    (visited_nodes, visited_edges, dist, parent)
}
```

动画完成后显示结果：

```rust
// 在draw函数中，当动画完成时显示结果
if animation_complete && !self.current_algorithm.is_empty() {
    log_lines.push("---- 结束 ----\n".to_string());
    
    match self.current_algorithm.as_str() {
        "Prim" => {
            log_lines.push(format!("最小生成树总长度: {}", self.prim_total_cost));
        }
        "Dijkstra" => {
            log_lines.push("最短距离:".to_string());
            
            for (&node, &dist) in sorted_nodes {
                // 从parent信息重建路径
                let mut path = vec![node];
                let mut current = node;
                while let Some(&prev) = self.dijkstra_parent.get(&current) {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                
                let path_str = path.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                
                log_lines.push(format!(
                    "  到节点{}: 距离={}, 路径={}",
                    node, dist, path_str
                ));
            }
        }
        _ => {}
    }
}
```

## UI布局

```
┌────────────────────────────────┬────────────┐
│                                │   Menu     │  30%
│                                ├────────────┤
│         Graph Canvas           │            │
│     (Force-directed layout)    │   Visit    │  70%
│                                │    Info    │
│                                │            │
└────────────────────────────────┴────────────┘
         80%                          20%
```

右侧面板垂直分割为3:7：
- 上方30%：算法选择菜单
- 下方70%：访问信息和结果显示

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
    .split(frame.area());

let right_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
    .split(chunks[1]);
```

## 技术栈

- **Rust**: 系统编程语言
- **Ratatui**: 终端UI框架
- **force_graph**: 力导向图布局算法
- **crossterm**: 跨平台终端控制

## 使用方法

### 编译运行

```bash
cargo build --release
cargo run
```

### 操作说明

- `j/k`: 上下移动菜单
- `l/Enter`: 选择算法
- `h`: 返回上级菜单
- `方向键`: 移动中心节点
- `+/-`: 调整节点半径
- `q`: 退出

## 性能优化

1. **增量渲染**: 只更新变化的部分
2. **状态缓存**: 避免重复计算
3. **固定帧率**: `dt = 0.005秒`，约200 FPS
4. **动画间隔**: 0.2秒/步，平衡速度和可读性

## 扩展建议

1. 添加更多算法（Kruskal、A*、Bellman-Ford）
2. 支持自定义图输入
3. 导出动画为GIF/视频
4. 添加算法步骤暂停/回退功能
5. 支持有向图和带权图的完整可视化

## 总结

本项目的核心创新在于**双状态分离**的染色控制机制：
- 算法一次性执行获取完整访问序列
- 动画控制器按时间步逐个元素显示
- 渲染系统只根据当前显示状态着色

这种设计使得算法逻辑与可视化逻辑完全解耦，易于扩展和维护。
