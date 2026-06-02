use regex::Regex;

pub fn cobol_to_java(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?mi)^\s*PROGRAM-ID\.\s*(\w+)").unwrap();
    let class_name = re.captures(&result)
        .map(|c| capitalize(&c[1]))
        .unwrap_or_else(|| "LegacyProgram".to_string());

    let re = Regex::new(r"(?mi)^\s*IDENTIFICATION\s+DIVISION\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*PROGRAM-ID\.\s*\w+\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*DATA\s+DIVISION\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*WORKING-STORAGE\s+SECTION\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*LINKAGE\s+SECTION\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(
        r"(?mi)^\s*(0[1-9]|[1-4]\d|77)\s+(\w+(?:-\w+)*)\s+PIC\s+(\w+(?:\([\dV]+\))?)(?:\s+VALUE\s+(.+?))?\.?\s*$"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = cobol_to_camel(&caps[2]);
        let pic = &caps[3];
        let java_type = pic_to_java_type(pic);
        let default = caps.get(4).map(|m| m.as_str().trim()).unwrap_or("");
        let init = match default {
            s if s.eq_ignore_ascii_case("SPACES") || s.eq_ignore_ascii_case("SPACE") =>
                if java_type == "String" { " = \"\"" } else { " = 0" },
            s if s.eq_ignore_ascii_case("ZERO") || s.eq_ignore_ascii_case("ZEROS") =>
                " = 0",
            s if s.starts_with('\'') && s.ends_with('\'') =>
                &format!(" = \"{}\"", &s[1..s.len()-1]),
            s if s.parse::<i64>().is_ok() =>
                &format!(" = {}", s),
            s if s.starts_with('"') && s.ends_with('"') =>
                &format!(" = {}", s),
            _ => "",
        };
        let comment = format!("{} {} PIC {}", &caps[1], &caps[2], pic);
        format!("{} {}{}; // {}", java_type, var_name, init, comment)
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*PROCEDURE\s+DIVISION\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*PROCEDURE\s+DIVISION\s+USING\s+.*\.?\s*").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?mi)^\s*(\w+(?:-\w+)*)-(?:SECTION|section)?\.\s*$").unwrap();
    result = re.replace_all(&result, "// $1:").to_string();

    let re = Regex::new(r"(?mi)^\s*MOVE\s+(.+?)\s+TO\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let src = caps[1].trim();
        let dst = cobol_to_camel(caps[2].trim());
        if src == "SPACES" || src == "SPACE" {
            format!("{} = \"\";", dst)
        } else if src == "ZERO" || src == "ZEROS" {
            format!("{} = 0;", dst)
        } else {
            format!("{} = {};", dst, cobol_to_camel(src))
        }
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*ADD\s+(.+?)\s+TO\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("{} += {};", cobol_to_camel(caps[2].trim()), cobol_to_camel(caps[1].trim()))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*SUBTRACT\s+(.+?)\s+FROM\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("{} -= {};", cobol_to_camel(caps[2].trim()), cobol_to_camel(caps[1].trim()))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*MULTIPLY\s+(.+?)\s+BY\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("{} *= {};", cobol_to_camel(caps[2].trim()), cobol_to_camel(caps[1].trim()))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*DIVIDE\s+(.+?)\s+INTO\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("{} /= {};", cobol_to_camel(caps[2].trim()), cobol_to_camel(caps[1].trim()))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*COMPUTE\s+(\w+(?:-\w+)*)\s*=\s*(.+)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var = cobol_to_camel(&caps[1]);
        let expr = caps[2].trim();
        format!("{} = {};", var, expr)
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*IF\s+(.+?)\s+THEN\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("if ({}) {{", caps[1].trim())
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*IF\s+(.+?)\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let cond = caps[1].trim();
        let cond = cond.strip_suffix('.').unwrap_or(cond);
        format!("if ({}) {{", cond)
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*ELSE\s*$").unwrap();
    result = re.replace_all(&result, "} else {").to_string();

    let re = Regex::new(r"(?mi)^\s*END-IF\.?\s*$").unwrap();
    result = re.replace_all(&result, "}").to_string();

    let re = Regex::new(r"(?mi)^\s*PERFORM\s+(\w+(?:-\w+)*)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("{}();", cobol_to_camel(&caps[1]))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*PERFORM\s+VARYING\s+(\w+(?:-\w+)*)\s+FROM\s+(\d+)\s+BY\s+(\d+)\s+UNTIL\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var = cobol_to_camel(&caps[1]);
        let from = &caps[2];
        let by = &caps[3];
        let until = caps[4].trim().strip_suffix('.').unwrap_or(caps[4].trim());
        format!("for (int {} = {}; !({}); {} += {}) {{", var, from, until, var, by)
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*CALL\s+'([^']+)'\s+USING\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let prog = &caps[1];
        let params: Vec<String> = caps[2].split_whitespace()
            .map(|p| cobol_to_camel(p).to_string())
            .collect();
        format!("{}({});", prog.to_lowercase(), params.join(", "))
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*DISPLAY\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let arg = caps[1].trim();
        if arg.starts_with('\'') || arg.starts_with('"') {
            format!("System.out.println({});", arg)
        } else {
            format!("System.out.println({});", cobol_to_camel(arg))
        }
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*STOP\s+RUN\.?\s*$").unwrap();
    result = re.replace_all(&result, "return;").to_string();

    let re = Regex::new(r"(?mi)^\s*GOBACK\.?\s*$").unwrap();
    result = re.replace_all(&result, "return;").to_string();

    let re = Regex::new(r"(?mi)^\s*EXIT\s+PROGRAM\.?\s*$").unwrap();
    result = re.replace_all(&result, "return;").to_string();

    let re = Regex::new(r"(?mi)^\s*END\s+PROGRAM\s+\w+\.?\s*$").unwrap();
    result = re.replace_all(&result, "}").to_string();

    let re = Regex::new(r"(?mi)^\s*INITIALIZE\s+(.+?)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let vars: Vec<String> = caps[1].split_whitespace()
            .map(|v| format!("{} = null;", cobol_to_camel(v)))
            .collect();
        vars.join("\n")
    }).to_string();

    let re = Regex::new(r"(?mi)^\s*STRING\s+(.+?)\s+DELIMITED\s+BY\s+SIZE\s+INTO\s+(\w+(?:-\w+)*)\.?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let parts: Vec<String> = caps[1].split_whitespace()
            .filter(|s| s != &" ")
            .map(|s| cobol_to_camel(s))
            .collect();
        format!("{} = {};", cobol_to_camel(&caps[2]), parts.join(" + "))
    }).to_string();

    let lines: Vec<String> = result.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("       ") && l.trim() != ".")
        .collect();

    let body = lines.join("\n    ");

    format!("public class {} {{\n    public static void main(String[] args) {{\n    {}\n    }}\n}}", class_name, body)
}

pub fn cobol_to_csharp(source: &str) -> String {
    let mut result = cobol_to_java(source);

    let re = Regex::new(r"public static void main\(String\[\] args\)").unwrap();
    result = re.replace_all(&result, "public static void Main(string[] args)").to_string();

    let re = Regex::new(r"\bString\b").unwrap();
    result = re.replace_all(&result, "string").to_string();

    let re = Regex::new(r"System\.out\.println").unwrap();
    result = re.replace_all(&result, "Console.WriteLine").to_string();

    result
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str().to_lowercase().as_str(),
    }
}

fn cobol_to_camel(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_lowercase()
            } else {
                capitalize(part)
            }
        })
        .collect()
}

fn pic_to_java_type(pic: &str) -> &'static str {
    let upper = pic.to_uppercase();
    if upper.starts_with('X') || upper.starts_with('A') {
        "String"
    } else if upper.starts_with('9') {
        if upper.contains('V') {
            "double"
        } else if upper.contains("(10") || upper.contains("(9") {
            "long"
        } else {
            "int"
        }
    } else if upper.starts_with('S') && upper.contains('9') {
        if upper.contains('V') {
            "double"
        } else {
            "int"
        }
    } else {
        "String"
    }
}
