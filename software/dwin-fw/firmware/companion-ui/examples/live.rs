//! live simulation runner

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use companion_ui::director::{Director, Settings};
use companion_ui::generated;
use companion_ui::model::Model;
use companion_ui::ui::Ui;
use companion_ui::vp::{self, Inputs, Parser};

/// write frame bytes
fn frame(vp: u16, words: &[u16]) -> Vec<u8> {
    let mut f = vec![0x5A, 0xA5, (3 + 2 * words.len()) as u8, 0x82];
    f.extend_from_slice(&vp.to_be_bytes());
    for w in words {
        f.extend_from_slice(&w.to_be_bytes());
    }
    f
}

fn text(s: &str) -> Vec<u16> {
    let mut b = s.as_bytes().to_vec();
    b.push(0);
    if b.len() % 2 == 1 {
        b.push(0);
    }
    b.chunks(2)
        .map(|p| u16::from_be_bytes([p[0], p[1]]))
        .collect()
}

struct Bench {
    inputs: Inputs,
    parser: Parser,
    model: Model,
    director: Director,
    ui: Ui,
}

impl Bench {
    /// send words bench
    fn send(&mut self, vp: u16, words: &[u16], now: u64) {
        let mut reply = [0u8; vp::REPLY_MAX];
        for (k, chunk) in words.chunks(120).enumerate() {
            for b in frame(vp + 120 * k as u16, chunk) {
                if let Some(req) = self.parser.push(b) {
                    let (n, result) = self.inputs.handle(&req, now, &mut reply);
                    assert_eq!(&reply[..n], &vp::ACK);
                    assert_eq!(result, Some(Ok(())), "write at {vp:#06x} refused");
                }
            }
        }
        self.inputs
            .apply(&mut self.model, &mut self.director.settings, now);
        self.director.decide(&self.model, now);
    }

    fn dump(&mut self, out: &Path, name: &str, now: u64) {
        self.inputs
            .apply(&mut self.model, &mut self.director.settings, now);
        let screen = self
            .inputs
            .screen()
            .unwrap_or_else(|| self.director.decide(&self.model, now).screen);
        let (img, dropped) = self.ui.render_only(screen, &self.model, now);
        assert_eq!(dropped, 0, "{name}: {dropped} sprites didn't fit");
        let mut json = String::new();
        let _ = write!(
            json,
            "{{\"page\": {}, \"base\": {}, \"words\": [",
            screen.page(),
            generated::VP_BASE
        );
        for (i, w) in img.words().iter().enumerate() {
            let _ = write!(json, "{}{w}", if i > 0 { "," } else { "" });
        }
        json.push_str("]}\n");
        fs::write(out.join(format!("live-{name}.json")), json).unwrap();
        println!("live-{name}: {}", screen.name());
    }
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or("../../build/vp".into()));
    fs::create_dir_all(&out).unwrap();
    let mut b = Bench {
        inputs: Inputs::new(),
        parser: Parser::new(),
        model: Model::default(),
        director: Director::new(Settings::default()),
        ui: Ui::new(),
    };
    let mut now = 3_600_000;
    let unix: u32 = 1_792_346_700; // test epoch timestamp

    // waiting live mode
    b.send(vp::MODE, &[1], now);
    b.dump(&out, "waiting", now);

    // boot step progress
    b.send(vp::BOOT1, &[2], now);
    b.send(
        vp::BOOT1 + vp::step::DETAIL,
        &text("6 of 6 responding"),
        now,
    );
    b.send(vp::BOOT2, &[1], now);
    b.dump(&out, "boot", now);
    for (k, detail) in [
        (2, "HomeNet · −52 dBm"),
        (3, "intellithings/living-room"),
        (4, "MCP ready"),
    ] {
        b.send(vp::BOOT_STEPS[k], &[2], now);
        b.send(vp::BOOT_STEPS[k] + vp::step::DETAIL, &text(detail), now);
    }

    // live home state
    b.send(
        vp::TEMPERATURE,
        &[223, 412, 118, 642, 320, 1180, 3, 6, 7, 1],
        now,
    );
    b.send(vp::WIFI, &[1, 1, 0], now);
    b.send(
        vp::UNIX_TIME,
        &[(unix >> 16) as u16, unix as u16, (-240i16) as u16],
        now,
    );
    let message = "Everything looks comfortable. Air is fresh, and the room is holding at 72°F.";
    b.send(vp::MESSAGE_TS, &[(unix >> 16) as u16, unix as u16], now);
    b.send(vp::MESSAGE_TEXT, &text(message), now);
    now += 10_000;
    b.dump(&out, "home", now);

    // agent action flow
    b.send(vp::CO2, &[1180], now);
    b.send(vp::THINKING, &text("Looking at the air in here…"), now);
    b.dump(&out, "thinking", now);
    b.send(vp::CHIP1, &[0], now);
    b.send(vp::CHIP1 + vp::chip::LABEL, &text("Fan"), now);
    b.send(vp::CHIP1 + vp::chip::VALUE, &text("40%"), now);
    let said = "CO₂ hit 1,180 ppm, so I turned the fan up to 40%. Opening a window would help too.";
    b.send(vp::MESSAGE_TEXT, &text(said), now);
    let mut toast = vec![1, 0];
    toast.extend(text("Fan set to 40%"));
    b.send(vp::TOAST_DEVICE, &toast[1..], now);
    b.send(
        vp::TOAST_DETAIL,
        &text("Living Room · CO₂ was 1,180 ppm"),
        now,
    );
    b.send(vp::TOAST_TRIGGER, &[1], now);
    now += 1_500;
    b.dump(&out, "toast", now);
    now += 8_000;
    b.dump(&out, "chips", now);

    // forced rooms page
    b.send(vp::ROOM2, &[1, 233, 480, 700, 30, 120, 1], now);
    b.send(vp::ROOM3, &[1, 239, 390, 610, 4, 95, 0], now);
    b.send(vp::ROOM5 + vp::room::NOTE, &text("Lights on"), now);
    b.send(vp::SCREEN, &[3], now);
    b.dump(&out, "rooms", now);
    b.send(vp::SCREEN, &[0], now);

    // takeover alert screen
    b.send(vp::ALERT_METRIC, &[3, 87, 120], now);
    b.send(vp::ALERT_TITLE, &text("Smoke or cooking particles"), now);
    b.send(vp::ALERT_PLACE, &text("Kitchen"), now);
    b.send(vp::ALERT_NORMAL, &text("Normal is under 12"), now);
    b.send(
        vp::ALERT_MESSAGE,
        &text("Probably the stove. I'll tell you when the air clears."),
        now,
    );
    for (base, device, label, value) in [
        (vp::ALERT_DONE1, 0, "Range hood", "High"),
        (vp::ALERT_DONE2, 3, "Purifier", "Turbo"),
        (vp::ALERT_ASK, 6, "Close the bedroom door", ""),
    ] {
        b.send(base, &[device], now);
        b.send(base + vp::chip::LABEL, &text(label), now);
        b.send(base + vp::chip::VALUE, &text(value), now);
    }
    b.send(vp::ALERT_ACTIVE, &[1], now);
    b.dump(&out, "alert", now);
    b.send(vp::ALERT_ACTIVE, &[0], now);

    // offline hub state
    b.send(vp::HUB, &[2, 0xFFFF, 42], now);
    b.dump(&out, "offline", now);
}
