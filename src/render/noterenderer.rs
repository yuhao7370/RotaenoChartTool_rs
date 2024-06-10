use macroquad::{time, ui::{self, Skin}};
use crate::chart::note::{Tap, Flick, Slide, Rotate, Catch, Trail};
use macroquad::prelude::*;

fn draw_arc(x: f32, y: f32, radius: f32, start: f32, end: f32, thickness: f32, color: Color) {
    let mut last_point = Vec2::new(x + radius * start.to_radians().cos(), y - radius * start.to_radians().sin());
    for i in 1..=100 {
        let point = Vec2::new(x + radius * (start + (end - start) * i as f32 / 100.0).to_radians().cos(), y - radius * (start + (end - start) * i as f32 / 100.0).to_radians().sin());
        draw_line(last_point.x, last_point.y, point.x, point.y, thickness, color);
        last_point = point;
    }
}

pub fn draw_rotate_simple(x: f32, y: f32, radius: f32, degree: f32, delta: f32) {
    let mut true_degree = (450.0 - degree) % 360.0;
    if true_degree < 0.0 {
        true_degree += 360.0;
    }

    let mul = radius / 325.0;

    let color = if delta > 0.0{
        RED
    } else{
        BLUE
    };
    // 画50像素粗的圆弧
    draw_arc(x, y, radius, true_degree, true_degree - delta, 40.0 * mul, color);
    // 画5像素粗的圆弧
    draw_arc(x, y, radius, true_degree - delta, true_degree + 180.0, 10.0 * mul, color);
    // 画50像素粗的圆弧
    draw_arc(x, y, radius, true_degree - delta + 180.0, true_degree + 180.0, 40.0 * mul, color);
    // 画5像素粗的圆弧
    draw_arc(x, y, radius, true_degree - delta + 180.0, true_degree + 360.0, 10.0 * mul, color);
    // println!("success")
}