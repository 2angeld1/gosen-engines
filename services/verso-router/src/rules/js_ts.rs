use regex::Regex;

pub fn js_to_ts(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)(function\s+\w+\s*\(([^)]*)\))").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let sig = &caps[1];
        let params = &caps[2];
        if params.trim().is_empty() {
            return sig.to_string();
        }
        let typed_params: Vec<String> = params.split(',')
            .map(|p| {
                let p = p.trim();
                if p.contains(':') { p.to_string() } else { format!("{}: any", p) }
            })
            .collect();
        sig.replacen(params, &typed_params.join(", "), 1)
    }).to_string();

    let re = Regex::new(r"(?m)(\(([^)]*)\)\s*=>)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let params = &caps[2];
        if params.trim().is_empty() {
            return caps[0].to_string();
        }
        let typed_params: Vec<String> = params.split(',')
            .map(|p| {
                let p = p.trim();
                if p.contains(':') { p.to_string() } else { format!("{}: any", p) }
            })
            .collect();
        caps[0].replacen(params, &typed_params.join(", "), 1)
    }).to_string();

    let re = Regex::new(r"(?m)(function\s+\w+\s*\([^)]*\))\s*\{").unwrap();
    result = re.replace_all(&result, "$1: any {").to_string();

    let re = Regex::new(r"(?m)\bvar\s+").unwrap();
    result = re.replace_all(&result, "let ").to_string();

    let re = Regex::new(r#"(?m)(const|let)\s+(\w+)\s*=\s*require\s*\(['"]([^'"]+)['"]\)"#).unwrap();
    result = re.replace_all(&result, "import $2 from '$3'").to_string();

    if !result.contains("export ") && result.contains("module.exports") {
        result = result.replace("module.exports", "export default");
    } else if !result.contains("export ") && result.contains("function ") {
        let re = Regex::new(r"(?m)^(function\s+\w+)").unwrap();
        if let Some(cap) = re.captures(&result) {
            result = result.replacen(&cap[1], &format!("export {}", &cap[1]), 1);
        }
    }

    result
}
