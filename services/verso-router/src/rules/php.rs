use regex::Regex;

pub fn php_to_php(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)\barray\s*\(([\s\S]*?)\)\s*;").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let inner = &caps[1];
        let items: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        format!("[{}];", items.join(", "))
    }).to_string();

    if !result.contains("declare(strict_types=1)") {
        result = format!("<?php\ndeclare(strict_types=1);\n\n{}", result.trim_start_matches("<?php").trim());
    }

    result
}
