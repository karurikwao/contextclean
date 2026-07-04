use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use tiktoken_rs::{o200k_base_singleton, CoreBPE};

pub const MIN_EXPLAINABLE_TRUNCATION_TOKENS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FitModel {
    #[serde(rename = "gpt-4.1")]
    Gpt41,
    #[serde(rename = "claude-sonnet")]
    ClaudeSonnet,
    #[serde(rename = "gemini-pro")]
    GeminiPro,
}

impl FitModel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Gpt41 => "gpt-4.1",
            Self::ClaudeSonnet => "claude-sonnet",
            Self::GeminiPro => "gemini-pro",
        }
    }

    pub fn max_tokens(self) -> usize {
        match self {
            Self::Gpt41 => 1_047_576,
            Self::ClaudeSonnet => 1_000_000,
            Self::GeminiPro => 1_048_576,
        }
    }

    pub fn model_id(self) -> &'static str {
        match self {
            Self::Gpt41 => "gpt-4.1",
            Self::ClaudeSonnet => "claude-sonnet-5",
            Self::GeminiPro => "gemini-2.5-pro",
        }
    }

    pub fn max_output_tokens(self) -> usize {
        match self {
            Self::Gpt41 => 32_768,
            Self::ClaudeSonnet => 128_000,
            Self::GeminiPro => 65_536,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackedText {
    pub content: String,
    pub tokens_removed: usize,
    pub applied: bool,
}

pub fn tokenizer_name() -> &'static str {
    "o200k_base"
}

pub fn count_tokens(input: &str) -> usize {
    bpe().encode_with_special_tokens(input).len()
}

pub fn pack_to_budget(input: &str, max_tokens: usize) -> PackedText {
    let original_tokens = count_tokens(input);
    if original_tokens <= max_tokens {
        return PackedText {
            content: input.to_string(),
            tokens_removed: 0,
            applied: false,
        };
    }

    if max_tokens == 0 {
        return PackedText {
            content: String::new(),
            tokens_removed: original_tokens,
            applied: true,
        };
    }

    let mut content_budget = max_tokens;
    for _ in 0..=max_tokens {
        let kept = semantic_prefix(input, content_budget);
        let kept_tokens = count_tokens(kept.trim_end());
        let removed = original_tokens.saturating_sub(kept_tokens);
        let footer = footer_for(removed, max_tokens);
        let candidate = if kept.trim().is_empty() {
            footer.clone()
        } else {
            format!("{}\n\n{footer}", kept.trim_end())
        };
        if count_tokens(&candidate) <= max_tokens {
            return PackedText {
                content: candidate,
                tokens_removed: removed,
                applied: true,
            };
        }

        if content_budget == 0 {
            break;
        }
        content_budget -= 1;
    }

    let fallback = hard_prefix("[Context Truncated]", max_tokens);
    PackedText {
        content: fallback,
        tokens_removed: original_tokens,
        applied: true,
    }
}

fn footer_for(tokens_removed: usize, max_tokens: usize) -> String {
    let full = format!(
        "[Context Truncated: Removed {tokens_removed} tokens to fit {max_tokens}-token budget]"
    );
    if count_tokens(&full) <= max_tokens {
        full
    } else {
        "[Context Truncated]".to_string()
    }
}

fn semantic_prefix(input: &str, max_tokens: usize) -> String {
    if max_tokens == 0 {
        return String::new();
    }

    let mut output = String::new();
    let chunks = semantic_chunks(input);
    for chunk in &chunks {
        let candidate = format!("{output}{chunk}");
        if count_tokens(&candidate) <= max_tokens {
            output = candidate;
        } else {
            break;
        }
    }

    if output.trim().is_empty() {
        if chunks
            .first()
            .map(|chunk| chunk.contains("```"))
            .unwrap_or(false)
        {
            return String::new();
        }
        hard_prefix(input, max_tokens)
    } else {
        output
    }
}

fn semantic_chunks(input: &str) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut in_fence = false;
    let mut previous_blank = false;

    for (offset, line) in input.split_inclusive('\n').enumerate() {
        let line_start = input
            .split_inclusive('\n')
            .take(offset)
            .map(str::len)
            .sum::<usize>();
        let line_end = line_start + line.len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && line.trim().is_empty() {
            if !previous_blank && line_end > start {
                chunks.push(&input[start..line_end]);
                start = line_end;
            }
            previous_blank = true;
        } else {
            previous_blank = false;
        }
    }

    if start < input.len() {
        chunks.push(&input[start..]);
    }
    chunks
}

fn hard_prefix(input: &str, max_tokens: usize) -> String {
    let mut low = 0usize;
    let mut high = input.len();
    let mut best = 0usize;
    while low <= high {
        let mid = (low + high) / 2;
        let candidate = floor_char_boundary(input, mid);
        if count_tokens(&input[..candidate]) <= max_tokens {
            best = candidate;
            low = candidate.saturating_add(1);
        } else if candidate == 0 {
            break;
        } else {
            high = candidate - 1;
        }
    }
    input[..best].trim_end().to_string()
}

fn floor_char_boundary(input: &str, mut index: usize) -> usize {
    index = index.min(input.len());
    while index > 0 && !input.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn bpe() -> &'static CoreBPE {
    static BPE: OnceLock<&'static CoreBPE> = OnceLock::new();
    BPE.get_or_init(o200k_base_singleton)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packer_keeps_output_under_budget_with_footer() {
        let input = "para one\n\npara two with more words\n\npara three with more words";
        let packed = pack_to_budget(input, 12);
        assert!(packed.applied);
        assert!(count_tokens(&packed.content) <= 12);
        assert!(packed.content.contains("Context Truncated"));
    }

    #[test]
    fn packer_does_not_split_oversized_first_code_fence() {
        let input = format!(
            "```rust\nfn very_long_first_fence() {{\n{}\n}}\n```\n\nTrailing paragraph",
            "println!(\"hello\");\n".repeat(80)
        );

        let packed = pack_to_budget(&input, 40);

        assert!(packed.applied);
        assert!(count_tokens(&packed.content) <= 40);
        assert!(packed.content.contains("Context Truncated"));
        assert_eq!(packed.content.matches("```").count() % 2, 0);
    }

    #[test]
    fn fit_presets_are_deterministic() {
        assert_eq!(FitModel::Gpt41.max_tokens(), 1_047_576);
        assert_eq!(FitModel::ClaudeSonnet.label(), "claude-sonnet");
        assert_eq!(FitModel::GeminiPro.max_tokens(), 1_048_576);
    }

    #[test]
    fn minimum_truncation_marker_budget_matches_tokenizer() {
        assert_eq!(
            count_tokens("[Context Truncated]"),
            MIN_EXPLAINABLE_TRUNCATION_TOKENS
        );
    }
}
