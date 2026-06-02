use regex::Regex;

pub fn html_to_tsx(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r##"\bclass\s*="([^"]*)"##).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let val = &caps[1];
        if val.contains('{') {
            caps[0].to_string()
        } else {
            format!("className=\"{}\"", val)
        }
    }).to_string();

    let re = Regex::new(r##"\bfor\s*="([^"]*)"##).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let val = &caps[1];
        if val.contains('{') {
            caps[0].to_string()
        } else {
            format!("htmlFor=\"{}\"", val)
        }
    }).to_string();

    let void_elements = ["br", "hr", "img", "input", "meta", "link", "area", "base", "col", "embed", "source", "track", "wbr"];
    for el in &void_elements {
        let re = Regex::new(&format!(r"(?m)<{}([^>]*[^/])>\s*</{}>", el, el)).unwrap();
        result = re.replace_all(&result, |_: &regex::Captures| format!("<{} />", el)).to_string();
    }

    let root_count = result.trim().matches("<").count() - result.trim().matches("</").count();
    if root_count > 2 {
        result = format!("<>\n{}\n</>", result);
    }

    result
}
