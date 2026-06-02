use regex::Regex;

pub fn objc_to_swift(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)^\s*#import\s+<([^>]+)>").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let path = &caps[1];
        let framework = path.split('/').next().unwrap_or(path);
        format!("import {}", framework)
    }).to_string();
    let re = Regex::new(r#"(?m)^\s*#import\s+"[^"]+"\s*$"#).unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"(?m)^\s*#(?:import|include|define|ifdef|ifndef|endif|pragma|undef)\s.*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?m)^\s*@interface\s+(\w+)\s*:\s*(\w+)(?:\s*<([^>]+)>)?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let class_name = &caps[1];
        let super_name = &caps[2];
        let proto = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        if proto.is_empty() {
            format!("class {}: {} {{", class_name, super_name)
        } else {
            format!("class {}: {}, {} {{", class_name, super_name, proto)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^\s*@interface\s+(\w+)\s*\(\s*\)\s*$").unwrap();
    result = re.replace_all(&result, "extension $1 {").to_string();

    let re = Regex::new(r"(?m)^\s*@interface\s+(\w+)\s*\((\w+)\)\s*$").unwrap();
    result = re.replace_all(&result, "// MARK: - $2\nextension $1 {").to_string();

    let re = Regex::new(r"(?m)^\s*@implementation\s+\w+(?:\s*\(.*?\))?\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?m)^\s*@end\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"(?m)^\s*@(synthesize|dynamic)\s+.*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(
        r"(?m)^\s*@property\s*(?:\(([^)]*)\))?\s*(?:IBOutlet\s+)?(.+?)\s*;\s*$"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let attrs = caps.get(1).map(|a| a.as_str()).unwrap_or("");
        let decl = caps[2].trim();
        let is_readonly = attrs.contains("readonly");
        let kw = if is_readonly { "let" } else { "var" };
        let mut tokens: Vec<&str> = decl.split_whitespace().collect();
        if tokens.len() >= 2 {
            let mut name = tokens.pop().unwrap_or("unknown").trim_start_matches('*').trim();
            if name.is_empty() {
                name = tokens.pop().unwrap_or("unknown");
            }
            let typ_raw = tokens.join(" ");
            let swift_type = objc_type_to_swift(&typ_raw);
            let swift_type = if typ_raw.ends_with('*') && !swift_type.ends_with('*') {
                swift_type.to_string()
            } else {
                swift_type
            };
            format!("{} {}: {}", kw, name, swift_type)
        } else if tokens.len() == 1 {
            let name = tokens[0].trim_start_matches('*');
            format!("{} {}: Any", kw, name)
        } else {
            format!("{} unknown: Any", kw)
        }
    }).to_string();

    let re = Regex::new(r"(^|[^.\w])_(\w+)\b").unwrap();
    result = re.replace_all(&result, "${1}self.${2}").to_string();

    let mut lines: Vec<String> = Vec::new();
    for line in result.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with('-') || trimmed.starts_with('+'))
            && trimmed.contains('(')
            && trimmed.ends_with(';')
            && !trimmed.contains('@')
        {
            continue;
        }
        if (trimmed.starts_with('-') || trimmed.starts_with('+'))
            && trimmed.contains('(')
            && trimmed.ends_with('{')
        {
            let swift_line = convert_objc_method_line(trimmed);
            lines.push(swift_line);
            continue;
        }
        lines.push(line.to_string());
    }
    result = lines.join("\n");

    let re = Regex::new(r"\[(\w+)\s+(\w+)((?:\s+\w+(?::\s*[^\]]+)?)*)\]").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let receiver = &caps[1];
        let selector_start = &caps[2];
        let rest = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let rest = rest.trim();
        if rest.is_empty() {
            format!("{}.{}()", receiver, selector_start)
        } else if !rest.contains(':') {
            format!("{}.{}({})", receiver, selector_start, rest)
        } else {
            let mut args = Vec::new();
            let re_arg = Regex::new(r"(\w+)\s*:\s*([^\s]+(?:\s+[^\s]+)*?)(?=\s+\w+\s*:|\s*$)").unwrap();
            for cap in re_arg.captures_iter(rest) {
                let label = &cap[1];
                let val = cap[2].trim();
                if args.is_empty() {
                    args.push(format!("{}: {}", label, val));
                } else {
                    args.push(format!("{}: {}", label, val));
                }
            }
            if args.is_empty() {
                format!("{}.{}()", receiver, selector_start)
            } else {
                format!("{}.{}({})", receiver, selector_start, args.join(", "))
            }
        }
    }).to_string();

    let re = Regex::new(r"\[\[(\w+)\s+alloc\]\s+(init\w*(?::[^\]]*)?)\]").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let cls = &caps[1];
        let init_selector = caps[2].trim();
        if init_selector == "init" {
            format!("{}()", cls)
        } else {
            let re_init = Regex::new(r"init\w*((?::[^:]+)*)").unwrap();
            if let Some(init_caps) = re_init.captures(&init_selector) {
                let args_str = init_caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if args_str.is_empty() {
                    format!("{}()", cls)
                } else {
                    let mut args = Vec::new();
                    for _cap in re_init.captures_iter(args_str) {
                        args.push("...".to_string());
                    }
                    format!("{}({})", cls, args.join(", "))
                }
            } else {
                format!("{}()", cls)
            }
        }
    }).to_string();

    let re = Regex::new(r#"NSLog\(@"([^"]*)"\s*(?:,\s*(.+?))?\)"#).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let fmt = &caps[1];
        let args = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if args.is_empty() {
            format!("print(\"{}\")", fmt)
        } else {
            format!("print(\"{}\", {})", fmt, args)
        }
    }).to_string();
    let re = Regex::new(r#"NSLog\(@"([^"]*)"\)"#).unwrap();
    result = re.replace_all(&result, "print(\"$1\")").to_string();
    let re = Regex::new(r"NSAssert\(([^,]+),\s*(.+)\)").unwrap();
    result = re.replace_all(&result, "assert($1, $2)").to_string();
    let re = Regex::new(r"@selector\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "#selector($1)").to_string();

    let re = Regex::new(r#"@\"([^"]*)\""#).unwrap();
    result = re.replace_all(&result, "\"$1\"").to_string();

    let re = Regex::new(r"@\(([^)]+)\)").unwrap();
    result = re.replace_all(&result, "$1").to_string();

    let re = Regex::new(r"@\{([^}]*)\}").unwrap();
    result = re.replace_all(&result, "[$1]").to_string();

    let re = Regex::new(r"@\[([^\]]*)\]").unwrap();
    result = re.replace_all(&result, "[$1]").to_string();

    let re = Regex::new(r"@YES\b").unwrap();
    result = re.replace_all(&result, "true").to_string();
    let re = Regex::new(r"@NO\b").unwrap();
    result = re.replace_all(&result, "false").to_string();

    let re = Regex::new(r"@(\d+\.?\d*(?:f|d)?)\b").unwrap();
    result = re.replace_all(&result, "$1").to_string();

    let re = Regex::new(r"\bYES\b").unwrap();
    result = re.replace_all(&result, "true").to_string();
    let re = Regex::new(r"\bNO\b").unwrap();
    result = re.replace_all(&result, "false").to_string();

    let re = Regex::new(r"(?m)^\s*(if|while|for)\s*\(([^)]*)\)\s*$").unwrap();
    result = re.replace_all(&result, "$1 $2 {").to_string();
    let re = Regex::new(r"(?m)^\s*(if|while|for)\s*\(([^)]*)\)\s*([^{;]+);\s*$").unwrap();
    result = re.replace_all(&result, "$1 $2 {\n    $3;\n}").to_string();

    let re = Regex::new(
        r"for\s*\(\s*(?:int|NSInteger|long)\s+(\w+)\s*=\s*(\d+)\s*;\s*\w+\s*([<>=!]+)\s*(\w+)\s*;\s*\w+\s*(\+\+|--)\s*\)"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let from = &caps[2];
        let op = &caps[3];
        let to = &caps[4];
        let inc = &caps[5];
        if inc == "++" && (op == "<" || op == "<=") {
            let exclusive = if op == "<=" { format!("({} + 1)", to) } else { to.to_string() };
            format!("for {} in {}..<{} {{", &caps[1], from, exclusive)
        } else if inc == "--" && (op == ">" || op == ">=") {
            let exclusive = if op == ">=" { format!("({} - 1)", to) } else { to.to_string() };
            format!("for {} in stride(from: {}, to: {}, by: -1) {{", &caps[1], from, exclusive)
        } else {
            format!("for {} in stride(from: {}, to: {}, by: 1) {{ // FIXME", &caps[1], from, to)
        }
    }).to_string();

    let re = Regex::new(r"for\s*\([^)]+?\s+(\w+)\s+in\s+(\w+)\s*\)").unwrap();
    result = re.replace_all(&result, "for $1 in $2 {").to_string();

    let re = Regex::new(r"@try\s*\{").unwrap();
    result = re.replace_all(&result, "do {").to_string();
    let re = Regex::new(r"@catch\s*\(([^)]*)\)\s*\{").unwrap();
    result = re.replace_all(&result, "catch { // FIXME: was catch($1)").to_string();
    let re = Regex::new(r"@finally\s*\{").unwrap();
    result = re.replace_all(&result, "defer {").to_string();

    let re = Regex::new(r"\((\w+\s*\**)\)\s*(\w+)").unwrap();
    result = re.replace_all(&result, "$1($2)").to_string();
    let re = Regex::new(r#"\(NSString\s*\*\)\s*"([^"]*)""#).unwrap();
    result = re.replace_all(&result, "\"$1\"").to_string();

    let type_map: [(&str, &str); 20] = [
        (r"\bNSString\s*\*", "String"),
        (r"\bNSMutableString\s*\*", "String"),
        (r"\bNSInteger\b", "Int"),
        (r"\bNSUInteger\b", "UInt"),
        (r"\bCGFloat\b", "CGFloat"),
        (r"\bCGRect\b", "CGRect"),
        (r"\bCGPoint\b", "CGPoint"),
        (r"\bCGSize\b", "CGSize"),
        (r"\bBOOL\b", "Bool"),
        (r"\binstancetype\b", "Self"),
        (r"\bid\b", "Any"),
        (r"\bNSArray\s*\*", "[Any]"),
        (r"\bNSMutableArray\s*\*", "[Any]"),
        (r"\bNSDictionary\s*\*", "[AnyHashable: Any]"),
        (r"\bNSMutableDictionary\s*\*", "[AnyHashable: Any]"),
        (r"\bNSSet\s*\*", "Set<AnyHashable>"),
        (r"\bNSMutableSet\s*\*", "Set<AnyHashable>"),
        (r"\bNSNumber\s*\*", "NSNumber"),
        (r"\bNSDate\s*\*", "Date"),
        (r"\bNSData\s*\*", "Data"),
    ];
    for (pattern, swift_type) in &type_map {
        let re = Regex::new(pattern).unwrap();
        result = re.replace_all(&result, *swift_type).to_string();
    }

    let re = Regex::new(r#"\(String\)\s*"([^"]*)""#).unwrap();
    result = re.replace_all(&result, "\"$1\"").to_string();

    let re = Regex::new(r"\b(\w+)\.length\b").unwrap();
    result = re.replace_all(&result, "$1.count").to_string();

    let re = Regex::new(
        r"dispatch_async\(dispatch_get_main_queue\(\)\s*,\s*\^\s*\{"
    ).unwrap();
    result = re.replace_all(&result, "DispatchQueue.main.async {").to_string();

    let re = Regex::new(r"\^\s*\(([^)]*)\)\s*\{").unwrap();
    result = re.replace_all(&result, "{ ($1) in").to_string();
    let re = Regex::new(r"\^\s*\{").unwrap();
    result = re.replace_all(&result, "{").to_string();

    let re = Regex::new(r"CGRectMake\(([^,]+),\s*([^,]+),\s*([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "CGRect(x: $1, y: $2, width: $3, height: $4)").to_string();
    let re = Regex::new(r"CGPointMake\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "CGPoint(x: $1, y: $2)").to_string();
    let re = Regex::new(r"CGSizeMake\(([^,]+),\s*([^)]+)\)").unwrap();
    result = re.replace_all(&result, "CGSize(width: $1, height: $2)").to_string();

    let re = Regex::new(r"(?m)^\s*typedef\s+(.+?)\s+(\w+);\s*$").unwrap();
    result = re.replace_all(&result, "typealias $2 = $1").to_string();

    let re = Regex::new(r"(?m)^\s*@protocol\s+(\w+)(?:\s*<([^>]+)>)?\s*$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let name = &caps[1];
        let parent = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if parent.is_empty() {
            format!("protocol {} {{", name)
        } else {
            format!("protocol {}: {} {{", name, parent)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^\s*@optional\s*$").unwrap();
    result = re.replace_all(&result, "// @objc optional").to_string();
    let re = Regex::new(r"(?m)^\s*@required\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"typedef\s+NS_ENUM\(([^,]+),\s*(\w+)\)\s*\{").unwrap();
    result = re.replace_all(&result, "enum $2: $1 {").to_string();
    let re = Regex::new(r"typedef\s+NS_OPTIONS\(([^,]+),\s*(\w+)\)\s*\{").unwrap();
    result = re.replace_all(&result, "struct $2: OptionSet {\n    let rawValue: $1").to_string();

    let re = Regex::new(r"\{\s*\n\s*\n").unwrap();
    result = re.replace_all(&result, "{\n").to_string();
    let re = Regex::new(r"\n\s*\n\s*\n").unwrap();
    result = re.replace_all(&result, "\n\n").to_string();

    if result.contains("class ") || result.contains("extension ") || result.contains("protocol ") {
        result.push_str("\n}");
    }

    result
}

fn objc_type_to_swift(typ: &str) -> String {
    match typ.trim() {
        "void" | "IBAction" => "Void".to_string(),
        "BOOL" => "Bool".to_string(),
        "NSInteger" | "NSInteger *" => "Int".to_string(),
        "NSUInteger" | "NSUInteger *" => "UInt".to_string(),
        "int" | "int *" => "Int32".to_string(),
        "long" => "Int".to_string(),
        "float" | "float *" => "Float".to_string(),
        "double" | "double *" => "Double".to_string(),
        "CGFloat" => "CGFloat".to_string(),
        "NSString" | "NSString *" => "String".to_string(),
        "NSMutableString" | "NSMutableString *" => "String".to_string(),
        "NSArray" | "NSArray *" => "[Any]".to_string(),
        "NSMutableArray" | "NSMutableArray *" => "[Any]".to_string(),
        "NSDictionary" | "NSDictionary *" => "[AnyHashable: Any]".to_string(),
        "NSMutableDictionary" | "NSMutableDictionary *" => "[AnyHashable: Any]".to_string(),
        "NSSet" | "NSSet *" => "Set<AnyHashable>".to_string(),
        "id" => "Any".to_string(),
        "instancetype" => "Self".to_string(),
        "Class" => "AnyClass".to_string(),
        "SEL" => "Selector".to_string(),
        "NSNumber" | "NSNumber *" => "NSNumber".to_string(),
        "NSDate" | "NSDate *" => "Date".to_string(),
        "NSData" | "NSData *" => "Data".to_string(),
        "NSURL" | "NSURL *" => "URL".to_string(),
        "NSError" | "NSError *" => "Error".to_string(),
        "NSException" | "NSException *" => "NSException".to_string(),
        _ => {
            let t = typ.trim().trim_end_matches('*').trim();
            if t.is_empty() { "Any".to_string() } else { t.to_string() }
        }
    }
}

fn convert_objc_method_line(line: &str) -> String {
    let line = line.trim();
    let is_class = line.starts_with('+');
    let kw = if is_class { "class func" } else { "func" };

    let after_prefix = line.trim_start_matches('-').trim_start_matches('+').trim_start();

    let re_ret = Regex::new(r"^\(([^)]*)\)\s*").unwrap();
    let ret_type = re_ret.captures(after_prefix)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| "void".to_string());

    let without_ret = re_ret.replace(after_prefix, "").to_string();

    let without_brace = without_ret.trim_end_matches('{').trim_end().to_string();

    let re_param = Regex::new(r":\s*\(([^)]*)\)\s*(\w+)").unwrap();
    let param_captures: Vec<(String, String)> = re_param.captures_iter(&without_brace)
        .map(|c| (c[1].trim().to_string(), c[2].to_string()))
        .collect();

    let method_name = without_brace.split(':').next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "method".to_string());

    let ret_str = if ret_type == "void" || ret_type == "IBAction" {
        String::new()
    } else {
        format!(" -> {}", objc_type_to_swift(&ret_type))
    };

    if param_captures.is_empty() {
        format!("{} {}(){} {{", kw, method_name, ret_str)
    } else {
        let params: Vec<String> = param_captures.iter()
            .map(|(ptype, pname)| {
                let swift_type = objc_type_to_swift(ptype);
                format!("_ {}: {}", pname, swift_type)
            })
            .collect();
        format!("{} {}({}){} {{", kw, method_name, params.join(", "), ret_str)
    }
}
