//! 出力検証（クラス図 StructureValidator / §1.4 F6 / §1.5 手順8）。
//! 4部構成（宛名→挨拶→本文→結び）の充足を検査する。

/// 4部構成の検出結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts {
    pub salutation: bool,
    pub greeting: bool,
    pub body: bool,
    pub closing: bool,
}

impl Parts {
    pub fn is_complete(self) -> bool {
        self.salutation && self.greeting && self.body && self.closing
    }
}

const SALUTATION_MARKERS: [&str; 5] = ["{{相手名}}", "様", "Dear", "殿", "各位"];
const GREETING_MARKERS: [&str; 6] = [
    "お世話", "いつも", "拝啓", "ご連絡", "Thank you", "Hello",
];
const CLOSING_MARKERS: [&str; 6] = [
    "{{自分の署名}}", "よろしく", "敬具", "Regards", "Sincerely", "何卒",
];

pub fn detect_parts(text: &str) -> Parts {
    let non_empty_lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    let has_any = |markers: &[&str]| markers.iter().any(|m| text.contains(m));
    let salutation = has_any(&SALUTATION_MARKERS);
    let greeting = has_any(&GREETING_MARKERS);
    let closing = has_any(&CLOSING_MARKERS);
    // 本文: 宛名・挨拶・結び以外に十分な行があること（多言語フォールバック）。
    let body = non_empty_lines.len() >= 4
        || (salutation && greeting && closing && non_empty_lines.len() >= 3);

    Parts {
        salutation,
        greeting,
        body,
        closing,
    }
}

/// 4部構成が揃っているか。
pub fn validate_4parts(text: &str) -> bool {
    detect_parts(text).is_complete()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 四部構成が揃えば有効() {
        let text = "{{相手名}} 様\n\nいつもお世話になっております。\n\n本件、承知いたしました。対応いたします。\n\n何卒よろしくお願いいたします。\n{{自分の署名}}";
        assert!(validate_4parts(text));
    }

    #[test]
    fn 結びが無ければ無効() {
        let text = "{{相手名}} 様\n\nいつもお世話になっております。\n\n本件承知しました。";
        assert!(!validate_4parts(text));
    }
}
