//! text wrap estimator

use companion_ui::generated::style;

const SAMPLES: [&str; 6] = [
    "CO₂ hit 1,180 ppm, so I turned the fan up to 40%. Opening a window would help too. I'll keep an eye on it and let you know when the air is fresh again, which should take about twenty minutes.",
    "The empty bedroom hit 75°F, so I nudged its thermostat down 2°. Everyone seems to be in the living room, so I left that one alone. Tell me if it feels too warm in here and I'll adjust it as well.",
    "It's after sunset and someone's here, so I warmed and dimmed the lights. The kitchen lights are still on at full brightness; want me to dim those too? Nobody has been in there for an hour or so.",
    "Everything looks comfortable. Air is fresh, and the room is holding at 72°F. Humidity is a little low at 34%, which is normal for October. Nothing needs your attention right now, enjoy the evening.",
    "Particles spiked in the kitchen, probably from cooking. I switched the range hood to high and the purifier to turbo. Closing the bedroom door would keep the smoke out of there until it clears.",
    "Good morning! It was a quiet night: temperatures stayed between 68 and 70°F and CO₂ never went above 900 ppm in the bedroom. Today looks sunny, so I'll open the blinds a little after nine.",
];

/// count shown characters
fn shown(text: &str, lines: usize) -> (usize, bool) {
    let mut n = 0;
    let mut cut = false;
    style::STATEMENT.wrap(text, 364 - 40, lines, |_, line, truncated| {
        n += line.chars().count() + 1;
        cut |= truncated;
    });
    (n.saturating_sub(1), cut)
}

fn main() {
    for lines in [4, 3] {
        let counts: Vec<usize> = SAMPLES.iter().map(|s| shown(s, lines).0).collect();
        let min = counts.iter().min().unwrap();
        let avg = counts.iter().sum::<usize>() / counts.len();
        println!(
            "{lines} lines: {min} to {} characters shown (average {avg})",
            counts.iter().max().unwrap()
        );
    }
    for s in SAMPLES {
        let (n, cut) = shown(s, 4);
        println!(
            "{n:4} {} | {}",
            if cut { "cut" } else { "fit" },
            s.chars().take(n).collect::<String>()
        );
    }
}
