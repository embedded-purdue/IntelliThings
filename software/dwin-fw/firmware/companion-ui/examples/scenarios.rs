//! scenario frame exporter

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use companion_ui::demo;
use companion_ui::director::{self, PatternClock};
use companion_ui::generated;
use companion_ui::lightbar::{self, Rgb};
use companion_ui::ui::Ui;

fn dump(ui: &mut Ui, out: &Path, file: &str, name: &str, t_ms: u64) {
    let now = 3_600_000 + t_ms; // offset past boot
    let (screen, model) = demo::scenario(name, t_ms, now).expect("unknown scenario");
    let (img, dropped) = ui.render_only(screen, &model, now);
    assert_eq!(
        dropped, 0,
        "{name} at {t_ms} ms: {dropped} sprites didn't fit"
    );
    let mut json = String::new();
    let _ = write!(
        json,
        "{{\"page\": {}, \"base\": {}, \"words\": [",
        screen.page(),
        generated::VP_BASE
    );
    for (i, w) in img.words().iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let _ = write!(json, "{w}");
    }
    json.push_str("]}\n");
    fs::write(out.join(file), json).unwrap();
}

/// export demo frames
fn demo_frames(ui: &mut Ui, out: &Path, fps: u64) {
    let mut clock = PatternClock::default();
    let total = demo::loop_ms() * fps / 1000;
    for k in 0..total {
        let ms = k * 1000 / fps;
        let now = 3_600_000 + ms;
        let (name, t) = demo::at(ms);
        let (screen, model) = demo::scenario(name, t, now).unwrap();
        let greeting = (name == "away")
            .then_some(now - t + 10_500)
            .filter(|g| *g <= now);
        let (pattern, fixed) = director::pattern_for(screen, &model, now, greeting, 120_000);
        let since = clock.update(pattern, fixed, now);
        let mut leds = [Rgb::default(); 24];
        lightbar::render(&pattern, since, 1.0, &mut leds);
        let (img, dropped) = ui.render_only(screen, &model, now);
        assert_eq!(dropped, 0, "{name} at {t} ms: {dropped} sprites didn't fit");
        let mut bytes = Vec::with_capacity(2 + 1 + name.len() + 72 + 2 * img.words().len());
        bytes.extend_from_slice(&screen.page().to_le_bytes());
        bytes.push(name.len() as u8);
        bytes.extend_from_slice(name.as_bytes());
        for c in leds {
            bytes.extend_from_slice(&[c.r, c.g, c.b]);
        }
        for w in img.words() {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        fs::write(out.join(format!("frame_{k:05}.bin")), bytes).unwrap();
    }
    println!("demo: {total} frames at {fps} fps");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = PathBuf::from(
        args.get(1)
            .cloned()
            .unwrap_or_else(|| "../../build/vp".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let mut ui = Box::new(Ui::new());
    match (args.get(2).map(String::as_str), args.get(3)) {
        (Some("demo"), fps) => {
            let fps: u64 = fps.and_then(|f| f.parse().ok()).unwrap_or(10);
            demo_frames(&mut ui, &out, fps);
        }
        (Some(name), fps) => {
            let fps: u64 = fps.and_then(|f| f.parse().ok()).unwrap_or(5);
            let secs = demo::SCENARIOS
                .iter()
                .find(|s| s.0 == name)
                .expect("unknown scenario")
                .1 as u64;
            for k in 0..secs * fps {
                dump(
                    &mut ui,
                    &out,
                    &format!("{name}_{k:03}.json"),
                    name,
                    k * 1000 / fps,
                );
            }
            println!("{name}: {} frames", secs * fps);
        }
        _ => {
            for (name, _, key) in demo::SCENARIOS {
                dump(
                    &mut ui,
                    &out,
                    &format!("{name}.json"),
                    name,
                    (key * 1000.0) as u64,
                );
                println!("{name}");
            }
        }
    }
}
