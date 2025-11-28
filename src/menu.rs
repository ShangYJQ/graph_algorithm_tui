use ratatui::{prelude::*, widgets::*};

//事件定义
pub enum MenuSignal {
    None,
    Selected(String),
}

// --- 数据模型 ---
#[derive(Clone, Debug)]
pub struct MenuItem {
    pub name: String,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn new(name: &str, children: Vec<MenuItem>) -> Self {
        Self {
            name: name.to_string(),
            children,
        }
    }
    pub fn leaf(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: vec![],
        }
    }
}

// 这个结构体负责逻辑：上下移动、进入退出、保存数据
#[derive(Debug)]
pub struct MenuState {
    pub root_items: Vec<MenuItem>,
    pub nav_stack: Vec<usize>,              // 导航栈
    pub list_state: ListState,              // Ratatui 的列表状态
    pub last_selected_item: Option<String>, // 供外部获取最终结果
}

impl MenuState {
    pub fn new(items: Vec<MenuItem>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            root_items: items,
            nav_stack: Vec::new(),
            list_state: state,
            last_selected_item: None,
        }
    }

    // 获取当前层级的数据
    pub fn get_current_items(&self) -> &[MenuItem] {
        let mut current_list = &self.root_items;
        for &index in &self.nav_stack {
            if let Some(item) = current_list.get(index) {
                current_list = &item.children;
            }
        }
        current_list
    }

    pub fn up(&mut self) {
        let len = self.get_current_items().len();
        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn down(&mut self) {
        let len = self.get_current_items().len();
        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn enter(&mut self) -> MenuSignal {
        if let Some(selected_idx) = self.list_state.selected() {
            let (has_children, name) = {
                let items = self.get_current_items();
                if let Some(item) = items.get(selected_idx) {
                    (!item.children.is_empty(), item.name.clone())
                } else {
                    (false, String::new())
                }
            };

            return if has_children {
                self.nav_stack.push(selected_idx);
                self.list_state.select(Some(0));
                MenuSignal::None
            } else {
                // 记录选中的叶子节点，供外部逻辑使用
                self.last_selected_item = Some(name.clone());
                MenuSignal::Selected(name)
            };
        }
        MenuSignal::None
    }

    pub fn back(&mut self) {
        if let Some(last_index) = self.nav_stack.pop() {
            self.list_state.select(Some(last_index));
            self.last_selected_item = None; // 此时可能想清除选中状态
        }
    }
}

pub struct Menu<'a> {
    block: Option<Block<'a>>,
    highlight_style: Style,
}

impl<'a> Menu<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }

    // 支持链式调用设置 Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    // 支持设置高亮样式
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
}

impl<'a> StatefulWidget for Menu<'a> {
    type State = MenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items_data = state.get_current_items();

        let list_items: Vec<ListItem> = items_data
            .iter()
            .map(|i| {
                let content = if i.children.is_empty() {
                    format!(" {} ", i.name)
                } else {
                    format!(" {} ->", i.name)
                };
                ListItem::new(content)
            })
            .collect();

        // 创建内部的 List 组件
        let mut list = List::new(list_items)
            .highlight_style(self.highlight_style)
            .highlight_symbol(">> ");

        if let Some(b) = self.block {
            list = list.block(b);
        }

        // 代理渲染给内部的 List
        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
