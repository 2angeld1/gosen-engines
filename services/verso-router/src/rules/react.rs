use regex::Regex;

pub fn react_class_to_hooks(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)^(\s*)class\s+(\w+)\s+extends\s+(?:React\.)?Component\s*\{").unwrap();
    result = re.replace_all(&result, "${1}function ${2}(props) {").to_string();

    let re = Regex::new(r"(?m)^\s*constructor\s*\([^)]*\)\s*\{[\s\S]*?^\s*\}").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"super\([^)]*\);?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"this\.state\s*=\s*\{").unwrap();
    let has_state = re.is_match(&result);
    if has_state {
        let re = Regex::new(r"(?m)^(\s*)this\.state\s*=\s*(\{[\s\S]*?\});?\s*$").unwrap();
        result = re.replace_all(&result, "${1}const [state, setState] = useState($2);").to_string();
    }
    let re = Regex::new(r"this\.state\.(\w+)").unwrap();
    result = re.replace_all(&result, "state.$1").to_string();
    let re = Regex::new(r"this\.setState\s*\(").unwrap();
    result = re.replace_all(&result, "setState(").to_string();

    let re = Regex::new(r"this\.props\.(\w+)").unwrap();
    result = re.replace_all(&result, "props.$1").to_string();
    let re = Regex::new(r"(?m)^\s*this\.\w+\s*=\s*this\.\w+\.bind\(this\);?\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?m)^(\s*)componentDidMount\s*\(\s*\)\s*\{").unwrap();
    result = re.replace_all(&result, "${1}useEffect(() => {").to_string();
    let re = Regex::new(r"(?m)^(\s*)componentDidUpdate\s*\([^)]*\)\s*\{").unwrap();
    result = re.replace_all(&result, "${1}useEffect(() => {").to_string();
    let re = Regex::new(r"(?m)^(\s*)componentWillUnmount\s*\(\s*\)\s*\{").unwrap();
    result = re.replace_all(&result, "${1}return () => {").to_string();

    let re = Regex::new(r"(?m)^(\s*)(\w+)\s*=\s*\(([^)]*)\)\s*=>").unwrap();
    result = re.replace_all(&result, "${1}const ${2} = (${3}) =>").to_string();

    let re = Regex::new(r"(?m)^(\s+)render\s*\(\s*\)\s*\{").unwrap();
    result = re.replace_all(&result, "${1}__RENDER_MARKER__").to_string();
    let re = Regex::new(r"(?m)^(\s+)(\w+)\s*\(([^)]*)\)\s*\{").unwrap();
    result = re.replace_all(&result, "${1}const ${2} = (${3}) => {").to_string();
    let re = Regex::new(r"__RENDER_MARKER__").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"this\.(\w+\s*\()").unwrap();
    result = re.replace_all(&result, "$1").to_string();

    if !result.contains("useState") && !result.contains("useEffect") {
    } else if result.contains("import React") {
        let re = Regex::new(r#"(import\s+React\s+from\s+['"]react['"];?)"#).unwrap();
        result = re.replace_all(&result, "$1\nimport { useState, useEffect } from 'react';").to_string();
    } else if result.contains("from 'react'") || result.contains("from \"react\"") {
        let re = Regex::new(r#"(from\s+['"]react['"];?)"#).unwrap();
        result = re.replace_all(&result, "from 'react'").to_string();
        let re = Regex::new(r#"import\s+\{([^}]*)\}\s+from\s+['"]react['"]"#).unwrap();
        if re.is_match(&result) {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let existing = &caps[1];
                let mut has_use_state = existing.contains("useState");
                let mut has_use_effect = existing.contains("useEffect");
                let mut new_imports = existing.trim().to_string();
                if !has_use_state { new_imports = format!("{}, useState", new_imports); }
                if !has_use_effect { new_imports = format!("{}, useEffect", new_imports); }
                format!("import {{{}}} from 'react'", new_imports)
            }).to_string();
        }
    } else {
        result = format!("import {{ useState, useEffect }} from 'react';\n{}", result);
    }

    result
}
