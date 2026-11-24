use super::*;

/// Format the single duration argument in supported Android Chronometer formats.
pub(super) fn format_chronometer(format: &str, duration: &str) -> String {
    let mut out=String::new();
    let mut rest=format;
    while let Some(index)=rest.find('%') {
        out.push_str(&rest[..index]);
        rest=&rest[index+1..];
        if let Some(next)=rest.strip_prefix("1$s").or_else(||rest.strip_prefix('s')) { out.push_str(duration); rest=next; }
        else if let Some(next)=rest.strip_prefix('%') { out.push('%');rest=next; }
        else if let Some(next)=rest.strip_prefix('n') { out.push('\n');rest=next; }
        else { return duration.to_owned(); }
    }
    out.push_str(rest);
    out
}

/// Android date patterns ignore quoted letters when deciding update frequency.
pub(super) fn clock_pattern_has_seconds(pattern: &str) -> bool {
    let mut chars = pattern.chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.peek() == Some(&'\'') { chars.next(); } else { quoted = !quoted; }
        } else if ch == 's' && !quoted { return true; }
    }
    false
}

pub(super) fn format_wall_clock(clock:WallClock, pattern:&str) -> String {
    let local_seconds = (clock.unix_seconds.floor() as i64).saturating_add(i64::from(clock.offset_seconds));
    let local = local_seconds.rem_euclid(86400);
    let hour = local/3600; let minute = local/60%60; let second = local%60;
    // Proleptic Gregorian civil date from days relative to the Unix epoch.
    let days = local_seconds.div_euclid(86400) + 719468;
    let era = days.div_euclid(146097);
    let day_of_era = days - era*146097;
    let year_of_era = (day_of_era - day_of_era/1460 + day_of_era/36524 - day_of_era/146096)/365;
    let mut year = year_of_era + era*400;
    let day_of_year = day_of_era - (365*year_of_era + year_of_era/4 - year_of_era/100);
    let month_prime = (5*day_of_year + 2)/153;
    let day = day_of_year - (153*month_prime+2)/5 + 1;
    let month = month_prime + if month_prime<10 { 3 } else { -9 };
    year += i64::from(month<=2);
    let mut out=String::new(); let mut chars=pattern.chars().peekable(); let mut quoted=false;
    while let Some(ch)=chars.next() {
        if ch=='\'' { if chars.peek()==Some(&'\'') { chars.next();out.push('\''); } else { quoted = !quoted; } continue; }
        if quoted { out.push(ch); continue; }
        if matches!(ch,'H'|'h'|'K'|'k'|'m'|'s'|'a'|'y'|'M'|'d') {
            let mut count=1;while chars.peek()==Some(&ch) { chars.next();count+=1; }
            if ch=='a' { out.push_str(if hour<12 { "AM" } else { "PM" });continue; }
            let value=match ch { 'H'=>hour,'h'=>if hour%12==0 {12} else {hour%12},'K'=>hour%12,'k'=>if hour==0 {24} else {hour},'m'=>minute,'s'=>second,'M'=>month,'d'=>day,_=>if count==2 { year.rem_euclid(100) } else { year } };
            if ch=='y' && count>=4 { out.push_str(&format!("{value:04}")); }
            else if count>=2 { out.push_str(&format!("{value:02}")); } else { out.push_str(&value.to_string()); }
        } else { out.push(ch); }
    }
    out
}
pub(super) fn ellipsize_single_line(text:&str, width:f32, style:TextStyle, metrics:TextMetrics, mode:u8) -> String {
    if metrics.width(text,style)<=width { return text.to_owned(); }
    let mut used=metrics.width("…",style);
    if used>width { return String::new(); }
    let parts:Vec<(usize,&str)>=text.grapheme_indices(true).collect();
    let mut left=0usize;let mut right=parts.len();let mut prefix_end=0usize;let mut suffix_start=text.len();
    while left<right {
        let preferred_front=mode==3 || (mode==2 && left<=parts.len()-right);
        let mut chosen=None;
        for front in [preferred_front,!preferred_front] {
            if mode!=2 && front!=preferred_front { break; }
            let part=if front { parts[left].1 } else { parts[right-1].1 };
            let advance=metrics.width(part,style)+style.letter_spacing;
            if used+advance<=width { chosen=Some((front,advance));break; }
        }
        let Some((front,advance))=chosen else { break; };
        used+=advance;
        if front { prefix_end=parts[left].0+parts[left].1.len();left+=1; }
        else { right-=1;suffix_start=parts[right].0; }
    }
    format!("{}…{}",&text[..prefix_end],&text[suffix_start..])
}
/// Append an ellipsis to a truncated line, keeping complete graphemes in one pass.
pub(super) fn ellipsize_truncated_line(text:&str, width:f32, style:TextStyle, metrics:TextMetrics) -> String {
    let mut used=metrics.width("…",style);
    if used>width { return String::new(); }
    let mut end=0;
    for (index,grapheme) in text.trim_end().grapheme_indices(true) {
        let advance=metrics.width(grapheme,style)+style.letter_spacing;
        if used+advance>width { break; }
        used+=advance;
        end=index+grapheme.len();
    }
    format!("{}…",&text[..end])
}
/// Whitespace that can be consumed at a line break; non-breaking spaces stay with their text.
pub(super) fn breakable_space(ch:char) -> bool {
    (ch.is_whitespace() && !matches!(ch,'\u{00a0}'|'\u{2007}'|'\u{202f}')) || ch=='\u{200b}'
}
pub(super) struct WrappedLine<'a> {
    pub(super) text: std::borrow::Cow<'a,str>,
    pub(super) discretionary_hyphen: bool,
}
/// Wrap UTF-8 at whitespace or grapheme boundaries, preserving explicit newlines.
pub(super) fn wrapped_text_lines<'a>(text: &'a str, width: f32, style: TextStyle, metrics: TextMetrics, mut emit: impl FnMut(WrappedLine<'a>)) -> usize {
    let mut count = 0;
    let mut paragraph_start=0;
    let mut ends_with_separator=false;
    for (paragraph_end,_) in linebreaks(text).filter(|(_,kind)| *kind==BreakOpportunity::Mandatory) {
        let segment=&text[paragraph_start..paragraph_end];
        let separator_len=if segment.ends_with("\r\n") { 2 } else {
            segment.chars().next_back().filter(|c| matches!(*c,'\n'|'\r'|'\u{000b}'|'\u{000c}'|'\u{0085}'|'\u{2028}'|'\u{2029}')).map_or(0,char::len_utf8)
        };
        let paragraph=&segment[..segment.len()-separator_len];
        paragraph_start=paragraph_end;
        ends_with_separator=separator_len>0;
        let mut rest = paragraph;
        let mut breaks=linebreaks(paragraph).filter_map(|(index,kind)| if kind==BreakOpportunity::Allowed { Some(index) } else { None }).peekable();
        if rest.is_empty() { emit(WrappedLine { text:std::borrow::Cow::Borrowed(""), discretionary_hyphen:false }); count += 1; continue; }
        while !rest.is_empty() {
            let offset=paragraph.len()-rest.len();
            while breaks.peek().is_some_and(|p| *p<=offset) { breaks.next(); }
            let mut end = 0; let mut last_break = None; let mut advance = 0.0; let mut has_visible=false;
            for (i, grapheme) in rest.grapheme_indices(true) {
                let invisible=grapheme.chars().all(|ch| matches!(ch,'\u{00ad}'|'\u{200b}'|'\u{2060}'|'\u{feff}'));
                let next = advance + metrics.width(grapheme, style) + if has_visible && !invisible { style.letter_spacing } else { 0.0 };
                if next > width.max(1.0) && end > 0 { break; }
                advance = next; end = i + grapheme.len();
                if !invisible { has_visible=true; }
                while breaks.peek().is_some_and(|p| *p<=offset+end) {
                    let opportunity=breaks.next().unwrap();
                    let hyphen_fits=grapheme!="\u{00ad}" || advance+metrics.width("-",style)+if has_visible { style.letter_spacing } else { 0.0 }<=width.max(1.0);
                    if opportunity==offset+end && hyphen_fits { last_break=Some(end); }
                }
                if next > width.max(1.0) { break; }
            }
            if end < rest.len() && let Some(point) = last_break.filter(|point| *point > 0) { end = point; }
            let raw=rest[..end].trim_end_matches(breakable_space);
            let show_hyphen=end<rest.len() && raw.ends_with('\u{00ad}') && last_break==Some(end);
            if raw.contains('\u{00ad}') {
                let mut display=raw.replace('\u{00ad}',"");
                if show_hyphen { display.push('-'); }
                emit(WrappedLine { text:std::borrow::Cow::Owned(display), discretionary_hyphen:show_hyphen });
            } else { emit(WrappedLine { text:std::borrow::Cow::Borrowed(raw), discretionary_hyphen:false }); }
            count += 1;
            rest = rest[end..].trim_start_matches(breakable_space);
        }
    }
    if ends_with_separator || count==0 { emit(WrappedLine { text:std::borrow::Cow::Borrowed(""), discretionary_hyphen:false });count+=1; }
    count
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum TextBreakStrategy {
    #[default]
    Simple,
    HighQuality,
    Balanced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum TextHyphenationFrequency {
    #[default]
    None,
    Normal,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum TextJustificationMode {
    #[default]
    None,
    InterWord,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TextLayoutExt {
    pub(super) break_strategy: TextBreakStrategy,
    pub(super) hyphenation: TextHyphenationFrequency,
    pub(super) justification: TextJustificationMode,
    pub(super) auto_min_sp: f32,
    pub(super) auto_max_sp: f32,
    pub(super) auto_step_sp: f32,
}

impl TextLayoutExt {
    pub(super) fn has_auto_size(&self) -> bool {
        self.auto_min_sp.is_finite()
            && self.auto_max_sp.is_finite()
            && self.auto_min_sp > 0.0
            && self.auto_max_sp >= self.auto_min_sp
    }

    pub(super) fn has_custom_layout(&self) -> bool {
        self.break_strategy != TextBreakStrategy::Simple
            || self.hyphenation != TextHyphenationFrequency::None
            || self.justification != TextJustificationMode::None
            || self.has_auto_size()
    }
}

pub(super) fn parse_text_layout_ext(anchor: &str) -> TextLayoutExt {
    let mut parts = anchor.split('|');
    let break_strategy = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "high_quality" | "highquality" | "1" => TextBreakStrategy::HighQuality,
        "balanced" | "2" => TextBreakStrategy::Balanced,
        _ => TextBreakStrategy::Simple,
    };
    let hyphenation = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "normal" | "normal_fast" | "1" | "3" => TextHyphenationFrequency::Normal,
        "full" | "full_fast" | "2" | "4" => TextHyphenationFrequency::Full,
        _ => TextHyphenationFrequency::None,
    };
    let justification = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "inter_word" | "interword" | "1" => TextJustificationMode::InterWord,
        _ => TextJustificationMode::None,
    };
    let auto_raw = parts.next().unwrap_or("").trim();
    let (auto_min_sp, auto_max_sp, auto_step_sp) = if auto_raw.is_empty() {
        (0.0, 0.0, 1.0)
    } else {
        let mut nums = auto_raw.split(',');
        let parse_f = |s: Option<&str>| -> f32 {
            s.and_then(|v| v.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite() && *v >= 0.0)
                .unwrap_or(0.0)
        };
        let mn = parse_f(nums.next());
        let mx = parse_f(nums.next());
        let st = parse_f(nums.next());
        (mn, mx, if st > 0.0 { st } else { 1.0 })
    };
    TextLayoutExt {
        break_strategy,
        hyphenation,
        justification,
        auto_min_sp,
        auto_max_sp,
        auto_step_sp,
    }
}

pub(super) struct ExtWrappedLine {
    pub(super) text: String,
    pub(super) discretionary_hyphen: bool,
    pub(super) is_paragraph_end: bool,
}

pub(super) fn split_oversized_word_hyphenated(
    word: &str,
    max_w: f32,
    style: TextStyle,
    metrics: TextMetrics,
) -> Vec<(String, bool)> {
    let clean_all = word.replace('\u{00ad}', "");
    let limit = max_w.max(1.0);
    if metrics.width(&clean_all, style) <= limit + 0.01 {
        return vec![(clean_all, false)];
    }
    let hyphen_extra = metrics.width("-", style) + style.letter_spacing.max(0.0);
    let prefix_limit = (limit - hyphen_extra).max(0.0);
    let mut out = Vec::new();
    let mut rem = word;
    while !rem.is_empty() {
        let clean_rem = rem.replace('\u{00ad}', "");
        if metrics.width(&clean_rem, style) <= limit + 0.01 {
            out.push((clean_rem, false));
            break;
        }
        // Collect grapheme end offsets for the visible string (preferring soft hyphen breaks if any fit).
        let mut best_shy: Option<usize> = None;
        let mut best_grapheme: Option<usize> = None;
        let mut first_visible_end: Option<usize> = None;
        let mut acc = String::new();
        for (i, g) in rem.grapheme_indices(true) {
            let end = i + g.len();
            if g == "\u{00ad}" {
                if !acc.is_empty() && end < rem.len() {
                    let w = metrics.width(&acc, style);
                    if w <= prefix_limit + 0.01 {
                        best_shy = Some(end);
                    }
                }
                continue;
            }
            acc.push_str(g);
            if first_visible_end.is_none() {
                first_visible_end = Some(end);
            }
            let remaining_has_visible = rem[end..].chars().any(|c| c != '\u{00ad}');
            if !remaining_has_visible {
                break;
            }
            let w = metrics.width(&acc, style);
            if w <= prefix_limit + 0.01 {
                best_grapheme = Some(end);
            } else {
                break;
            }
        }
        if let Some(split_at) = best_shy.or(best_grapheme) {
            let mut piece = rem[..split_at].replace('\u{00ad}', "");
            piece.push('-');
            out.push((piece, true));
            rem = &rem[split_at..];
        } else if let Some(split_at) = first_visible_end && split_at < rem.len() {
            let piece = rem[..split_at].replace('\u{00ad}', "");
            out.push((piece, false));
            rem = &rem[split_at..];
        } else {
            out.push((clean_rem, false));
            break;
        }
    }
    if out.is_empty() {
        out.push((String::new(), false));
    }
    out
}

pub(super) fn wrap_text_lines_ext(
    text: &str,
    max_w: f32,
    style: TextStyle,
    metrics: TextMetrics,
    ext: TextLayoutExt,
) -> Vec<ExtWrappedLine> {
    let limit = max_w.max(1.0);
    let mut result = Vec::new();
    let mut paragraph_start = 0usize;
    let mut ends_with_separator = false;
    let space_adv = metrics.width(" ", style) + 2.0 * style.letter_spacing;

    for (paragraph_end, _) in linebreaks(text).filter(|(_, kind)| *kind == BreakOpportunity::Mandatory) {
        let segment = &text[paragraph_start..paragraph_end];
        let separator_len = if segment.ends_with("\r\n") {
            2
        } else {
            segment
                .chars()
                .next_back()
                .filter(|c| matches!(*c, '\n' | '\r' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'))
                .map_or(0, char::len_utf8)
        };
        let paragraph = &segment[..segment.len() - separator_len];
        paragraph_start = paragraph_end;
        ends_with_separator = separator_len > 0;

        if paragraph.is_empty() {
            result.push(ExtWrappedLine {
                text: String::new(),
                discretionary_hyphen: false,
                is_paragraph_end: true,
            });
            continue;
        }
        if ext.break_strategy == TextBreakStrategy::Simple && ext.hyphenation == TextHyphenationFrequency::None {
            let start_idx = result.len();
            wrapped_text_lines(paragraph, limit, style, metrics, |line| {
                result.push(ExtWrappedLine {
                    text: line.text.into_owned(),
                    discretionary_hyphen: line.discretionary_hyphen,
                    is_paragraph_end: false,
                });
            });
            if result.len() > start_idx {
                result.last_mut().unwrap().is_paragraph_end = true;
            }
            continue;
        }

        let raw_words: Vec<&str> = paragraph
            .split(breakable_space)
            .filter(|w| !w.is_empty())
            .collect();
        if raw_words.is_empty() {
            result.push(ExtWrappedLine {
                text: String::new(),
                discretionary_hyphen: false,
                is_paragraph_end: true,
            });
            continue;
        }

        let mut tokens: Vec<(String, bool)> = Vec::with_capacity(raw_words.len());
        for w in raw_words {
            if ext.hyphenation != TextHyphenationFrequency::None {
                tokens.extend(split_oversized_word_hyphenated(w, limit, style, metrics));
            } else {
                tokens.push((w.replace('\u{00ad}', ""), false));
            }
        }
        let n = tokens.len();
        let mut pref = vec![0.0_f32; n + 1];
        for (i, (tok, _)) in tokens.iter().enumerate() {
            pref[i + 1] = pref[i] + metrics.width(tok, style);
        }
        let span_width = |i: usize, j: usize| -> f32 {
            let words_w = pref[j] - pref[i];
            let gaps = j.saturating_sub(i + 1) as f32;
            words_w + gaps * space_adv
        };
        let has_forced_internal_break = n > 1 && tokens[..n - 1].iter().any(|(_, h)| *h);
        if !has_forced_internal_break && span_width(0, n) <= limit + 0.01 {
            let joined = tokens.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(" ");
            result.push(ExtWrappedLine {
                text: joined,
                discretionary_hyphen: tokens[n - 1].1,
                is_paragraph_end: true,
            });
            continue;
        }

        if ext.break_strategy == TextBreakStrategy::Simple {
            let mut i = 0usize;
            while i < n {
                let mut j = i + 1;
                while j < n && !tokens[j - 1].1 && span_width(i, j + 1) <= limit + 0.01 {
                    j += 1;
                }
                let joined = tokens[i..j]
                    .iter()
                    .map(|(t, _)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push(ExtWrappedLine {
                    text: joined,
                    discretionary_hyphen: tokens[j - 1].1,
                    is_paragraph_end: j == n,
                });
                i = j;
            }
        } else {
            let mut dp = vec![f64::INFINITY; n + 1];
            let mut line_cnt = vec![0usize; n + 1];
            let mut next_break = vec![n; n];
            dp[n] = 0.0;
            for i in (0..n).rev() {
                let mut best_cost = f64::INFINITY;
                let mut best_lines = usize::MAX;
                let mut best_j = i + 1;
                for j in (i + 1)..=n {
                    let lw = span_width(i, j);
                    if j > i + 1 && lw > limit + 0.01 {
                        break;
                    }
                    let is_last = j == n;
                    let slack = (limit - lw).max(0.0) as f64;
                    let line_cost = if is_last && ext.break_strategy == TextBreakStrategy::HighQuality {
                        0.0
                    } else {
                        slack * slack
                    };
                    let total = line_cost + dp[j];
                    let lines = 1 + line_cnt[j];
                    if total + 1e-3 < best_cost || ((total - best_cost).abs() <= 1e-3 && lines <= best_lines) {
                        best_cost = total;
                        best_lines = lines;
                        best_j = j;
                    }
                    if tokens[j - 1].1 {
                        break;
                    }
                }
                dp[i] = best_cost;
                line_cnt[i] = best_lines;
                next_break[i] = best_j;
            }
            let mut i = 0usize;
            while i < n {
                let j = next_break[i].max(i + 1).min(n);
                let joined = tokens[i..j]
                    .iter()
                    .map(|(t, _)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                result.push(ExtWrappedLine {
                    text: joined,
                    discretionary_hyphen: tokens[j - 1].1,
                    is_paragraph_end: j == n,
                });
                i = j;
            }
        }
    }

    if ends_with_separator || result.is_empty() {
        result.push(ExtWrappedLine {
            text: String::new(),
            discretionary_hyphen: false,
            is_paragraph_end: true,
        });
    }
    result
}

pub(super) fn paint_justified_line(
    f: &mut Frame<'_>,
    bounds: Rect,
    line: &str,
    color: Color,
    style: TextStyle,
    metrics: TextMetrics,
    rtl: bool,
) {
    let words: Vec<&str> = line.split(breakable_space).filter(|w| !w.is_empty()).collect();
    if words.len() < 2 {
        f.styled_text(bounds, line, color, style);
        return;
    }
    let word_style = TextStyle { align: 0, padding: 0.0, ..style };
    let widths: Vec<f32> = words.iter().map(|w| metrics.width(w, word_style)).collect();
    let words_sum: f32 = widths.iter().sum();
    if bounds.width <= words_sum + 0.01 {
        f.styled_text(bounds, line, color, style);
        return;
    }
    let gap = (bounds.width - words_sum) / (words.len() - 1) as f32;
    if !rtl {
        let mut x = bounds.x;
        for (word, w) in words.iter().zip(widths) {
            f.styled_text(Rect { x, y: bounds.y, width: w, height: bounds.height }, word, color, word_style);
            x += w + gap;
        }
    } else {
        let mut x = bounds.x + bounds.width;
        for (word, w) in words.iter().zip(widths) {
            x -= w;
            f.styled_text(Rect { x, y: bounds.y, width: w, height: bounds.height }, word, color, word_style);
            x -= gap;
        }
    }
}

pub(super) const TEXT_LINK_WEB: u8 = 1;
pub(super) const TEXT_LINK_EMAIL: u8 = 2;
pub(super) const TEXT_LINK_PHONE: u8 = 4;

#[derive(Clone, Copy, Debug)]
pub(super) struct TextLinksSpec {
    pub(super) mask: u8,
    pub(super) link_color: Option<Color>,
    pub(super) clickable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TextLinkSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn parse_text_links_spans(anchor: &str) -> TextLinksSpec {
    let pipes = anchor.bytes().filter(|&b| b == b'|').count();
    let (mask_raw, rgba_raw, click_raw) = if pipes >= 2 {
        let (head, c) = anchor.rsplit_once('|').unwrap_or((anchor, "1"));
        let (m, col) = head.rsplit_once('|').unwrap_or((head, ""));
        (m, col, c)
    } else if pipes == 1 {
        let (m, col) = anchor.split_once('|').unwrap_or((anchor, ""));
        (m, col, "1")
    } else {
        (anchor, "", "1")
    };
    let mut mask = 0u8;
    for tok in mask_raw.split([',', '|', '+', ' ']).map(str::trim).filter(|s| !s.is_empty()) {
        match tok.to_ascii_lowercase().as_str() {
            "all" => mask |= TEXT_LINK_WEB | TEXT_LINK_EMAIL | TEXT_LINK_PHONE,
            "web" => mask |= TEXT_LINK_WEB,
            "email" => mask |= TEXT_LINK_EMAIL,
            "phone" => mask |= TEXT_LINK_PHONE,
            "none" => {}
            _ => {}
        }
    }
    let link_color = parse_rgba_csv(rgba_raw.trim());
    let clickable = !matches!(click_raw.trim().to_ascii_lowercase().as_str(), "0" | "false");
    TextLinksSpec { mask, link_color, clickable }
}

pub(super) fn starts_with_ascii_ci(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len()
        && s.is_char_boundary(prefix.len())
        && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

pub(super) fn match_web_link(line: &str, start: usize, prev: Option<char>) -> Option<usize> {
    let tail = &line[start..];
    let (prefix_len, is_www) = if starts_with_ascii_ci(tail, "https://") {
        (8, false)
    } else if starts_with_ascii_ci(tail, "http://") {
        (7, false)
    } else if starts_with_ascii_ci(tail, "www.") {
        (4, true)
    } else {
        return None;
    };
    if is_www {
        if prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '/' | '@')) {
            return None;
        }
    } else if prev.is_some_and(char::is_alphanumeric) {
        return None;
    }
    let mut end = start + prefix_len;
    for (off, ch) in tail[prefix_len..].char_indices() {
        if ch.is_whitespace() || ch.is_control() || matches!(ch, '<' | '>' | '"') {
            break;
        }
        end = start + prefix_len + off + ch.len_utf8();
    }
    while end > start + prefix_len {
        let slice = &line[start..end];
        let Some(ch) = slice.chars().next_back() else { break };
        let strip = if matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"') {
            true
        } else if ch == ')' {
            slice.bytes().filter(|&b| b == b')').count() > slice.bytes().filter(|&b| b == b'(').count()
        } else if ch == ']' {
            slice.bytes().filter(|&b| b == b']').count() > slice.bytes().filter(|&b| b == b'[').count()
        } else if ch == '}' {
            slice.bytes().filter(|&b| b == b'}').count() > slice.bytes().filter(|&b| b == b'{').count()
        } else {
            false
        };
        if strip {
            end -= ch.len_utf8();
        } else {
            break;
        }
    }
    let body = &line[start + prefix_len..end];
    if body.is_empty() {
        return None;
    }
    if is_www {
        let host = body.split(['/', '?', '#']).next().unwrap_or("");
        let (a, b) = host.split_once('.')?;
        if a.is_empty() || b.is_empty() || !a.chars().any(char::is_alphanumeric) || !b.chars().any(char::is_alphanumeric) {
            return None;
        }
    } else {
        let first = body.chars().next()?;
        if !first.is_alphanumeric() && first != '[' {
            return None;
        }
    }
    Some(end)
}

pub(super) fn is_email_local_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

pub(super) fn match_email_link(line: &str, start: usize, prev: Option<char>) -> Option<usize> {
    if prev.is_some_and(is_email_local_char) {
        return None;
    }
    let tail = &line[start..];
    let first = tail.chars().next()?;
    if !first.is_ascii_alphanumeric() {
        return None;
    }
    let mut local_end = start;
    let mut at_pos = None;
    for (off, ch) in tail.char_indices() {
        if ch == '@' {
            at_pos = Some(start + off);
            break;
        }
        if !is_email_local_char(ch) {
            return None;
        }
        local_end = start + off + ch.len_utf8();
    }
    let at = at_pos?;
    if local_end <= start || line[start..local_end].ends_with('.') || line[start..local_end].contains("..") {
        return None;
    }
    let domain_start = at + 1;
    let mut end = domain_start;
    for (off, ch) in line[domain_start..].char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-') {
            end = domain_start + off + ch.len_utf8();
        } else {
            break;
        }
    }
    while end > domain_start {
        let b = line.as_bytes()[end - 1];
        if b == b'.' || b == b'-' {
            end -= 1;
        } else {
            break;
        }
    }
    if line[end..].starts_with('@') {
        return None;
    }
    let domain = &line[domain_start..end];
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return None;
    }
    for label in &labels {
        if label.is_empty() || label.starts_with('-') || label.ends_with('-') {
            return None;
        }
    }
    let tld = labels.last()?;
    if tld.len() < 2 || !tld.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    Some(end)
}

pub(super) fn match_phone_link(line: &str, start: usize, prev: Option<char>) -> Option<usize> {
    if prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '+' | '@')) {
        return None;
    }
    let tail = &line[start..];
    let mut chars = tail.chars();
    let first = chars.next()?;
    let valid_start = match first {
        '0'..='9' => true,
        '+' | '(' => chars.next().is_some_and(|c| c.is_ascii_digit()),
        _ => false,
    };
    if !valid_start {
        return None;
    }
    let mut end = start;
    let mut prev_sep = false;
    for (off, ch) in tail.char_indices() {
        let allowed = match ch {
            '0'..='9' => {
                prev_sep = false;
                true
            }
            '+' => off == 0,
            '-' | '.' => {
                if prev_sep {
                    false
                } else {
                    prev_sep = true;
                    true
                }
            }
            ' ' => {
                if prev_sep {
                    false
                } else {
                    let rest = &tail[off + 1..];
                    let ok = rest.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '(');
                    prev_sep = true;
                    ok
                }
            }
            '(' | ')' => {
                prev_sep = false;
                true
            }
            _ => false,
        };
        if !allowed {
            break;
        }
        end = start + off + ch.len_utf8();
    }
    while end > start {
        let slice = &line[start..end];
        let Some(ch) = slice.chars().next_back() else { break };
        if ch.is_ascii_digit() {
            break;
        }
        if ch == ')'
            && slice.bytes().filter(|&b| b == b')').count()
                == slice.bytes().filter(|&b| b == b'(').count()
        {
            break;
        }
        end -= ch.len_utf8();
    }
    if end <= start {
        return None;
    }
    let matched = &line[start..end];
    if matched.bytes().filter(|&b| b == b'(').count() != matched.bytes().filter(|&b| b == b')').count() {
        return None;
    }
    if line[end..].chars().next().is_some_and(char::is_alphanumeric) {
        return None;
    }
    let digits = matched.bytes().filter(|b| b.is_ascii_digit()).count();
    if !(7..=20).contains(&digits) {
        return None;
    }
    Some(end)
}

pub(super) fn find_text_link_spans(line: &str, mask: u8) -> Vec<TextLinkSpan> {
    let mut spans = Vec::new();
    if mask == 0 || line.is_empty() {
        return spans;
    }
    let mut prev: Option<char> = None;
    let mut cursor = 0usize;
    while cursor < line.len() {
        let matched = if mask & TEXT_LINK_WEB != 0 {
            match_web_link(line, cursor, prev)
        } else {
            None
        }
        .or_else(|| {
            if mask & TEXT_LINK_EMAIL != 0 {
                match_email_link(line, cursor, prev)
            } else {
                None
            }
        })
        .or_else(|| {
            if mask & TEXT_LINK_PHONE != 0 {
                match_phone_link(line, cursor, prev)
            } else {
                None
            }
        });
        if let Some(end) = matched
            && end > cursor
        {
            spans.push(TextLinkSpan { start: cursor, end });
            prev = line[cursor..end].chars().next_back();
            cursor = end;
        } else if let Some(ch) = line[cursor..].chars().next() {
            prev = Some(ch);
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }
    spans
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_text_line_with_links(
    f: &mut Frame<'_>,
    bounds: Rect,
    line: &str,
    text_color: Color,
    link_color: Color,
    style: TextStyle,
    metrics: TextMetrics,
    mask: u8,
    clickable: bool,
    enabled: bool,
    id: u64,
    link_counter: &mut usize,
    clicked_link: &mut Option<String>,
    underline_h: f32,
    fallback_field_clicked: bool,
) {
    let spans = find_text_link_spans(line, mask);
    if spans.is_empty() {
        f.styled_text(bounds, line, text_color, style);
        return;
    }
    let total_w = metrics.width(line, style);
    let line_x = match style.align {
        1 => bounds.x + (bounds.width - total_w) * 0.5,
        2 => bounds.x + bounds.width - style.padding - total_w,
        _ => bounds.x + style.padding,
    };
    let seg_style = TextStyle { align: 0, padding: 0.0, ..style };
    let seg_x_at = |byte_idx: usize| -> f32 {
        if byte_idx == 0 {
            line_x
        } else {
            let pw = metrics.width(&line[..byte_idx], seg_style);
            if pw > 0.0 {
                line_x + pw + style.letter_spacing
            } else {
                line_x
            }
        }
    };
    let baseline_y = bounds.y + bounds.height * 0.5 + style.size * 0.35;
    let underline_y = (baseline_y + underline_h.max(1.0))
        .min(bounds.y + bounds.height - underline_h)
        .max(bounds.y);
    let mut cursor = 0usize;
    for span in spans {
        if span.start > cursor {
            let seg = &line[cursor..span.start];
            let x = seg_x_at(cursor);
            let w = metrics.width(seg, seg_style);
            if w > 0.0 {
                f.styled_text(
                    Rect { x, y: bounds.y, width: w, height: bounds.height },
                    seg,
                    text_color,
                    seg_style,
                );
            }
        }
        let link_text = &line[span.start..span.end];
        let sx = seg_x_at(span.start);
        let sw = metrics.width(link_text, seg_style);
        let span_rect = Rect { x: sx, y: bounds.y, width: sw, height: bounds.height };
        if sw > 0.0 {
            f.styled_text(span_rect, link_text, link_color, seg_style);
            f.paint_rect(
                Rect { x: sx, y: underline_y, width: sw, height: underline_h },
                0.0,
                link_color,
            );
            if clickable {
                let link_id = anonymous_child_id(id, 42_000 + *link_counter);
                *link_counter += 1;
                let hit = f.target(link_id, span_rect, link_text, enabled, Role::Link);
                if hit.clicked
                    || (fallback_field_clicked
                        && enabled
                        && f.input.pointer.is_some_and(|p| span_rect.intersect(f.clip).contains(p)))
                {
                    *clicked_link = Some(link_text.to_owned());
                }
            }
        }
        cursor = span.end;
    }
    if cursor < line.len() {
        let seg = &line[cursor..];
        let x = seg_x_at(cursor);
        let w = metrics.width(seg, seg_style);
        if w > 0.0 {
            f.styled_text(
                Rect { x, y: bounds.y, width: w, height: bounds.height },
                seg,
                text_color,
                seg_style,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_justified_line_with_links(
    f: &mut Frame<'_>,
    bounds: Rect,
    line: &str,
    text_color: Color,
    link_color: Color,
    style: TextStyle,
    metrics: TextMetrics,
    rtl: bool,
    mask: u8,
    clickable: bool,
    enabled: bool,
    id: u64,
    link_counter: &mut usize,
    clicked_link: &mut Option<String>,
    underline_h: f32,
) {
    let spans = find_text_link_spans(line, mask);
    if spans.is_empty() {
        paint_justified_line(f, bounds, line, text_color, style, metrics, rtl);
        return;
    }
    let mut words: Vec<(usize, usize, &str)> = Vec::new();
    let mut in_word = false;
    let mut w_start = 0usize;
    for (idx, ch) in line.char_indices() {
        if breakable_space(ch) {
            if in_word {
                words.push((w_start, idx, &line[w_start..idx]));
                in_word = false;
            }
        } else if !in_word {
            w_start = idx;
            in_word = true;
        }
    }
    if in_word {
        words.push((w_start, line.len(), &line[w_start..]));
    }
    if words.len() < 2 {
        paint_text_line_with_links(
            f,
            bounds,
            line,
            text_color,
            link_color,
            style,
            metrics,
            mask,
            clickable,
            enabled,
            id,
            link_counter,
            clicked_link,
            underline_h,
            false,
        );
        return;
    }
    let word_style = TextStyle { align: 0, padding: 0.0, ..style };
    let widths: Vec<f32> = words.iter().map(|(_, _, w)| metrics.width(w, word_style)).collect();
    let words_sum: f32 = widths.iter().sum();
    if bounds.width <= words_sum + 0.01 {
        paint_text_line_with_links(
            f,
            bounds,
            line,
            text_color,
            link_color,
            style,
            metrics,
            mask,
            clickable,
            enabled,
            id,
            link_counter,
            clicked_link,
            underline_h,
            false,
        );
        return;
    }
    let gap = (bounds.width - words_sum) / (words.len() - 1) as f32;
    let mut word_xs = Vec::with_capacity(words.len());
    if !rtl {
        let mut x = bounds.x;
        for &w in &widths {
            word_xs.push(x);
            x += w + gap;
        }
    } else {
        let mut x = bounds.x + bounds.width;
        for &w in &widths {
            x -= w;
            word_xs.push(x);
            x -= gap;
        }
    }
    let baseline_y = bounds.y + bounds.height * 0.5 + style.size * 0.35;
    let underline_y = (baseline_y + underline_h.max(1.0))
        .min(bounds.y + bounds.height - underline_h)
        .max(bounds.y);
    let mut span_extents: Vec<(f32, f32)> = vec![(f32::INFINITY, f32::NEG_INFINITY); spans.len()];
    for (wi, &(ws, we, word)) in words.iter().enumerate() {
        let wx = word_xs[wi];
        let seg_x_in_word = |rel: usize| -> f32 {
            if rel == 0 {
                wx
            } else {
                let pw = metrics.width(&word[..rel], word_style);
                if pw > 0.0 { wx + pw + style.letter_spacing } else { wx }
            }
        };
        let mut cur = ws;
        for (si, span) in spans.iter().enumerate() {
            if span.end <= cur || span.start >= we {
                continue;
            }
            let ls = span.start.max(ws);
            let le = span.end.min(we);
            if ls > cur {
                let seg = &line[cur..ls];
                let x = seg_x_in_word(cur - ws);
                let w = metrics.width(seg, word_style);
                if w > 0.0 {
                    f.styled_text(Rect { x, y: bounds.y, width: w, height: bounds.height }, seg, text_color, word_style);
                }
            }
            let link_part = &line[ls..le];
            let lx = seg_x_in_word(ls - ws);
            let lw = metrics.width(link_part, word_style);
            if lw > 0.0 {
                f.styled_text(Rect { x: lx, y: bounds.y, width: lw, height: bounds.height }, link_part, link_color, word_style);
                span_extents[si].0 = span_extents[si].0.min(lx);
                span_extents[si].1 = span_extents[si].1.max(lx + lw);
            }
            cur = le;
        }
        if cur < we {
            let seg = &line[cur..we];
            let x = seg_x_in_word(cur - ws);
            let w = metrics.width(seg, word_style);
            if w > 0.0 {
                f.styled_text(Rect { x, y: bounds.y, width: w, height: bounds.height }, seg, text_color, word_style);
            }
        }
    }
    for (si, span) in spans.iter().enumerate() {
        let (min_x, max_x) = span_extents[si];
        if min_x.is_finite() && max_x > min_x {
            let sw = max_x - min_x;
            let span_rect = Rect { x: min_x, y: bounds.y, width: sw, height: bounds.height };
            f.paint_rect(
                Rect { x: min_x, y: underline_y, width: sw, height: underline_h },
                0.0,
                link_color,
            );
            if clickable {
                let link_text = &line[span.start..span.end];
                let link_id = anonymous_child_id(id, 42_000 + *link_counter);
                *link_counter += 1;
                let hit = f.target(link_id, span_rect, link_text, enabled, Role::Link);
                if hit.clicked {
                    *clicked_link = Some(link_text.to_owned());
                }
            }
        }
    }
}

