use std::sync::OnceLock;

use regex::{Captures, Regex};

use crate::config::CleanMode;
use crate::models::{NoiseSource, NoiseSourceKind, RemovedSection, RemovedSectionKind};

struct CodeBlock {
    placeholder: String,
    rendered: String,
}

pub(crate) fn remove_html_noise_blocks(
    input: &str,
    mode: CleanMode,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    if !looks_like_html(input) {
        return input.to_string();
    }

    let mut content = input.to_string();
    let patterns = [
        ("div", noise_div_regex()),
        ("section", noise_section_regex()),
        ("form", noise_form_regex()),
        ("dialog", noise_dialog_regex()),
    ];

    for (tag, regex) in patterns {
        let (next_content, matches) = remove_noise_blocks_for_tag(&content, tag, regex);
        if matches.is_empty() {
            continue;
        }

        let removed_tokens = matches.iter().map(|value| estimate_tokens(value)).sum();
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::HtmlBoilerplate,
            label: format!("HTML boilerplate block: {tag} noise"),
            tokens_removed: removed_tokens,
            count: matches.len(),
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::HtmlBoilerplate,
            label: format!("HTML boilerplate block: {tag} noise"),
            tokens_removed: removed_tokens,
        });
        content = next_content;
    }

    let matches: Vec<_> = noise_iframe_regex()
        .find_iter(&content)
        .map(|m| m.as_str().to_string())
        .collect();
    if !matches.is_empty() {
        let removed_tokens = matches.iter().map(|value| estimate_tokens(value)).sum();
        removed_sections.push(RemovedSection {
            kind: RemovedSectionKind::HtmlBoilerplate,
            label: "HTML boilerplate block: iframe noise".to_string(),
            tokens_removed: removed_tokens,
            count: matches.len(),
        });
        noise_sources.push(NoiseSource {
            kind: NoiseSourceKind::HtmlBoilerplate,
            label: "HTML boilerplate block: iframe noise".to_string(),
            tokens_removed: removed_tokens,
        });
        content = noise_iframe_regex().replace_all(&content, "\n").to_string();
    }

    if matches!(mode, CleanMode::Aggressive) {
        content = remove_aggressive_html_blocks(&content, removed_sections, noise_sources);
    }

    content
}

pub(crate) fn html_to_readable_content(input: &str, mode: CleanMode) -> String {
    if !looks_like_html(input) {
        return input.to_string();
    }

    let (mut content, code_blocks) = protect_code_blocks(input);
    content = convert_tables(&content);
    content = convert_links(&content);
    content = convert_headings(&content);
    content = convert_inline_code(&content);
    let (mut content, inline_code_blocks) = protect_markdown_inline_code(&content);
    content = replace_structural_tags(&content);
    content = tag_regex().replace_all(&content, "").to_string();
    content = cleanup_readable_lines(&content);
    content = restore_code_blocks(&content, &inline_code_blocks);
    content = restore_code_blocks(&content, &code_blocks);
    content = blank_lines_regex()
        .replace_all(&content, "\n\n")
        .trim()
        .to_string();

    if matches!(mode, CleanMode::Aggressive) {
        content = collapse_sparse_markdown_separators(&content);
    }

    content
}

fn remove_noise_blocks_for_tag(
    input: &str,
    tag: &str,
    open_regex: &Regex,
) -> (String, Vec<String>) {
    let mut output = String::new();
    let mut removed = Vec::new();
    let mut cursor = 0usize;

    while let Some(open_match) = open_regex.find_at(input, cursor) {
        output.push_str(&input[cursor..open_match.start()]);
        let block_end =
            find_matching_tag_end(input, tag, open_match.end()).unwrap_or(open_match.end());
        removed.push(input[open_match.start()..block_end].to_string());
        output.push('\n');
        cursor = block_end;
    }

    output.push_str(&input[cursor..]);
    (output, removed)
}

fn find_matching_tag_end(input: &str, tag: &str, scan_from: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut cursor = scan_from;

    while let Some(tag_match) = tag_regex().find_at(input, cursor) {
        if html_tag_name(tag_match.as_str())
            .map(|name| name.eq_ignore_ascii_case(tag))
            .unwrap_or(false)
        {
            if is_closing_html_tag(tag_match.as_str()) {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(tag_match.end());
                }
            } else if !is_self_closing_html_tag(tag_match.as_str()) {
                depth += 1;
            }
        }
        cursor = tag_match.end();
    }

    None
}

fn html_tag_name(tag_text: &str) -> Option<&str> {
    let body = tag_text.strip_prefix('<')?.trim_start();
    let body = body.strip_prefix('/').unwrap_or(body).trim_start();
    let end = body
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':' || ch == '-'))
        .unwrap_or(body.len());
    if end == 0 {
        None
    } else {
        Some(&body[..end])
    }
}

fn is_closing_html_tag(tag_text: &str) -> bool {
    tag_text
        .strip_prefix('<')
        .map(|body| body.trim_start().starts_with('/'))
        .unwrap_or(false)
}

fn is_self_closing_html_tag(tag_text: &str) -> bool {
    tag_text.trim_end().ends_with("/>")
}

fn remove_aggressive_html_blocks(
    input: &str,
    removed_sections: &mut Vec<RemovedSection>,
    noise_sources: &mut Vec<NoiseSource>,
) -> String {
    let matches: Vec<_> = aggressive_block_regex()
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect();
    if matches.is_empty() {
        return input.to_string();
    }

    let removed_tokens = matches.iter().map(|value| estimate_tokens(value)).sum();
    removed_sections.push(RemovedSection {
        kind: RemovedSectionKind::HtmlBoilerplate,
        label: "aggressive promotional/tracking HTML blocks".to_string(),
        tokens_removed: removed_tokens,
        count: matches.len(),
    });
    noise_sources.push(NoiseSource {
        kind: NoiseSourceKind::HtmlBoilerplate,
        label: "aggressive promotional/tracking HTML blocks".to_string(),
        tokens_removed: removed_tokens,
    });

    aggressive_block_regex()
        .replace_all(input, "\n")
        .to_string()
}

fn protect_code_blocks(input: &str) -> (String, Vec<CodeBlock>) {
    let mut blocks = Vec::new();
    let content = pre_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            let raw = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let code = code_close_regex()
                .replace_all(&code_open_regex().replace_all(raw, ""), "")
                .to_string();
            let index = blocks.len();
            let placeholder = code_placeholder(input, index);
            blocks.push(CodeBlock {
                placeholder: placeholder.clone(),
                rendered: format!("\n\n```\n{}\n```\n\n", code.trim_matches('\n')),
            });
            format!("\n{placeholder}\n")
        })
        .to_string();

    (content, blocks)
}

fn protect_markdown_inline_code(input: &str) -> (String, Vec<CodeBlock>) {
    let mut blocks = Vec::new();
    let content = markdown_inline_code_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            let rendered = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let index = blocks.len();
            let placeholder = code_placeholder(input, index);
            blocks.push(CodeBlock {
                placeholder: placeholder.clone(),
                rendered: rendered.to_string(),
            });
            placeholder
        })
        .to_string();

    (content, blocks)
}

fn restore_code_blocks(input: &str, blocks: &[CodeBlock]) -> String {
    let mut content = input.to_string();
    for block in blocks {
        content = content.replace(&block.placeholder, &block.rendered);
    }
    content
}

fn code_placeholder(input: &str, index: usize) -> String {
    let mut attempt = 0usize;
    loop {
        let placeholder = format!("__CTX_CLEAN_INTERNAL_CODE_BLOCK_{index}_{attempt}__");
        if !input.contains(&placeholder) {
            return placeholder;
        }
        attempt += 1;
    }
}

fn convert_tables(input: &str) -> String {
    table_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            table_to_markdown(caps.get(1).map(|m| m.as_str()).unwrap_or_default())
        })
        .to_string()
}

fn table_to_markdown(table_body: &str) -> String {
    let mut rows = Vec::new();
    for row_caps in tr_regex().captures_iter(table_body) {
        let row_body = row_caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let cells: Vec<_> = cell_regex()
            .captures_iter(row_body)
            .map(|cell_caps| {
                let raw = cell_caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                escape_table_cell(&strip_tags_to_text(raw))
            })
            .filter(|cell| !cell.is_empty())
            .collect();
        if !cells.is_empty() {
            rows.push(cells);
        }
    }

    if rows.is_empty() {
        return "\n".to_string();
    }

    let mut rendered = String::from("\n\n");
    rendered.push_str("| ");
    rendered.push_str(&rows[0].join(" | "));
    rendered.push_str(" |\n");
    rendered.push_str("| ");
    rendered.push_str(&vec!["---"; rows[0].len()].join(" | "));
    rendered.push_str(" |\n");
    for row in rows.iter().skip(1) {
        rendered.push_str("| ");
        rendered.push_str(&row.join(" | "));
        rendered.push_str(" |\n");
    }
    rendered.push('\n');
    rendered
}

fn convert_links(input: &str) -> String {
    anchor_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            let href = caps.get(1).map(|m| m.as_str()).unwrap_or_default().trim();
            let label = strip_tags_to_text(caps.get(2).map(|m| m.as_str()).unwrap_or_default());
            if href.is_empty() || label.is_empty() {
                label
            } else {
                format!("[{label}]({href})")
            }
        })
        .to_string()
}

fn convert_headings(input: &str) -> String {
    heading_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            let level = caps
                .get(1)
                .and_then(|m| m.as_str().parse::<usize>().ok())
                .unwrap_or(2)
                .clamp(1, 6);
            let text = strip_tags_to_text(caps.get(2).map(|m| m.as_str()).unwrap_or_default());
            if text.is_empty() {
                "\n".to_string()
            } else {
                format!("\n\n{} {text}\n\n", "#".repeat(level))
            }
        })
        .to_string()
}

fn convert_inline_code(input: &str) -> String {
    inline_code_regex()
        .replace_all(input, |caps: &Captures<'_>| {
            let code =
                cleanup_inline_code_text(caps.get(1).map(|m| m.as_str()).unwrap_or_default());
            if code.is_empty() {
                String::new()
            } else {
                format!("`{code}`")
            }
        })
        .to_string()
}

fn replace_structural_tags(input: &str) -> String {
    let mut content = input.to_string();
    content = br_regex().replace_all(&content, "\n").to_string();
    content = li_open_regex().replace_all(&content, "\n- ").to_string();
    content = li_close_regex().replace_all(&content, "\n").to_string();
    content = block_close_regex()
        .replace_all(&content, "\n\n")
        .to_string();
    content = block_open_regex().replace_all(&content, "\n").to_string();
    content
}

fn strip_tags_to_text(input: &str) -> String {
    let stripped = tag_regex().replace_all(input, "");
    cleanup_inline_whitespace(stripped.as_ref())
}

fn cleanup_inline_code_text(input: &str) -> String {
    whitespace_regex()
        .replace_all(input, " ")
        .trim()
        .to_string()
}

fn cleanup_readable_lines(input: &str) -> String {
    let mut lines = Vec::new();
    for line in input.lines() {
        let trimmed = cleanup_inline_whitespace(line);
        lines.push(trimmed);
    }
    blank_lines_regex()
        .replace_all(&lines.join("\n"), "\n\n")
        .trim()
        .to_string()
}

fn cleanup_inline_whitespace(input: &str) -> String {
    whitespace_regex()
        .replace_all(input, " ")
        .trim()
        .to_string()
}

fn collapse_sparse_markdown_separators(input: &str) -> String {
    input
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.len() > 6
                && trimmed
                    .chars()
                    .all(|ch| ch == '-' || ch == '_' || ch == '='))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_table_cell(input: &str) -> String {
    input.replace('|', "\\|")
}

fn looks_like_html(input: &str) -> bool {
    html_hint_regex().is_match(input)
}

fn estimate_tokens(input: &str) -> usize {
    if input.is_empty() {
        0
    } else {
        input.chars().count().div_ceil(4)
    }
}

fn html_hint_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?is)<\s*/?\s*(html|body|main|article|section|div|p|h[1-6]|a|ul|ol|li|table|tr|td|th|pre|code|script|style|nav|footer|aside|svg)\b",
        )
        .expect("valid regex")
    })
}

fn noise_attr_pattern() -> &'static str {
    r#"(?:id|class|role|aria-label|data-testid|data-component|data-module)\s*=\s*["'][^"']*(?:cookie|consent|gdpr|modal|popup|newsletter|subscribe|advertisement|advertising|ad-banner|ad_|ad-|adservice|adsbygoogle|sponsored|promoted|tracking|analytics|promo|share-widget|social-share|paywall)[^"']*["']"#
}

fn noise_div_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(&format!(r"(?is)<div\b[^>]*{}[^>]*>", noise_attr_pattern()))
            .expect("valid regex")
    })
}

fn noise_section_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(&format!(
            r"(?is)<section\b[^>]*{}[^>]*>",
            noise_attr_pattern()
        ))
        .expect("valid regex")
    })
}

fn noise_form_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(&format!(r"(?is)<form\b[^>]*{}[^>]*>", noise_attr_pattern()))
            .expect("valid regex")
    })
}

fn noise_dialog_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(&format!(
            r"(?is)<dialog\b[^>]*{}[^>]*>",
            noise_attr_pattern()
        ))
        .expect("valid regex")
    })
}

fn noise_iframe_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?is)<iframe\b[^>]*(?:adservice|doubleclick|tracking|analytics|pixel|advert)[^>]*>.*?</iframe>"#)
            .expect("valid regex")
    })
}

fn aggressive_block_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?is)<(?:div|section|aside)\b[^>]*(?:class|id)\s*=\s*["'][^"']*(?:related|recommended|more-stories|sponsored|outbrain|taboola)[^"']*["'][^>]*>.*?</(?:div|section|aside)>"#)
            .expect("valid regex")
    })
}

fn pre_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<pre\b[^>]*>(.*?)</pre>").expect("valid regex"))
}

fn code_open_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<code\b[^>]*>").expect("valid regex"))
}

fn code_close_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)</code>").expect("valid regex"))
}

fn table_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<table\b[^>]*>(.*?)</table>").expect("valid regex"))
}

fn tr_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<tr\b[^>]*>(.*?)</tr>").expect("valid regex"))
}

fn cell_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?is)<(th|td)\b[^>]*>(.*?)</(?:th|td)>").expect("valid regex"))
}

fn anchor_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?is)<a\b[^>]*href\s*=\s*["']([^"']+)["'][^>]*>(.*?)</a>"#)
            .expect("valid regex")
    })
}

fn heading_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<h([1-6])\b[^>]*>(.*?)</h[1-6]>").expect("valid regex"))
}

fn inline_code_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<code\b[^>]*>(.*?)</code>").expect("valid regex"))
}

fn markdown_inline_code_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"`[^`\n]+`").expect("valid regex"))
}

fn br_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<br\s*/?>").expect("valid regex"))
}

fn li_open_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)<li\b[^>]*>").expect("valid regex"))
}

fn li_close_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?is)</li>").expect("valid regex"))
}

fn block_close_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)</(?:p|div|section|article|main|header|blockquote|ul|ol|tr)>")
            .expect("valid regex")
    })
}

fn block_open_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)<(?:p|div|section|article|main|header|blockquote|ul|ol|tr)\b[^>]*>")
            .expect("valid regex")
    })
}

fn tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?is)</?[a-z][a-z0-9:-]*(?:\s+[^<>]*)?/?>").expect("valid regex")
    })
}

fn whitespace_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[ \t]{2,}").expect("valid regex"))
}

fn blank_lines_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n{3,}").expect("valid regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_visible_html_to_markdownish_context() {
        let input = r#"
        <main>
          <h1>Keep This Article</h1>
          <p>Read the <a href="https://example.com/report">full report</a>.</p>
          <table><tr><th>Name</th><th>Status</th></tr><tr><td>API</td><td>Failing</td></tr></table>
          <pre><code>const value = api.user.id;</code></pre>
        </main>
        "#;

        let output = html_to_readable_content(input, CleanMode::Standard);

        assert!(output.contains("# Keep This Article"));
        assert!(output.contains("[full report](https://example.com/report)"));
        assert!(output.contains("| Name | Status |"));
        assert!(output.contains("```"));
        assert!(output.contains("const value = api.user.id;"));
    }

    #[test]
    fn removes_cookie_modal_and_ad_blocks() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = r#"
        <main><p>KEEP_VISIBLE_SENTINEL</p></main>
        <div id="cookie-banner">Accept all cookies</div>
        <dialog class="newsletter-modal">Subscribe now</dialog>
        <section class="ad-banner">Buy this</section>
        "#;

        let output = remove_html_noise_blocks(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("KEEP_VISIBLE_SENTINEL"));
        assert!(!output.contains("Accept all cookies"));
        assert!(!output.contains("Subscribe now"));
        assert!(!output.contains("Buy this"));
        assert!(!removed.is_empty());
        assert!(!noise.is_empty());
    }

    #[test]
    fn code_placeholders_do_not_replace_visible_text() {
        let output = html_to_readable_content(
            r#"<main><p>Literal __CTX_CLEAN_INTERNAL_CODE_BLOCK_0_0__ stays text.</p><pre><code>actual_code()</code></pre></main>"#,
            CleanMode::Standard,
        );

        assert!(output.contains("Literal __CTX_CLEAN_INTERNAL_CODE_BLOCK_0_0__ stays text."));
        assert!(output.contains("actual_code()"));
    }

    #[test]
    fn inline_code_preserves_angle_brackets_after_entity_decode() {
        let output = html_to_readable_content(
            r#"<main><p>Keep <code>if a < b && b > c</code> visible.</p></main>"#,
            CleanMode::Standard,
        );

        assert!(output.contains("`if a < b && b > c`"));
    }

    #[test]
    fn inline_code_preserves_tag_like_generics() {
        let output = html_to_readable_content(
            r#"<main><p>Return <code>Vec<usize></code> from the scanner.</p></main>"#,
            CleanMode::Standard,
        );

        assert!(output.contains("`Vec<usize>`"));
    }

    #[test]
    fn noise_patterns_do_not_drop_legitimate_ads_substrings() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = r#"
        <main>
          <section class="leads uploads roads">KEEP_LEADS_SECTION</section>
          <div class="cookie-consent">Drop cookies</div>
        </main>
        "#;

        let output = remove_html_noise_blocks(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("KEEP_LEADS_SECTION"));
        assert!(!output.contains("Drop cookies"));
    }

    #[test]
    fn removes_nested_cookie_banner_blocks() {
        let mut removed = Vec::new();
        let mut noise = Vec::new();
        let input = r#"
        <main><p>KEEP_ARTICLE_BODY</p></main>
        <div id="cookie-banner">
          <div class="inner">DROP_COOKIE_TEXT</div>
          <button>DROP_COOKIE_BUTTON</button>
        </div>
        "#;

        let output = remove_html_noise_blocks(input, CleanMode::Standard, &mut removed, &mut noise);

        assert!(output.contains("KEEP_ARTICLE_BODY"));
        assert!(!output.contains("DROP_COOKIE_TEXT"));
        assert!(!output.contains("DROP_COOKIE_BUTTON"));
        assert_eq!(removed.len(), 1);
    }
}
