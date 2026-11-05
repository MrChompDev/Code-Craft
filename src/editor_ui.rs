// Multi-tab code editor: line numbers, active line, folding, syntax colors.
// Since the engine build-out, the code view is a real editable TextEdit with
// the syntax colorizer applied through a custom layouter.
use eframe::egui;
use crate::icons;
use crate::project::{ft_of, Ft};
use crate::theme::*;
use crate::App;
use egui::{Color32, FontFamily, FontId, Pos2, Sense, Vec2};

const ROW_H: f32 = 20.0;
const FONT_SIZE: f32 = 13.2;
/// Files above this size render without syntax coloring to stay smooth.
const HIGHLIGHT_CAP: usize = 200_000;

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
        let dirty = app.buffers.get(&id).map(|b| b.dirty).unwrap_or(false);
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
            egui::Align2::LEFT_CENTER,
            name.clone(),
            FontId::new(12.0, FontFamily::Proportional),
            if active { TEXT } else { TEXT2 },
        );
        // modified dot
        if dirty {
            p.circle_filled(Pos2::new(trect.right() - 24.0, trect.center().y), 2.6, ACCENT);
        }
        // close X
        let xr = egui::Rect::from_center_size(
            Pos2::new(trect.right() - 12.0, trect.center().y),
            Vec2::splat(14.0),
        );
        let xresp = ui.interact(xr, egui::Id::new("tabx").with(i), Sense::click());
        let xcolor = if xresp.hovered() { TEXT } else { TEXT3 };
        let cx = xr.center();
        let d = 3.2;
        p.line_segment([Pos2::new(cx.x - d, cx.y - d), Pos2::new(cx.x + d, cx.y + d)], egui::Stroke::new(1.4, xcolor));
        p.line_segment([Pos2::new(cx.x - d, cx.y + d), Pos2::new(cx.x + d, cx.y - d)], egui::Stroke::new(1.4, xcolor));
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
            // Make sure a buffer exists for the active tab (open_file usually did).
            if !app.buffers.contains_key(&id) {
                let text = app.project.read_file(id).unwrap_or_default();
                app.buffers.insert(id, crate::Buffer { text, dirty: false });
            }
            let ft = ft_of(&app.project.nodes[id].name);

            // gutter metrics
            let probe = ui.painter();
            let mono = FontId::monospace(FONT_SIZE);
            let cw = char_width(probe, &mono);
            let line_count = app.buffer_text(id).lines().count().max(1);
            let digits = line_count.to_string().len().max(2);
            let gutter_w = 14.0 + digits as f32 * cw + 14.0;

            // cursor line for gutter highlight
            let cur_line = app.cur_line.get(&id).copied().unwrap_or(1).min(line_count);

            ui.horizontal(|ui| {
                // ---- gutter ----
                let gutter_h = (line_count as f32 * ROW_H + 20.0).max(ui.available_height() - 8.0);
                let (grect, _) = ui.allocate_exact_size(Vec2::new(gutter_w, gutter_h), Sense::hover());
                let gp = ui.painter_at(grect);
                gp.rect_filled(grect, 0.0, EDITOR_BG);
                let y = grect.top() + 4.0 + (cur_line - 1) as f32 * ROW_H;
                gp.rect_filled(
                    egui::Rect::from_min_size(Pos2::new(grect.left(), y), Vec2::new(grect.width(), ROW_H)),
                    0.0,
                    ACTIVE_LINE,
                );
                for i in 1..=line_count {
                    let cy = grect.top() + 4.0 + (i - 1) as f32 * ROW_H + ROW_H / 2.0;
                    gp.text(
                        Pos2::new(grect.right() - 8.0, cy),
                        egui::Align2::RIGHT_CENTER,
                        i.to_string(),
                        mono.clone(),
                        if i == cur_line { GUTTER_NUM_CUR } else { GUTTER_NUM },
                    );
                }

                // ---- editable text ----
                let Some(buf) = app.buffers.get_mut(&id) else { return };
                let text = buf.text.clone();
                // Tokenize once per text revision; the layouter reuses the
                // cached tokens whenever the buffer text is unchanged.
                let text_hash = hash_str(&text);
                let cached_toks = match app.token_cache.get(&id) {
                    Some((h, toks)) if *h == text_hash => Some(toks.clone()),
                    _ => {
                        let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
                        let toks = tokens_for(ft, &lines);
                        app.token_cache.insert(id, (text_hash, toks.clone()));
                        Some(toks)
                    }
                };
                let mut layouter =
                    |ui: &egui::Ui, src: &dyn egui::TextBuffer, _wrap: f32| {
                        let s = src.as_str();
                        // If the user typed this very frame, colorize fresh
                        // against the live text (cache catches up next frame).
                        let toks = if s == text.as_str() {
                            cached_toks.clone().unwrap_or_default()
                        } else {
                            let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
                            tokens_for(ft, &lines)
                        };
                        let job = build_job(s, &toks);
                        ui.fonts(|f| f.layout_job(job))
                    };
                let out = egui::TextEdit::multiline(&mut buf.text)
                    .id(egui::Id::new("editor").with(id))
                    .font(egui::TextStyle::Name("editor".into()))
                    .desired_width(ui.available_width())
                    .frame(false)
                    .layouter(&mut layouter)
                    .show(ui);

                // track cursor line
                if let Some(cursor) = out.cursor_range {
                    let idx = cursor.primary.index;
                    let line = text[..idx.min(text.len())].matches('\n').count() + 1;
                    app.cur_line.insert(id, line);
                }
                if out.response.changed() {
                    app.mark_dirty(id, true);
                }
                if app.scroll_to_cursor {
                    app.scroll_to_cursor = false;
                    out.response.scroll_to_me(Some(egui::Align::Center));
                }
            });
        });
}

/// Assemble the syntax-colored LayoutJob from precomputed line tokens.
/// Falls back to plain text for very large files.
fn build_job(text: &str, toks: &[Vec<Tok>]) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mono = FontId::monospace(FONT_SIZE);
    if text.len() > HIGHLIGHT_CAP {
        job.append(text, 0.0, egui::TextFormat::simple(mono, SYN_PLAIN));
        return job;
    }
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let line_toks = toks.get(i).cloned().unwrap_or_default();
        if line_toks.is_empty() {
            job.append(line, 0.0, egui::TextFormat::simple(mono.clone(), SYN_PLAIN));
        } else {
            let mut pos = 0usize;
            for t in &line_toks {
                if t.s > pos {
                    let seg: String = line.chars().skip(pos).take(t.s - pos).collect();
                    job.append(seg.as_str(), 0.0, egui::TextFormat::simple(mono.clone(), SYN_PLAIN));
                }
                if t.e > t.s {
                    let seg: String = line.chars().skip(t.s).take(t.e - t.s).collect();
                    job.append(seg.as_str(), 0.0, egui::TextFormat::simple(mono.clone(), t.c));
                }
                pos = pos.max(t.e);
            }
            if pos < line.chars().count() {
                let seg: String = line.chars().skip(pos).collect();
                job.append(seg.as_str(), 0.0, egui::TextFormat::simple(mono.clone(), SYN_PLAIN));
            }
        }
        // newline keeps the job text identical to the buffer text
        job.append("\n", 0.0, egui::TextFormat::simple(mono.clone(), SYN_PLAIN));
    }
    job
}

/// Small stable hash for the token memo (not cryptographic).
pub fn hash_str(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
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
