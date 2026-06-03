use tree_sitter::{Language, Parser};

pub struct LanguageDetector {
    parsers: Vec<(&'static str, Parser)>,
}

pub fn get_language(name: &str) -> Option<Language> {
    match name {
        "PHP" => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        "Python" => Some(tree_sitter_python::LANGUAGE.into()),
        "JavaScript" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "TypeScript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "TSX" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "Rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "Go" => Some(tree_sitter_go::LANGUAGE.into()),
        "Java" => Some(tree_sitter_java::LANGUAGE.into()),
        "C#" => Some(tree_sitter_c_sharp::LANGUAGE.into()),
        "Ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
        "C++" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "COBOL" => Some(arborium_cobol::language().into()),
        _ => None,
    }
}

impl LanguageDetector {
    pub fn new() -> Self {
        let mut parsers = Vec::new();
        let names = [
            "Python", "JavaScript", "TypeScript", "TSX", "Rust",
            "Go", "Java", "C#", "Ruby", "C++",
        ];

        for name in &names {
            if let Some(lang) = get_language(name) {
                let mut parser = Parser::new();
                if parser.set_language(&lang).is_ok() {
                    parsers.push((*name, parser));
                }
            }
        }

        LanguageDetector { parsers }
    }

    pub fn detect(&mut self, source: &str) -> String {
        let mut best: Option<(&str, usize)> = None;

        for (name, parser) in &mut self.parsers {
            if let Some(tree) = parser.parse(source, None) {
                let errors = count_errors(tree.root_node());
                if errors == 0 {
                    return name.to_string();
                }
                match best {
                    None => best = Some((name, errors)),
                    Some((_, best_err)) if errors < best_err => best = Some((name, errors)),
                    _ => {}
                }
            }
        }

        best.map(|(name, _)| name.to_string()).unwrap_or_else(|| "Unknown".to_string())
    }
}

fn count_errors(node: tree_sitter::Node) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_error() || child.is_missing() {
            count += 1;
        }
        count += count_errors(child);
    }
    count
}
