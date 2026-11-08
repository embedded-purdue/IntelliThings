//! ui frame controller

use embedded_io_async::Write;

use crate::canvas::{self, Canvas};
use crate::dgus::{Link, VpImage};
use crate::generated;
use crate::model::Model;
use crate::screens::{self, Extras, Screen};

pub type Image = VpImage<{ generated::VP_WORDS }>;

// signature vp bounds
const _: () = {
    let end = generated::VP_BASE as usize + generated::VP_WORDS;
    assert!(generated::SESSION_VP as usize >= end && generated::SIGNATURE_VP as usize >= end);
};

/// frame statistics summary
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameStats {
    pub bytes: usize,
    pub dropped: u16,
    pub page_switched: bool,
}

pub struct Ui {
    img: Image,
    canvas: Canvas,
    page: Option<u16>,
    backlight: Option<u8>,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub const fn new() -> Self {
        Self {
            img: VpImage::new(generated::VP_BASE),
            canvas: Canvas::new(),
            page: None,
            backlight: None,
        }
    }

    /// invalidate panel state
    pub fn invalidate(&mut self) {
        self.img.invalidate();
        self.page = None;
        self.backlight = None;
    }

    pub fn page(&self) -> Option<u16> {
        self.page
    }

    /// screen backlight level
    pub fn backlight_for(screen: Screen) -> u8 {
        match screen {
            Screen::Night => 30,
            Screen::Away => 40,
            _ => 80,
        }
    }

    /// render single frame
    pub async fn frame<W: Write>(
        &mut self,
        link: &mut Link<W>,
        screen: Screen,
        model: &Model,
        now_ms: u64,
    ) -> Result<FrameStats, W::Error> {
        let mut stats = FrameStats::default();
        let page = screen.page();
        if self.page != Some(page) {
            // clear shared layers
            canvas::clear_layers(&mut self.img);
            stats.bytes += self.img.flush(link).await?;
            link.set_page(page).await?;
            self.page = Some(page);
            stats.page_switched = true;
        }
        let level = Self::backlight_for(screen);
        if self.backlight != Some(level) {
            link.set_backlight(level).await?;
            self.backlight = Some(level);
        }
        self.canvas.clear();
        screens::render(
            screen,
            &mut self.canvas,
            &mut Extras { img: &mut self.img },
            model,
            now_ms,
        );
        self.canvas.encode(&mut self.img);
        stats.dropped = self.canvas.dropped;
        stats.bytes += self.img.flush(link).await?;
        Ok(stats)
    }

    /// test render image
    pub fn render_only(&mut self, screen: Screen, model: &Model, now_ms: u64) -> (&Image, u16) {
        canvas::clear_layers(&mut self.img);
        self.canvas.clear();
        screens::render(
            screen,
            &mut self.canvas,
            &mut Extras { img: &mut self.img },
            model,
            now_ms,
        );
        self.canvas.encode(&mut self.img);
        (&self.img, self.canvas.dropped)
    }
}
