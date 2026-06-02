use regex::Regex;

pub fn vb_to_csharp(source: &str) -> String {
    let mut result = source.to_string();

    let re = Regex::new(r"(?m)^\s*'").unwrap();
    result = re.replace_all(&result, "//").to_string();
    let re = Regex::new(r"Rem\s+").unwrap();
    result = re.replace_all(&result, "// ").to_string();

    let re = Regex::new(r"(?m)^Attribute\s+\w+\s*=\s*.*$").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"(?m)^VERSION\s+.*$").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"(?m)^Begin\s+\{[^}]+\}\s+\w+.*$").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"(?m)^Begin\s+\w+.*$").unwrap();
    result = re.replace_all(&result, "").to_string();
    let re = Regex::new(r"(?m)^End\s*$").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(
        r"(?m)^(\s*)(?:Dim|Private|Public|Static|WithEvents)\s+(?:ByVal\s+|ByRef\s+|Optional\s+)?(\w+)\s+As\s+(New\s+)?(\w+(?:\([^)]*\))?(?:\s*\*)?(?:\s*\d+)?)(?:\s*=\s*(.+))?"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let name = &caps[2];
        let is_new = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let raw_type = caps.get(4).map(|m| m.as_str()).unwrap_or("Object");
        let init = caps.get(5).map(|m| m.as_str().trim()).unwrap_or("");

        let cstype = vb_type_to_csharp(raw_type);

        if !is_new.is_empty() && init.is_empty() {
            format!("{}{} {} = new {}();", indent, cstype, name, cstype)
        } else if !init.is_empty() {
            format!("{}{} {} = {};", indent, cstype, name, init)
        } else {
            format!("{}{} {};", indent, cstype, name)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:Public|Private)?\s*Const\s+(\w+)\s+As\s+(\w+)\s*=\s*(.+)$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let name = &caps[2];
        let typ = vb_type_to_csharp(&caps[3]);
        let val = &caps[4];
        format!("{}const {} {} = {};", indent, typ, name, val)
    }).to_string();

    let re = Regex::new(
        r"(?m)^(\s*)((?:Public|Private|Friend)\s+)?(?:Static\s+)?(Sub|Function|Property\s+(?:Get|Let|Set))(?:\s+(\w+))?\s*\(([^)]*)\)\s*(?:As\s+(\w+))?"
    ).unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let vis = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("private");
        let kind = &caps[3];
        let name = &caps[4];
        let params_raw = caps.get(5).map(|m| m.as_str().trim()).unwrap_or("");
        let ret_type = caps.get(6).map(|m| vb_type_to_csharp(m.as_str()));

        let csharp_vis = match vis {
            "Friend" => "internal",
            "Public" => "public",
            "Private" => "private",
            _ => vis,
        };

        let params: Vec<String> = if params_raw.is_empty() {
            Vec::new()
        } else {
            params_raw.split(',').map(|p| {
                let p = p.trim();
                if p.is_empty() { return String::new(); }
                let re_p = Regex::new(r"(?:Optional\s+)?(?:ByVal|ByRef)?\s*(\w+)\s+As\s+(\w+)").unwrap();
                if let Some(cap) = re_p.captures(p) {
                    let pname = &cap[1];
                    let ptype = vb_type_to_csharp(&cap[2]);
                    format!("{} {}", ptype, pname)
                } else {
                    p.to_string()
                }
            }).collect()
        };

        match kind {
            "Sub" => {
                format!("{}{} void {}({}) {{", indent, csharp_vis, name, params.join(", "))
            }
            "Function" => {
                let ret = ret_type.as_deref().unwrap_or("void");
                format!("{}{} {} {}({}) {{", indent, csharp_vis, ret, name, params.join(", "))
            }
            "Property Get" => {
                let ret = ret_type.as_deref().unwrap_or("object");
                format!("{}{} {} {} {{ get {{", indent, csharp_vis, ret, name)
            }
            "Property Let" | "Property Set" => {
                if params.is_empty() {
                    format!("{}set {{", indent)
                } else {
                    format!("{}set {{", indent)
                }
            }
            _ => caps[0].to_string(),
        }
    }).to_string();

    let re = Regex::new(r"(?m)^(\s*)End\s+(Sub|Function|Property|If|Select|With|Type|Class|Enum)\s*$").unwrap();
    result = re.replace_all(&result, "$1}").to_string();

    let re = Regex::new(r"If[^\S\n]+([^\n]+?)[^\S\n]+Then[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "if ($1) { $2; }").to_string();

    let re = Regex::new(r"(?m)^(\s*)If\s+(.+?)\s+Then\s*$").unwrap();
    result = re.replace_all(&result, "${1}if (${2}) {").to_string();

    let re = Regex::new(r"(?m)^(\s*)ElseIf\s+(.+?)\s+Then\s*$").unwrap();
    result = re.replace_all(&result, "$1} else if ($2) {").to_string();

    let re = Regex::new(r"(?m)^(\s*)Else\s*$").unwrap();
    result = re.replace_all(&result, "$1} else {").to_string();

    let re = Regex::new(r"For[^\S\n]+(\w+)[^\S\n]*=[^\S\n]*([^\s]+)[^\S\n]+To[^\S\n]+([^\s]+)(?:[^\S\n]+Step[^\S\n]+(-?\d+))?").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var = &caps[1];
        let start = &caps[2];
        let end = &caps[3];
        let step = caps.get(4).map(|m| m.as_str()).unwrap_or("1");
        if step == "-1" || step.starts_with('-') {
            format!("for (int {} = {}; {} >= {}; {}--)", var, start, var, end, var)
        } else if step != "1" {
            format!("for (int {} = {}; {} <= {}; {} += {})", var, start, var, end, var, step)
        } else {
            format!("for (int {} = {}; {} <= {}; {}++)", var, start, var, end, var)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^(\s*)Next(?:\s+\w+)?\s*$").unwrap();
    result = re.replace_all(&result, "$1}").to_string();

    let re = Regex::new(r"For[^\S\n]+Each[^\S\n]+(\w+)[^\S\n]+In[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "foreach (var $1 in $2)").to_string();

    let re = Regex::new(r"Do[^\S\n]+While[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "while ($1) {").to_string();
    let re = Regex::new(r"Do[^\S\n]+Until[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "while (!($1)) {").to_string();
    let re = Regex::new(r"(?m)^\s*Do\s*$").unwrap();
    result = re.replace_all(&result, "do {").to_string();
    let re = Regex::new(r"(?m)^(\s*)Loop\s*(?:While|Until)?\s*(.*)?$").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let indent = &caps[1];
        let cond = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        if cond.is_empty() {
            format!("{}}} while (true);", indent)
        } else {
            format!("{}}} while ({});", indent, cond)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^(\s*)Wend\s*$").unwrap();
    result = re.replace_all(&result, "$1}").to_string();

    let re = Regex::new(r"Select[^\S\n]+Case[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "switch ($1) {").to_string();
    let re = Regex::new(r"(?m)^(\s*)Case\s+(.+?)(?::|$)\s*$").unwrap();
    result = re.replace_all(&result, "${1}case ${2}:").to_string();
    let re = Regex::new(r"(?m)^(\s*)Case\s+Else\s*$").unwrap();
    result = re.replace_all(&result, "${1}default:").to_string();
    let re = Regex::new(r"(?m)^(\s*)End\s+Select\s*$").unwrap();
    result = re.replace_all(&result, "$1}").to_string();

    let re = Regex::new(r"With[^\S\n]+([^\n]+)").unwrap();
    result = re.replace_all(&result, "with ($1) {").to_string();

    let re = Regex::new(r"<>").unwrap();
    result = re.replace_all(&result, "!=").to_string();

    let re = Regex::new(r"\bAnd\b").unwrap();
    result = re.replace_all(&result, "&&").to_string();
    let re = Regex::new(r"\bOr\b").unwrap();
    result = re.replace_all(&result, "||").to_string();
    let re = Regex::new(r"\bNot\b").unwrap();
    result = re.replace_all(&result, "!").to_string();
    let re = Regex::new(r"\bXor\b").unwrap();
    result = re.replace_all(&result, "^").to_string();
    let re = Regex::new(r"\bMod\b").unwrap();
    result = re.replace_all(&result, "%").to_string();

    let re = Regex::new(r"\s*&\s*").unwrap();
    result = re.replace_all(&result, " + ").to_string();

    let re = Regex::new(r"MsgBox\s*\(([^\n]+)\)").unwrap();
    result = re.replace_all(&result, "MessageBox.Show($1);").to_string();
    let re = Regex::new(r"(?m)^(\s*)MsgBox\s+([^\n]+)\s*$").unwrap();
    result = re.replace_all(&result, "${1}MessageBox.Show(${2});").to_string();

    let re = Regex::new(r"InputBox\s*\((.+?)\)").unwrap();
    result = re.replace_all(&result, "Interaction.InputBox($1)").to_string();

    let convs: [(&str, &str); 12] = [
        (r"CInt\s*\((.+?)\)", "Convert.ToInt32($1)"),
        (r"CLng\s*\((.+?)\)", "Convert.ToInt64($1)"),
        (r"CSng\s*\((.+?)\)", "Convert.ToSingle($1)"),
        (r"CDbl\s*\((.+?)\)", "Convert.ToDouble($1)"),
        (r"CStr\s*\((.+?)\)", "Convert.ToString($1)"),
        (r"CBool\s*\((.+?)\)", "Convert.ToBoolean($1)"),
        (r"CDate\s*\((.+?)\)", "Convert.ToDateTime($1)"),
        (r"CByte\s*\((.+?)\)", "Convert.ToByte($1)"),
        (r"CCur\s*\((.+?)\)", "Convert.ToDecimal($1)"),
        (r"CStr\s*\((.+?)\)", "Convert.ToString($1)"),
        (r"Val\s*\((.+?)\)", "Convert.ToDouble($1)"),
        (r"CVar\s*\((.+?)\)", "$1"),
    ];
    for (pat, repl) in &convs {
        let re = Regex::new(pat).unwrap();
        result = re.replace_all(&result, *repl).to_string();
    }

    let str_funcs: [(&str, &str); 15] = [
        (r"Len\s*\((.+?)\)", "$1.Length"),
        (r"Trim\s*\((.+?)\)", "$1.Trim()"),
        (r"LTrim\s*\((.+?)\)", "$1.TrimStart()"),
        (r"RTrim\s*\((.+?)\)", "$1.TrimEnd()"),
        (r"UCase\s*\((.+?)\)", "$1.ToUpper()"),
        (r"LCase\s*\((.+?)\)", "$1.ToLower()"),
        (r"Space\s*\((.+?)\)", "new string(' ', $1)"),
        (r"Asc\s*\((.+?)\)", "(int)$1[0]"),
        (r"Chr\s*\((.+?)\)", "(char)$1"),
        (r"AscW\s*\((.+?)\)", "(int)$1[0]"),
        (r"ChrW\s*\((.+?)\)", "(char)$1"),
        (r"Abs\s*\((.+?)\)", "Math.Abs($1)"),
        (r"Sqr\s*\((.+?)\)", "Math.Sqrt($1)"),
        (r"Sgn\s*\((.+?)\)", "Math.Sign($1)"),
        (r"Int\s*\((.+?)\)", "Math.Truncate($1)"),
    ];
    for (pat, repl) in &str_funcs {
        let re = Regex::new(pat).unwrap();
        result = re.replace_all(&result, *repl).to_string();
    }

    let re = Regex::new(r"Left\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Substring(0, $2)").to_string();
    let re = Regex::new(r"Right\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Substring($1.Length - $2)").to_string();
    let re = Regex::new(r"Mid\s*\((.+?),\s*(.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Substring(($2) - 1, $3)").to_string();
    let re = Regex::new(r"Mid\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Substring(($2) - 1)").to_string();

    let re = Regex::new(r"Replace\s*\((.+?),\s*(.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Replace($2, $3)").to_string();
    let re = Regex::new(r"Split\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.Split($2)").to_string();
    let re = Regex::new(r"Join\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "string.Join($2, $1)").to_string();

    let re = Regex::new(r"InStr\s*\((\d+),\s*(.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$2.IndexOf($3, $1 - 1)").to_string();
    let re = Regex::new(r"InStr\s*\((.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1.IndexOf($2)").to_string();

    let re = Regex::new(r"IIf\s*\((.+?),\s*(.+?),\s*(.+?)\)").unwrap();
    result = re.replace_all(&result, "$1 ? $2 : $3").to_string();

    let re = Regex::new(r"Set\s+(\w+)\s*=\s*(Nothing|New\s+\w+(?:\([^)]*\))?)").unwrap();
    result = re.replace_all(&result, |caps: &regex::Captures| {
        let var = &caps[1];
        let val = &caps[2];
        if val == "Nothing" {
            format!("{} = null;", var)
        } else {
            format!("{} = {};", var, val)
        }
    }).to_string();

    let re = Regex::new(r"(?m)^\s*On\s+Error\s+GoTo\s+\w+\s*$").unwrap();
    result = re.replace_all(&result, "try {").to_string();
    let re = Regex::new(r"(?m)^\s*On\s+Error\s+Resume\s+Next\s*$").unwrap();
    result = re.replace_all(&result, "// On Error Resume Next").to_string();
    let re = Regex::new(r"(?m)^\s*Resume\s+Next\s*$").unwrap();
    result = re.replace_all(&result, "// continue;").to_string();
    let re = Regex::new(r"(?m)^\s*Resume\s*$").unwrap();
    result = re.replace_all(&result, "// continue;").to_string();

    let re = Regex::new(r"Err\.Number").unwrap();
    result = re.replace_all(&result, "// Err.Number").to_string();
    let re = Regex::new(r"Err\.Description").unwrap();
    result = re.replace_all(&result, "// Err.Description").to_string();
    let re = Regex::new(r"Err\.Clear").unwrap();
    result = re.replace_all(&result, "// Err.Clear").to_string();

    let re = Regex::new(r"(?m)^(\s*)Type\s+(\w+)\s*$").unwrap();
    result = re.replace_all(&result, "${1}struct ${2} {").to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:Public\s+|Private\s+)?Enum\s+(\w+)\s*$").unwrap();
    result = re.replace_all(&result, "${1}enum ${2} {").to_string();

    let re = Regex::new(r"(\w+)_(Load|Click|DblClick|KeyDown|KeyPress|KeyUp|MouseDown|MouseUp|MouseMove|Change|GotFocus|LostFocus|Resize|Terminate|Initialize)\s*\(").unwrap();
    result = re.replace_all(&result, "$1_$2(object sender, EventArgs e)").to_string();

    let re = Regex::new(r"(?m)^(\s*)(?:Public\s+|Private\s+)?Property\s+(\w+)\s+As\s+(\w+)\s*$").unwrap();
    result = re.replace_all(&result, "$1$3 $2 { get; set; }").to_string();

    let re = Regex::new(r"Optional\s+(\w+)\s+As\s+(\w+)\s*=\s*(.+)").unwrap();
    result = re.replace_all(&result, "$2 $1 = $3").to_string();

    let re = Regex::new(r"Call\s+").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"\s+Then\s*$").unwrap();
    result = re.replace_all(&result, " {").to_string();

    let re = Regex::new(r"Let\s+").unwrap();
    result = re.replace_all(&result, "").to_string();

    let re = Regex::new(r"\bIs\s+Not\b").unwrap();
    result = re.replace_all(&result, "!=").to_string();
    let re = Regex::new(r"\bIs\b").unwrap();
    result = re.replace_all(&result, "==").to_string();

    let re = Regex::new(r#"(\w+)\s+Like\s+"([^"]*)""#).unwrap();
    result = re.replace_all(&result, "$1.Contains(\"$2\")").to_string();

    let re = Regex::new(r"DoEvents").unwrap();
    result = re.replace_all(&result, "System.Windows.Forms.Application.DoEvents()").to_string();

    let re = Regex::new(r"\bNothing\b").unwrap();
    result = re.replace_all(&result, "null").to_string();

    let re = Regex::new(r"\bTrue\b").unwrap();
    result = re.replace_all(&result, "true").to_string();
    let re = Regex::new(r"\bFalse\b").unwrap();
    result = re.replace_all(&result, "false").to_string();

    let re = Regex::new(r"AddressOf\s+(\w+)").unwrap();
    result = re.replace_all(&result, "new EventHandler($1)").to_string();

    let re = Regex::new(r"(?m)^(\s*)((?:return\s+)?[\w.\[\]()]+(?:\s*=\s*[^;{]+|(?:\s*\([^)]*\))?))\s*$").unwrap();
    result = re.replace_all(&result, "$1$2;").to_string();

    result
}

fn vb_type_to_csharp(typ: &str) -> String {
    let t = typ.trim();
    match t {
        "Integer" => "int".to_string(),
        "Long" => "long".to_string(),
        "Single" => "float".to_string(),
        "Double" => "double".to_string(),
        "String" => "string".to_string(),
        "String *" => "string".to_string(),
        "Boolean" => "bool".to_string(),
        "Byte" => "byte".to_string(),
        "Date" => "DateTime".to_string(),
        "Currency" => "decimal".to_string(),
        "Variant" => "object".to_string(),
        "Object" => "object".to_string(),
        "Integer[]" => "int[]".to_string(),
        "Long[]" => "long[]".to_string(),
        "String[]" => "string[]".to_string(),
        s if s.ends_with('*') => s.trim_end_matches('*').trim().to_string(),
        s if s.chars().any(|c| c.is_ascii_digit()) => {
            let no_digits = s.trim_end_matches(|c: char| c.is_ascii_digit()).trim();
            vb_type_to_csharp(no_digits)
        }
        _ => {
            t.to_string()
        }
    }
}
