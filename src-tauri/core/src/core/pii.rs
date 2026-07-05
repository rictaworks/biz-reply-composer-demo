//! 個人情報マスキング（クラス図 PiiMasker / §1.5 手順9・仕様 §8）。
//! 氏名らしき文字列・メールアドレス・電話番号を保存前に伏字化する。
//! 宛名・署名は {{相手名}} {{自分の署名}} トークンで扱い実名は保持しない。

/// メール・電話・敬称付き氏名を伏字化する。過剰マスクよりも保存前の確実な除去を優先。
pub fn mask(text: &str) -> String {
    let mut out = mask_emails(text);
    out = mask_phones(&out);
    out = mask_honorific_names(&out);
    out
}

const REDACTED: &str = "〔伏字〕";

fn mask_emails(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for token in split_keep(text, |c| c.is_whitespace()) {
        if looks_like_email(token) {
            result.push_str(REDACTED);
        } else {
            result.push_str(token);
        }
    }
    result
}

fn looks_like_email(token: &str) -> bool {
    let at = token.find('@');
    match at {
        Some(i) if i > 0 && i < token.len() - 1 => token[i + 1..].contains('.'),
        _ => false,
    }
}

fn mask_phones(text: &str) -> String {
    // 数字・ハイフン・括弧が続く長さ10以上の並びを電話番号とみなす。
    let mut result = String::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || matches!(ch, '-' | '(' | ')' | '+') {
            buf.push(ch);
        } else {
            flush_phone(&mut buf, &mut result);
            result.push(ch);
        }
    }
    flush_phone(&mut buf, &mut result);
    result
}

fn flush_phone(buf: &mut String, result: &mut String) {
    let digits = buf.chars().filter(|c| c.is_ascii_digit()).count();
    if digits >= 10 {
        result.push_str(REDACTED);
    } else {
        result.push_str(buf);
    }
    buf.clear();
}

/// 「〇〇様/さん/氏/殿」の直前の語を伏字化する（簡易）。
fn mask_honorific_names(text: &str) -> String {
    const HONORIFICS: [&str; 4] = ["様", "さん", "氏", "殿"];
    let mut result = text.to_string();
    for h in HONORIFICS {
        result = mask_before_honorific(&result, h);
    }
    result
}

fn mask_before_honorific(text: &str, honorific: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(pos) = rest.find(honorific) {
        let before = &rest[..pos];
        // 敬称直前の連続する非空白・非記号の語を名前候補とみなす。
        let name_start = before
            .char_indices()
            .rev()
            .take_while(|(_, c)| !c.is_whitespace() && !is_boundary(*c))
            .last()
            .map(|(i, _)| i);
        match name_start {
            Some(i) if i < before.len() => {
                out.push_str(&before[..i]);
                out.push_str(REDACTED);
            }
            _ => out.push_str(before),
        }
        out.push_str(honorific);
        rest = &rest[pos + honorific.len()..];
    }
    out.push_str(rest);
    out
}

fn is_boundary(c: char) -> bool {
    matches!(c, '、' | '。' | '「' | '」' | '（' | '）' | ',' | '.' | '\n')
}

/// 空白などの区切りを保持したまま分割する。
fn split_keep<'a>(text: &'a str, is_sep: impl Fn(char) -> bool) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if is_sep(c) {
            if start < i {
                parts.push(&text[start..i]);
            }
            parts.push(&text[i..i + c.len_utf8()]);
            start = i + c.len_utf8();
        }
    }
    if start < text.len() {
        parts.push(&text[start..]);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn メールアドレスを伏字化() {
        let masked = mask("連絡先は taro.yamada@example.com です");
        assert!(!masked.contains("@example.com"));
        assert!(masked.contains(REDACTED));
    }

    #[test]
    fn 電話番号を伏字化() {
        let masked = mask("電話 090-1234-5678 まで");
        assert!(!masked.contains("090-1234-5678"));
        assert!(masked.contains(REDACTED));
    }

    #[test]
    fn 敬称付き氏名を伏字化() {
        let masked = mask("山田太郎様、ご連絡ありがとうございます");
        assert!(!masked.contains("山田太郎"));
        assert!(masked.contains("様"));
    }
}
