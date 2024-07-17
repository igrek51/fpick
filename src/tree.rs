use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::filesystem::FileNode;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub relevance: i32,
    pub kind: TreeNodeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeNodeType {
    FileNode(FileNode),
    SelfReference,
}

impl TreeNode {
    pub fn render_list_item(&self) -> ListItem {
        match &self.kind {
            TreeNodeType::FileNode(file_node) => self.render_file_node(file_node),
            TreeNodeType::SelfReference => self.render_self_reference(),
        }
    }

    pub fn indexed_name(&self) -> &str {
        match &self.kind {
            TreeNodeType::FileNode(file_node) => file_node.lowercase_name.as_str(),
            TreeNodeType::SelfReference => ".",
        }
    }

    pub fn name(&self) -> &str {
        match &self.kind {
            TreeNodeType::FileNode(file_node) => file_node.name.as_str(),
            TreeNodeType::SelfReference => ".",
        }
    }

    pub fn is_directory(&self) -> bool {
        match &self.kind {
            TreeNodeType::FileNode(file_node) => file_node.is_directory,
            TreeNodeType::SelfReference => true,
        }
    }

    pub fn render_file_node(&self, file_node: &FileNode) -> ListItem {
        let display: String = file_node.name.clone();
        let mut suffix = String::new();
        let mut style = Style::default();
        if file_node.is_symlink {
            suffix = format!("{suffix}@");
            style = Style::default().fg(ratatui::style::Color::LightCyan).bold();
        }
        if file_node.is_directory {
            suffix = format!("{suffix}/");
            style = Style::default().fg(ratatui::style::Color::LightBlue).bold();
        }
        Line::from(vec![Span::styled(display, style), Span::raw(suffix)]).into()
    }

    pub fn render_self_reference(&self) -> ListItem {
        let style = Style::default()
            .fg(ratatui::style::Color::LightYellow)
            .bold();
        Line::from(vec![Span::styled(".", style)]).into()
    }
}

pub fn evaluate_relevance(text: &str, words: &Vec<String>) -> i32 {
    if words.is_empty() {
        return 0;
    }
    let mut relevance = 0;
    for word in words {
        if text.contains(word) {
            relevance += 1;
        } else {
            return 0;
        }
    }
    let firs_word = words.first().unwrap();
    if text.starts_with(firs_word) {
        relevance += 10;
    }
    relevance
}

pub fn render_tree_nodes(child_nodes: &Vec<FileNode>, filter_text: &str) -> Vec<TreeNode> {
    let filter_words: Vec<String> = filter_text
        .to_lowercase()
        .split_whitespace()
        .map(|it| it.to_lowercase())
        .collect();

    let mut current_tree_nodes: Vec<TreeNode> = child_nodes
        .iter()
        .map(|it: &FileNode| TreeNode {
            relevance: 0,
            kind: TreeNodeType::FileNode(it.clone()),
        })
        .collect();
    current_tree_nodes.insert(
        0,
        TreeNode {
            relevance: 0,
            kind: TreeNodeType::SelfReference,
        },
    );
    current_tree_nodes = current_tree_nodes
        .iter()
        .map(|it: &TreeNode| TreeNode {
            relevance: evaluate_relevance(it.indexed_name(), &filter_words),
            kind: it.kind.clone(),
        })
        .collect();

    if !filter_text.is_empty() {
        current_tree_nodes = current_tree_nodes
            .into_iter()
            .filter(|it| it.relevance > 0)
            .collect();
    }

    current_tree_nodes.sort_by(|a: &TreeNode, b: &TreeNode| {
        let first_cmp = a.relevance.cmp(&b.relevance).reverse();
        first_cmp
            .then(a.is_directory().cmp(&b.is_directory()).reverse())
            .then(a.indexed_name().cmp(b.indexed_name()))
    });

    current_tree_nodes
}
