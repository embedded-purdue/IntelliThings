//! sprite canvas encoder

use crate::dgus::VpImage;
use crate::generated::{self, Part};
use crate::text::{Align, GlyphStyle, Placement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// skin layer sprites
    Skin,
    /// glyph and icon layer
    Main,
    /// overlay toast layer
    Over,
}

pub const W: i16 = 800;
pub const H: i16 = 480;

pub struct Canvas {
    skin: heapless::Vec<Placement, 64>,
    main: heapless::Vec<Placement, 640>,
    over: heapless::Vec<Placement, 192>,
    /// dropped sprite count
    pub dropped: u16,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub const fn new() -> Self {
        Self {
            skin: heapless::Vec::new(),
            main: heapless::Vec::new(),
            over: heapless::Vec::new(),
            dropped: 0,
        }
    }

    pub fn clear(&mut self) {
        self.skin.clear();
        self.main.clear();
        self.over.clear();
        self.dropped = 0;
    }

    pub fn sprite(&mut self, layer: Layer, sprite: u16, x: i16, y: i16) {
        // sprite bounds check
        if !(0..W).contains(&x) || !(0..H).contains(&y) {
            self.dropped += 1;
            return;
        }
        let p = Placement { sprite, x, y };
        let ok = match layer {
            Layer::Skin => self.skin.push(p).is_ok(),
            Layer::Main => self.main.push(p).is_ok(),
            Layer::Over => self.over.push(p).is_ok(),
        };
        if !ok {
            self.dropped += 1;
        }
    }

    /// centered sprite placement
    pub fn sprite_at(&mut self, layer: Layer, sprite: u16, cx: i16, cy: i16) {
        let (w, h) = size(sprite);
        self.sprite(layer, sprite, cx - (w as i16) / 2, cy - (h as i16) / 2);
    }

    /// render text string
    pub fn text(
        &mut self,
        layer: Layer,
        style: &GlyphStyle,
        s: &str,
        x: i16,
        y: i16,
        align: Align,
    ) -> i16 {
        let mut placed: heapless::Vec<Placement, 160> = heapless::Vec::new();
        let w = style.place(s, x, y, align, |p| {
            let _ = placed.push(p);
        });
        for p in placed {
            self.sprite(layer, p.sprite, p.x, p.y);
        }
        w
    }

    /// encode canvas layers
    pub fn encode<const N: usize>(&mut self, img: &mut VpImage<N>) {
        let mut dropped = 0;
        dropped += encode_layer(&self.skin, &generated::SKIN, img);
        dropped += encode_layer(&self.main, &generated::MAIN, img);
        dropped += encode_layer(&self.over, &generated::OVER, img);
        self.dropped += dropped;
    }
}

pub fn size(sprite: u16) -> (u16, u16) {
    generated::SPRITE_SIZE
        .get(sprite as usize)
        .copied()
        .unwrap_or((0, 0))
}

/// encode layer parts
fn encode_layer<const N: usize>(items: &[Placement], parts: &[Part], img: &mut VpImage<N>) -> u16 {
    let mut used = [false; 640];
    for part in parts {
        let mut n = 0u16;
        for (k, p) in items.iter().enumerate() {
            if n == part.capacity {
                break;
            }
            if used[k] || !(part.first..part.end).contains(&p.sprite) {
                continue;
            }
            used[k] = true;
            img.set_slice(
                part.vp + 2 + 3 * n,
                &[p.x as u16, p.y as u16, p.sprite - part.first],
            );
            n += 1;
        }
        img.set(part.vp, ((part.icl as u16) << 8) | 0x07);
        img.set(part.vp + 1, n);
        img.set(part.vp + 2 + 3 * n, 0xFF00);
    }
    used[..items.len()].iter().filter(|u| !**u).count() as u16
}

/// clear all layers
pub fn clear_layers<const N: usize>(img: &mut VpImage<N>) {
    for part in generated::SKIN
        .iter()
        .chain(generated::MAIN.iter())
        .chain(generated::OVER.iter())
    {
        img.set(part.vp, ((part.icl as u16) << 8) | 0x07);
        img.set(part.vp + 1, 0);
        img.set(part.vp + 2, 0xFF00);
    }
}
