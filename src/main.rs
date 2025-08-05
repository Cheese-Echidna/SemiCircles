mod easing;

use crate::easing::{ease_in_out_elastic, ease_out_elastic};
use nannou::prelude::*;
use nannou::rand;
use nannou::rand::prelude::SliceRandom;
use nannou::rand::thread_rng;
use nannou::winit::event::VirtualKeyCode;
use palette::{IntoColor, Okhsl};
use rand_derive2::RandGen;
use std::f32::consts::FRAC_PI_2;

const PALETTE_LEN: usize = 8;

fn main() {
    nannou::app(model).update(update).fullscreen().run();
}

// TODO:
//  - Maybe swap palette every 60 seconds
//  - Maybe have SC return to origin after some time?

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
            Transition::Rotation(r2) => object.orientation = r2,
            Transition::Translation(p2) => object.pos = p2,
        }
        object.transition = None;
    }
}

#[derive(Debug, RandGen)]
enum SemiCircleType {
    Filled,
    Striped,
}

type Palette = [LinSrgb; PALETTE_LEN];

struct Model {
    _window: WindowId,
    palette: Palette,
    grid: GridInfo,
    objects: Vec<SemiCircle>,
    last_anim_time: f32,
    last_p_swap_time: f32,
    new_palette: Palette,
}

struct GridInfo {
    wh: Vec2,
    tile_size: f32,
    num_tiles: Vec2,
    transition_duration: f32,
    transition_delay: f32,
}

impl GridInfo {
    fn contains(&self, point: Vec2) -> bool {
        !(point.x >= self.num_tiles.x
            || point.x < 0.0
            || point.y >= self.num_tiles.y
            || point.y < 0.0)
    }
}

struct SemiCircle {
    transition: Option<(Transition, f32)>,
    pos: Vec2,
    orientation: f32,
    semi_circle_type: SemiCircleType,
    colour: usize,
}

impl SemiCircle {
    fn draw(&self, draw: &Draw, colours: &[LinSrgb], grid_info: &GridInfo, since_start: f32) {
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

    fn get_pos(&self, grid: &GridInfo, since_start: f32) -> Vec2 {
        let p = if let Some((Transition::Translation(p2), d)) = self.transition {
            let p1 = self.pos;
            let t = ((since_start - d) / grid.transition_duration)
                .clamp(0.0, 1.0);
            p1.lerp(p2, ease_out_elastic(t))
        } else {
            self.pos
        };

        let bottom_left = -grid.wh / 2.0;
        let offset = Vec2::splat(grid.tile_size / 2.0);
        let tile = Vec2::splat(grid.tile_size) * p;
        bottom_left + offset + tile
    }

    fn get_orientation(&self, grid: &GridInfo, since_start: f32) -> f32 {
        if let Some((Transition::Rotation(o2), d)) = self.transition {
            let o1 = self.orientation;
            let t = ((since_start - d) / grid.transition_duration)
                .clamp(0.0, 1.0);
            lerp(o1, o2, ease_in_out_elastic(t))
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

        let tile_size = width / num_tiles_wide;

        let num_tiles_tall = (height / tile_size).floor();
        let height = num_tiles_tall * tile_size;

        let wh = Vec2::new(width, height);
        let num_tiles = Vec2::new(num_tiles_wide, num_tiles_tall);

        Self {
            wh,
            tile_size,
            num_tiles,
            transition_duration: 3.0,
            transition_delay: 1.0,
        }
    }
}

fn new_objects(grid: &GridInfo, palette_len: usize) -> Vec<SemiCircle> {
    let mut objects = vec![];

    for x in 0..(grid.num_tiles.x as i32) {
        for y in 0..(grid.num_tiles.y as i32) {
            for _ in 0..2 {
                // if random_f32() > 0.25 {
                objects.push(SemiCircle {
                    transition: None,
                    pos: Vec2::new(x as f32, y as f32),
                    orientation: random_orientation(),
                    semi_circle_type: SemiCircleType::generate_random(),
                    // colour: (x / 2) as usize,
                    colour: random_range(0, palette_len),
                })
                // }
            }
        }
    }

    objects.shuffle(&mut thread_rng());

    objects
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

    // app.set
    let _window = app.window(window_id).unwrap();
    _window.set_cursor_visible(false);
    // let egui = Egui::from_window(&window);

    let grid = GridInfo::new(app);

    Model {
        _window: window_id,
        palette: palette(),
        objects: new_objects(&grid, PALETTE_LEN),
        grid,
        last_anim_time: 0.0,
        last_p_swap_time: 0.0,
        new_palette: palette(),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw_palette_bg(app, model, &draw);
    // frame.clear(LIGHTYELLOW);

    for semi in model.objects.iter() {
        semi.draw(&draw, &model.palette, &model.grid, app.time);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn draw_palette_bg(app: &App, model: &Model, draw: &Draw) {
    let max = model.palette.len();
    // let swap_time = 10.0;
    // let offset = (app.time / swap_time).floor() as usize % max;
    let rect = app.window_rect();
    for i in 0..max {
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
        // draw.polygon().points(points).color(model.palette[(i + 4) % max]);
        draw.polygon().points(points).color(model.palette[i]);
    }
}

fn event(app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyReleased(key) => match key {
            VirtualKeyCode::R => model.palette = palette(),
            _ => {}
        },
        MouseMoved(_) => {
            if app.time > 0.2 {
                app.quit()
            }
            // move moves seem to be generated on fullscreen
        }
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for object in model.objects.iter_mut() {
        if let Some((transition, start)) = &object.transition.clone() {
            if app.time - start
                > model.grid.transition_duration * 1.2
            {
                transition.finalise(object);
            }
        }
    }

    if app.time - model.last_anim_time > model.grid.transition_delay {
        trigger(model, app.time);
        model.last_anim_time = app.time;
    }

    // just before swap
    let swap_spacing = 60.0;
    let seconds_left_until = model.last_p_swap_time + swap_spacing - app.time;
    let swap_time = 2.0;
    if seconds_left_until <= swap_time {
        let f = (1.0 - seconds_left_until / swap_time) * (PALETTE_LEN as f32);
        let u = f.floor() as usize;
        if !(u >= PALETTE_LEN) {
            model.palette[u] = model.new_palette[u];
        }
    }

    if model.last_p_swap_time + swap_spacing <= app.time {
        model.palette = model.new_palette;
        model.new_palette = palette();
        model.last_p_swap_time = app.time;
    }
}

fn trigger(model: &mut Model, time: f32) {
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
            return;
        }
        Transition::Translation(new_pos)
    };

    object.transition = Some((transition, time));
}

fn palette() -> [LinSrgb; PALETTE_LEN] {
    let h_init: f32 = random::<f32>();
    let h_range = 1.0;

    // let min_range = lerp(0.0, 0.35, random());
    // let max_range = lerp(0.65, 1.0, random());

    let min_range = 0.3;
    let max_range = 0.55;

    let max = PALETTE_LEN;

    [0; PALETTE_LEN]
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
