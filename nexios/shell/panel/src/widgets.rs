//! Desktop widget cards — calendar, clock, weather and photos — in the style
//! of the modern macOS desktop: small frosted squares on the top-left of the
//! workspace, under the wallpaper and the windows.
//!
//! The cards are passive: they take no clicks, so the surface they live on
//! (`nexios-widgets`, Layer::Bottom) gives its whole input region away. Data
//! comes from the same files the dock reads: `weather.txt` for the weather
//! card, `~/Pictures` for the photos card, the system clock for the calendar.

use crate::canvas::{text_width, Canvas, Fonts, Icon, Rgba};
use crate::material;
use crate::tokens::Theme;
use chrono::{Datelike, Local, NaiveDate};

/// Square card side, the macOS "small" widget size.
pub const SQ: i32 = 140;
pub const GAP: i32 = 10;
/// Photos card width — two small cards' worth, like the reference layout.
pub const PHOTO_W: i32 = 300;
pub const W: i32 = SQ + GAP + PHOTO_W;
pub const H: i32 = SQ * 2 + GAP;
pub const RADIUS: f32 = 18.0;

const CARD_PAD: i32 = 14;

/// Card rectangles in surface coordinates, for the compositor blur region.
pub fn card_rects() -> [(i32, i32, i32, i32); 4] {
    [
        (0, 0, SQ, SQ),                    // calendar
        (SQ + GAP, 0, SQ, SQ),             // clock
        (0, SQ + GAP, SQ, SQ),             // weather
        (SQ + GAP, SQ + GAP, PHOTO_W, SQ), // photos
    ]
}

/// What the weather card shows. `weather.txt` lines: 1 temperature
/// (numeric gets a "°" like the dock), 2 city, 3 condition, 4 `H:hi L:lo`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Weather {
    pub temp: String,
    pub city: String,
    pub condition: String,
    pub range: String,
}

pub struct Cal {
    /// e.g. "NOVEMBER".
    pub month: String,
    /// Column (0 = Monday) of the first day of the month.
    pub first_col: i32,
    pub days: i32,
    pub today: i32,
}

pub struct View {
    pub cal: Cal,
    /// `None` when weather.txt is missing or empty — the card then shows
    /// where to write the data instead of inventing a forecast.
    pub weather: Option<Weather>,
    /// Newest decodable image in ~/Pictures; `None` draws the placeholder.
    pub photo: Option<Icon>,
}

/// Pure parser so tests do not touch the filesystem.
pub fn parse_weather(text: Option<&str>) -> Option<Weather> {
    let text = text?;
    let mut lines = text.lines().map(str::trim);
    let temp = lines.next()?;
    if temp.is_empty() {
        return None;
    }
    let temp = if temp.chars().all(|c| c.is_ascii_digit() || c == '-') {
        format!("{temp}°")
    } else {
        temp.to_string()
    };
    let line = |s: Option<&str>| s.unwrap_or("").to_string();
    Some(Weather {
        temp,
        city: line(lines.next()),
        condition: line(lines.next()),
        range: line(lines.next()),
    })
}

/// Month grid facts for a date, pure for tests.
pub fn cal_of(date: NaiveDate) -> Cal {
    let first = date.with_day(1).unwrap();
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    };
    Cal {
        month: date.format("%B").to_string().to_uppercase(),
        first_col: first.weekday().num_days_from_monday() as i32,
        days: (next_month - first).num_days() as i32,
        today: date.day() as i32,
    }
}

/// Today's view: clock for the calendar, `weather.txt` for the weather card,
/// the newest PNG in ~/Pictures for the photos card.
pub fn live() -> View {
    let today = Local::now().date_naive();
    let weather_text =
        nexios_conf::config_path("weather.txt").and_then(|p| std::fs::read_to_string(p).ok());
    let photo = newest_photo().and_then(|b| crate::canvas::decode_png(&b));
    View {
        cal: cal_of(today),
        weather: parse_weather(weather_text.as_deref()),
        photo,
    }
}

fn newest_photo() -> Option<Vec<u8>> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::path::Path::new(&home).join("Pictures");
    let mut best: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let is_png = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("png"));
        if !is_png {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if best.as_ref().is_none_or(|(t, _)| modified > *t) {
            best = Some((modified, path));
        }
    }
    best.and_then(|(_, p)| std::fs::read(p).ok())
}

fn card(cv: &mut Canvas<'_>, slab: &material::Slab, x: i32, y: i32, w: i32, h: i32) {
    material::paint(cv, slab, x, y, w, h, RADIUS);
}

/// The calendar's ink: the widget is a light card whatever the theme, like
/// the reference, so its text is always dark.
fn cal_ink() -> (Rgba, Rgba) {
    (
        Rgba::rgb(0x24, 0x26, 0x2b),
        Rgba::rgba(0x24, 0x26, 0x2b, 150),
    )
}

fn paint_cal(cv: &mut Canvas<'_>, fonts: &Fonts, cal: &Cal, x: i32, y: i32) {
    let red = Rgba::rgb(0xf5, 0x45, 0x36);
    let (ink, dim) = cal_ink();
    cv.text(
        fonts.semibold.as_ref(),
        &cal.month,
        x + CARD_PAD,
        y + CARD_PAD + 12,
        13.0,
        red,
    );

    // Grid: weekday header then weeks, Monday-first, centred in each cell.
    let grid_x = x + CARD_PAD;
    let grid_w = SQ - CARD_PAD * 2;
    let cell_w = grid_w as f32 / 7.0;
    let header_y = y + CARD_PAD + 26;
    for (i, d) in "MTWTFSS".chars().enumerate() {
        let s = d.to_string();
        let tw = text_width(fonts.regular.as_ref(), &s, 9.5);
        let cx = grid_x + (i as f32 * cell_w + cell_w / 2.0 - tw as f32 / 2.0) as i32;
        cv.text(fonts.regular.as_ref(), &s, cx, header_y, 9.5, dim);
    }
    let rows = 6;
    let row_h = ((SQ - CARD_PAD) - (header_y - y) - 8) as f32 / rows as f32;
    let day0_y = header_y + 12;
    for day in 1..=cal.days {
        let cell = cal.first_col + day - 1;
        let (col, row) = (cell % 7, cell / 7);
        let s = day.to_string();
        let cy = day0_y + (row as f32 * row_h) as i32;
        let cx_f = grid_x as f32 + col as f32 * cell_w + cell_w / 2.0;
        if day == cal.today {
            cv.circle(cx_f, cy as f32 + 4.5, 9.5, red);
            let tw = text_width(fonts.semibold.as_ref(), &s, 11.5);
            cv.text(
                fonts.semibold.as_ref(),
                &s,
                (cx_f - tw as f32 / 2.0) as i32,
                cy + 9,
                11.5,
                Rgba::rgb(255, 255, 255),
            );
        } else {
            let tw = text_width(fonts.regular.as_ref(), &s, 11.5);
            cv.text(
                fonts.regular.as_ref(),
                &s,
                (cx_f - tw as f32 / 2.0) as i32,
                cy + 9,
                11.5,
                ink,
            );
        }
    }
}

fn paint_weather(cv: &mut Canvas<'_>, fonts: &Fonts, w: &Option<Weather>, x: i32, y: i32) {
    // The weather card keeps its own blue, as on the reference desktop.
    cv.round_rect_vgrad(
        x,
        y,
        SQ,
        SQ,
        RADIUS,
        Rgba::rgb(0x46, 0x9b, 0xe8),
        Rgba::rgb(0x2b, 0x68, 0xce),
    );
    cv.round_rect_glass_rim(
        x,
        y,
        SQ,
        SQ,
        RADIUS,
        Rgba::rgba(255, 255, 255, 120),
        Rgba::rgba(255, 255, 255, 30),
        Rgba::rgba(0, 0, 0, 26),
    );
    let white = Rgba::rgb(255, 255, 255);
    let soft = Rgba::rgba(255, 255, 255, 190);
    let pad = CARD_PAD;
    match w {
        Some(wx) => {
            let city = if wx.city.is_empty() {
                "Weather"
            } else {
                &wx.city
            };
            cv.text(
                fonts.semibold.as_ref(),
                city,
                x + pad,
                y + pad + 14,
                14.0,
                white,
            );
            cv.text(
                fonts.regular.as_ref(),
                &wx.temp,
                x + pad - 2,
                y + pad + 66,
                44.0,
                white,
            );
            let mut ty = y + SQ - pad - 16;
            if !wx.range.is_empty() {
                cv.text(fonts.regular.as_ref(), &wx.range, x + pad, ty, 12.0, soft);
                ty -= 16;
            }
            if !wx.condition.is_empty() {
                cv.text(
                    fonts.regular.as_ref(),
                    &wx.condition,
                    x + pad,
                    ty,
                    12.0,
                    soft,
                );
            }
        }
        None => {
            cv.text(
                fonts.semibold.as_ref(),
                "Weather",
                x + pad,
                y + pad + 14,
                14.0,
                white,
            );
            cv.text(
                fonts.regular.as_ref(),
                "Write a temperature to",
                x + pad,
                y + SQ - pad - 30,
                11.0,
                soft,
            );
            cv.text(
                fonts.regular.as_ref(),
                "weather.txt",
                x + pad,
                y + SQ - pad - 16,
                11.0,
                soft,
            );
        }
    }
}

/// Cover-blit `ic` clipped to a rounded rect — the photo fills its card.
/// Works in raw pixels; `blend` already takes physical coordinates.
fn blit_cover(cv: &mut Canvas<'_>, ic: &Icon, x: i32, y: i32, w: i32, h: i32, r: f32) {
    let (iw, ih) = (ic.width as f32, ic.height as f32);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let s = cv.scale;
    let (x, y, w, h, r) = (x * s, y * s, w * s, h * s, r * s as f32);
    let (pw, ph) = (cv.stride / 4, cv.pixels.len() / cv.stride.max(1));
    let scale = (w as f32 / iw).max(h as f32 / ih);
    let (src_w, src_h) = (w as f32 / scale, h as f32 / scale);
    let (src_x, src_y) = ((iw - src_w) / 2.0, (ih - src_h) / 2.0);
    let (half_w, half_h) = (w as f32 * 0.5, h as f32 * 0.5);
    let (cx, cy) = (x as f32 + half_w, y as f32 + half_h);
    for py in y.max(0)..(y + h).min(ph as i32) {
        let qy = (py as f32 + 0.5 - cy).abs() - (half_h - r);
        let sy = (src_y + (py - y) as f32 / scale) as u32;
        if sy >= ic.height {
            continue;
        }
        for px in x.max(0)..(x + w).min(pw as i32) {
            let qx = (px as f32 + 0.5 - cx).abs() - (half_w - r);
            let sdf = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;
            let cov = (0.5 - sdf).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            let sx = (src_x + (px - x) as f32 / scale) as u32;
            if sx >= ic.width {
                continue;
            }
            let off = ((sy * ic.width + sx) * 4) as usize;
            let col = Rgba::rgba(
                ic.pixels[off],
                ic.pixels[off + 1],
                ic.pixels[off + 2],
                ic.pixels[off + 3],
            );
            cv.blend(px, py, col, cov);
        }
    }
}

fn paint_photos(cv: &mut Canvas<'_>, fonts: &Fonts, photo: Option<&Icon>, x: i32, y: i32) {
    if let Some(ic) = photo {
        blit_cover(cv, ic, x, y, PHOTO_W, SQ, RADIUS);
        cv.round_rect_glass_rim(
            x,
            y,
            PHOTO_W,
            SQ,
            RADIUS,
            Rgba::rgba(255, 255, 255, 110),
            Rgba::rgba(255, 255, 255, 30),
            Rgba::rgba(0, 0, 0, 40),
        );
        return;
    }
    let (_, dim) = cal_ink();
    cv.draw_app_squircle(x + PHOTO_W / 2 - 22, y + 18, 44, "photos", None, fonts);
    let lines = [
        "Photos will appear here",
        "when they are added to",
        "your library",
    ];
    for (i, l) in lines.iter().enumerate() {
        let tw = text_width(fonts.regular.as_ref(), l, 11.0);
        cv.text(
            fonts.regular.as_ref(),
            l,
            x + (PHOTO_W - tw) / 2,
            y + 82 + i as i32 * 14,
            11.0,
            dim,
        );
    }
}

/// The clock's analog face: light dial, dark hands, red second hand — the
/// macOS Clock widget. `now` is a parameter so previews and tests can pin it.
fn paint_clock(cv: &mut Canvas<'_>, fonts: &Fonts, x: i32, y: i32, now: chrono::DateTime<Local>) {
    use chrono::Timelike;
    let (ink, dim) = cal_ink();
    cv.text(
        fonts.semibold.as_ref(),
        "Local",
        x + CARD_PAD,
        y + CARD_PAD + 13,
        12.0,
        ink,
    );
    let cx = x as f32 + SQ as f32 / 2.0;
    let cy = y as f32 + SQ as f32 / 2.0 + 12.0;
    let r = 42.0;
    // Dial: warm white face, hairline rim.
    cv.circle(cx, cy, r, Rgba::rgb(0xfa, 0xfa, 0xf7));
    cv.stroke_round_rect(
        (cx - r) as i32,
        (cy - r) as i32,
        (r * 2.0) as i32,
        (r * 2.0) as i32,
        r,
        1.0,
        Rgba::rgba(0x24, 0x26, 0x2b, 60),
    );
    for i in 0..12 {
        let a = i as f32 / 12.0 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let major = i % 3 == 0;
        let (inner, thick) = if major { (r - 9.0, 2.2) } else { (r - 5.0, 1.4) };
        cv.line(
            cx + a.cos() * inner,
            cy + a.sin() * inner,
            cx + a.cos() * (r - 2.0),
            cy + a.sin() * (r - 2.0),
            thick,
            if major { ink } else { dim },
        );
    }
    let secs = now.num_seconds_from_midnight() as f32 + now.timestamp_subsec_millis() as f32 / 1000.0;
    let mut hand = |frac: f32, len: f32, thick: f32, col: Rgba| {
        let a = frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        cv.line(cx, cy, cx + a.cos() * len, cy + a.sin() * len, thick, col);
    };
    hand(secs / 43200.0, r * 0.5, 3.6, ink); // hour
    hand(secs / 3600.0 % 1.0, r * 0.72, 2.6, ink); // minute
    hand(secs / 60.0 % 1.0, r * 0.78, 1.1, Rgba::rgb(0xf5, 0x45, 0x36)); // second
    cv.circle(cx, cy, 3.0, ink);
    cv.circle(cx, cy, 1.3, Rgba::rgb(0xf5, 0x45, 0x36));
}

/// Paint all four cards with their top-left at `x`,`y`. `slab` is the glass
/// material the bar and dock use, so the widgets share the same frost.
pub fn paint(
    cv: &mut Canvas<'_>,
    _th: &Theme,
    fonts: &Fonts,
    v: &View,
    slab: &material::Slab,
    x: i32,
    y: i32,
) {
    let [(cx, cy, cw, ch), (kx, ky, kw, kh), (wx, wy, ww, wh), (px, py, pw, ph)] =
        card_rects();
    card(cv, slab, x + cx, y + cy, cw, ch);
    card(cv, slab, x + kx, y + ky, kw, kh);
    card(cv, slab, x + wx, y + wy, ww, wh);
    card(cv, slab, x + px, y + py, pw, ph);
    paint_cal(cv, fonts, &v.cal, x + cx, y + cy);
    paint_clock(cv, fonts, x + kx, y + ky, Local::now());
    paint_weather(cv, fonts, &v.weather, x + wx, y + wy);
    paint_photos(cv, fonts, v.photo.as_ref(), x + px, y + py);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Material;
    use crate::tokens::{Accent, ThemeKind};

    #[test]
    fn weather_text_parses_all_four_lines() {
        let w = parse_weather(Some("11\nTokyo\nMostly Sunny\nH:14° L:3°")).unwrap();
        assert_eq!(w.temp, "11°");
        assert_eq!(w.city, "Tokyo");
        assert_eq!(w.condition, "Mostly Sunny");
        assert_eq!(w.range, "H:14° L:3°");
        // Back-compat: a lone temperature still fills the card.
        let w = parse_weather(Some("22")).unwrap();
        assert_eq!(w.temp, "22°");
        assert!(w.city.is_empty());
        // A value that already carries its unit is kept verbatim.
        assert_eq!(parse_weather(Some("11°C")).unwrap().temp, "11°C");
        assert!(parse_weather(None).is_none());
        assert!(parse_weather(Some("")).is_none());
    }

    #[test]
    fn calendar_grid_is_monday_first() {
        // November 2026 starts on a Sunday → column 6 of a Monday-first grid.
        let cal = cal_of(NaiveDate::from_ymd_opt(2026, 11, 22).unwrap());
        assert_eq!(cal.month, "NOVEMBER");
        assert_eq!(cal.first_col, 6);
        assert_eq!(cal.days, 30);
        assert_eq!(cal.today, 22);
        let feb = cal_of(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
        assert_eq!(feb.first_col, 6); // Feb 1 2026 is a Sunday.
        assert_eq!(feb.days, 28);
    }

    #[test]
    fn cards_fit_the_surface() {
        for (x, y, w, h) in card_rects() {
            assert!(x >= 0 && y >= 0 && x + w <= W && y + h <= H);
        }
    }

    #[test]
    fn paint_fills_every_pixel() {
        let fonts = Fonts::load();
        let th = Theme::with(ThemeKind::Light, Accent::Blue);
        let mut buf = vec![0u8; (W * H * 4) as usize];
        let mut cv = Canvas::new(&mut buf, W as u32, H as u32, 1);
        let tint = Rgba::rgb(0xf0, 0x7d, 0x3e);
        let slab = material::slab(Material::Glass, th.surface, Some(tint));
        let v = View {
            cal: cal_of(NaiveDate::from_ymd_opt(2026, 11, 22).unwrap()),
            weather: parse_weather(Some("11\nTokyo\nMostly Sunny\nH:14° L:3°")),
            photo: None,
        };
        paint(&mut cv, &th, &fonts, &v, &slab, 0, 0);
        // Card bodies paint; clear of every card and its shadow stays empty.
        let at = |x: usize, y: usize| buf[(y * W as usize + x) * 4 + 3];
        assert!(at(SQ as usize / 2, SQ as usize / 2) > 0);
        assert!(at(SQ as usize / 2, (SQ + GAP + SQ / 2) as usize) > 0);
        assert!(
            at(
                (SQ + GAP + PHOTO_W / 2) as usize,
                (SQ + GAP + SQ / 2) as usize
            ) > 0
        );
        assert_eq!(at(W as usize - 2, 2), 0);
    }
}
