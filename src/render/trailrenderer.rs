use macroquad::{time, ui::{self, Skin}};
use crate::chart::note::{Note, Tap, Flick, Slide, Rotate, Catch, Trail};
use crate::chart::{chart::Chart, traildistance::TrailDistance};
use macroquad::prelude::*;

pub fn dwaw_arc(x: f32, y: f32, radius: f32, start: f32, end: f32, thickness: f32, color: Color){
    let mut last_point = Vec2::new(x + radius * start.to_radians().cos(), y - radius * start.to_radians().sin());
    for i in 1..=100 {
        let point = Vec2::new(x + radius * (start + (end - start) * i as f32 / 100.0).to_radians().cos(), y - radius * (start + (end - start) * i as f32 / 100.0).to_radians().sin());
        draw_line(last_point.x, last_point.y, point.x, point.y, thickness, color);
        last_point = point;
    }
}

pub fn distance_to_radius(max_radius: f32, distance: f32, start_distance: f32, end_distance: f32) -> f32 {
    let x: f32 = (distance - start_distance) / (end_distance - start_distance); //还有%多少到判定区

    let x1 = 1.0 - x;

    fn func(x: f32) -> f32 {
        let (a, b, c, d) = (1.4354973119363346,1.7027444700980798,1.154638454723781,-0.11566204049406854);
    
        a * (b * (x - c)).exp() + d
    }
    
    max_radius * func(x1) // 使用 powf 函数的倒数来实现变化率随 (distance - start_distance) 减小而增大
}

pub fn draw_trail(chart: &Chart, start_distance: f32, end_distance: f32, debug: bool){
    let color = Color::new(1.0, 1.0, 1.0, 0.5);

    for i in 0..chart.trail_distance.len() - 1{
        let mut trail: TrailDistance;
        let mut next_trail:TrailDistance;
        trail = chart.trail_distance[i]; // 获取当前下标的trail
        next_trail = chart.trail_distance[i + 1]; // 获取下一个trail

        if next_trail.distance < start_distance || trail.distance > end_distance {
            continue; // 如果下一个trail的距离小于显示的起始距离，或者当前trail的距离大于显示的结束距离，那么就不显示
            // 该结果已经把所有没有交集的情况都排除了
        }

        if trail.distance >= start_distance && next_trail.distance <= end_distance {
            // 全部在内
            for i in 0..100 {
                let this_distance1 = trail.distance + (next_trail.distance - trail.distance) * i as f32 / 100.0;
                let this_distance2 = trail.distance + (next_trail.distance - trail.distance) * (i + 1) as f32 / 100.0;
                let time1 = chart.find_time_by_distance(this_distance1);
                let time2 = chart.find_time_by_distance(this_distance2);

                let radius1 = distance_to_radius(325.0, this_distance1, start_distance, end_distance);
                let progress1 = (this_distance1 - trail.distance) / (next_trail.distance - trail.distance);
                let degree1 = chart.find_degree_by_2_trails(trail, next_trail, progress1);
                let truedegree1  = 450.0 - degree1;
                let (x1, y1) = (600.0 + radius1 * truedegree1.to_radians().cos(), 400.0 - radius1 * truedegree1.to_radians().sin());

                let radius2 = distance_to_radius(325.0, this_distance2, start_distance, end_distance);
                let progress2 = (this_distance2 - trail.distance) / (next_trail.distance - trail.distance);
                let degree2 = chart.find_degree_by_2_trails(trail, next_trail, progress2);
                let truedegree2  = 450.0 - degree2;
                let (x2, y2) = (600.0 + radius2 * truedegree2.to_radians().cos(), 400.0 - radius2 * truedegree2.to_radians().sin());
                
                draw_line(x1, y1, x2, y2, 4.0, color);
                draw_line(1200.0 - x1, 800.0 - y1, 1200.0 - x2, 800.0 - y2, 4.0, color);
            }
        }
        else if trail.distance < start_distance && next_trail.distance > start_distance && next_trail.distance < end_distance {
            // 尾在内头不在内
            for i in 0..100 {
                let this_distance1 = start_distance + (next_trail.distance - start_distance) * i as f32 / 100.0;
                let this_distance2 = start_distance + (next_trail.distance - start_distance) * (i + 1) as f32 / 100.0;
                let time1 = chart.find_time_by_distance(this_distance1);
                let time2 = chart.find_time_by_distance(this_distance2);
                if(this_distance1 < start_distance || this_distance2 > end_distance){
                    continue;
                }
                let radius1 = distance_to_radius(325.0, this_distance1, start_distance, end_distance);
                let progress1 = (this_distance1 - trail.distance) / (next_trail.distance - trail.distance);
                let degree1 = chart.find_degree_by_2_trails(trail, next_trail, progress1);
                let truedegree1  = 450.0 - degree1;
                let (x1, y1) = (600.0 + radius1 * truedegree1.to_radians().cos(), 400.0 - radius1 * truedegree1.to_radians().sin());

                
                let radius2 = distance_to_radius(325.0, this_distance2, start_distance, end_distance);
                let progress2 = (this_distance2 - trail.distance) / (next_trail.distance - trail.distance);
                let degree2 = chart.find_degree_by_2_trails(trail, next_trail, progress2);
                let truedegree2  = 450.0 - degree2;
                let (x2, y2) = (600.0 + radius2 * truedegree2.to_radians().cos(), 400.0 - radius2 * truedegree2.to_radians().sin());
            
                draw_line(x1, y1, x2, y2, 4.0, color);
                draw_line(1200.0 - x1, 800.0 - y1, 1200.0 - x2, 800.0 - y2, 4.0, color);
                // draw_text(&format!("{:.2}", progress1), x1, y1, 20.0, WHITE);
            }              
        }
        else if trail.distance > start_distance && trail.distance < end_distance && next_trail.distance > end_distance {
            // draw_text(&format!("{:.1} {:.1}", trail.distance, next_trail.distance), 200.0, 200.0, 20.0, WHITE);
            // 头在内尾不在内
            for i in 0..100 {
                let this_distance1 = trail.distance + (end_distance - trail.distance) * i as f32 / 100.0;
                let this_distance2 = trail.distance + (end_distance - trail.distance) * (i + 1) as f32 / 100.0;
                let time1 = chart.find_time_by_distance(this_distance1);
                let time2 = chart.find_time_by_distance(this_distance2);
                if(this_distance1 < start_distance || this_distance2 > end_distance){
                    continue;
                }

                let radius1 = distance_to_radius(325.0, this_distance1, start_distance, end_distance);
                let progress1 = (this_distance1 - trail.distance) / (next_trail.distance - trail.distance);
                let degree1 = chart.find_degree_by_2_trails(trail, next_trail, progress1);
                let truedegree1  = 450.0 - degree1;
                let (x1, y1) = (600.0 + radius1 * truedegree1.to_radians().cos(), 400.0 - radius1 * truedegree1.to_radians().sin());

                let radius2 = distance_to_radius(325.0, this_distance2, start_distance, end_distance);
                let progress2 = (this_distance2 - trail.distance) / (next_trail.distance - trail.distance);
                let degree2 = chart.find_degree_by_2_trails(trail, next_trail, progress2);
                let truedegree2  = 450.0 - degree2;
                let (x2, y2) = (600.0 + radius2 * truedegree2.to_radians().cos(), 400.0 - radius2 * truedegree2.to_radians().sin());
            
                draw_line(x1, y1, x2, y2, 4.0, color);
                // draw_circle(x1, y1, 5.0, RED);
                // draw_circle(x2, y2, 5.0, BLUE);
                draw_line(1200.0 - x1, 800.0 - y1, 1200.0 - x2, 800.0 - y2, 4.0, color);
                
            }
        }
        else{
            for i in 0..100 {
                let this_distance1 = start_distance + (end_distance - start_distance) * i as f32 / 100.0;
                let this_distance2 = start_distance + (end_distance - start_distance) * (i + 1) as f32 / 100.0;
                let time1 = chart.find_time_by_distance(this_distance1);
                let time2 = chart.find_time_by_distance(this_distance2);
                if this_distance2 > end_distance || this_distance1 < start_distance{
                    continue;
                }
                let radius1 = distance_to_radius(325.0, this_distance1, start_distance, end_distance);
                let progress1 = (this_distance1 - trail.distance) / (next_trail.distance - trail.distance);
                let degree1 = chart.find_degree_by_2_trails(trail, next_trail, progress1);
                let truedegree1  = 450.0 - degree1;
                let (x1, y1) = (600.0 + radius1 * truedegree1.to_radians().cos(), 400.0 - radius1 * truedegree1.to_radians().sin());

                let radius2 = distance_to_radius(325.0, this_distance2, start_distance, end_distance);
                let progress2 = (this_distance2 - trail.distance) / (next_trail.distance - trail.distance);
                let degree2 = chart.find_degree_by_2_trails(trail, next_trail, progress2);
                let truedegree2  = 450.0 - degree2;
                let (x2, y2) = (600.0 + radius2 * truedegree2.to_radians().cos(), 400.0 - radius2 * truedegree2.to_radians().sin());
            
                draw_line(x1, y1, x2, y2, 4.0, color);
                draw_line(1200.0 - x1, 800.0 - y1, 1200.0 - x2, 800.0 - y2, 4.0, color);
                // draw_text(&format!("{:.2}", progress1), x1, y1, 20.0, WHITE);
            }
        }
    }
    
    let start_chart_time = chart.find_time_by_distance(start_distance);
    let end_chart_time = chart.find_time_by_distance(end_distance);
    for i in 0..chart.note.len() {
        // continue;
        let note = &chart.note[i];
        if chart.find_distance_by_time(note.get_time()) < start_distance || chart.find_distance_by_time(note.get_time()) > end_distance{
            continue;
        }
        match note {
            Note::Tap(tap) => { 
                continue;
            },
            Note::Flick(flick) => {
                continue;
            },
            Note::Slide(slide) => {
                continue;
            },
            Note::Rotate(rotate) => {
                continue;
            },
            Note::Catch(catch) => {
                continue;
            },
            Note::Trail(trail) => {
                let degree = trail.degree;
                let time = trail.time;
                let this_distance = chart.find_distance_by_time(time);
                let mut truedegree  = 450.0 - degree;
                let radius = distance_to_radius(325.0, this_distance, start_distance, end_distance);

                if trail.delta != 0.0 { // 画弧的情况
                    if this_distance < start_distance || this_distance > end_distance { // 弧必须在显示范围内
                        continue;
                    }
                    let thickness = 3.0 + 3.0 * (1.0 - (this_distance - start_distance) / (end_distance - start_distance));
                    if trail.delta.abs() >= 180.0{
                        dwaw_arc(600.0, 400.0, radius, 0.0, 360.0, thickness, color);
                    }
                    else{
                        dwaw_arc(600.0, 400.0, radius, truedegree, truedegree - trail.delta, thickness, color);
                        dwaw_arc(600.0, 400.0, radius, truedegree + 180.0, truedegree - trail.delta + 180.0, thickness, color);
                    }
                }

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