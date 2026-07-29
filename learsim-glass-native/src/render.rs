//! SDL2 render loop — runs on the main thread, draws directly to the display
//! (KMSDRM on a headless Pi; X11/Wayland in dev). Renders local views itself and
//! blits decoded glassout frames; forwards taps as input events.

use crate::state::Shared;
use crate::types::{InputEvent, LatestFrame};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/Library/Fonts/Arial.ttf",
];

const BG: Color = Color::RGB(5, 7, 10);
const FG: Color = Color::RGB(232, 238, 245);
const MUTED: Color = Color::RGB(92, 107, 122);
const ACCENT: Color = Color::RGB(41, 197, 255);

/// Snapshot of the target screen's state for one frame.
struct ScreenView {
    view_id: String,
    device_name: String,
    screen_name: String,
    fit: String,
    message: String,
    hour_12: bool,
    show_seconds: bool,
    show_date: bool,
    utc: bool,
    engine_url: String,
}

fn read_screen_view(shared: &Shared, screen_id: &str) -> ScreenView {
    let cfg = shared.config.lock().unwrap();
    let device_name = cfg.device.name.clone();
    let screen = cfg.screen(screen_id);
    match screen {
        Some(s) => ScreenView {
            view_id: s.view_id.clone(),
            device_name,
            screen_name: s.name.clone(),
            fit: s.setting_str("fit").unwrap_or("contain").to_string(),
            message: s.setting_str("message").unwrap_or("Standby").to_string(),
            hour_12: s.setting_str("hourFormat") == Some("12h"),
            show_seconds: !matches!(s.settings.get("showSeconds"), Some(serde_json::Value::Bool(false))),
            show_date: !matches!(s.settings.get("showDate"), Some(serde_json::Value::Bool(false))),
            utc: s.setting_bool("utc"),
            engine_url: s.setting_str("engineUrl").unwrap_or("").to_string(),
        },
        None => ScreenView {
            view_id: "standby".into(),
            device_name,
            screen_name: screen_id.to_string(),
            fit: "contain".into(),
            message: "Standby".into(),
            hour_12: false,
            show_seconds: true,
            show_date: true,
            utc: false,
            engine_url: String::new(),
        },
    }
}

/// Destination rectangle for a source frame within the output, per fit mode.
fn fit_rect(fit: &str, src_w: u32, src_h: u32, out_w: u32, out_h: u32) -> Rect {
    match fit {
        "stretch" => Rect::new(0, 0, out_w, out_h),
        "native" => {
            let x = (out_w as i32 - src_w as i32) / 2;
            let y = (out_h as i32 - src_h as i32) / 2;
            Rect::new(x, y, src_w, src_h)
        }
        _ => {
            let scale = (out_w as f32 / src_w as f32).min(out_h as f32 / src_h as f32);
            let w = (src_w as f32 * scale).round().max(1.0) as u32;
            let h = (src_h as f32 * scale).round().max(1.0) as u32;
            let x = (out_w as i32 - w as i32) / 2;
            let y = (out_h as i32 - h as i32) / 2;
            Rect::new(x, y, w, h)
        }
    }
}

/// Draw text centered at (cx, cy) scaled so its height is `target_h` px.
fn draw_text(
    canvas: &mut Canvas<Window>,
    tc: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    color: Color,
    cx: i32,
    cy: i32,
    target_h: u32,
) {
    if text.is_empty() {
        return;
    }
    let surface = match font.render(text).blended(color) {
        Ok(s) => s,
        Err(_) => return,
    };
    let texture = match tc.create_texture_from_surface(&surface) {
        Ok(t) => t,
        Err(_) => return,
    };
    let (tw, th) = (surface.width(), surface.height());
    if th == 0 {
        return;
    }
    let w = (tw as f32 * (target_h as f32 / th as f32)).round() as u32;
    let dst = Rect::new(cx - w as i32 / 2, cy - target_h as i32 / 2, w, target_h);
    let _ = canvas.copy(&texture, None, Some(dst));
}

pub fn run(
    shared: Arc<Shared>,
    screen_id: String,
    latest: LatestFrame,
    input_tx: mpsc::UnboundedSender<InputEvent>,
) -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // Smooth (bilinear) texture scaling — without this, scaled text and panels
    // use nearest-neighbour and look jagged/choppy. Must be set before any
    // texture is created.
    let _ = sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "linear");

    let window = video
        .window("learsim-glass", 1280, 720)
        .fullscreen_desktop()
        .borderless()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    sdl.mouse().show_cursor(false);

    let tc = canvas.texture_creator();
    let (out_w, out_h) = canvas.output_size()?;

    // Load a font (optional — glassout still works without one).
    // Render glyphs large so most views downscale (crisp) rather than upscale.
    let font = FONT_PATHS
        .iter()
        .find_map(|p| ttf.load_font(*p, 256).ok());
    if font.is_none() {
        eprintln!("[render] no system font found; text views will be blank. Install fonts-dejavu-core.");
    }

    let mut event_pump = sdl.event_pump()?;

    // Panel frame texture (recreated when the source size changes).
    let mut panel_tex: Option<Texture> = None;
    let mut panel_dims: (u32, u32) = (0, 0);
    let mut panel_rect: Option<Rect> = None;
    let mut last_frame_at: Option<Instant> = None;

    'running: loop {
        let sv = read_screen_view(&shared, &screen_id);

        // --- events ---------------------------------------------------------
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    forward_tap(x as f32, y as f32, &panel_rect, panel_dims, &input_tx);
                }
                Event::FingerDown { x, y, .. } => {
                    forward_tap(x * out_w as f32, y * out_h as f32, &panel_rect, panel_dims, &input_tx);
                }
                _ => {}
            }
        }

        // --- draw -----------------------------------------------------------
        canvas.set_draw_color(BG);
        canvas.clear();

        match sv.view_id.as_str() {
            "glassout" => {
                // Upload a newly-arrived frame, if any.
                if let Some(frame) = latest.lock().unwrap().take() {
                    if panel_tex.is_none() || panel_dims != (frame.width, frame.height) {
                        panel_tex = tc
                            .create_texture_streaming(PixelFormatEnum::RGB24, frame.width, frame.height)
                            .ok();
                        panel_dims = (frame.width, frame.height);
                    }
                    if let Some(tex) = panel_tex.as_mut() {
                        let pitch = (frame.width * 3) as usize;
                        let _ = tex.update(None, &frame.rgb, pitch);
                    }
                    last_frame_at = Some(Instant::now());
                }

                let fresh = last_frame_at.map(|t| t.elapsed() < Duration::from_secs(3)).unwrap_or(false);
                if let (Some(tex), true) = (panel_tex.as_ref(), fresh) {
                    let dst = fit_rect(&sv.fit, panel_dims.0, panel_dims.1, out_w, out_h);
                    panel_rect = Some(dst);
                    let _ = canvas.copy(tex, None, Some(dst));
                } else {
                    // Not connected / no frames yet → branded placeholder.
                    panel_rect = None;
                    if let Some(font) = font.as_ref() {
                        let msg = if sv.engine_url.is_empty() {
                            "Glassout not configured".to_string()
                        } else {
                            format!("Connecting to {}", sv.engine_url)
                        };
                        draw_text(&mut canvas, &tc, font, &msg, FG, out_w as i32 / 2, out_h as i32 / 2, 44);
                        draw_id(&mut canvas, &tc, font, &sv, out_w, out_h);
                    }
                }
            }
            "clock" => {
                panel_rect = None;
                if let Some(font) = font.as_ref() {
                    draw_clock(&mut canvas, &tc, font, &sv, out_w, out_h);
                }
            }
            "testpattern" => {
                panel_rect = None;
                draw_test_pattern(&mut canvas, out_w, out_h);
            }
            _ => {
                // standby
                panel_rect = None;
                if let Some(font) = font.as_ref() {
                    draw_standby(&mut canvas, &tc, font, &sv, out_w, out_h);
                }
            }
        }

        canvas.present();
    }

    Ok(())
}

fn forward_tap(
    x: f32,
    y: f32,
    panel_rect: &Option<Rect>,
    panel_dims: (u32, u32),
    input_tx: &mpsc::UnboundedSender<InputEvent>,
) {
    let (Some(rect), (sw, sh)) = (panel_rect, panel_dims) else {
        return;
    };
    if sw == 0 || sh == 0 || rect.width() == 0 || rect.height() == 0 {
        return;
    }
    let rx = x - rect.x() as f32;
    let ry = y - rect.y() as f32;
    if rx < 0.0 || ry < 0.0 || rx >= rect.width() as f32 || ry >= rect.height() as f32 {
        return;
    }
    let sx = (rx / rect.width() as f32 * sw as f32) as u32;
    let sy = (ry / rect.height() as f32 * sh as f32) as u32;
    let _ = input_tx.send(InputEvent { x: sx, y: sy });
}

fn draw_id(
    canvas: &mut Canvas<Window>,
    tc: &TextureCreator<WindowContext>,
    font: &Font,
    sv: &ScreenView,
    out_w: u32,
    out_h: u32,
) {
    let id = format!("{} · {}", sv.device_name, sv.screen_name);
    draw_text(canvas, tc, font, &id, MUTED, out_w as i32 / 2, out_h as i32 - 40, 22);
}

fn draw_standby(
    canvas: &mut Canvas<Window>,
    tc: &TextureCreator<WindowContext>,
    font: &Font,
    sv: &ScreenView,
    out_w: u32,
    out_h: u32,
) {
    draw_text(canvas, tc, font, "learsim · glass", MUTED, out_w as i32 / 2, out_h as i32 / 2 - 120, 26);
    draw_text(canvas, tc, font, &sv.message, FG, out_w as i32 / 2, out_h as i32 / 2, 96);
    draw_id(canvas, tc, font, sv, out_w, out_h);
}

fn draw_clock(
    canvas: &mut Canvas<Window>,
    tc: &TextureCreator<WindowContext>,
    font: &Font,
    sv: &ScreenView,
    out_w: u32,
    out_h: u32,
) {
    use chrono::{Local, Utc};

    let (time_str, date_str) = if sv.utc {
        format_time(&Utc::now(), sv)
    } else {
        format_time(&Local::now(), sv)
    };

    draw_text(canvas, tc, font, &time_str, FG, out_w as i32 / 2, out_h as i32 / 2 - 40, (out_h / 4).max(48));
    if sv.show_date {
        draw_text(canvas, tc, font, &date_str, MUTED, out_w as i32 / 2, out_h as i32 / 2 + (out_h as i32 / 8), 40);
    }
    let mut id = format!("{} · {}", sv.device_name, sv.screen_name);
    if sv.utc {
        id.push_str(" · ZULU");
    }
    draw_text(canvas, tc, font, &id, MUTED, out_w as i32 / 2, out_h as i32 - 40, 22);
}

fn format_time<Tz: chrono::TimeZone>(now: &chrono::DateTime<Tz>, sv: &ScreenView) -> (String, String)
where
    Tz::Offset: std::fmt::Display,
{
    let time_fmt = match (sv.hour_12, sv.show_seconds) {
        (true, true) => "%I:%M:%S %p",
        (true, false) => "%I:%M %p",
        (false, true) => "%H:%M:%S",
        (false, false) => "%H:%M",
    };
    let mut time_str = now.format(time_fmt).to_string();
    if sv.utc {
        time_str.push('Z');
    }
    (time_str, now.format("%A %d %B %Y").to_string())
}

fn draw_test_pattern(canvas: &mut Canvas<Window>, w: u32, h: u32) {
    let (wi, hi) = (w as i32, h as i32);
    // grid
    canvas.set_draw_color(Color::RGB(30, 42, 56));
    let mut x = 0;
    while x <= wi {
        let _ = canvas.draw_line((x, 0), (x, hi));
        x += 100;
    }
    let mut y = 0;
    while y <= hi {
        let _ = canvas.draw_line((0, y), (wi, y));
        y += 100;
    }
    // frame
    canvas.set_draw_color(ACCENT);
    let _ = canvas.draw_rect(Rect::new(1, 1, w - 2, h - 2));
    // cross + diagonals
    canvas.set_draw_color(Color::RGB(61, 220, 132));
    let _ = canvas.draw_line((wi / 2, 0), (wi / 2, hi));
    let _ = canvas.draw_line((0, hi / 2), (wi, hi / 2));
    let _ = canvas.draw_line((0, 0), (wi, hi));
    let _ = canvas.draw_line((wi, 0), (0, hi));
    // colour bars
    let bars = [
        Color::WHITE,
        Color::RGB(255, 255, 0),
        Color::RGB(0, 255, 255),
        Color::RGB(0, 255, 0),
        Color::RGB(255, 0, 255),
        Color::RGB(255, 0, 0),
        Color::RGB(0, 0, 255),
        Color::BLACK,
    ];
    let bw = w / bars.len() as u32;
    let bh = (h as f32 * 0.08).max(40.0) as u32;
    for (i, c) in bars.iter().enumerate() {
        canvas.set_draw_color(*c);
        let _ = canvas.fill_rect(Rect::new(i as i32 * bw as i32, hi - bh as i32, bw, bh));
    }
}
