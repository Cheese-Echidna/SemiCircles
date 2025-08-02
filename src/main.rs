use nannou::prelude::*;
use nannou::rand;
use nannou::winit::event::VirtualKeyCode;
use palette::{IntoColor, Okhsl};
use rand_derive2::RandGen;
use std::f32::consts::FRAC_PI_2;
use std::time::{Duration, Instant};

fn main() {
    nannou::app(model).update(update).fullscreen().run();
}

// TODO:
//  - Add rotations
//  - Expand grid to fill screen (ideally line up with the palette colours) (will need to fix bounds check for movement)
//  - Fix random movement (currently too variable)


#[derive(Debug, RandGen)]
enum Orientation {
    Left,
    Right,
    Top,
    Bottom,
}

impl Orientation {
    fn angle(&self) -> f32 {
        (match self {
            Orientation::Left => 0.5,
            Orientation::Right => 1.5,
            Orientation::Top => 0.0,
            Orientation::Bottom => 1.0,
        }) * PI
    }
    fn offset(&self) -> Vec2 {
        match self {
            Orientation::Left => {-Vec2::X}
            Orientation::Right => {Vec2::X}
            Orientation::Top => {Vec2::Y}
            Orientation::Bottom => {-Vec2::Y}
        }
    }
}

#[derive(Debug, RandGen)]
enum SemiCircleType {
    Filled,
    Striped,
}

struct Model {
    _window: WindowId,
    palette: [LinSrgb; 8],
    grid: Grid,
}

#[derive(Debug)]
struct Grid {
    width: f32,
    tile_size: f32,
    num_tiles_wide: f32,
    objects: Vec<SemiCircle>,
    slide_time: f32
}

#[derive(Debug)]
struct SemiCircle {
    pos: Vec2,
    movement: Option<(Vec2, Duration)>,
    orientation: Orientation,
    semi_circle_type: SemiCircleType,
    colour: usize,

}

impl SemiCircle {
    fn draw(&self, draw: &Draw, centre: Vec2, radius: f32, colours: &[LinSrgb]) {
        let segments = 200;

        let arc = |radius: f32| -> Vec<Vec2> {
            (0..=segments)
                .map(|i| {
                    let t = map_range(i, 0, segments, 0.0, PI) + self.orientation.angle();
                    Vec2::new(t.cos(), t.sin()) * radius + centre
                })
                .collect()
        };

        match self.semi_circle_type {
            SemiCircleType::Filled => {
                draw.polygon().points(arc(radius)).color(colours[self.colour]);
            }
            SemiCircleType::Striped => {
                let num_rings = 3;
                for i in 0..num_rings {
                    let i = i as f32;
                    let num_rings = num_rings as f32;
                    let incr = 1.0  / (2.0 * num_rings);
                    let new_radius = incr * (2.0 * i + 1.5);
                    draw.polyline().weight(radius * incr).points(arc(radius * new_radius)).color(colours[self.colour]);
                }
            }
        }
    }
}

impl Grid {
    fn new(app: &App, palette_len: usize) -> Self {
        let wh = app.window_rect().w_h();

        let num_tiles_wide_u = 8_i32;
        let num_tiles_wide = num_tiles_wide_u as f32;

        let width = wh.0.min(wh.1) * 0.95;

        let mut objects = vec![];

        for x in 0..num_tiles_wide_u {
            for y in 0..num_tiles_wide_u {
                for _ in 0..2 {
                    objects.push(SemiCircle {
                        pos: Vec2::new(x as f32, y as f32),
                        movement: None,
                        orientation: Orientation::generate_random(),
                        semi_circle_type: SemiCircleType::generate_random(),
                        colour: random_range(0, palette_len),
                    })
                }
            }
        }

        Self {
            width,
            tile_size: width / num_tiles_wide,
            num_tiles_wide,
            objects,
            slide_time: 1.5,
        }
    }
    fn moving_pos_of(&self, p1: Vec2, p2: Vec2, time: Duration) -> Vec2 {
        let t = (time.as_secs_f32() / self.slide_time).clamp(0.0, 1.0);
        let p = p1.lerp(p2, ease_out_circ(t));
        self.pos_of(p)
    }

    fn pos_of(&self, p: Vec2) -> Vec2 {
        let bottom_left = -Vec2::splat(self.width / 2.0);
        let offset = Vec2::splat(self.tile_size / 2.0);
        let tile = Vec2::splat(self.tile_size) * p;
        bottom_left + offset + tile
    }
}

fn ease_out_circ(x: f32) -> f32 {
    (1.0 - (x - 1.0).powi(2)).sqrt()
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
    let grid = Grid::new(app, palette.len());

    Model {
        _window: window_id,
        palette,
        grid,
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
        let points = vec![
            Vec2::new(x_start, top),
            Vec2::new(x_end, top),
            Vec2::new(x_end, bottom),
            Vec2::new(x_start, bottom),
        ];
        draw.polygon().points(points).color(*colour);
    }

    for semi in model.grid.objects.iter() {
        let pos = if let Some((p2, duration)) = semi.movement {
            model.grid.moving_pos_of(semi.pos, p2, app.duration.since_start.abs_diff(duration))
        } else {
            model.grid.pos_of(semi.pos)
        };

        semi.draw(
            &draw,
            pos,
            model.grid.tile_size / 2.0,
            &model.palette,
        );
        // draw.ellipse().xy(model.grid.pos_of(semi.pos)).color(RED).radius(3.0);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyReleased(key) => match key {
            VirtualKeyCode::R => model.palette = palette(),
            _ => {}
        },
        _ => {}
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let slide_time = model.grid.slide_time;
    for object in model.grid.objects.iter_mut() {
        if let Some((new_pos, start)) = object.movement {
            if start.abs_diff(update.since_start).as_secs_f32() > slide_time * 1.2 {
                object.pos = new_pos;
                object.movement = None;
            }
        }
    }

    if random_f32() > 0.99 {
        let i = random_range(0, model.grid.objects.len());
        let object = &mut model.grid.objects[i];
        if object.movement.is_some() {
            return;
        }
        let new_pos = object.pos + Orientation::generate_random().offset();
        if new_pos.max_element() >= model.grid.num_tiles_wide || new_pos.min_element() < 0.0 {
            return;
        }

        object.movement = Some((new_pos, update.since_start));
    }
}

fn palette() -> [LinSrgb; 8] {
    let h_init: f32 = random::<f32>();
    let h_range = 1.5;

    // let min_range = lerp(0.0, 0.35, random());
    // let max_range = lerp(0.65, 1.0, random());

    let min_range = 0.3;
    let max_range = 1.0;

    let max = 8;

    [0; 8]
        .iter()
        .enumerate()
        .map(|(i, _)| i as f32 / max as f32)
        .map(|t| (t, lerp(min_range, max_range, t)))
        .map(|(t, x)| {
            let c = curve(x);
            (lerp(h_init, h_init + h_range, t) % 1.0, c.0, c.1)
        })
        // .inspect(|(h, s, v)| println!("HSV = {}, {}, {}", h, s, v))
        .map(|(h, s, v)| hsv(h, s * 1.2, v * 0.8))
        .collect::<Vec<LinSrgb>>()
        .try_into()
        .unwrap()
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
