use regex::Regex;

pub fn jq_to_vanilla(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"\$\(document\)\.ready\(function\s*\([^)]*\)\s*\{").unwrap();
    result = re.replace_all(&result, "document.addEventListener('DOMContentLoaded', function() {").to_string();

    let re = Regex::new(r"\$\(function\s*\([^)]*\)\s*\{").unwrap();
    result = re.replace_all(&result, "document.addEventListener('DOMContentLoaded', function() {").to_string();

    let re = Regex::new(r#"\$\(['"]<(\w+)[^>]*>['"]\)"#).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        format!("document.createElement('{}')", &caps[1])
    }).to_string();

    let re = Regex::new(r"\$\(this\)").unwrap();
    result = re.replace_all(&result, "this").to_string();

    let re = Regex::new(r#"\$\(['"]([^'"]+)['"]\)"#).unwrap();
    result = re.replace_all(&result, "document.querySelector('$1')").to_string();

    let re = Regex::new(r"\$\(([a-zA-Z_]\w*)\)").unwrap();
    result = re.replace_all(&result, "$1").to_string();

    let re = Regex::new(r"\$\.ajax\(\s*\{").unwrap();
    result = re.replace_all(&result, "fetch({").to_string();

    let re = Regex::new(r"\$\.get\(([^,]+)(?:,\s*([^,]+))?(?:,\s*([^)]+))?\s*\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let url = &caps[1];
        let data = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let success = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        if !success.is_empty() {
            format!("fetch({}, {{method:'GET'}}).then(r => r.text()).then({})", url, success)
        } else {
            format!("fetch({})", url)
        }
    }).to_string();

    let re = Regex::new(r"\$\.post\(([^,]+)(?:,\s*([^,]+))?(?:,\s*([^)]+))?\s*\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let url = &caps[1];
        let data = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let success = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        if !success.is_empty() {
            format!("fetch({}, {{method:'POST', body: {}}}).then(r => r.text()).then({})", url, data, success)
        } else {
            format!("fetch({}, {{method:'POST', body: {}}})", url, data)
        }
    }).to_string();

    let re = Regex::new(r"\$\.getJSON\(([^,]+)(?:,\s*([^,]+))?(?:,\s*([^)]+))?\s*\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let url = &caps[1];
        let success = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        if !success.is_empty() {
            format!("fetch({}).then(r => r.json()).then({})", url, success)
        } else {
            format!("fetch({}).then(r => r.json())", url)
        }
    }).to_string();

    let re = Regex::new(r"\$\.each\(([^,]+),\s*(function\s*\([^)]*\)|[a-zA-Z_]\w*)\)").unwrap();
    result = re.replace_all(&result, "$1.forEach($2)").to_string();

    let re = Regex::new(r"\$\.map\(([^,]+),\s*(function\s*\([^)]*\)|[a-zA-Z_]\w*)\)").unwrap();
    result = re.replace_all(&result, "$1.map($2)").to_string();

    let re = Regex::new(r"\$\.grep\(([^,]+),\s*(function\s*\([^)]*\)|[a-zA-Z_]\w*)\)").unwrap();
    result = re.replace_all(&result, "$1.filter($2)").to_string();

    let re = Regex::new(r"\$\.trim\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "$1.trim()").to_string();

    let re = Regex::new(r"\$\.isArray\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "Array.isArray($1)").to_string();

    let re = Regex::new(r"\$\.type\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "typeof $1").to_string();

    let re = Regex::new(r"\$\.inArray\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "$2.indexOf($1)").to_string();

    let re = Regex::new(r"\$\.extend\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "Object.assign($1)").to_string();

    let re = Regex::new(r"\$\.proxy\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "$1.bind($2)").to_string();

    let re = Regex::new(r"\$\.noConflict\(\)").unwrap();
    result = re.replace_all(&result, "/* $.noConflict() removed */").to_string();

    let re = Regex::new(r"\.text\(\)").unwrap();
    result = re.replace_all(&result, ".textContent").to_string();

    let re = Regex::new(r"\.text\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".textContent = $1").to_string();

    let re = Regex::new(r"\.html\(\)").unwrap();
    result = re.replace_all(&result, ".innerHTML").to_string();

    let re = Regex::new(r"\.html\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".innerHTML = $1").to_string();

    let re = Regex::new(r"\.val\(\)").unwrap();
    result = re.replace_all(&result, ".value").to_string();

    let re = Regex::new(r"\.val\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".value = $1").to_string();

    let re = Regex::new(r"\.attr\(([^,)]+)\)").unwrap();
    result = re.replace_all(&result, ".getAttribute($1)").to_string();

    let re = Regex::new(r"\.attr\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".setAttribute($1, $2)").to_string();

    let re = Regex::new(r#"\.css\(['"]?([a-zA-Z-]+)['"]?\)"#).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let prop = &caps[1];
        format!(".style['{}']", prop)
    }).to_string();

    let re = Regex::new(r#"\.css\(['"]?([a-zA-Z-]+)['"]?,\s*([^)]+)\)"#).unwrap();
    result = re.replace_all(&result, ".style['$1'] = $2").to_string();

    let re = Regex::new(r"\.addClass\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".classList.add($1)").to_string();

    let re = Regex::new(r"\.removeClass\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".classList.remove($1)").to_string();

    let re = Regex::new(r"\.toggleClass\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".classList.toggle($1)").to_string();

    let re = Regex::new(r"\.hasClass\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".classList.contains($1)").to_string();

    let re = Regex::new(r"\.on\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".addEventListener($1, $2)").to_string();

    let re = Regex::new(r"\.off\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".removeEventListener($1, $2)").to_string();

    let re = Regex::new(r"\.trigger\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".dispatchEvent(new Event($1))").to_string();

    let re = Regex::new(r"\.show\(\)").unwrap();
    result = re.replace_all(&result, ".style.display = ''").to_string();

    let re = Regex::new(r"\.hide\(\)").unwrap();
    result = re.replace_all(&result, ".style.display = 'none'").to_string();

    let re = Regex::new(r"\.remove\(\)").unwrap();
    result = re.replace_all(&result, ".remove()").to_string();

    let re = Regex::new(r"\.empty\(\)").unwrap();
    result = re.replace_all(&result, ".innerHTML = ''").to_string();

    let re = Regex::new(r"\.append\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".insertAdjacentHTML('beforeend', $1)").to_string();

    let re = Regex::new(r"\.prepend\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".insertAdjacentHTML('afterbegin', $1)").to_string();

    let re = Regex::new(r"\.before\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".insertAdjacentHTML('beforebegin', $1)").to_string();

    let re = Regex::new(r"\.after\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".insertAdjacentHTML('afterend', $1)").to_string();

    let re = Regex::new(r"\.find\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".querySelectorAll($1)").to_string();

    let re = Regex::new(r"\.closest\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, ".closest($1)").to_string();

    let re = Regex::new(r"\.parent\(\)").unwrap();
    result = re.replace_all(&result, ".parentElement").to_string();

    let re = Regex::new(r"\.children\(\)").unwrap();
    result = re.replace_all(&result, ".children").to_string();

    let re = Regex::new(r"\.first\(\)").unwrap();
    result = re.replace_all(&result, ".firstElementChild").to_string();

    let re = Regex::new(r"\.last\(\)").unwrap();
    result = re.replace_all(&result, ".lastElementChild").to_string();

    let re = Regex::new(r"\.slideUp\(([^)]*)\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let dur = caps.get(1).map(|m| m.as_str()).unwrap_or("400");
        let d = if dur.is_empty() { "400" } else { dur };
        format!(".style.transition = 'height {}ms'; .style.height = '0'; .style.overflow = 'hidden'", d)
    }).to_string();

    let re = Regex::new(r"\.fadeIn\(([^)]*)\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let dur = caps.get(1).map(|m| m.as_str()).unwrap_or("400");
        let d = if dur.is_empty() { "400" } else { dur };
        format!(".style.transition = 'opacity {}ms'; .style.opacity = '1'", d)
    }).to_string();

    let re = Regex::new(r"\.fadeOut\(([^)]*)\)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let dur = caps.get(1).map(|m| m.as_str()).unwrap_or("400");
        let d = if dur.is_empty() { "400" } else { dur };
        format!(".style.transition = 'opacity {}ms'; .style.opacity = '0'", d)
    }).to_string();

    let re = Regex::new(r"\$\.Deferred\(\)").unwrap();
    result = re.replace_all(&result, "new Promise((resolve, reject) => { const def = { resolve, reject, promise: null }; def.promise = () => def; return def; })").to_string();

    let re = Regex::new(r"\$\.when\(([^)]+)\)\.then").unwrap();
    result = re.replace_all(&result, "Promise.all([$1]).then").to_string();

    let re = Regex::new(r"\$\.merge\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "[...$1, ...$2]").to_string();

    result
}
