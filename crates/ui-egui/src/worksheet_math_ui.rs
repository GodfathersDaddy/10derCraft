//! A small two-dimensional text surface. Carets retain source byte offsets while
//! fractions, powers and subscripts are laid out in two dimensions. Evaluation
//! belongs exclusively to the calculation engine.
use crate::theme::Tokens;
use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use std::ops::Range;
#[derive(Clone)]
struct Glyph {
    text: String,
    at: Pos2,
    size: f32,
    span: Range<usize>,
    extent: Vec2,
}
#[derive(Clone)]
struct Caret {
    index: usize,
    at: Pos2,
    height: f32,
}
#[derive(Clone, Default)]
pub struct MathBox {
    pub size: Vec2,
    pub baseline: f32,
    glyphs: Vec<Glyph>,
    carets: Vec<Caret>,
    lines: Vec<[Pos2; 2]>,
}
impl MathBox {
    fn shifted(mut self, by: Vec2) -> Self {
        for g in &mut self.glyphs {
            g.at += by;
        }
        for c in &mut self.carets {
            c.at += by;
        }
        for line in &mut self.lines {
            for point in line {
                *point += by;
            }
        }
        self
    }
    fn append(&mut self, other: Self) {
        self.glyphs.extend(other.glyphs);
        self.carets.extend(other.carets);
        self.lines.extend(other.lines);
    }
    pub fn nearest(&self, point: Pos2) -> usize {
        self.carets
            .iter()
            .min_by(|a, b| {
                let distance = |c: &Caret| {
                    let y = (point.y - c.at.y).clamp(0., c.height) + c.at.y;
                    (point.x - c.at.x).powi(2) + (point.y - y).powi(2) * 3.
                };
                distance(a).total_cmp(&distance(b))
            })
            .map_or(0, |c| c.index)
    }
    fn caret(&self, index: usize) -> Option<&Caret> {
        self.carets.iter().min_by_key(|c| c.index.abs_diff(index))
    }
    pub fn paint(&self, ui: &Ui, at: Pos2, editor: Option<&Editor>, highlight: Option<Range<usize>>, accent: bool) {
        let p = ui.painter();
        let t = Tokens::get();
        for g in &self.glyphs {
            let selected = editor.is_some_and(|e| ranges_intersect(e.selection(), g.span.clone()));
            if selected || highlight.as_ref().is_some_and(|r| ranges_intersect(r.clone(), g.span.clone())) {
                p.rect_filled(Rect::from_min_size(at + g.at.to_vec2(), g.extent).expand(1.), 2., t.accent_soft);
            }
            p.text(at + g.at.to_vec2(), Align2::LEFT_TOP, &g.text, FontId::proportional(g.size), if accent { t.accent } else { t.text });
        }
        for line in &self.lines {
            p.line_segment([at + line[0].to_vec2(), at + line[1].to_vec2()], Stroke::new(1.2, t.text));
        }
        if let Some(editor) = editor
            && let Some(caret) = self.caret(editor.cursor)
        {
            let x = at + caret.at.to_vec2();
            if ui.input(|i| i.time).rem_euclid(1.) < 0.65 {
                p.line_segment([x, x + vec2(0., caret.height)], Stroke::new(1.4, t.accent));
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
    pub fn caret_handles(&self, id: &str, at: Pos2) {
        for caret in &self.carets {
            crate::scenario::publish_handle(&format!("worksheet:caret:{id}:{}", caret.index), at + caret.at.to_vec2() + vec2(0., caret.height * 0.5));
        }
    }
}
fn ranges_intersect(a: Range<usize>, b: Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}
fn plain(ui: &Ui, text: &str, offset: usize, size: f32) -> MathBox {
    let mut out = MathBox { baseline: size * 0.84, ..Default::default() };
    let mut x = 0.;
    for (i, c) in text.char_indices() {
        let value = if c == '*' { "·".to_owned() } else { c.to_string() };
        let extent = ui.painter().layout_no_wrap(value.clone(), FontId::proportional(size), Tokens::get().text).size();
        out.carets.push(Caret { index: offset + i, at: pos2(x, 0.), height: extent.y });
        out.glyphs.push(Glyph { text: value, at: pos2(x, 0.), size, span: offset + i..offset + i + c.len_utf8(), extent });
        x += extent.x;
        out.size.y = out.size.y.max(extent.y);
    }
    out.carets.push(Caret { index: offset + text.len(), at: pos2(x, 0.), height: size * 1.2 });
    out.size.x = x.max(2.);
    out.size.y = out.size.y.max(size * 1.2);
    out
}
fn horizontal(items: Vec<MathBox>, gap: f32) -> MathBox {
    let baseline = items.iter().map(|x| x.baseline).fold(0., f32::max);
    let below = items.iter().map(|x| x.size.y - x.baseline).fold(0., f32::max);
    let mut out = MathBox { size: vec2(0., baseline + below), baseline, ..Default::default() };
    for item in items {
        let width = item.size.x;
        let y = baseline - item.baseline;
        out.append(item.shifted(vec2(out.size.x, y)));
        out.size.x += width + gap;
    }
    out.size.x = (out.size.x - gap).max(2.);
    out
}
fn operator(text: &str, allowed: &[char], first: bool) -> Option<usize> {
    let mut level = 0i32;
    let mut found = None;
    for (i, c) in text.char_indices() {
        if c == '(' || c == '[' {
            level += 1;
        } else if c == ')' || c == ']' {
            level -= 1;
        }
        if level == 0 && i > 0 && allowed.contains(&c) {
            let before = text[..i].trim_end().chars().next_back();
            let numeric_exponent = matches!(before, Some('e' | 'E'))
                && text[..i].chars().any(|c| c.is_ascii_digit())
                && text[..i].chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E'));
            if !matches!(before, Some('+' | '-' | '*' | '/' | '^' | '(' | ',' | ':')) && !numeric_exponent {
                found = Some(i);
                if first {
                    break;
                }
            }
        }
    }
    found
}
fn outer_group(text: &str) -> bool {
    if !text.starts_with('(') || !text.ends_with(')') {
        return false;
    }
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        if c == '(' {
            depth += 1;
        }
        if c == ')' {
            depth -= 1;
            if depth == 0 && i + 1 < text.len() {
                return false;
            }
        }
    }
    depth == 0
}
fn build(ui: &Ui, text: &str, offset: usize, size: f32, depth: usize, strip: bool) -> MathBox {
    let leading = text.len() - text.trim_start().len();
    let offset = offset + leading;
    let text = text.trim();
    if depth > 24 || text.len() > 4096 {
        return plain(ui, text, offset, size);
    }
    if text.is_empty() {
        return plain(ui, "", offset, size);
    }
    if outer_group(text) {
        let inner = build(ui, &text[1..text.len() - 1], offset + 1, size, depth + 1, false);
        if strip {
            let mut inner = inner;
            inner.carets.push(Caret { index: offset, at: Pos2::ZERO, height: size * 1.2 });
            inner.carets.push(Caret { index: offset + text.len(), at: pos2(inner.size.x, 0.), height: size * 1.2 });
            return inner;
        }
        return horizontal(vec![plain(ui, "(", offset, size), inner, plain(ui, ")", offset + text.len() - 1, size)], 1.);
    }
    if let Some(i) = text.find(":=") {
        return horizontal(
            vec![
                build(ui, &text[..i], offset, size, depth + 1, false),
                plain(ui, ":=", offset + i, size),
                build(ui, &text[i + 2..], offset + i + 2, size, depth + 1, false),
            ],
            7.,
        );
    }
    if let Some(i) = text.find('=') {
        return horizontal(
            vec![
                build(ui, &text[..i], offset, size, depth + 1, false),
                plain(ui, "=", offset + i, size),
                build(ui, &text[i + 1..], offset + i + 1, size, depth + 1, false),
            ],
            7.,
        );
    }
    if let Some(i) = operator(text, &['+', '-'], false) {
        return horizontal(
            vec![
                build(ui, &text[..i], offset, size, depth + 1, false),
                plain(ui, &text[i..i + 1], offset + i, size),
                build(ui, &text[i + 1..], offset + i + 1, size, depth + 1, false),
            ],
            5.,
        );
    }
    if let Some(i) = operator(text, &['*', '/'], false) {
        if text.as_bytes().get(i) == Some(&b'*') {
            return horizontal(
                vec![
                    build(ui, &text[..i], offset, size, depth + 1, false),
                    plain(ui, "*", offset + i, size),
                    build(ui, &text[i + 1..], offset + i + 1, size, depth + 1, false),
                ],
                4.,
            );
        }
        let numerator = build(ui, &text[..i], offset, size * 0.94, depth + 1, true);
        let denominator = build(ui, &text[i + 1..], offset + i + 1, size * 0.94, depth + 1, true);
        let width = numerator.size.x.max(denominator.size.x) + 12.;
        let bar = numerator.size.y + 4.;
        let mut out = MathBox { size: vec2(width, bar + 6. + denominator.size.y), baseline: bar + size * 0.3, ..Default::default() };
        let nx = (width - numerator.size.x) * 0.5;
        let dx = (width - denominator.size.x) * 0.5;
        out.append(numerator.shifted(vec2(nx, 0.)));
        out.append(denominator.shifted(vec2(dx, bar + 6.)));
        out.lines.push([pos2(0., bar), pos2(width, bar)]);
        out.carets.push(Caret { index: offset + i, at: pos2(width * 0.5, bar - 3.), height: 6. });
        out
    } else if let Some(i) = operator(text, &['^'], true) {
        let base = build(ui, &text[..i], offset, size, depth + 1, false);
        let exponent = build(ui, &text[i + 1..], offset + i + 1, size * 0.68, depth + 1, true);
        let down = exponent.size.y * 0.65;
        let x = base.size.x;
        let mut out = MathBox {
            size: vec2(x + exponent.size.x, (base.size.y + down).max(exponent.size.y)),
            baseline: base.baseline + down,
            ..Default::default()
        };
        out.append(base.shifted(vec2(0., down)));
        out.append(exponent.shifted(vec2(x, 0.)));
        out.carets.push(Caret { index: offset + i, at: pos2(x, down), height: size });
        out
    } else if let Some(i) = text.find('_')
        && !text.contains(['(', ')', ' '])
    {
        let base = plain(ui, &text[..i], offset, size);
        let sub = plain(ui, &text[i + 1..], offset + i + 1, size * 0.68);
        let x = base.size.x;
        let down = size * 0.46;
        let mut out = MathBox { size: vec2(x + sub.size.x, base.size.y.max(sub.size.y + down)), baseline: base.baseline, ..Default::default() };
        out.append(base);
        out.append(sub.shifted(vec2(x, down)));
        out.carets.push(Caret { index: offset + i, at: pos2(x, 0.), height: size });
        out
    } else if let Some(i) = text.find('(')
        && i > 0
        && outer_group(&text[i..])
    {
        horizontal(vec![plain(ui, &text[..i], offset, size), build(ui, &text[i..], offset + i, size, depth + 1, false)], 1.)
    } else {
        plain(ui, text, offset, size)
    }
}
pub fn layout(ui: &Ui, text: &str, size: f32) -> MathBox {
    build(ui, text, 0, size, 0, false)
}
pub fn field(
    ui: &mut Ui,
    id: &str,
    text: &str,
    at: Pos2,
    size: f32,
    editor: Option<&Editor>,
    highlight: Option<Range<usize>>,
    accent: bool,
) -> (MathBox, egui::Response) {
    let math = layout(ui, text, size);
    let rect = Rect::from_min_size(at, math.size.max(vec2(12., size * 1.2))).expand(4.);
    let response = ui.interact(rect, egui::Id::new(("worksheet_math", id)), Sense::click_and_drag()).on_hover_cursor(egui::CursorIcon::Text);
    math.paint(ui, at, editor, highlight, accent);
    if editor.is_some() {
        math.caret_handles(id, at);
    }
    (math, response)
}
#[derive(Clone, Default)]
pub struct Editor {
    undo: Vec<(String, usize, usize)>,
    redo: Vec<(String, usize, usize)>,
    pub text: String,
    pub cursor: usize,
    pub anchor: usize,
}
pub fn owns_keyboard(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<egui::Id>(egui::Id::new("worksheet_math_keyboard"))).is_some_and(|id| ctx.memory(|m| m.has_focus(id)))
}
#[derive(Default)]
pub struct Input {
    pub changed: bool,
    pub evaluate: bool,
    pub finish: bool,
    pub cancel: bool,
}
impl Editor {
    pub fn new(text: String, index: usize) -> Self {
        let index = index.min(text.len());
        let index = floor_boundary(&text, index);
        Self { text, cursor: index, anchor: index, ..Self::default() }
    }
    pub fn insert_text(&mut self, text: &str) {
        self.replace(text);
    }
    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }
    fn history(&mut self, forward: bool) -> bool {
        let saved = if forward { self.redo.pop() } else { self.undo.pop() };
        let Some((text, cursor, anchor)) = saved else { return false };
        let current = (self.text.clone(), self.cursor, self.anchor);
        if forward {
            self.undo.push(current);
        } else {
            self.redo.push(current);
        }
        self.text = text;
        self.cursor = cursor;
        self.anchor = anchor;
        true
    }
    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
    }
    fn replace(&mut self, text: &str) {
        let range = self.selection();
        if self.text.len() - range.len() + text.len() > 4096 {
            return;
        }
        if self.undo.len() >= 100 {
            self.undo.remove(0);
        }
        self.undo.push((self.text.clone(), self.cursor, self.anchor));
        self.redo.clear();
        self.text.replace_range(range.clone(), text);
        self.cursor = range.start + text.len();
        self.anchor = self.cursor;
    }
    pub fn mouse(&mut self, math: &MathBox, point: Pos2, extend: bool, double: bool) {
        self.cursor = floor_boundary(&self.text, math.nearest(point).min(self.text.len()));
        if !extend {
            self.anchor = self.cursor;
        }
        if double {
            let range = identifier_range(&self.text, self.cursor);
            self.anchor = range.start;
            self.cursor = range.end;
        }
    }
    pub fn input(&mut self, ui: &mut Ui, id: egui::Id, math: &MathBox) -> Input {
        if !ui.memory(|m| m.has_focus(id)) {
            return Input::default();
        }
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("worksheet_math_keyboard"), id));
        ui.memory_mut(|m| m.set_focus_lock_filter(id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true }));
        let events = ui.input_mut(|input| {
            let mut events = Vec::new();
            input.events.retain(|e| {
                if matches!(e, egui::Event::Text(_) | egui::Event::Paste(_) | egui::Event::Copy | egui::Event::Cut | egui::Event::Key { .. }) {
                    events.push(e.clone());
                    false
                } else {
                    true
                }
            });
            events
        });
        let mut out = Input::default();
        for event in events {
            match event {
                egui::Event::Text(text) if text == "=" && !self.text[..self.cursor].ends_with(':') => {
                    out.evaluate = true;
                    break;
                }
                egui::Event::Text(text) | egui::Event::Paste(text) => {
                    self.replace(&text.replace(['\r', '\n'], " "));
                    out.changed = true;
                }
                egui::Event::Copy | egui::Event::Cut => {
                    let range = self.selection();
                    if !range.is_empty() {
                        ui.ctx().copy_text(self.text[range].to_owned());
                    }
                    if matches!(event, egui::Event::Cut) {
                        self.replace("");
                        out.changed = true;
                    }
                }
                egui::Event::Key { key, pressed: true, modifiers, .. } => match key {
                    egui::Key::Z if modifiers.command => {
                        out.changed |= self.history(modifiers.shift);
                    }
                    egui::Key::Y if modifiers.command => {
                        out.changed |= self.history(true);
                    }
                    egui::Key::A if modifiers.command => self.select_all(),
                    egui::Key::Enter | egui::Key::Tab => {
                        out.finish = true;
                        break;
                    }
                    egui::Key::Escape => {
                        out.cancel = true;
                        break;
                    }
                    egui::Key::Backspace => {
                        if self.selection().is_empty() {
                            self.anchor = previous(&self.text, self.cursor);
                        }
                        self.replace("");
                        out.changed = true;
                    }
                    egui::Key::Delete => {
                        if self.selection().is_empty() {
                            self.anchor = next(&self.text, self.cursor);
                        }
                        self.replace("");
                        out.changed = true;
                    }
                    egui::Key::ArrowLeft | egui::Key::ArrowRight | egui::Key::Home | egui::Key::End => {
                        self.cursor = match key {
                            egui::Key::Home => 0,
                            egui::Key::End => self.text.len(),
                            egui::Key::ArrowLeft => previous(&self.text, self.cursor),
                            _ => next(&self.text, self.cursor),
                        };
                        if !modifiers.shift {
                            self.anchor = self.cursor;
                        }
                    }
                    egui::Key::ArrowUp | egui::Key::ArrowDown => {
                        if let Some(current) = math.caret(self.cursor) {
                            let direction = if key == egui::Key::ArrowUp { -1. } else { 1. };
                            if let Some(target) = math.carets.iter().filter(|c| (c.at.y - current.at.y) * direction > 4.).min_by(|a, b| {
                                let distance = |c: &Caret| (c.at.x - current.at.x).abs() + (c.at.y - current.at.y).abs() * 2.;
                                distance(a).total_cmp(&distance(b))
                            }) {
                                self.cursor = floor_boundary(&self.text, target.index.min(self.text.len()));
                                if !modifiers.shift {
                                    self.anchor = self.cursor;
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if out.changed {
            ui.ctx().request_repaint();
        }
        out
    }
}
fn floor_boundary(text: &str, mut index: usize) -> usize {
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
fn previous(text: &str, index: usize) -> usize {
    text[..index].char_indices().next_back().map_or(0, |(i, _)| i)
}
fn next(text: &str, index: usize) -> usize {
    text[index..].chars().next().map_or(index, |c| index + c.len_utf8())
}
pub fn identifier_range(text: &str, index: usize) -> Range<usize> {
    let mut start = floor_boundary(text, index.min(text.len()));
    let mut end = start;
    while start > 0 {
        let i = previous(text, start);
        if text[i..start].chars().all(|c| c.is_alphanumeric() || c == '_') {
            start = i;
        } else {
            break;
        }
    }
    while end < text.len() {
        let i = next(text, end);
        if text[end..i].chars().all(|c| c.is_alphanumeric() || c == '_') {
            end = i;
        } else {
            break;
        }
    }
    start..end
}
pub fn number(value: f64, digits: usize) -> String {
    if value == 0. {
        return "0".into();
    }
    let digits = digits.clamp(2, 16);
    let exponent = value.abs().log10().floor() as i32;
    let trim = |s: String| {
        if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_owned() } else { s }
    };
    if exponent >= digits as i32 || exponent < -3 {
        let scientific = format!("{:.*e}", digits - 1, value);
        let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
        format!("{}*10^({exponent})", trim(mantissa.into()))
    } else {
        trim(format!("{:.*}", (digits as i32 - 1 - exponent).max(0) as usize, value))
    }
}
