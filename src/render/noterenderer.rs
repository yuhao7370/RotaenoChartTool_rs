use macroquad::{time, ui::{self, Skin}};
use macroquad::prelude::*;

use crate::chart::chart::ChartProperties;
use crate::chart::{chart::Chart, traildistance::TrailDistance};
use crate::chart::note::{Note, Tap, Flick, Slide, Rotate, Catch, Trail, Bomb};
use crate::chart::bpm::BPM;
use crate::chart::speed::Speed;

use crate::render::texturemanager::TextureManager;
use crate::render::trailrenderer::{dwaw_arc, distance_to_radius};

pub struct NoteTextureManager {
    tap_texture: TextureManager,
    flick_texture: TextureManager,
    big_slide_texture: TextureManager,
    small_slide_texture: TextureManager,
    catch_texture: TextureManager,
    bomb_texture: TextureManager,
}

impl NoteTextureManager {
    pub async fn init(
        tap_texture_path: &str,
        flick_texture_path: &str,
        big_slide_texture_path: &str,
        small_slide_texture_path: &str,
        catch_texture_path: &str,
        bomb_texture_path: &str,
    ) -> Self {
        let tap_texture = TextureManager::new(tap_texture_path).await.unwrap_or_else(|_| TextureManager::default());
        let flick_texture = TextureManager::new(flick_texture_path).await.unwrap_or_else(|_| TextureManager::default());
        let big_slide_texture = TextureManager::new(big_slide_texture_path).await.unwrap_or_else(|_| TextureManager::default());
        let small_slide_texture = TextureManager::new(small_slide_texture_path).await.unwrap_or_else(|_| TextureManager::default());
        let catch_texture = TextureManager::new(catch_texture_path).await.unwrap_or_else(|_| TextureManager::default());
        let bomb_texture = TextureManager::new(bomb_texture_path).await.unwrap_or_else(|_| TextureManager::default());

        Self {
            tap_texture,
            flick_texture,
            small_slide_texture,
            big_slide_texture,
            catch_texture,
            bomb_texture,
        }
    }
}

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

    let mul = radius / 327.5;

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

pub fn draw_note(chart: Chart, chart_property: &ChartProperties, note_texture_manager: &NoteTextureManager, debug: bool) {
    // note部分
    for i in 0..chart.note.len() {
        // continue;
        let note = &chart.note[i];
        match note {
            Note::Slide(slide) => {
                let snaptime = 60.0 / chart.bpm[0].bpm * 1000.0;
                let time2 = slide.time + slide.amount as f32 * snaptime / slide.snap as f32;
                if time2 < chart_property.start_chart_time || slide.time > chart_property.end_chart_time{
                    continue;
                }
            }
            _ => {
                if (note.get_time() < chart_property.start_chart_time || note.get_time() > chart_property.end_chart_time){
                    continue;
                }
            }
        }

        match note {
            Note::Tap(tap) => { 
                let degree = tap.degree;
                let time = tap.time;
                let this_distance = chart.find_distance_by_time(time);
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);
                let truedegree  = 450.0 - degree;
                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());
                // 40,30 - 300,500
                let yscale: f32 = (radius / 327.5) * 300.0 + 40.0;
                let xscale: f32 = (radius / 327.5) * 420.0 + 30.0;
                // draw_text(&format!("beat: {:.3}", chart.chart_time_to_beat(time)), x, y, 20.0, WHITE);
                note_texture_manager.tap_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
            },
            Note::Flick(flick) => {
                let degree = flick.degree;
                let time = flick.time;
                let this_distance = chart.find_distance_by_time(time);
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);
                let truedegree  = 450.0 - degree;
                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());

                let yscale: f32 = (radius / 327.5) * 300.0 + 40.0;
                let xscale: f32 = (radius / 327.5) * 420.0 + 30.0;
                note_texture_manager.flick_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
            },
            Note::Slide(slide) => {
                let degree = slide.degree;
                let time = slide.time;
                let slidetype = slide.slidetype;
                let this_distance = chart.find_distance_by_time(time);
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);
                let truedegree  = 450.0 - degree;
                let yscale: f32 = (radius / 327.5) * 300.0 + 40.0;
                let xscale: f32 = (radius / 327.5) * 420.0 + 30.0;
                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());
                if slide.time > chart_property.start_chart_time && slide.time < chart_property.end_chart_time{
                    match slidetype{
                        0 => {
                            note_texture_manager.tap_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
                        },
                        1 => {
                            note_texture_manager.flick_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
                        },
                        2 => {
                            note_texture_manager.small_slide_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
                        },
                        3 => {
                            note_texture_manager.big_slide_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
                        },
                        _ => {}
                    }
                }

                for j in 1..slide.amount {
                    // 填4就是4分音符
                    let snaptime = 60.0 / chart.bpm[0].bpm * 1000.0;
                    let time = slide.time + j as f32 * snaptime / slide.snap as f32;
                    let enddegree = slide.end_degree;
                    let end_distance_1 = chart.find_distance_by_time(slide.time + slide.amount as f32 * snaptime / slide.snap as f32);
                    let this_distance_1 = chart.find_distance_by_time(time);
                    if this_distance_1 > chart_property.end_distance || this_distance_1 < chart_property.start_distance{
                        continue;
                    }
                    let progress = (time - slide.time) / (slide.amount as f32 * snaptime / slide.snap as f32);
                    let degree = chart.get_y_from_x(slide.degree, enddegree, slide.prev_curv / 100.0, slide.next_curv / 100.0, progress);
                    // let degree = slide.degree + (enddegree - slide.degree) * j as f32 / slide.amount as f32;
                    let radius = distance_to_radius(327.5, this_distance_1, chart_property.start_distance, chart_property.end_distance);
                    let truedegree  = 450.0 - degree;
                    let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());
                    // 40,30 - 300,500
                    let yscale: f32 = (radius / 327.5) * 300.0 + 40.0;
                    let xscale: f32 = (radius / 327.5) * 420.0 + 30.0;
                    note_texture_manager.small_slide_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree + 90.0);
                }
            },
            Note::Rotate(rotate) => {
                // println!("rotate");
                let degree = rotate.degree;
                let time = rotate.time;
                let this_distance = chart.find_distance_by_time(time);
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);
                let truedegree  = 450.0 - degree;
                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());
                // draw_circle(x, y, 5.0, WHITE);
                
                draw_rotate_simple(600.0, 400.0, radius, degree, rotate.delta);
            },
            Note::Catch(catch) => {
                let deg = chart.find_degree_by_time(catch.time);
                let mut degree = if deg <= 180.0 && deg > 0.0 {deg} else {deg - 180.0};
                // let mut degree = deg % 180.0;
                degree += catch.degree * 180.0;
                let time = catch.time;
                let this_distance = chart.find_distance_by_time(time);
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);
                let truedegree  = 450.0 - degree;
                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());

                let yscale: f32 = (radius / 327.5) * 300.0 + 40.0;
                let xscale: f32 = (radius / 327.5) * 420.0 + 30.0;
                note_texture_manager.catch_texture.middle_draw(x, y, xscale / 750.0, yscale / 750.0, degree - 90.0);
                // draw_text(&format!("deg: {}", deg), x, y, 20.0, GREEN);
            },
            Note::Trail(trail) => {
                continue;
                let degree = trail.degree;
                let time = trail.time;
                let this_distance = chart.find_distance_by_time(time);
                let mut truedegree  = 450.0 - degree;
                let radius = distance_to_radius(327.5, this_distance, chart_property.start_distance, chart_property.end_distance);

                let (x, y) = (600.0 + radius * truedegree.to_radians().cos(), 400.0 - radius * truedegree.to_radians().sin());
                // draw_circle(x, y, 8.0, GREEN);
            },
            Note::Bomb(bomb) => {
                continue;
            },
            _ => {}
            
        }
    }


}