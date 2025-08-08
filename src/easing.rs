#![allow(dead_code)]

use std::f32::consts::PI;

pub fn ease_out_elastic(x: f32) -> f32 {
    const C4: f32 = (2.0 * PI) / 3.0;

    if x == 0.0 {
        0.0
    } else if x == 1.0 {
        1.0
    } else {
        (2.0f32).powf(-10.0 * x) * ((x * 10.0 - 0.75) * C4).sin() + 1.0
    }
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

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - ((-2.0 * x + 2.0).powi(3)) * 0.5
    }
}
