// Multi-tab code editor: line numbers, active line, folding, syntax colors.
use eframe::egui;
use crate::icons::{self, Chev};
use crate::project::{ft_of, Ft};
use crate::theme::*;
use crate::App;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Sense, Vec2};

const ROW_H: f32 = 20.0;
const FONT_SIZE: f32 = 13.2;

#[derive(Clone, Copy)]
pub struct Tok {
    s: usize,
    e: usize,
    c: Color32,
}

pub fn draw(app: &mut App, ui: &mut egui::Ui) {
    tabs_bar(app, ui);
    code_view(app, ui);
}

// ---------------- tabs ----------------

fn tabs_bar(app: &mut App, ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, PANEL_BG);
    p.line_segment(
        [Pos2::new(rect.left(), rect.bottom()), Pos2::new(rect.right(), rect.bottom())],
        egui::Stroke::new(1.0, BORDER),
    );

    let mono = FontId::new(12.0, FontFamily::Proportional);
    let mut x = rect.left() + 2.0;
    let tabs = app.open_tabs.clone();
    for (i, &id) in tabs.iter().enumerate() {
        let name = app.project.nodes[id].name.clone();
        let active = app.active == Some(id);
        let name_w = p
            .layout_no_wrap(name.clone(), mono.clone(), TEXT)
            .rect
            .width();
        let w = (10.0 + 14.0 + 6.0 + name_w + 6.0 + 14.0 + 10.0).min(230.0);
        let trect = egui::Rect::from_min_size(Pos2::new(x, rect.top() + 3.0), Vec2::new(w, 28.0));

        if active {
            p.rect_filled(trect, 0.0, EDITOR_BG);
            p.line_segment(
                [Pos2::new(trect.left(), trect.bottom()), Pos2::new(trect.right(), trect.bottom())],
                egui::Stroke::new(2.0, ACCENT),
            );
        }
        let hover_resp = ui.interact(trect, egui::Id::new("tab").with(i), Sense::click());
        if !active && hover_resp.hovered() {
            p.rect_filled(trect, 0.0, HOVER);
        }

        icons::badge(p, Pos2::new(trect.left() + 15.0, trect.center().y), ft_of(&name), false);
        p.text(
            Pos2::new(trect.left() + 26.0, trect.center().y),
            Align2::LEFT_CENTER,
            &name,
            mono.clone(),
            if active { TEXT } else { TEXT2 },
        );

        // close button
        let xc = Pos2::new(trect.right() - 14.0, trect.center().y);
        let xr = egui::Rect::from_center_size(xc, Vec2::splat(16.0));
        let xresp = ui.interact(xr, egui::Id::new("tabx").with(i), Sense::click());
        let xcol = if xresp.hovered() { TEXT } else { TEXT3 };
        if xresp.hovered() {
            p.circle_filled(xc, 7.0, if active { HOVER } else { SEL_BG });
        }
        icons::x_mark(p, xc, 3.2, xcol);
        if xresp.clicked() {
            app.close_tab(id);
        } else if hover_resp.clicked() && !xresp.hovered() {
            app.active = Some(id);
            app.selected = Some(id);
            app.term_focus = false;
        }
        x += w + 1.0;
    }
}

// ---------------- code view ----------------

fn code_view(app: &mut App, ui: &mut egui::Ui) {
    egui::ScrollArea::both()
        .auto_shrink(false)
        .show(ui, |ui| {
            let Some(id) = app.active else {
                empty_state(ui);
                return;
            };
            let lines: Vec<String> = match &app.project.nodes[id].content {
                Some(c) => c.clone(),
                None => Vec::new(),
            };
            let ft = ft_of(&app.project.nodes[id].name);
            let toks = tokens_for(ft, &lines);

            let mono = FontId::monospace(FONT_SIZE);
            let p0 = ui.painter();
            let cw = char_width(p0, &mono);
            let digits = (lines.len().max(1)).to_string().len().max(2);
            let gutter_w = 14.0 + digits as f32 * cw + 16.0;
            let max_text_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f32 * cw;
            let total_w = gutter_w + max_text_w + 40.0;
            let total_h = lines.len() as f32 * ROW_H + 24.0;

            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(total_w.max(ui.available_width()), total_h), Sense::click());
            let p = &ui.painter_at(rect);
            p.rect_filled(rect, 0.0, EDITOR_BG);

            let cur_line = {
                let e = app.cur_line.entry(id).or_insert(1);
                *e = (*e).min(lines.len().max(1));
                *e
            };

            // active line highlight
            if !lines.is_empty() {
                let y = rect.top() + 8.0 + (cur_line - 1) as f32 * ROW_H;
                p.rect_filled(
                    egui::Rect::from_min_size(
                        Pos2::new(rect.left(), y),
                        Vec2::new(rect.width(), ROW_H),
                    ),
                    0.0,
                    ACTIVE_LINE,
                );
            }

            // fold ranges
            let ranges = fold_ranges(&lines);
            let folded: std::collections::HashSet<usize> = ranges
                .iter()
                .filter(|&&(s, _)| *app.folds.get(&(id, s)).unwrap_or(&false))
                .map(|&(s, _)| s)
                .collect();
            if let Some(collapse) = app.fold_action.take() {
                for &(s, _) in &ranges {
                    app.folds.insert((id, s), collapse);
                }
            }
            let hidden: std::collections::HashSet<usize> = ranges
                .iter()
                .filter(|&&(s, _)| folded.contains(&s))
                .flat_map(|&(s, e)| (s + 1)..=e)
                .collect();

            // gutter numbers + code
            let mut row_rects: Vec<(usize, egui::Rect)> = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                let li = i + 1;
                if hidden.contains(&i) {
                    continue;
                }
                let y = rect.top() + 8.0 + i as f32 * ROW_H;
                let cy = y + ROW_H / 2.0;
                let is_cur = li == cur_line;
                p.text(
                    Pos2::new(rect.left() + gutter_w - 12.0 - cw * 1.0, cy),
                    Align2::RIGHT_CENTER,
                    li.to_string(),
                    mono.clone(),
                    if is_cur { GUTTER_NUM_CUR } else { GUTTER_NUM },
                );
                // fold chevron
                if ranges.iter().any(|&(s, _)| s == i) {
                    let cc = Pos2::new(rect.left() + gutter_w - 4.0, cy);
                    let collapsed = folded.contains(&i);
                    icons::chevron(
                        p,
                        cc,
                        4.5,
                        if collapsed { TEXT2 } else { TEXT3 },
                        if collapsed { Chev::Right } else { Chev::Down },
                    );
                    row_rects.push((
                        i,
                        egui::Rect::from_center_size(cc, Vec2::splat(14.0)),
                    ));
                }
                // tokens
                let line_toks = toks.get(i).cloned().unwrap_or_default();
                let tx = rect.left() + gutter_w + 6.0;
                if line_toks.is_empty() {
                    if !line.is_empty() {
                        p.text(Pos2::new(tx, cy), Align2::LEFT_CENTER, line.clone(), mono.clone(), SYN_PLAIN);
                    }
                } else {
                    for t in &line_toks {
                        if t.s >= line.chars().count() {
                            continue;
                        }
                        let seg: String = line.chars().skip(t.s).take(t.e - t.s).collect();
                        p.text(
                            Pos2::new(tx + t.s as f32 * cw, cy),
                            Align2::LEFT_CENTER,
                            seg,
                            mono.clone(),
                            t.c,
                        );
                    }
                }
                // folded placeholder
                if folded.contains(&i) {
                    let end = ranges.iter().find(|&&(s, _)| s == i).map(|&(_, e)| e).unwrap_or(i);
                    let dots_x = tx + line.chars().count() as f32 * cw + 6.0;
                    p.text(
                        Pos2::new(dots_x, cy),
                        Align2::LEFT_CENTER,
                        format!("… +{} lines", end - i),
                        mono.clone(),
                        FOLD_DOTS,
                    );
                }
            }

            // interactions
            if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let rel = (pos.y - rect.top() - 8.0) / ROW_H;
                    let ln = (rel.floor() as usize + 1).clamp(1, lines.len().max(1));
                    app.cur_line.insert(id, ln);
                }
                app.term_focus = false;
            }
            for (i, cr) in row_rects {
                let r = ui.interact(cr, egui::Id::new("fold").with(id).with(i), Sense::click());
                if r.clicked() {
                    let e = app.folds.entry((id, i)).or_insert(false);
                    *e = !*e;
                }
                if r.hovered() {
                    let hp = ui.painter_at(rect);
                    hp.circle_filled(cr.center(), 7.0, HOVER);
                    let collapsed = folded.contains(&i);
                    icons::chevron(
                        &hp,
                        cr.center(),
                        4.5,
                        TEXT,
                        if collapsed { Chev::Right } else { Chev::Down },
                    );
                }
            }
        });
}

fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.35);
        ui.label(
            egui::RichText::new("Search Everywhere: Double \u{21e7}")
                .size(13.0)
                .color(TEXT3),
        );
        ui.label(egui::RichText::new("Go to File: Ctrl+N").size(13.0).color(TEXT3));
        ui.label(egui::RichText::new("Recent Files: Ctrl+E").size(13.0).color(TEXT3));
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Drop files here to open them").size(13.0).color(TEXT3));
    });
}

fn char_width(p: &egui::Painter, font: &FontId) -> f32 {
    (p.layout_no_wrap("0".repeat(32), font.clone(), TEXT).rect.width() / 32.0).max(6.0)
}

// ---------------- folding ----------------

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

/// Indentation-based fold ranges: (open_line, last_hidden_line).
pub fn fold_ranges(lines: &[String]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in 0..lines.len() {
        let base = indent_of(&lines[i]);
        if lines[i].trim().is_empty() {
            continue;
        }
        if i + 1 >= lines.len() || indent_of(&lines[i + 1]) <= base {
            continue;
        }
        let mut end = i;
        let mut j = i + 1;
        while j < lines.len() {
            let ind = indent_of(&lines[j]);
            if ind > base {
                end = j;
                j += 1;
            } else {
                break;
            }
        }
        // include the closing brace/paren line if it follows
        if end + 1 < lines.len() {
            let t = lines[end + 1].trim_start();
            if t.starts_with('}') || t.starts_with(')') || t.starts_with(']') || t.starts_with("</") {
                end += 1;
            }
        }
        if end > i {
            out.push((i, end));
        }
    }
    out
}

// ---------------- syntax highlighting ----------------

struct LangCfg {
    kw: &'static [&'static str],
    line_comment: Option<&'static str>,
    block_comment: Option<(&'static str, &'static str)>,
    strings: &'static [char],
}

fn cfg_for(ft: Ft) -> Option<LangCfg> {
    match ft {
        Ft::Js => Some(LangCfg {
            kw: &[
                "const", "let", "var", "function", "class", "extends", "constructor", "this",
                "return", "if", "else", "for", "of", "in", "while", "await", "async", "throw",
                "new", "typeof", "import", "export", "from", "default", "try", "catch", "true",
                "false", "null", "undefined", "break", "continue", "instanceof", "delete", "void",
                "yield", "static", "get", "set",
            ],
            line_comment: Some("//"),
            block_comment: Some(("/*", "*/")),
            strings: &['\'', '"', '`'],
        }),
        Ft::Py => Some(LangCfg {
            kw: &[
                "def", "class", "import", "from", "as", "if", "elif", "else", "for", "while",
                "return", "in", "not", "and", "or", "with", "try", "except", "finally", "pass",
                "raise", "lambda", "global", "yield", "assert", "del", "is", "async", "await",
                "True", "False", "None", "print",
            ],
            line_comment: Some("#"),
            block_comment: None,
            strings: &['\'', '"'],
        }),
        Ft::Json => Some(LangCfg {
            kw: &["true", "false", "null"],
            line_comment: None,
            block_comment: None,
            strings: &['"'],
        }),
        Ft::Rs => Some(LangCfg {
            kw: &[
                "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "use", "mod",
                "match", "if", "else", "for", "in", "while", "loop", "return", "self", "Self",
                "true", "false", "const", "static", "where", "as", "dyn", "crate",
            ],
            line_comment: Some("//"),
            block_comment: Some(("/*", "*/")),
            strings: &['"'],
        }),
        Ft::Md | Ft::Txt | Ft::Other => None,
        Ft::Html | Ft::Css => None, // custom scanners below
    }
}

pub fn tokens_for(ft: Ft, lines: &[String]) -> Vec<Vec<Tok>> {
    match ft {
        Ft::Html => scan_html(lines),
        Ft::Css => scan_css(lines),
        Ft::Md => scan_md(lines),
        _ => match cfg_for(ft) {
            Some(cfg) => scan_generic(cfg, ft, lines),
            None => vec![Vec::new(); lines.len()],
        },
    }
}

fn scan_generic(cfg: LangCfg, ft: Ft, lines: &[String]) -> Vec<Vec<Tok>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut in_block = false;
    let (bs, be) = cfg.block_comment.unwrap_or(("", ""));
    for line in lines {
        let mut toks: Vec<Tok> = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut prev_word = String::new();
        while i < chars.len() {
            if in_block {
                let rest: String = chars[i..].iter().collect();
                if let Some(pos) = rest.find(be) {
                    toks.push(Tok { s: i, e: i + pos + be.len(), c: SYN_COM });
                    i += pos + be.len();
                    in_block = false;
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                }
                continue;
            }
            let rest: String = chars[i..].iter().collect();
            if !bs.is_empty() && rest.starts_with(bs) {
                let rest2 = &rest[bs.len()..];
                if let Some(pos) = rest2.find(be) {
                    toks.push(Tok { s: i, e: i + bs.len() + pos + be.len(), c: SYN_COM });
                    i += bs.len() + pos + be.len();
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                    in_block = true;
                }
                continue;
            }
            if let Some(lc) = cfg.line_comment {
                if rest.starts_with(lc) {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                    continue;
                }
            }
            let c = chars[i];
            if cfg.strings.contains(&c) {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != c {
                    if chars[j] == '\\' {
                        j += 1;
                    }
                    j += 1;
                }
                let end = (j + 1).min(chars.len());
                // JSON: key vs value
                let color = if ft == Ft::Json {
                    let after: String = chars[end..].iter().collect();
                    if after.trim_start().starts_with(':') {
                        SYN_FN
                    } else {
                        SYN_STR
                    }
                } else {
                    SYN_STR
                };
                toks.push(Tok { s: i, e: end, c: color });
                i = end;
                prev_word.clear();
                continue;
            }
            if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).map_or(false, |d| d.is_ascii_digit())) {
                let mut j = i;
                while j < chars.len()
                    && (chars[j].is_ascii_alphanumeric()
                        || chars[j] == '.'
                        || chars[j] == '_'
                        || chars[j] == 'x')
                {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_NUM });
                i = j;
                prev_word.clear();
                continue;
            }
            if c == '#' && ft == Ft::Css {
                // hex color
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_alphanumeric() {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_NUM });
                i = j;
                continue;
            }
            if c == '@' && ft == Ft::Py {
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_TYPE });
                i = j;
                continue;
            }
            if c == '@' && ft == Ft::Css {
                let mut j = i;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '-' || chars[j] == '@') {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_KW });
                i = j;
                continue;
            }
            if c.is_alphabetic() || c == '_' {
                let mut j = i;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '-') {
                    j += 1;
                }
                let word: String = chars[i..j].iter().collect();
                let after: String = chars[j..].iter().collect();
                let color = if cfg.kw.contains(&word.as_str()) {
                    SYN_KW
                } else if prev_word == "def" || prev_word == "function" || after.trim_start().starts_with('(') {
                    SYN_FN
                } else if prev_word == "class" || word.chars().next().map_or(false, |ch| ch.is_uppercase()) {
                    SYN_TYPE
                } else {
                    SYN_PLAIN
                };
                toks.push(Tok { s: i, e: j, c: color });
                prev_word = word;
                i = j;
                continue;
            }
            // punctuation / whitespace
            let mut j = i + 1;
            while j < chars.len() && !(chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '"' || chars[j] == '\'') {
                j += 1;
            }
            toks.push(Tok { s: i, e: j, c: SYN_PLAIN });
            if chars[i] == ':' || chars[i] == '=' || chars[i] == '(' || chars[i] == '.' {
                prev_word.clear();
            }
            i = j;
        }
        out.push(toks);
    }
    out
}

fn scan_html(lines: &[String]) -> Vec<Vec<Tok>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut in_comment = false;
    let mut in_tag = false;
    for line in lines {
        let mut toks: Vec<Tok> = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let rest: String = chars[i..].iter().collect();
            if in_comment {
                if let Some(pos) = rest.find("-->") {
                    toks.push(Tok { s: i, e: i + pos + 3, c: SYN_COM });
                    i += pos + 3;
                    in_comment = false;
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                }
                continue;
            }
            if rest.starts_with("<!--") {
                if let Some(pos) = rest[4..].find("-->") {
                    toks.push(Tok { s: i, e: i + 4 + pos + 3, c: SYN_COM });
                    i += 4 + pos + 3;
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                    in_comment = true;
                }
                continue;
            }
            if !in_tag && chars[i] == '<' {
                let mut j = i;
                let mut closed = false;
                while j < chars.len() {
                    if chars[j] == '>' {
                        j += 1;
                        closed = true;
                        break;
                    }
                    if chars[j] == '"' || chars[j] == '\'' {
                        let q = chars[j];
                        j += 1;
                        while j < chars.len() && chars[j] != q {
                            j += 1;
                        }
                    }
                    j += 1;
                }
                // tokenize the tag: brackets + name orange, attr names purple, strings green
                let mut k = i;
                let mut word_start = None;
                let mut flush = |toks: &mut Vec<Tok>, chars: &[char], a: usize, b: usize| {
                    if b > a {
                        let w: String = chars[a..b].iter().collect();
                        let col = if w.starts_with('<') || w.starts_with('/') || w == ">" {
                            SYN_TAG
                        } else {
                            SYN_ATTR
                        };
                        toks.push(Tok { s: a, e: b, c: col });
                    }
                };
                while k < j {
                    let c = chars[k];
                    if c.is_alphabetic() || c == '/' || c == '<' || c == '>' || c == '!' {
                        if c == '"' || c == '\'' {
                            flush(&mut toks, &chars, word_start.take().unwrap_or(k), k);
                            let q = c;
                            let mut m = k + 1;
                            while m < j && chars[m] != q {
                                m += 1;
                            }
                            toks.push(Tok { s: k, e: (m + 1).min(j), c: SYN_STR });
                            k = m + 1;
                            continue;
                        }
                        match word_start {
                            None => word_start = Some(k),
                            Some(_) => {
                                if !c.is_alphanumeric() && c != '-' && c != '/' {
                                    flush(&mut toks, &chars, word_start.take().unwrap_or(k), k);
                                    word_start = Some(k);
                                }
                            }
                        }
                        if c == '<' || c == '>' {
                            flush(&mut toks, &chars, word_start.take().unwrap_or(k), k + 1);
                        }
                    } else if c == '=' {
                        flush(&mut toks, &chars, word_start.take().unwrap_or(k), k);
                        toks.push(Tok { s: k, e: k + 1, c: SYN_PLAIN });
                    }
                    k += 1;
                }
                if let Some(a) = word_start {
                    flush(&mut toks, &chars, a, j);
                }
                i = j;
                in_tag = !closed;
                continue;
            }
            // plain text until next '<'
            let mut j = i;
            while j < chars.len() && chars[j] != '<' {
                j += 1;
            }
            if j > i {
                toks.push(Tok { s: i, e: j, c: SYN_PLAIN });
            }
            i = j;
        }
        out.push(toks);
    }
    out
}

fn scan_css(lines: &[String]) -> Vec<Vec<Tok>> {
    let mut out = Vec::with_capacity(lines.len());
    let mut in_comment = false;
    for line in lines {
        let mut toks: Vec<Tok> = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let trimmed = line.trim();
        let is_selector = trimmed.ends_with('{')
            || (trimmed.contains('{') && !trimmed.contains(':'))
            || trimmed.starts_with('@');
        while i < chars.len() {
            let rest: String = chars[i..].iter().collect();
            if in_comment {
                if let Some(pos) = rest.find("*/") {
                    toks.push(Tok { s: i, e: i + pos + 2, c: SYN_COM });
                    i += pos + 2;
                    in_comment = false;
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                }
                continue;
            }
            if rest.starts_with("/*") {
                if let Some(pos) = rest[2..].find("*/") {
                    toks.push(Tok { s: i, e: i + 2 + pos + 2, c: SYN_COM });
                    i += 2 + pos + 2;
                } else {
                    toks.push(Tok { s: i, e: chars.len(), c: SYN_COM });
                    i = chars.len();
                    in_comment = true;
                }
                continue;
            }
            let c = chars[i];
            if c == '"' || c == '\'' {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != c {
                    j += 1;
                }
                toks.push(Tok { s: i, e: (j + 1).min(chars.len()), c: SYN_STR });
                i = j + 1;
                continue;
            }
            if is_selector {
                // color whole selector-ish line up to '{'
                let mut j = i;
                while j < chars.len() && chars[j] != '{' {
                    j += 1;
                }
                if j > i {
                    toks.push(Tok { s: i, e: j, c: SYN_SEL });
                }
                if j < chars.len() {
                    toks.push(Tok { s: j, e: j + 1, c: SYN_PLAIN });
                }
                i = j + 1;
                continue;
            }
            if c == '#' {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_alphanumeric() {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_NUM });
                i = j;
                continue;
            }
            if c.is_ascii_digit()
                || (c == '.' && chars.get(i + 1).map_or(false, |d| d.is_ascii_digit()))
            {
                let mut j = i;
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '.' || chars[j] == '%') {
                    j += 1;
                }
                toks.push(Tok { s: i, e: j, c: SYN_NUM });
                i = j;
                continue;
            }
            if c.is_alphabetic() || c == '-' || c == '@' {
                let mut j = i;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '-' || chars[j] == '@') {
                    j += 1;
                }
                let word: String = chars[i..j].iter().collect();
                let after: String = chars[j..].iter().collect();
                let col = if word.starts_with('@') {
                    SYN_KW
                } else if after.trim_start().starts_with(':') || after.starts_with(':') {
                    SYN_FN
                } else {
                    SYN_PLAIN
                };
                toks.push(Tok { s: i, e: j, c: col });
                i = j;
                continue;
            }
            toks.push(Tok { s: i, e: i + 1, c: SYN_PLAIN });
            i += 1;
        }
        out.push(toks);
    }
    out
}

fn scan_md(lines: &[String]) -> Vec<Vec<Tok>> {
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let mut toks: Vec<Tok> = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let n = chars.len();
        if n == 0 {
            out.push(toks);
            continue;
        }
        let t = line.trim_start();
        if t.starts_with('#') {
            toks.push(Tok { s: 0, e: n, c: SYN_HEAD });
        } else if t.starts_with("```") {
            toks.push(Tok { s: 0, e: n, c: SYN_COM });
        } else if t.starts_with('>') {
            toks.push(Tok { s: 0, e: n, c: SYN_COM });
        } else if t.starts_with('-') || t.starts_with('*') {
            toks.push(Tok { s: 0, e: 1 + (line.len() - t.len()), c: SYN_KW });
            toks.push(Tok { s: 1 + (line.len() - t.len()), e: n, c: SYN_PLAIN });
        } else {
            // inline `code` and **bold**
            let mut i = 0;
            while i < n {
                if chars[i] == '`' {
                    let mut j = i + 1;
                    while j < n && chars[j] != '`' {
                        j += 1;
                    }
                    toks.push(Tok { s: i, e: (j + 1).min(n), c: SYN_STR });
                    i = j + 1;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
                    let mut j = i + 2;
                    while j + 1 < n && !(chars[j] == '*' && chars[j + 1] == '*') {
                        j += 1;
                    }
                    let end = (j + 2).min(n);
                    toks.push(Tok { s: i, e: end, c: SYN_KW });
                    i = end;
                } else if chars[i] == '[' {
                    let mut j = i + 1;
                    while j < n && chars[j] != ']' {
                        j += 1;
                    }
                    let end = (j + 1).min(n);
                    toks.push(Tok { s: i, e: end, c: SYN_FN });
                    i = end;
                } else {
                    let mut j = i;
                    while j < n && chars[j] != '`' && !(chars[j] == '*' && chars.get(j + 1) == Some(&'*')) && chars[j] != '[' {
                        j += 1;
                    }
                    if j > i {
                        toks.push(Tok { s: i, e: j, c: SYN_PLAIN });
                    }
                    i = j.max(i + 1);
                }
            }
        }
        out.push(toks);
    }
    out
}
