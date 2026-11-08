//! glyph text layout

/// glyph style metrics
pub struct GlyphStyle {
    /// first sprite id
    pub first: u16,
    /// supported style characters
    pub chars: &'static str,
    /// ascii glyph lookup
    pub ascii: &'static [u8; 96],
    /// glyph pen advance
    pub advance: &'static [u16],
    /// glyph x offset
    pub x_offset: &'static [i8],
    /// cell ascent offset
    pub ascent: u8,
    pub height: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub sprite: u16,
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Centre,
    Right,
}

impl GlyphStyle {
    /// find glyph index
    pub fn index(&self, c: char) -> Option<usize> {
        if (' '..='\u{7f}').contains(&c) {
            let i = self.ascii[c as usize - 0x20];
            if i != 255 {
                return Some(i as usize);
            }
        }
        let find = |c: char| self.chars.chars().position(|x| x == c);
        find(c).or_else(|| match c {
            '\u{2019}' | '\u{2018}' | '`' => find('\''),
            '\u{201C}' | '\u{201D}' => find('"'),
            '\u{2013}' | '\u{2014}' | '\u{2212}' => find('-'),
            '\u{00A0}' | '\t' | '\n' => find(' '),
            _ => None,
        })
    }

    fn advance_of(&self, c: char) -> u32 {
        self.index(c).map_or(0, |i| self.advance[i] as u32)
    }

    /// string pixel width
    pub fn width(&self, s: &str) -> i16 {
        ((s.chars().map(|c| self.advance_of(c)).sum::<u32>() + 8) / 16) as i16
    }

    /// place string glyphs
    pub fn place(
        &self,
        s: &str,
        x: i16,
        y: i16,
        align: Align,
        mut emit: impl FnMut(Placement),
    ) -> i16 {
        let w = self.width(s);
        let x0 = match align {
            Align::Left => x,
            Align::Centre => x - w / 2,
            Align::Right => x - w,
        };
        let top = y - self.ascent as i16;
        let mut pen16: i32 = x0 as i32 * 16;
        for c in s.chars() {
            if let Some(i) = self.index(c) {
                if c != ' ' {
                    emit(Placement {
                        sprite: self.first + i as u16,
                        x: ((pen16 + 8) / 16) as i16 + self.x_offset[i] as i16,
                        y: top,
                    });
                }
                pen16 += self.advance[i] as i32;
            }
        }
        w
    }

    /// word wrap lines
    pub fn wrap<'a>(
        &self,
        s: &'a str,
        width: i16,
        max_lines: usize,
        mut line: impl FnMut(usize, &'a str, bool),
    ) {
        let mut rest = s.trim();
        let mut n = 0;
        while !rest.is_empty() && n < max_lines {
            let last = n + 1 == max_lines;
            let mut end = self.fit(rest, width);
            let mut truncated = false;
            if last && !rest[end..].trim().is_empty() {
                // ellipsis room margin
                end = self.fit(rest, width - self.width("\u{2026}"));
                truncated = true;
            }
            line(n, rest[..end].trim_end(), truncated);
            rest = rest[end..].trim_start();
            n += 1;
        }
    }

    /// fit line prefix
    fn fit(&self, rest: &str, width: i16) -> usize {
        let mut end = 0;
        for (bi, c) in rest
            .char_indices()
            .chain(core::iter::once((rest.len(), ' ')))
        {
            if c != ' ' && c != '\n' {
                continue;
            }
            if self.width(&rest[..bi]) > width {
                break;
            }
            end = bi;
            if c == '\n' {
                return end;
            }
        }
        if end == 0 {
            // hard word break
            end = rest
                .char_indices()
                .take_while(|&(bi, c)| self.width(&rest[..bi + c.len_utf8()]) <= width)
                .last()
                .map_or(rest.chars().next().map_or(0, char::len_utf8), |(bi, c)| {
                    bi + c.len_utf8()
                });
        }
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    // test mono style
    static ADV: [u16; 96] = [160; 96];
    static OFF: [i8; 96] = [0; 96];
    const CHARS: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{2026}";
    static NO_ASCII: [u8; 96] = [255; 96];
    static MONO: GlyphStyle = GlyphStyle {
        first: 100,
        chars: CHARS,
        ascii: &NO_ASCII,
        advance: &ADV,
        x_offset: &OFF,
        ascent: 20,
        height: 28,
    };

    #[test]
    fn places_glyphs_on_the_baseline() {
        let mut got = Vec::new();
        let w = MONO.place("Hi 7", 10, 50, Align::Left, |p| got.push(p));
        assert_eq!(w, 40);
        assert_eq!(got.len(), 3); // space lacks sprite
        assert_eq!(
            got[0],
            Placement {
                sprite: 100 + 40,
                x: 10,
                y: 30
            }
        );
        assert_eq!(got[2].x, 40);
        let mut got = Vec::new();
        MONO.place("ab", 100, 50, Align::Right, |p| got.push(p));
        assert_eq!(got[0].x, 80);
    }

    #[test]
    fn folds_typographic_quotes() {
        assert_eq!(MONO.index('\u{2019}'), MONO.index('\''));
        assert_eq!(MONO.index('\u{2014}'), MONO.index('-'));
        assert_eq!(MONO.index('é'), None);
    }

    #[test]
    fn wraps_words_and_truncates() {
        let mut lines = Vec::new();
        MONO.wrap(
            "Everything looks comfortable and the air is fresh",
            120,
            3,
            |i, s, t| lines.push((i, s, t)),
        );
        assert_eq!(lines[0], (0, "Everything", false));
        assert_eq!(lines[1], (1, "looks", false));
        assert_eq!(lines[2], (2, "comfortable", true));
        let mut lines = Vec::new();
        MONO.wrap("short one", 200, 3, |i, s, t| lines.push((i, s, t)));
        assert_eq!(lines, [(0, "short one", false)]);
    }

    #[test]
    fn hard_breaks_long_words() {
        let mut lines = Vec::new();
        MONO.wrap("abcdefghijklmnop", 50, 4, |_, s, _| lines.push(s));
        assert_eq!(lines, ["abcde", "fghij", "klmno", "p"]);
    }
}
