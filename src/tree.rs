use crate::filesystem::FileNode;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub display_name: String,
    pub indent: i32,
    pub relevance: i32,
    pub file_node: FileNode,
}

pub fn evaluate_relevance(text: &str, words: &Vec<String>) -> i32 {
    if words.is_empty() {
        return 0;
    }
    let mut relevance = 0;
    for word in words {
        if text.contains(word) {
            relevance += 1;
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
            display_name: it.display_name(),
            indent: 0,
            relevance: evaluate_relevance(it.lowercase_name.as_str(), &filter_words),
            file_node: it.clone(),
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
            .then(
                a.file_node
                    .is_directory
                    .cmp(&b.file_node.is_directory)
                    .reverse(),
            )
            .then(a.file_node.lowercase_name.cmp(&b.file_node.lowercase_name))
    });

    current_tree_nodes
}
