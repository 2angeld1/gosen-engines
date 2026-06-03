use serde_json::{json, Value};
use std::collections::HashMap;

struct Lang {
    label: &'static str,
    target_versions: &'static [&'static str],
    source_versions: &'static [&'static str],
    can_translate_to: &'static [&'static str],
    target_lang: &'static str,
}

static LANGUAGES: once_cell::sync::Lazy<HashMap<&'static str, Lang>> = once_cell::sync::Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("php", Lang {
        label: "PHP",
        target_versions: &["8.0", "8.1", "8.2", "8.3", "8.4"],
        source_versions: &["5.6", "7.0", "7.1", "7.2", "7.3", "7.4"],
        can_translate_to: &["PHP", "Python", "JavaScript", "Go", "Java", "Rust", "C#", "Ruby", "TypeScript", "COBOL", "C++"],
        target_lang: "PHP",
    });
    m.insert("javascript", Lang {
        label: "JavaScript",
        target_versions: &["TS 5.x"],
        source_versions: &["ES5", "ES6", "ES2016+"],
        can_translate_to: &["JavaScript", "Python", "Go", "Java", "Rust", "TypeScript", "PHP", "C#", "Ruby", "COBOL", "C++"],
        target_lang: "TypeScript",
    });
    m.insert("python", Lang {
        label: "Python",
        target_versions: &["3.11", "3.12", "3.13"],
        source_versions: &["2.7", "3.6", "3.7", "3.8", "3.9", "3.10", "3.11", "3.12", "3.13"],
        can_translate_to: &["Python", "Go", "JavaScript", "Rust", "Java", "C#", "Ruby", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "Python",
    });
    m.insert("java", Lang {
        label: "Java",
        target_versions: &["17", "21"],
        source_versions: &["8", "11", "17", "21"],
        can_translate_to: &["Java", "Python", "JavaScript", "Go", "Rust", "Kotlin", "C#", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "Java",
    });
    m.insert("go", Lang {
        label: "Go",
        target_versions: &["1.22", "1.23"],
        source_versions: &["1.16", "1.17", "1.18", "1.19", "1.20", "1.21", "1.22"],
        can_translate_to: &["Go", "Python", "JavaScript", "Java", "Rust", "C#", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "Go",
    });
    m.insert("rust", Lang {
        label: "Rust",
        target_versions: &["2021", "2024"],
        source_versions: &["2015", "2018", "2021", "2024"],
        can_translate_to: &["Rust", "Python", "JavaScript", "Go", "Java", "C#", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "Rust",
    });
    m.insert("csharp", Lang {
        label: "C#",
        target_versions: &["10", "11", "12"],
        source_versions: &["7.x", "8", "9", "10", "11", "12"],
        can_translate_to: &["C#", "Python", "JavaScript", "Go", "Java", "Rust", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "C#",
    });
    m.insert("ruby", Lang {
        label: "Ruby",
        target_versions: &["3.0", "3.1", "3.2", "3.3"],
        source_versions: &["2.7", "3.0", "3.1", "3.2", "3.3"],
        can_translate_to: &["Ruby", "Python", "JavaScript", "Go", "Java", "Rust", "PHP", "TypeScript", "COBOL", "C++"],
        target_lang: "Ruby",
    });
    m.insert("kotlin", Lang {
        label: "Kotlin",
        target_versions: &["1.8", "2.0"],
        source_versions: &["1.6", "1.8", "2.0"],
        can_translate_to: &["Kotlin", "Java", "Python", "JavaScript", "Go", "TypeScript", "Dart", "COBOL", "C++"],
        target_lang: "Kotlin",
    });
    m.insert("typescript", Lang {
        label: "TypeScript",
        target_versions: &["5.x"],
        source_versions: &["ES5", "ES6", "ES2016+", "TS 3.x", "TS 4.x", "TS 5.x"],
        can_translate_to: &["TypeScript", "Python", "JavaScript", "Go", "Java", "Rust", "C#", "PHP", "Ruby", "COBOL", "C++"],
        target_lang: "TypeScript",
    });
    m.insert("cobol", Lang {
        label: "COBOL",
        target_versions: &["COBOL-2002", "COBOL-2014"],
        source_versions: &["COBOL-85", "COBOL-2002", "COBOL-2014"],
        can_translate_to: &["COBOL", "Python", "Java", "C#", "Go", "Rust", "PHP", "JavaScript", "TypeScript", "C++"],
        target_lang: "COBOL",
    });
    m.insert("cpp", Lang {
        label: "C++",
        target_versions: &["C++17", "C++20", "C++23"],
        source_versions: &["C++98", "C++11", "C++14", "C++17", "C++20", "C++23"],
        can_translate_to: &["C++", "Python", "Java", "C#", "Go", "Rust", "PHP", "JavaScript", "TypeScript", "COBOL"],
        target_lang: "C++",
    });
    m.insert("visualbasic", Lang {
        label: "VB6/VB.NET",
        target_versions: &[""],
        source_versions: &["VB6", "VB.NET"],
        can_translate_to: &["Python", "Java", "C#", "Go", "Rust", "PHP", "JavaScript", "TypeScript", "COBOL", "C++"],
        target_lang: "C#",
    });
    m.insert("dart", Lang {
        label: "Dart/Flutter",
        target_versions: &["3.x"],
        source_versions: &["2.x", "3.x"],
        can_translate_to: &["Dart", "Python", "JavaScript", "Go", "Java", "Rust", "C#", "PHP", "TypeScript", "COBOL", "C++", "Kotlin"],
        target_lang: "Dart",
    });
    m.insert("jquery", Lang {
        label: "jQuery",
        target_versions: &["ES6"],
        source_versions: &["1.x", "2.x", "3.x"],
        can_translate_to: &["JavaScript", "React", "Vue"],
        target_lang: "JavaScript",
    });
    m.insert("react", Lang {
        label: "React",
        target_versions: &["18", "19"],
        source_versions: &["15", "16", "17", "18"],
        can_translate_to: &["React"],
        target_lang: "React",
    });
    m.insert("mysql", Lang {
        label: "MySQL",
        target_versions: &["8.0", "8.4"],
        source_versions: &["5.x", "8.0"],
        can_translate_to: &["PostgreSQL", "SQLite"],
        target_lang: "PostgreSQL",
    });
    m.insert("postgresql", Lang {
        label: "PostgreSQL",
        target_versions: &["15", "16", "17"],
        source_versions: &["13", "14", "15", "16", "17"],
        can_translate_to: &["PostgreSQL", "MySQL", "SQLite"],
        target_lang: "PostgreSQL",
    });
    m
});

pub fn normalize(name: &str) -> String {
    match name.to_lowercase().replace('-', " ").replace('_', " ").trim() {
        n if n == "c#" || n == "c sharp" || n == "csharp" => "csharp".to_string(),
        n if n == "c++" || n == "cpp" || n == "c plus plus" => "cpp".to_string(),
        n if n == "js" || n == "javascript" => "javascript".to_string(),
        n if n == "ts" || n == "typescript" => "typescript".to_string(),
        n if n == "py" || n == "python" => "python".to_string(),
        n if n == "rs" || n == "rust" => "rust".to_string(),
        n if n == "golang" || n == "go" => "go".to_string(),
        n if n == "kt" || n == "kotlin" => "kotlin".to_string(),
        n if n == "rb" || n == "ruby" => "ruby".to_string(),
        n if n == "cobol" || n == "cbl" || n == "cob" => "cobol".to_string(),
        n if n == "objective c" || n == "objectivec" || n == "objc" || n == "obj c" => "objectivec".to_string(),
        n if n == "dart" || n == "flutter" => "dart".to_string(),
        n if n == "vb" || n == "vb6" || n == "vb.net" || n == "visual basic" || n == "visualbasic" || n == "vba" => "visualbasic".to_string(),
        n if n == "jquery" => "jquery".to_string(),
        n if n == "react" || n == "reactjs" => "react".to_string(),
        n if n == "mysql" || n == "mysqli" => "mysql".to_string(),
        n if n == "pg" || n == "postgres" || n == "postgresql" => "postgresql".to_string(),
        n => n.to_string(),
    }
}

pub fn get_all() -> Value {
    let mut map = serde_json::Map::new();
    for (key, lang) in LANGUAGES.iter() {
        map.insert(key.to_string(), json!({
            "label": lang.label,
            "target_versions": lang.target_versions,
            "source_versions": lang.source_versions,
            "can_translate_to": lang.can_translate_to,
            "target_lang": lang.target_lang,
        }));
    }
    Value::Object(map)
}
