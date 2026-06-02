use regex::Regex;

pub fn java_to_kotlin(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)^\s*package\s+([^;]+);").unwrap();
    result = re.replace_all(&result, "package $1").to_string();

    let re = Regex::new(
        r"(?m)^(\s*)(?:public\s+)?(?:abstract\s+)?(?:final\s+)?class\s+(\w+)(?:\s+extends\s+(\w+))?(?:\s+implements\s+([^{]+))?\s*\{"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let name = &caps[2];
        let ext = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let impls = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        if !ext.is_empty() && !impls.is_empty() {
            format!("{}class {} : {}(), {} {{", indent, name, ext, impls)
        } else if !ext.is_empty() {
            format!("{}class {} : {}() {{", indent, name, ext)
        } else if !impls.is_empty() {
            format!("{}class {} : {} {{", indent, name, impls)
        } else {
            format!("{}class {} {{", indent, name)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:public\s+)?interface\s+(\w+)(?:\s+extends\s+([^{]+))?\s*\{").unwrap();
    result = re.replace_all(&result, "${1}interface ${2} {").to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:public\s+)?@interface\s+(\w+)").unwrap();
    result = re.replace_all(&result, "${1}annotation class ${2} {").to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:public\s+)?enum\s+(\w+)(?:\s+implements\s+([^{]+))?\s*\{").unwrap();
    result = re.replace_all(&result, "${1}enum class ${2} {").to_string();

    let re = Regex::new(
        r"(?m)^(\s*)((?:public|private|protected)\s+)?(?:static\s+)?(?:final\s+)?(\w+(?:\[\])?(?:<[^>]+>)?)\s+(\w+)(?:\s*=\s*([^;]+))?\s*;\s*$"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let vis = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let typ = java_type_to_kotlin(caps[3].trim());
        let name = &caps[4];
        let init = caps.get(5).map(|m| m.as_str().trim()).unwrap_or("");

        let raw_type = caps[3].trim();
        if matches!(raw_type, "return" | "if" | "while" | "for" | "switch" | "case" | "break" | "continue" | "throw" | "catch" | "finally" | "new" | "try") {
            return caps[0].to_string();
        }

        let is_final = caps[0].contains("final");
        let is_static = caps[0].contains("static");
        let kw = if (is_final && is_static && !init.is_empty()) || (caps[0].contains("static final")) {
            "const val"
        } else if is_final || name.starts_with("val") {
            "val"
        } else {
            "var"
        };

        let init_val = init.replace("new ", "");

        let init_str: String = if init_val == "null" || init_val.is_empty() {
            if typ == "String" || typ.contains('?') {
                " = null".to_string()
            } else if typ == "Int" || typ == "Long" || typ == "Short" || typ == "Byte" {
                " = 0".to_string()
            } else if typ == "Float" {
                " = 0.0f".to_string()
            } else if typ == "Double" {
                " = 0.0".to_string()
            } else if typ == "Boolean" {
                " = false".to_string()
            } else if typ == "Char" {
                " = '\\u0000'".to_string()
            } else {
                String::new()
            }
        } else {
            let init_val = init_val.trim_end_matches(';');
            format!(" = {}", init_val)
        };

        let mut out = format!("{}{}{} {}: {}", indent, vis, kw, name, typ);
        out.push_str(&init_str);
        out
    }).to_string();

    let re = Regex::new(
        r"(?m)^(\s*)((?:public|private|protected)\s+)?([A-Z]\w+)\s*\(([^)]*)\)\s*(?:\s*throws\s+[^{]+)?\s*\{"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let params_raw = caps.get(4).map(|m| m.as_str().trim()).unwrap_or("");
        let params: Vec<String> = if params_raw.is_empty() {
            Vec::new()
        } else {
            params_raw.split(',').map(|p| {
                let p = p.trim();
                let parts: Vec<&str> = p.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ptype = java_type_to_kotlin(parts[..parts.len()-1].join(" ").trim());
                    let pname = parts.last().unwrap_or(&"arg");
                    format!("{}: {}", pname, ptype)
                } else {
                    p.to_string()
                }
            }).collect()
        };
        format!("{}constructor({}) {{", indent, params.join(", "))
    }).to_string();

    let re = Regex::new(
        r"(?m)^(\s*)((?:public|private|protected)\s+)?(?:static\s+)?(?:final\s+)?(\w+(?:\[\])?(?:<[^>]+>)?)\s+(\w+)\s*\(([^)]*)\)\s*(?:\s*throws\s+[^{]+)?\s*\{"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let vis = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let ret_type = caps[3].trim();
        let name = &caps[4];
        let params_raw = caps[5].trim();
        let is_static = caps[0].contains("static");

        let ret_str = if ret_type == "void" { "Unit".to_string() } else { java_type_to_kotlin(ret_type) };

        let params: Vec<String> = if params_raw.is_empty() {
            Vec::new()
        } else {
            params_raw.split(',').map(|p| {
                let p = p.trim();
                let parts: Vec<&str> = p.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ptype = java_type_to_kotlin(parts[..parts.len()-1].join(" ").trim());
                    let pname = parts.last().unwrap_or(&"arg");
                    let pname = pname.trim_end_matches("[]");
                    format!("{}: {}", pname, ptype)
                } else {
                    p.to_string()
                }
            }).collect()
        };

        if name == "main" && params.len() == 1 && params[0].contains("Array<String>") {
            format!("{}@JvmStatic\n{}fun main(args: Array<String>) {{", indent, indent)
        } else {
            if is_static {
                format!("{}@JvmStatic\n{}fun {}({}): {} {{", indent, indent, name, params.join(", "), ret_str)
            } else {
                format!("{}{}fun {}({}): {} {{", indent, vis, name, params.join(", "), ret_str)
            }
        }
    }).to_string();

    let re = Regex::new(r"@Override\s*").unwrap();
    result = re.replace_all(&result, "override ").to_string();
    let re = Regex::new(r"override\s+public\s+").unwrap();
    result = re.replace_all(&result, "override ").to_string();
    let re = Regex::new(r"@Deprecated").unwrap();
    result = re.replace_all(&result, "@Deprecated").to_string();
    let re = Regex::new(r"@SuppressWarnings\([^)]*\)").unwrap();
    result = re.replace_all(&result, "@Suppress").to_string();
    let re = Regex::new(r"@Nullable").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"@NotNull").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"System\.(out|err)\.(println|print|printf)").unwrap();
    result = re.replace_all(&result, "$2").to_string();

    let re = Regex::new(r"String\.format\(([^,]+),\s*(.+)\)").unwrap();
    result = re.replace_all(&result, "$1.format($2)").to_string();

    let re = Regex::new(r"new\s+ArrayList\s*<[^>]*>\s*\(\)").unwrap();
    result = re.replace_all(&result, "mutableListOf()").to_string();
    let re = Regex::new(r"new\s+HashMap\s*<[^>]*>\s*\(\)").unwrap();
    result = re.replace_all(&result, "mutableMapOf()").to_string();
    let re = Regex::new(r"new\s+HashSet\s*<[^>]*>\s*\(\)").unwrap();
    result = re.replace_all(&result, "mutableSetOf()").to_string();
    let re = Regex::new(r"new\s+(\w+)\s*\(").unwrap();
    result = re.replace_all(&result, "$1(").to_string();

    let re = Regex::new(r"(\w+)\s+instanceof\s+(\w+)").unwrap();
    result = re.replace_all(&result, "$1 is $2").to_string();

    let re = Regex::new(r"\((\w+)\)\s*(\w+)").unwrap();
    result = re.replace_all(&result, "$2 as $1").to_string();

    let re = Regex::new(r"for\s*\((\w+(?:\[\])?(?:<[^>]+>)?)\s+(\w+)\s*:\s*(\w+)\)").unwrap();
    result = re.replace_all(&result, "for ($2 in $3)").to_string();

    let re = Regex::new(r"switch\s*\(([^)]+)\)\s*\{").unwrap();
    result = re.replace_all(&result, "when ($1) {").to_string();
    let re = Regex::new(r"case\s+(.+?)\s*:").unwrap();
    result = re.replace_all(&result, "$1 ->").to_string();
    let re = Regex::new(r"break;").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"default:").unwrap();
    result = re.replace_all(&result, "else ->").to_string();

    let re = Regex::new(r"catch\s*\((\w+)\s+(\w+)\)").unwrap();
    result = re.replace_all(&result, "catch ($2: $1)").to_string();

    let re = Regex::new(
        r"for\s*\(\s*(?:\w+(?:\[\])?(?:<[^>]+>)?)\s+(\w+)\s*=\s*([^;]+)\s*;\s*\w+\s*([<>=!]+)\s*([^;]+)\s*;\s*\w+\s*(\+\+|--|\+=\s*\w+|--=\s*\w+)\s*\)"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var = &caps[1];
        let init = &caps[2];
        let limit = &caps[4];
        let init = init.trim();
        let limit = limit.trim();
        if init == "0" {
            format!("for ({} in 0 until {})", var, limit)
        } else {
            format!("for ({} in {} until {})", var, init, limit)
        }
    }).to_string();

    let type_mappings: [(&str, &str); 10] = [
        (r"\bvoid\b", "Unit"),
        (r"\bBoolean\b", "Boolean"),
        (r"\bInteger\b", "Int"),
        (r"\bArrayList\b", "ArrayList"),
        (r"\bHashMap\b", "HashMap"),
        (r"\bHashSet\b", "HashSet"),
        (r"\bList\b", "List"),
        (r"\bMap\b", "Map"),
        (r"\bSet\b", "Set"),
        (r"\bObject\b", "Any"),
    ];
    for (pattern, kt_type) in &type_mappings {
        let re = Regex::new(pattern).unwrap();
        result = re.replace_all(&result, *kt_type).to_string();
    }

    let re = Regex::new(r";(\s*[)}])").unwrap();
    result = re.replace_all(&result, "$1").to_string();
    let re = Regex::new(r";\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r";\s*\n").unwrap();
    result = re.replace_all(&result, "\n").to_string();
    let re = Regex::new(r"\s*throws\s+\w+(?:\s*,\s*\w+)*").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"\{\s*\n\s*\n").unwrap();
    result = re.replace_all(&result, "{\n").to_string();
    let re = Regex::new(r"\n\s*\n\s*\n").unwrap();
    result = re.replace_all(&result, "\n\n").to_string();

    result
}

fn java_type_to_kotlin(typ: &str) -> String {
    let t = typ.trim();
    match t {
        "void" => "Unit".to_string(),
        "boolean" | "Boolean" => "Boolean".to_string(),
        "byte" | "Byte" => "Byte".to_string(),
        "short" | "Short" => "Short".to_string(),
        "int" | "Integer" => "Int".to_string(),
        "long" | "Long" => "Long".to_string(),
        "float" | "Float" => "Float".to_string(),
        "double" | "Double" => "Double".to_string(),
        "char" | "Character" => "Char".to_string(),
        "String" => "String".to_string(),
        "int[]" => "IntArray".to_string(),
        "byte[]" => "ByteArray".to_string(),
        "char[]" => "CharArray".to_string(),
        "String[]" => "Array<String>".to_string(),
        "Object" => "Any".to_string(),
        "Void" => "Unit".to_string(),
        "boolean[]" => "BooleanArray".to_string(),
        "long[]" => "LongArray".to_string(),
        "double[]" => "DoubleArray".to_string(),
        "float[]" => "FloatArray".to_string(),
        s if s.starts_with("List<") || s.starts_with("ArrayList<") => format!("MutableList<{}>", &t[5..t.len()-1]),
        s if s.starts_with("Map<") || s.starts_with("HashMap<") => format!("MutableMap<{}>", &t[4..t.len()-1]),
        s if s.starts_with("Set<") || s.starts_with("HashSet<") => format!("MutableSet<{}>", &t[4..t.len()-1]),
        _ => {
            let t = t.trim_end_matches("[]");
            if t.is_empty() { "Any".to_string() } else { t.to_string() }
        }
    }
}
