use nannou::rand;
use std::f32::consts::FRAC_PI_2;
use nannou::prelude::*;
use nannou::winit::event::VirtualKeyCode;
use palette::{IntoColor, Okhsl};
use rand_derive2::RandGen;

fn main() {
    nannou::app(model)
        .update(update)
        .fullscreen()
        .run();
}

#[derive(Debug, RandGen)]
enum Orientation {
    Left, Right, Top, Bottom
}

impl Orientation {
    fn angle(&self) -> f32 {
        (match self {
            Orientation::Left => {0.5}
            Orientation::Right => {1.5}
            Orientation::Top => {0.0}
            Orientation::Bottom => {1.0}
        }) * PI
    }
}

#[derive(Debug, RandGen)]
enum SemiCircleType {
    Filled, Striped
}

struct Model {
    _window: WindowId,
    palette: [LinSrgb; 8],
    grid: Grid
}

#[derive(Debug)]
struct Grid {
    width: f32,
    tile_size: f32,
    num_tiles_wide: f32,
    objects: Vec<SemiCircle>
}

#[derive(Debug)]
struct SemiCircle {
    pos: IVec2,
    orientation: Orientation,
    semi_circle_type: SemiCircleType,
    colour: usize,
}

impl SemiCircle {
    fn draw(&self, draw: &Draw, centre: Vec2, radius: f32, colours: &[LinSrgb]) {
        let segments = 100;
        let points: Vec<Vec2> = (0..=segments)
            .map(|i| {
                let t = map_range(i, 0, segments, 0.0, PI) + self.orientation.angle();
                Vec2::new(t.cos(), t.sin()) * radius + centre
            })
            .collect();
        draw.polygon().points(points).color(colours[self.colour]);
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
                    objects.push(SemiCircle{
                        pos:IVec2::new(x, y),
                        orientation: Orientation::generate_random(),
                        semi_circle_type: SemiCircleType::generate_random(),
                        colour: random_range(0, palette_len)
                    })
                }
            }
        }

        Self {
            width,
            tile_size: width / num_tiles_wide,
            num_tiles_wide,
            objects,
        }
    }
    fn pos_of(&self, p: IVec2) -> Vec2 {
        let bottom_left = -Vec2::splat(self.width / 2.0);
        let offset = Vec2::splat(self.tile_size / 2.0);
        let tile = Vec2::splat(self.tile_size) * p.as_f32();
        bottom_left + offset + tile
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

    let window = app.window(window_id).unwrap();
    // let egui = Egui::from_window(&window);

    let palette = palette();
    let grid = Grid::new(app, palette.len());

    // println!("{:?}", grid);
    println!("{:?}", grid.objects.iter().map(|semi| grid.pos_of(semi.pos)).collect::<Vec<Vec2>>());
    // println!("{:?}", grid.objects);


    // println!("{}", palette.iter().map(|x| format!("({}, {}, {})", x.red, x.green, x.blue)).collect::<Vec<String>>().join("\n"));

    Model {
        _window: window_id,
        palette,
        grid,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    view_semi(app, model, frame);
}

fn view_semi(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    frame.clear(DARKBLUE);

    for semi in model.grid.objects.iter() {
        semi.draw(&draw, model.grid.pos_of(semi.pos), model.grid.tile_size / 2.0, &model.palette);
        // draw.ellipse().xy(model.grid.pos_of(semi.pos)).color(RED).radius(3.0);
    }

    draw.to_frame(app, &frame).unwrap()
}

fn view_palette(app: &App, model: &Model, frame: Frame) {
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
    let h_range = 0.8;

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
            (lerp(h_init, h_init + h_range, t) % 1.0, c.0, c.1)
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
