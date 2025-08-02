use nannou::prelude::*;
use nannou::rand;
use nannou::winit::event::VirtualKeyCode;
use palette::{IntoColor, Okhsl};
use rand_derive2::RandGen;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use nannou::draw::background::new;

fn main() {
    nannou::app(model).update(update).fullscreen().run();
}

// TODO:
//  - Add rotations
//  - Expand grid to fill screen (ideally line up with the palette colours) (will need to fix bounds check for movement)
//  - Fix random movement (currently too variable)

fn random_orientation() -> f32 {
    random_range(0_u8, 4) as f32 / 4.0 * TAU
}

fn random_direction() -> Vec2 {
    [-Vec2::X, Vec2::X, Vec2::Y, -Vec2::Y][random_range(0, 4)]
}


#[derive(Copy, Clone, Debug)]
enum Transition {
    Rotation(f32),
    Translation(Vec2),
}

impl Transition {
    pub(crate) fn finalise(self, object: &mut SemiCircle) {
        match self {
            Transition::Rotation(r2) => { object.orientation = r2}
            Transition::Translation(p2) => { object.pos = p2}
        }
        object.transition = None;
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
    grid: GridInfo,
    objects: Vec<SemiCircle>,
    last_anim: Duration,
}

struct GridInfo {
    wh: Vec2,
    tile_size: f32,
    num_tiles: Vec2,
    slide_time: f32,
}

impl GridInfo {
    fn contains(&self, point: Vec2) -> bool {
        !(point.x >= self.num_tiles.x || point.x < 0.0 || point.y >= self.num_tiles.y || point.y < 0.0)
    }
}

struct SemiCircle {
    transition: Option<(Transition, Duration)>,
    pos: Vec2,
    orientation: f32,
    semi_circle_type: SemiCircleType,
    colour: usize,
}

impl SemiCircle {
    fn draw(&self, draw: &Draw, colours: &[LinSrgb], grid_info: &GridInfo, since_start: Duration) {
        let centre = self.get_pos(grid_info, since_start);
        let radius = grid_info.tile_size / 2.0;
        let orientation = self.get_orientation(grid_info, since_start);

        let segments = 200;

        let arc = |radius: f32| -> Vec<Vec2> {
            (0..=segments)
                .map(|i| {
                    let t = map_range(i, 0, segments, 0.0, PI) + orientation;
                    Vec2::new(t.cos(), t.sin()) * radius + centre
                })
                .collect()
        };

        match self.semi_circle_type {
            SemiCircleType::Filled => {
                draw.polygon()
                    .points(arc(radius))
                    .color(colours[self.colour]);
            }
            SemiCircleType::Striped => {
                let num_rings = 3;
                for i in 0..num_rings {
                    let i = i as f32;
                    let num_rings = num_rings as f32;
                    let incr = 1.0 / (2.0 * num_rings);
                    let new_radius = incr * (2.0 * i + 1.5);
                    draw.polyline()
                        .weight(radius * incr)
                        .points(arc(radius * new_radius))
                        .color(colours[self.colour]);
                }
            }
        }
    }


    fn get_pos(&self, grid: &GridInfo, since_start: Duration) -> Vec2 {
        let p = if let Some((Transition::Translation(p2), d)) = self.transition {
            let p1 = self.pos;
            let t = ((since_start.abs_diff(d)).as_secs_f32() / grid.slide_time).clamp(0.0, 1.0);
            p1.lerp(p2, ease(t))
        } else {
            self.pos
        };

        let bottom_left = -grid.wh / 2.0;
        let offset = Vec2::splat(grid.tile_size / 2.0);
        let tile = Vec2::splat(grid.tile_size) * p;
        bottom_left + offset + tile
    }

    fn get_orientation(&self, grid: &GridInfo, since_start: Duration) -> f32 {
        if let Some((Transition::Rotation(o2), d)) = self.transition {
            let o1 = self.orientation;
            let t = ((since_start.abs_diff(d)).as_secs_f32() / grid.slide_time).clamp(0.0, 1.0);
            lerp(o1, o2, ease(t))
        } else {
            self.orientation
        }
    }
}

impl GridInfo {
    fn new(app: &App) -> Self {
        let (width, height) = app.window_rect().w_h();

        let num_tiles_wide_u = 16_i32;
        let num_tiles_wide = num_tiles_wide_u as f32;

        let tile_size= width / num_tiles_wide;

        let num_tiles_tall = (height / tile_size).floor();
        let height = num_tiles_tall * tile_size;

        let wh = Vec2::new(width, height);
        let num_tiles = Vec2::new(num_tiles_wide, num_tiles_tall);

        Self {
            wh,
            tile_size,
            num_tiles,
            slide_time: 2.0,
        }
    }
}


fn new_objects(grid: &GridInfo, palette_len: usize) -> Vec<SemiCircle> {
    let mut objects = vec![];

    for x in 0..(grid.num_tiles.x as i32) {
        for y in 0..(grid.num_tiles.y as i32) {
            for _ in 0..2 {
                objects.push(SemiCircle {
                    transition: None,
                    pos: Vec2::new(x as f32, y as f32),
                    orientation: random_orientation(),
                    semi_circle_type: SemiCircleType::generate_random(),
                    colour: random_range(0, palette_len),
                })
            }
        }
    }

    objects
}

fn ease(x: f32) -> f32 {
    ease_in_out_elastic(x)
}

pub fn ease_in_out_elastic(x: f32) -> f32 {
    let c5 = (2.0 * PI) / 4.5;

    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else if x < 0.5 {
        -((2.0f32).powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0
    } else {
        ((2.0f32).powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0 + 1.0
    }
}

fn ease_out_circ(x: f32) -> f32 {
    (1.0 - (x - 1.0).powi(2)).sqrt()
}

pub fn ease_out_bounce(mut x: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if x < 1.0 / D1 {
        N1 * x * x
    } else if x < 2.0 / D1 {
        x -= 1.5 / D1;
        N1 * x * x + 0.75
    } else if x < 2.5 / D1 {
        x -= 2.25 / D1;
        N1 * x * x + 0.9375
    } else {
        x -= 2.625 / D1;
        N1 * x * x + 0.984375
    }
}

pub fn ease_in_bounce(x: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - x)
}

pub fn ease_in_out_bounce(x: f32) -> f32 {
    if x < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * x)) * 0.5
    } else {
        (1.0 + ease_out_bounce(2.0 * x - 1.0)) * 0.5
    }
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

    let _window = app.window(window_id).unwrap();
    // let egui = Egui::from_window(&window);

    let palette = palette();
    let grid = GridInfo::new(app);

    Model {
        _window: window_id,
        palette,
        objects: new_objects(&grid, palette.len()),
        grid,
        last_anim: Duration::from_millis(0),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw_palette_bg(app, model, &draw);

    for semi in model.objects.iter() {
        semi.draw(&draw, &model.palette, &model.grid, app.duration.since_start);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn draw_palette_bg(app: &App, model: &Model, draw: &Draw) {
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
    for object in model.objects.iter_mut() {
        if let Some((transition, start)) = &object.transition.clone() {
            if start.abs_diff(update.since_start).as_secs_f32() > slide_time * 1.2 {
                transition.finalise(object);
            }
        }
    }

    if model.last_anim.abs_diff(update.since_start).as_secs_f32() > slide_time {
        trigger(model, update);
        model.last_anim = update.since_start;
    }
}

fn trigger(model: &mut Model, update: Update) {
    let i = random_range(0, model.objects.len());
    let object = &mut model.objects[i];

    if object.transition.is_some() {
        return;
    }

    let transition = if random_f32() >= 0.5 {
        let rand_rot = if random_f32() >= 0.5 {
            FRAC_PI_2
        } else {
            -FRAC_PI_2
        };
        let new_orientation = object.orientation + rand_rot;
        Transition::Rotation(new_orientation)
    } else {
        let rand_dir = random_direction();
        let new_pos = object.pos + rand_dir;
        if !model.grid.contains(new_pos) {
            return
        }
        Transition::Translation(new_pos)
    };

    object.transition = Some((transition, update.since_start));
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
