use std::f32::consts::FRAC_PI_2;
use nannou::color::IntoLinSrgba;
use nannou::prelude::*;
use nannou::winit::event::VirtualKeyCode;
use palette::{Hsl, IntoColor, Okhsl};

fn main() {
    nannou::app(model)
        .update(update)
        .fullscreen()
        .run();
}

struct Model {
    _window: WindowId,
    palette: [LinSrgb; 8]
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("wallpaper")
        .view(view)
        .event(event)
        // .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    // let egui = Egui::from_window(&window);

    let palette = palette();

    // println!("{}", palette.iter().map(|x| format!("({}, {}, {})", x.red, x.green, x.blue)).collect::<Vec<String>>().join("\n"));

    Model {
        _window: window_id,
        palette,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let max = 8;
    let rect = app.window_rect();
    for (i, colour) in model.palette.iter().enumerate() {
        let prop = i as f32 / max as f32;
        let x_start = rect.x.lerp(prop);
        let x_end = rect.x.lerp(prop + 1.0 / max as f32);
        let top = rect.y.start;
        let bottom = rect.y.end;
        let points = vec![Vec2::new(x_start, top), Vec2::new(x_end, top), Vec2::new(x_end, bottom), Vec2::new(x_start, bottom)];
        draw.polygon().points(points).color(*colour);
    }
    draw.to_frame(app, &frame).unwrap()
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => {
            match key {
                VirtualKeyCode::R => {model.palette = palette()}
                _ => {}
            }
        }
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, update: Update) {

}

fn palette() -> [LinSrgb; 8] {
    let h_init: f32 = random::<f32>();

    // let min_range = lerp(0.0, 0.35, random());
    // let max_range = lerp(0.65, 1.0, random());

    let min_range = 0.4;
    let max_range = 1.0;

    let max = 8;

    [0; 8].iter().enumerate()
        .map(|(i, _)| i as f32 / max as f32)
        .map(|t| lerp(min_range, max_range, t))
        .map(|t| {
            let c = curve(t);
            (lerp(h_init, h_init + 0.4, t) % 1.0, c.0, c.1)
        })
        // .inspect(|(h, s, v)| println!("HSV = {}, {}, {}", h, s, v))
        .map(|(h, s, v)|hsv(h, s, v) )
        .collect::<Vec<LinSrgb>>().try_into().unwrap()
}

fn curve(t: f32) -> (f32, f32) {
    let t = t * FRAC_PI_2;
    let n = 1.2;
    (t.sin().powf(n), t.cos().powf(n))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

fn hsv(h: f32, s: f32, v: f32) -> LinSrgb {
    let x: palette::Srgb<f32> = Okhsl::new(h * 360., s, v).into_color();
    let y: palette::LinSrgb<f32> = x.into_linear();
    LinSrgb::from_components(y.into_components())
}

// fn gui(app: &App, model: &mut Model, update: Update) {}

// fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
//     // Let egui handle things like keyboard and mouse input.
//     model.gui.handle_raw_event(event);
// }
