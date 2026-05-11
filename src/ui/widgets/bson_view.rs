use iced::{widget::{column, container, row, text}, Color, Element, Length, Padding};
use crate::{data::model::{BsonDoc, BsonVal}, theme::Palette};

/// Regex-like check for BSON call types (ObjectId, ISODate, etc.)
fn is_call(s: &str) -> bool {
    s.starts_with("ObjectId")
        || s.starts_with("ISODate")
        || s.starts_with("NumberDecimal")
        || s.starts_with("NumberLong")
        || s.starts_with("Timestamp")
        || s.starts_with("$$NOW")
        || s.starts_with("UUID")
}

fn needs_quotes(key: &str) -> bool {
    key.starts_with('$') || key.contains('.')
}

struct Ctx<'a, Msg> {
    palette: &'a Palette,
    fs: f32,
    _phantom: std::marker::PhantomData<Msg>,
}

impl<'a, Msg: 'static> Ctx<'a, Msg> {
    fn tok(&self, s: impl Into<String>, color: Color) -> Element<'static, Msg> {
        text(s.into()).size(self.fs).color(color).font(iced::Font::MONOSPACE).into()
    }

    fn line(&self, depth: usize, children: Vec<Element<'static, Msg>>) -> Element<'static, Msg> {
        let indent = (8.0 + depth as f32 * 14.0) as u16;
        container(row(children).spacing(0))
            .padding(Padding { left: indent as f32, top: 1.0, bottom: 1.0, right: 4.0 })
            .width(Length::Fill)
            .into()
    }

    fn render_val(&self, val: &BsonVal, depth: usize, lines: &mut Vec<Element<'static, Msg>>) {
        match val {
            BsonVal::Null => lines.push(self.line(depth, vec![
                self.tok("null", self.palette.tok_lit)
            ])),
            BsonVal::Bool(b) => lines.push(self.line(depth, vec![
                self.tok(b.to_string(), self.palette.tok_lit)
            ])),
            BsonVal::Int(n) => lines.push(self.line(depth, vec![
                self.tok(n.to_string(), self.palette.tok_num)
            ])),
            BsonVal::Float(f) => lines.push(self.line(depth, vec![
                self.tok(format!("{f}"), self.palette.tok_num)
            ])),
            BsonVal::NumberLong(n) => lines.push(self.line(depth, vec![
                self.tok(format!("NumberLong({n})"), self.palette.tok_call)
            ])),
            BsonVal::ObjectId(s) => lines.push(self.line(depth, vec![
                self.tok(s.clone(), self.palette.tok_call)
            ])),
            BsonVal::IsoDate(s) => lines.push(self.line(depth, vec![
                self.tok(s.clone(), self.palette.tok_call)
            ])),
            BsonVal::Timestamp(s) => lines.push(self.line(depth, vec![
                self.tok(s.clone(), self.palette.tok_call)
            ])),
            BsonVal::Str(s) => {
                let (color, rendered) = if is_call(s) {
                    (self.palette.tok_call, s.clone())
                } else {
                    (self.palette.tok_str, format!("\"{}\"", s))
                };
                lines.push(self.line(depth, vec![self.tok(rendered, color)]));
            }
            BsonVal::Array(arr) => {
                lines.push(self.line(depth, vec![self.tok("[", self.palette.tok_br)]));
                for v in arr { self.render_val(v, depth + 1, lines); }
                lines.push(self.line(depth, vec![self.tok("]", self.palette.tok_br)]));
            }
            BsonVal::Doc(doc) => {
                self.render_doc(doc, depth, lines);
            }
        }
    }

    fn render_kv(&self, key: &str, val: &BsonVal, depth: usize, comma: bool, lines: &mut Vec<Element<'static, Msg>>) {
        let key_str = if needs_quotes(key) { format!("\"{}\"", key) } else { key.to_string() };
        let key_tok = self.tok(key_str, self.palette.tok_key);
        let colon_tok = self.tok(": ", self.palette.tok_colon);
        let comma_tok: Option<Element<'static, Msg>> =
            if comma { Some(self.tok(",", self.palette.tok_colon)) } else { None };

        match val {
            BsonVal::Doc(doc) => {
                // Key: { on its own line
                let opening: Vec<Element<'static, Msg>> = vec![key_tok, colon_tok, self.tok("{", self.palette.tok_br)];
                lines.push(self.line(depth, opening));
                let keys: Vec<_> = doc.keys().cloned().collect();
                for (i, k) in keys.iter().enumerate() {
                    self.render_kv(k, &doc[k], depth + 1, i < keys.len() - 1, lines);
                }
                let mut closing: Vec<Element<'static, Msg>> = vec![self.tok("}", self.palette.tok_br)];
                if let Some(c) = comma_tok { closing.push(c); }
                lines.push(self.line(depth, closing));
            }
            BsonVal::Array(arr) if arr.is_empty() => {
                let mut parts: Vec<Element<'static, Msg>> = vec![key_tok, colon_tok, self.tok("[ ]", self.palette.tok_br)];
                if let Some(c) = comma_tok { parts.push(c); }
                lines.push(self.line(depth, parts));
            }
            _ => {
                // Inline value
                let mut parts: Vec<Element<'static, Msg>> = vec![key_tok, colon_tok];
                self.inline_val(val, &mut parts);
                if let Some(c) = comma_tok { parts.push(c); }
                lines.push(self.line(depth, parts));
            }
        }
    }

    fn inline_val(&self, val: &BsonVal, out: &mut Vec<Element<'static, Msg>>) {
        match val {
            BsonVal::Null => out.push(self.tok("null", self.palette.tok_lit)),
            BsonVal::Bool(b) => out.push(self.tok(b.to_string(), self.palette.tok_lit)),
            BsonVal::Int(n) => out.push(self.tok(n.to_string(), self.palette.tok_num)),
            BsonVal::Float(f) => out.push(self.tok(format!("{f}"), self.palette.tok_num)),
            BsonVal::NumberLong(n) => out.push(self.tok(format!("NumberLong({n})"), self.palette.tok_call)),
            BsonVal::ObjectId(s) | BsonVal::IsoDate(s) | BsonVal::Timestamp(s) => {
                out.push(self.tok(s.clone(), self.palette.tok_call));
            }
            BsonVal::Str(s) => {
                let (color, rendered) = if is_call(s) {
                    (self.palette.tok_call, s.clone())
                } else {
                    (self.palette.tok_str, format!("\"{}\"", s))
                };
                out.push(self.tok(rendered, color));
            }
            BsonVal::Array(arr) => {
                out.push(self.tok("[", self.palette.tok_br));
                for (i, v) in arr.iter().enumerate() {
                    self.inline_val(v, out);
                    if i < arr.len() - 1 { out.push(self.tok(", ", self.palette.tok_colon)); }
                }
                out.push(self.tok("]", self.palette.tok_br));
            }
            BsonVal::Doc(_) => out.push(self.tok("{…}", self.palette.tok_br)),
        }
    }

    fn render_doc(&self, doc: &BsonDoc, depth: usize, lines: &mut Vec<Element<'static, Msg>>) {
        lines.push(self.line(depth, vec![self.tok("{", self.palette.tok_br)]));
        let keys: Vec<_> = doc.keys().cloned().collect();
        for (i, k) in keys.iter().enumerate() {
            self.render_kv(k, &doc[k], depth + 1, i < keys.len() - 1, lines);
        }
        lines.push(self.line(depth, vec![self.tok("}", self.palette.tok_br)]));
    }
}

pub fn bson_view<Msg: 'static>(doc: &BsonDoc, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let ctx: Ctx<Msg> = Ctx { palette, fs, _phantom: std::marker::PhantomData };
    let mut lines: Vec<Element<'static, Msg>> = Vec::new();
    ctx.render_doc(doc, 0, &mut lines);
    column(lines).spacing(0).into()
}
