//! Token 计数模块。
//!
//! Phase 0 仅对比 OpenAI 的两种 BPE 编码：cl100k_base（GPT-4 家族）与
//! o200k_base（GPT-4o 家族）。
//!
//! 关于「Claude tokenizer」：Anthropic 未开源 tokenizer，tiktoken-rs 只内置
//! OpenAI BPE 编码。AGENTS.md §3 / §4.2 提到的「自定义 Claude 逻辑」属于
//! Phase 3+ 范围，Phase 0 跳过（用户在需求确认环节也已确认 cl100k vs o200k）。
//!
//! 编码器构造较慢（一次性解压嵌入的 BPE 表，约 30–60ms），用 OnceLock 延迟初始化，
//! 整个进程只构造一次。

use std::sync::OnceLock;

use serde::Serialize;
use tiktoken_rs::CoreBPE;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TokenCounts {
    pub cl100k: usize,
    pub o200k: usize,
}

impl TokenCounts {
    pub fn delta_pct(self) -> f64 {
        if self.cl100k == 0 {
            return 0.0;
        }
        ((self.o200k as f64 - self.cl100k as f64) / self.cl100k as f64) * 100.0
    }
}

fn cl100k() -> &'static CoreBPE {
    static BPE: OnceLock<CoreBPE> = OnceLock::new();
    BPE.get_or_init(|| {
        tiktoken_rs::cl100k_base()
            .expect("cl100k_base vocab is embedded in tiktoken-rs; init-time failure is fatal")
    })
}

fn o200k() -> &'static CoreBPE {
    static BPE: OnceLock<CoreBPE> = OnceLock::new();
    BPE.get_or_init(|| {
        tiktoken_rs::o200k_base()
            .expect("o200k_base vocab is embedded in tiktoken-rs; init-time failure is fatal")
    })
}

/// 对同一段文本用 cl100k 与 o200k 编码计数。
///
/// 使用 `encode_ordinary` 而非 `encode_with_special_tokens`：后者会把
/// `<|endoftext|>` 之类的特殊 token 计为单个 token，对真实用户文本无意义。
pub fn count_tokens(text: &str) -> TokenCounts {
    TokenCounts {
        cl100k: cl100k().encode_ordinary(text).len(),
        o200k: o200k().encode_ordinary(text).len(),
    }
}

/// PoC 报告用的示例文本：中英混合 + emoji + 代码片段。
/// 这类内容几乎必然让 cl100k 与 o200k 产出不同 token 数（o200k 对多字节/emoji 更友好）。
pub fn sample_text() -> String {
    "TokenHusk Phase 0 🚀：用确定性规则压缩 Prompt，把垃圾扔掉。\
     fn main() { println!(\"hello\"); }\n\
     用户：你好，请把上面这个 Rust 函数改成异步版本，保留错误处理。"
        .to_string()
}

// ───────────────────── 测试 ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_nonzero() {
        let c = count_tokens("hello world");
        assert!(c.cl100k > 0 && c.o200k > 0);
    }

    #[test]
    fn counts_are_deterministic() {
        let text = "the quick brown fox jumps over the lazy dog";
        let a = count_tokens(text);
        let b = count_tokens(text);
        assert_eq!(a.cl100k, b.cl100k);
        assert_eq!(a.o200k, b.o200k);
    }

    #[test]
    fn cl100k_and_o200k_differ_on_multilingual_with_emoji() {
        // o200k 对中文 / emoji 编码更紧凑，两者在该样本上一定不同。
        let c = count_tokens(&sample_text());
        assert_ne!(
            c.cl100k, c.o200k,
            "expected cl100k != o200k on multilingual+emoji sample (got both = {})",
            c.cl100k
        );
    }

    #[test]
    fn delta_pct_is_finite() {
        let c = count_tokens("abc");
        assert!(c.delta_pct().is_finite());
    }
}
