use serde::{Deserialize, Serialize};
use tree_sitter::{Parser, Node};
use crate::parser::get_language;

#[derive(Deserialize)]
pub struct ExtractAstRequest {
    pub source: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct AstNode {
    pub node_type: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Serialize)]
pub struct ExtractAstResponse {
    pub nodes: Vec<AstNode>,
    pub error: Option<String>,
}

pub fn extract_ast(req: &ExtractAstRequest) -> ExtractAstResponse {
    let lang_opt = get_language(&req.language);
    
    if lang_opt.is_none() {
        return ExtractAstResponse {
            nodes: vec![],
            error: Some(format!("Unsupported language: {}", req.language)),
        };
    }
    
    let language = lang_opt.unwrap();
    let mut parser = Parser::new();
    
    if let Err(e) = parser.set_language(&language) {
        return ExtractAstResponse {
            nodes: vec![],
            error: Some(format!("Failed to set language: {:?}", e)),
        };
    }
    
    let tree = match parser.parse(&req.source, None) {
        Some(t) => t,
        None => {
            return ExtractAstResponse {
                nodes: vec![],
                error: Some("Failed to parse source code".to_string()),
            };
        }
    };

    let mut nodes = Vec::new();
    walk_and_extract(tree.root_node(), &req.source, &mut nodes);

    ExtractAstResponse {
        nodes,
        error: None,
    }
}

fn walk_and_extract(node: Node, source: &str, results: &mut Vec<AstNode>) {
    let kind = node.kind();
    
    // Generic heuristic for structural nodes
    if kind.contains("function") || kind.contains("method") || kind.contains("class") {
        let name = extract_name(node, source);
        if !name.is_empty() {
            results.push(AstNode {
                node_type: kind.to_string(),
                name,
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_and_extract(child, source, results);
    }
}

fn extract_name(node: Node, source: &str) -> String {
    // Try to find an identifier child (e.g. "name" field or generic identifier node)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "identifier" || kind == "name" {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                return text.to_string();
            }
        }
    }
    
    // Fallback: check if node itself is an identifier (rare for declarations, but possible)
    if node.kind() == "identifier" {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return text.to_string();
        }
    }
    
    "".to_string()
}
