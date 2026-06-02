pub mod php;
pub mod js_ts;
pub mod cobol;
pub mod objc_swift;
pub mod java_kotlin;
pub mod vb_csharp;
pub mod jquery;
pub mod react;
pub mod mysql_postgres;
pub mod html_tsx;

pub fn translate(source: &str, source_lang: &str, target_lang: &str) -> Option<String> {
    match (source_lang.to_lowercase().as_str(), target_lang.to_lowercase().as_str()) {
        ("php", "php") => Some(php::php_to_php(source)),
        ("javascript", "typescript") | ("js", "typescript") => Some(js_ts::js_to_ts(source)),
        ("html", "tsx") | ("html", "react") => Some(html_tsx::html_to_tsx(source)),
        ("cobol", "java") => Some(cobol::cobol_to_java(source)),
        ("cobol", "csharp") => Some(cobol::cobol_to_csharp(source)),
        ("objectivec", "swift") => Some(objc_swift::objc_to_swift(source)),
        ("java", "kotlin") => Some(java_kotlin::java_to_kotlin(source)),
        ("visualbasic", "csharp") | ("visualbasic", "c#") => Some(vb_csharp::vb_to_csharp(source)),
        ("jquery", "javascript") | ("jquery", "js") => Some(jquery::jq_to_vanilla(source)),
        ("react", "react") => Some(react::react_class_to_hooks(source)),
        ("mysql", "postgresql") | ("mysql", "postgres") => Some(mysql_postgres::mysql_to_postgres(source)),
        _ => None,
    }
}
