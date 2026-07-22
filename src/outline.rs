//! Deterministic outline generator — the deny payload.
//!
//! Header + structure map + retrieval instructions. Target <= 60 lines / ~2.5KB.

use std::path::Path;

/// Per-entry width cap: a minified one-line file must never leak megabytes
/// into the deny reason.
const ENTRY_MAX_CHARS: usize = 200;
/// Ceiling on the assembled reason; above it fall back to header + footer only.
const REASON_MAX_BYTES: usize = 8192;

/// Clip a single outline entry to `ENTRY_MAX_CHARS` chars (char-boundary safe).
fn clip(entry: &str) -> String {
    match entry.char_indices().nth(ENTRY_MAX_CHARS) {
        Some((byte_idx, _)) => format!("{}\u{2026}", &entry[..byte_idx]),
        None => entry.to_string(),
    }
}

const CODE_EXTS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "go", "java", "cs", "c", "cpp", "cc",
    "h", "hpp", "rb", "php", "kt", "swift", "scala",
];

pub const IMAGE_PDF_NOTEBOOK_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tif", "tiff", "pdf", "ipynb",
];

fn ext_of(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

pub fn is_image_pdf_or_notebook(path: &str) -> bool {
    IMAGE_PDF_NOTEBOOK_EXTS.contains(&ext_of(path).as_str())
}

/// Full deny payload: header + entries (capped) + footer.
pub fn generate(path: &str, content: &str, file_bytes: u64, threshold: u64, max_lines: usize) -> String {
    let total_lines = content.lines().count();
    let kb = file_bytes as f64 / 1024.0;
    let threshold_kb = threshold as f64 / 1024.0;
    let header = format!(
        "stk clamp: {path}, {kb:.1} KB, {total_lines} lines (threshold {threshold_kb:.0} KB). Outline below;\nfetch only what you need with Read(file_path, offset, limit)."
    );

    let mut entries: Vec<String> = entries_for(path, content).iter().map(|e| clip(e)).collect();

    // Hard cap: if outline would exceed max_lines, keep first 60 + count.
    if entries.len() > max_lines {
        let keep = 60.min(max_lines);
        let dropped = entries.len() - keep;
        entries.truncate(keep);
        entries.push(format!("\u{2026} (+{dropped} more entries)"));
    }

    let footer = format!(
        "Re-read a symbol's body: Read with offset=<line>, limit=<span>. Whole file only if truly\nneeded: re-Read with offset=1, limit={total_lines}."
    );

    let reason = format!("{header}\n\n{}\n\n{footer}", entries.join("\n"));
    if reason.len() > REASON_MAX_BYTES {
        return format!("{header}\n\n{footer}");
    }
    reason
}

/// Structure-map entry lines for the file, family-dispatched by extension.
fn entries_for(path: &str, content: &str) -> Vec<String> {
    let ext = ext_of(path);
    if CODE_EXTS.contains(&ext.as_str()) {
        code_entries(content)
    } else if ext == "md" || ext == "markdown" {
        markdown_entries(content)
    } else if ext == "json" {
        json_entries(content)
    } else {
        other_entries(content)
    }
}

fn numbered(line_no: usize, text: &str) -> String {
    format!("{line_no:>4}  {text}")
}

fn is_import_line(trimmed: &str) -> bool {
    trimmed.starts_with("import ")
        || trimmed.starts_with("import{")
        || trimmed.starts_with("use ")
        || trimmed.starts_with("from ")
        || trimmed.starts_with("#include")
        || trimmed.starts_with("require(")
        || trimmed.starts_with("const ") && trimmed.contains("require(")
        || trimmed.starts_with("using ")
        || trimmed.starts_with("extern crate ")
}

fn is_decl_line(trimmed: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "fn ", "pub ", "class ", "struct ", "impl ", "interface ", "def ", "export ",
        "function ", "enum ", "trait ", "mod ", "type ", "async fn ", "async def ",
        "func ", "abstract ", "static ", "public ", "private ", "protected ",
    ];
    let decl = KEYWORDS.iter().any(|k| trimmed.starts_with(k));
    if !decl {
        return false;
    }
    // Only keep access-modifier lines that actually declare something.
    if trimmed.starts_with("public ")
        || trimmed.starts_with("private ")
        || trimmed.starts_with("protected ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("abstract ")
        || trimmed.starts_with("pub ")
        || trimmed.starts_with("export ")
    {
        const INNER: &[&str] = &[
            "fn", "class", "struct", "impl", "interface", "def", "function", "enum", "trait",
            "mod", "type", "async", "func", "const", "static", "abstract", "record",
        ];
        let rest: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
        return rest
            .iter()
            .any(|w| INNER.contains(w) || w.ends_with('(') || w.contains('('))
            || rest.first().map(|w| INNER.contains(w)).unwrap_or(false);
    }
    true
}

fn code_entries(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        // Skip attribute/annotation lines like #[test], @Test, decorators.
        if trimmed.starts_with("#[") || trimmed.starts_with("@") {
            i += 1;
            continue;
        }
        if is_import_line(trimmed) {
            // Collapse the consecutive import run into one count line.
            let start = i;
            while i < lines.len() {
                let t = lines[i].trim_start();
                if is_import_line(t) || t.is_empty() && i + 1 < lines.len() && is_import_line(lines[i + 1].trim_start()) {
                    i += 1;
                } else {
                    break;
                }
            }
            let count = lines[start..i]
                .iter()
                .filter(|l| is_import_line(l.trim_start()))
                .count();
            out.push(numbered(start + 1, &format!("import {{ \u{2026} }} ({count} import lines)")));
            continue;
        }
        // Top-level or indented declaration lines (indent <= 8 keeps method-level decls).
        let indent = line.len() - trimmed.len();
        if indent <= 8 && is_decl_line(trimmed) {
            let text = trimmed.trim_end().trim_end_matches('{').trim_end();
            out.push(numbered(i + 1, &format!("{}{}", " ".repeat(indent.min(4)), text)));
        }
        i += 1;
    }
    if out.is_empty() {
        return other_entries(content);
    }
    out
}

fn markdown_entries(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut fences = 0usize;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !in_fence {
                fences += 1;
            }
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && trimmed.starts_with('#') {
            out.push(numbered(idx + 1, trimmed));
        }
    }
    out.push(format!("      ({fences} fenced code blocks)"));
    out
}

fn json_entries(content: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return other_entries(content);
    };
    let mut out = Vec::new();
    describe_json(&value, "", 0, &mut out);
    out
}

fn json_shape(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(m) => format!("object ({} keys)", m.len()),
        serde_json::Value::Array(a) => format!("array ({} items)", a.len()),
        serde_json::Value::String(_) => "string".into(),
        serde_json::Value::Number(_) => "number".into(),
        serde_json::Value::Bool(_) => "bool".into(),
        serde_json::Value::Null => "null".into(),
    }
}

/// Keys to depth 2, array lengths — never values.
fn describe_json(v: &serde_json::Value, key: &str, depth: usize, out: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    match v {
        serde_json::Value::Object(map) => {
            if depth == 0 {
                out.push(format!("object ({} top-level keys)", map.len()));
            } else {
                out.push(format!("{indent}{key}: {}", json_shape(v)));
            }
            if depth < 2 {
                for (k, child) in map {
                    describe_json(child, k, depth + 1, out);
                }
            }
        }
        serde_json::Value::Array(_) if depth == 0 => {
            out.push(json_shape(v));
        }
        _ => {
            out.push(format!("{indent}{key}: {}", json_shape(v)));
        }
    }
}

fn other_entries(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    let mut out = Vec::new();
    for (idx, line) in lines.iter().take(10).enumerate() {
        out.push(numbered(idx + 1, line.trim_end()));
    }
    if total > 15 {
        out.push("\u{2026}".into());
    }
    if total > 10 {
        let start = total.saturating_sub(5).max(10);
        for (offset, line) in lines[start..].iter().enumerate() {
            out.push(numbered(start + offset + 1, line.trim_end()));
        }
    }
    out.push(format!("      ({total} lines total)"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_family_rust() {
        let src = "use std::io;\nuse std::fs;\n\nfn helper() {}\n\npub struct Thing {\n    x: u32,\n}\n\nimpl Thing {\n    pub fn new() -> Thing { Thing { x: 0 } }\n}\n";
        let out = generate("C:\\repo\\lib.rs", src, 20000, 16384, 80);
        assert!(out.starts_with("stk clamp: C:\\repo\\lib.rs"));
        assert!(out.contains("(2 import lines)"), "{out}");
        assert!(out.contains("fn helper()"));
        assert!(out.contains("pub struct Thing"));
        assert!(out.contains("impl Thing"));
        assert!(out.contains("pub fn new() -> Thing"));
        assert!(out.contains("offset=1, limit=12"));
    }

    #[test]
    fn code_family_skips_test_attributes() {
        let src = "#[test]\nfn my_test() {}\n";
        let out = generate("a.rs", src, 20000, 16384, 80);
        assert!(!out.contains("#[test]"));
        assert!(out.contains("fn my_test()"));
    }

    #[test]
    fn markdown_family() {
        let md = "# Title\n\nbody\n\n## Section A\n\n```rust\nfn x() {}\n```\n\n### Sub\n";
        let out = generate("doc.md", md, 20000, 16384, 80);
        assert!(out.contains("# Title"));
        assert!(out.contains("## Section A"));
        assert!(out.contains("### Sub"));
        assert!(out.contains("(1 fenced code blocks)"));
        // Heading inside fence must not leak — fn x is code, not heading; also
        // check code content is not treated as heading.
        assert!(!out.contains("fn x()"));
    }

    #[test]
    fn json_family_keys_no_values() {
        let js = r#"{"name":"secret-value","deps":{"serde":"1.0","clap":"4.0"},"tags":["a","b","c"]}"#;
        let out = generate("pkg.json", js, 20000, 16384, 80);
        assert!(out.contains("3 top-level keys"));
        assert!(out.contains("deps: object (2 keys)"));
        assert!(out.contains("tags: array (3 items)"));
        assert!(!out.contains("secret-value"), "values must never appear: {out}");
        assert!(!out.contains("1.0"));
    }

    #[test]
    fn other_text_family() {
        let txt: String = (1..=40).map(|i| format!("line {i}\n")).collect();
        let out = generate("notes.txt", &txt, 20000, 16384, 80);
        assert!(out.contains("line 1"));
        assert!(out.contains("line 10"));
        assert!(!out.contains("line 20\n"));
        assert!(out.contains("line 36"));
        assert!(out.contains("line 40"));
        assert!(out.contains("(40 lines total)"));
    }

    #[test]
    fn hard_cap_truncates() {
        let src: String = (0..200).map(|i| format!("fn f{i}() {{}}\n")).collect();
        let out = generate("big.rs", &src, 90000, 16384, 80);
        let entry_lines: Vec<&str> = out.lines().filter(|l| l.contains("fn f")).collect();
        assert_eq!(entry_lines.len(), 60);
        assert!(out.contains("(+140 more entries)"), "{out}");
    }

    #[test]
    fn entries_clipped_to_max_chars() {
        // One giant minified line starting with "function " (decl match).
        let src = format!("function f() {{ {} }}\n", "x=1;".repeat(100_000));
        let out = generate("bundle.js", &src, src.len() as u64, 16384, 80);
        assert!(out.len() < REASON_MAX_BYTES + 512, "reason too big: {} bytes", out.len());
        for line in out.lines() {
            assert!(line.chars().count() <= ENTRY_MAX_CHARS + 16, "line too long");
        }
    }

    #[test]
    fn reason_ceiling_falls_back_to_header_footer() {
        // 70 decl lines of ~170 chars each -> ~12KB of entries, under the
        // max_lines cap but over REASON_MAX_BYTES.
        let src: String = (0..70)
            .map(|i| format!("fn f{i}_{}() {{}}\n", "a".repeat(160)))
            .collect();
        let out = generate("wide.rs", &src, src.len() as u64, 16384, 80);
        assert!(out.len() <= REASON_MAX_BYTES, "ceiling not applied: {} bytes", out.len());
        assert!(out.starts_with("stk clamp:"));
        assert!(out.contains("offset=1, limit=70"));
        assert!(!out.contains("fn f0_"), "fallback must drop entries: {out}");
    }

    #[test]
    fn total_line_count_always_present() {
        let txt = "a\nb\nc\n";
        let out = generate("x.unknownext", txt, 20000, 16384, 80);
        assert!(out.contains("offset=1, limit=3"));
    }
}
