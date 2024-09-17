// chart/chart.rs

// BPM:
// 		time/BPM
// 	Speed:
// 		time/speed/smooth
// 	Note:
// 		0(tap)/time/degree
// 		1(flick)/time/degree
// 		2(slide)/time/degree/slidetype/end_degree/snap/amount/prev_curv/next_curv
// 		4(rotate)/time/degree/delta/prev_curv/next_curv
// 		5(catch)/time/degree
//      6(bomb)/time/degree
// 		11(trail)/time/degree/delta/prev_curv/next_curv

use core::time;
use std::fs::File;
use std::io::{self, BufRead, Error};
use std::io::{Write, Read};
use std::path::Path;
use std::io::Result as IoResult;
use std::rc::Rc;


use serde::{de, Deserialize, Serialize};
use serde_json::{Result, Value, json};
use serde_json::ser::{Serializer, PrettyFormatter};

use log::{info, warn, error};

use super::bpm::BPM;
use super::speed::Speed;
use super::note::Note;

use super::note::Tap;
use super::note::Flick;
use super::note::Slide;
use super::note::Rotate;
use super::note::Catch;
use super::note::Trail;
use super::note::Bomb;

use super::speeddistance::{self, SpeedDistance};
use super::traildistance::{self, TrailDistance};

#[derive(Clone)]
pub struct HitSound {
    pub time: f32,
    pub note_type: i32,
    pub played: bool,
} // 这里要注意，note_type跟谱面里不太一样，0是tap，1是flick，2是slide(谱面里2和3是slise)，3和4是rotateL和rotateR，5是catch

impl HitSound {
    pub fn new(time: f32, note_type: i32) -> Self {
        Self {
            time,
            note_type,
            played: false,
        }
    }

    pub fn play(&mut self) {
        self.played = true;
    }
}

#[derive(Clone)]
pub struct Chart {
    pub version: i32,
    pub offset: f32,
    pub bpm: Vec<BPM>,
    pub speed: Vec<Speed>,
    pub note: Vec<Note>,
    pub speed_distance: Vec<SpeedDistance>,
    pub speed_distance_plain: Vec<SpeedDistance>,
    pub trail_distance: Vec<TrailDistance>,
    pub trail_distance_plain: Vec<TrailDistance>,

    pub hitsound_list: Vec<HitSound>,

    pub phone_trail_distance: Vec<TrailDistance>,
    // pub single_trail_distance: Vec<TrailDistance>,
}

pub struct ChartProperties {
    pub start_chart_time: f32,
    pub end_chart_time: f32,
    pub start_distance: f32,
    pub end_distance: f32,
    pub show_distance: f32,
    pub cur_degree: f32,
}
impl ChartProperties {
    pub fn new() -> Self {
        Self {
            start_chart_time: 0.0,
            end_chart_time: 0.0,
            start_distance: 0.0,
            end_distance: 0.0,
            show_distance: 0.0,
            cur_degree: 0.0,
        }
    }
    
    pub fn init(start_chart_time: f32, end_chart_time: f32, start_distance: f32, end_distance: f32, show_distance: f32, cur_degree: f32) -> Self {
        Self {
            start_chart_time,
            end_chart_time,
            start_distance,
            end_distance,
            show_distance,
            cur_degree,
        }
    }
}


pub enum Section {
    None,
    BpmSection,
    SpeedSection,
    NoteSection,
}

pub trait DistanceGetter {
    fn get_distance(&self) -> f32;
}

impl DistanceGetter for SpeedDistance {
    fn get_distance(&self) -> f32 {
        self.distance
    }
}

impl DistanceGetter for TrailDistance {
    fn get_distance(&self) -> f32 {
        self.distance
    }
}

pub trait       TimeGetter {
    fn get_time(&self) -> f32;
}

impl TimeGetter for TrailDistance {
    fn get_time(&self) -> f32 {
        self.time
    }
}

impl TimeGetter for SpeedDistance {
    fn get_time(&self) -> f32 {
        self.time
    }
}

impl TimeGetter for Speed {
    fn get_time(&self) -> f32 {
        self.time
    }
}

impl TimeGetter for Note {
    fn get_time(&self) -> f32 {
        self.get_time()
    }
}

impl TimeGetter for BPM {
    fn get_time(&self) -> f32 {
        self.time
    }
}

impl Chart {
    pub fn create_empty_chart() -> Self {
        Self {
            version: 0,
            offset: 0.0,
            bpm: Vec::new(),
            speed: Vec::new(),
            note: Vec::new(),
            speed_distance: Vec::new(),
            speed_distance_plain: Vec::new(),
            trail_distance: Vec::new(),
            trail_distance_plain: Vec::new(),

            hitsound_list: Vec::new(),

            phone_trail_distance: Vec::new(),
            // single_trail_distance: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.sort_chart();
        self.distance_preprocessing();
        self.update_plain_chart();
        self.update_hitsound();
    }

    pub fn reset_hitsound(&mut self, chart_time: f32){
        for i in 0..self.hitsound_list.len() {
            if self.hitsound_list[i].time >= chart_time {
                self.hitsound_list[i].played = false;
            }
            else{
                self.hitsound_list[i].played = true;
            }
        }
    }

    pub fn update_hitsound(&mut self){
        self.hitsound_list.clear();
        for i in 0..self.note.len() {
            let note = &self.note[i];
            match note {
                Note::Tap(tap) => { 
                    let time = tap.time;
                    let note_type = 0;
                    let hitsound = HitSound::new(time, note_type);
                    self.hitsound_list.push(hitsound);
                },
                Note::Flick(flick) => {
                    let degree = flick.degree;
                    let time = flick.time;
                    let note_type = 1;
                    let hitsound = HitSound::new(time, note_type);
                    self.hitsound_list.push(hitsound);
                },
                Note::Slide(slide) => {
                    let degree = slide.degree;
                    let time = slide.time;
                    let slidetype = slide.slidetype;
                    if slide.slidetype != 3 {
                        let hitsound = HitSound::new(time, slidetype);
                        self.hitsound_list.push(hitsound);
                    } else {
                        let hitsound = HitSound::new(time, 2);
                        self.hitsound_list.push(hitsound);
                    }

                    for j in 1..slide.amount {
                        // 填4就是4分音符
                        // let snaptime = 60.0 / self.bpm[0].bpm * 1000.0; // TODO chart get bpm
                        let snaptime = 60.0 / self.find_bpm_by_time(slide.time) * 1000.0;
                        let time = slide.time + j as f32 * snaptime / slide.snap as f32;
                        let hitsound = HitSound::new(time, 2);
                        self.hitsound_list.push(hitsound);
                    }
                },
                Note::Rotate(rotate) => {
                    let delta = rotate.delta;
                    let time = rotate.time;
                    let note_type = if delta < 0.0 { 3 } else { 4 };
                    let hitsound = HitSound::new(time, note_type);
                    self.hitsound_list.push(hitsound);
                },
                Note::Catch(catch) => {
                    let time = catch.time;
                    let note_type = 5;
                    let hitsound = HitSound::new(time, note_type);
                    self.hitsound_list.push(hitsound);
                    // draw_text(&format!("deg: {}", deg), x, y, 20.0, GREEN);
                },
                Note::Trail(trail) => {
                    continue;
                },
                Note::Bomb(bomb) => {
                    continue;
                },
                _ => {}
                
            }
        }
        self.hitsound_list.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn update_plain_chart(&mut self){
        let mut plain_chart = self.clone();
        plain_chart.speed = vec![Speed::new(0.0, 1.0, 0)];
        plain_chart.trail_distance = vec![];
        plain_chart.speed_distance = vec![];
        plain_chart.distance_preprocessing();
        self.trail_distance_plain = plain_chart.trail_distance;
        self.speed_distance_plain = plain_chart.speed_distance;
    }

    pub fn get_plain_chart(&self) -> Chart{
        let mut plain_chart = self.clone();
        plain_chart.speed = vec![Speed::new(0.0, 1.0, 0)];
        plain_chart.trail_distance = plain_chart.trail_distance_plain.clone();
        plain_chart.speed_distance = plain_chart.speed_distance_plain.clone();
        plain_chart
    }

    pub fn load_chart_from_official(path: &str) -> io::Result<Self> {
        log::info!("Loading chart from official format: {}", path);
        let file: File = File::open(Path::new(path))?;
        let reader: io::BufReader<File> = io::BufReader::new(file);

        let mut section: Section = Section::None;
        let mut chart: Chart = Chart::create_empty_chart();

        for line in reader.lines() {
            let line: String = line?;
            match line.as_str() {
                _ if line.starts_with("# Version") => chart.version = line.split_whitespace().last().unwrap().parse().unwrap(),
                _ if line.starts_with("# BPM") => section = Section::BpmSection,
                _ if line.starts_with("# Speed") => section = Section::SpeedSection,
                _ if line.starts_with("# Note") => section = Section::NoteSection,
                _ => match section {
                    Section::BpmSection => {
                        let parts: Vec<&str> = line.split(',').collect();
                        if line.is_empty(){
                            log::warn!("Empty line, Skipping");
                            continue;
                        }
                        if parts.len() != 2 {
                            log::error!("Invalid BPM line: {}, Skipping", line);
                            continue;
                        }
                        chart.bpm.push(BPM::new(parts[0].parse().unwrap(), parts[1].parse().unwrap()));
                    }
                    Section::SpeedSection => {
                        let parts: Vec<&str> = line.split(',').collect();
                        if line.is_empty(){
                            log::warn!("Empty line, Skipping");
                            continue;
                        }
                        if parts.len() != 3 {
                            log::error!("Invalid Speed line: {}, Skipping", line);
                            continue;
                        }
                        chart.speed.push(Speed::new(parts[0].parse().unwrap(), parts[1].parse().unwrap(), parts[2].parse().unwrap()));
                    }
                    Section::NoteSection => {
                        let parts: Vec<&str> = line.split(',').collect();
                        if line.is_empty() {
                            log::warn!("Empty line, Skipping");
                            continue;
                        }
                        if parts.len() < 3 {
                            log::error!("Invalid Note line: {}, Skipping", line);
                            continue;
                        }
                        let note_type: i32 = parts[0].parse().unwrap();
                        let note = match note_type {
                            0 => {
                                if parts.len() != 3{log::error!("Invalid Tap note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Tap note");
                                Note::Tap(Tap::new(parts[1].parse().unwrap(), parts[2].parse().unwrap()))
                            },
                            1 => {
                                if parts.len() != 3{log::error!("Invalid Flick note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Flick note");
                                Note::Flick(Flick::new(parts[1].parse().unwrap(), parts[2].parse().unwrap()))
                            },
                            2 => {
                                if parts.len() != 9{log::error!("Invalid Slide note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Slide note");
                                Note::Slide(Slide::new(parts[1].parse().unwrap(), parts[2].parse().unwrap(), parts[3].parse().unwrap(), parts[4].parse().unwrap(), parts[5].parse().unwrap(), parts[6].parse().unwrap(), parts[7].parse().unwrap(), parts[8].parse().unwrap()))
                            },
                            4 => {
                                if parts.len() != 6{log::error!("Invalid Rotate note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Rotate note");
                                Note::Rotate(Rotate::new(parts[1].parse().unwrap(), parts[2].parse().unwrap(), parts[3].parse().unwrap(), parts[4].parse().unwrap(), parts[5].parse().unwrap()))
                            },
                            5 => {
                                if parts.len() != 3{log::error!("Invalid Catch note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Catch note");
                                Note::Catch(Catch::new(parts[1].parse().unwrap(), parts[2].parse().unwrap()))
                            },
                            6 => {
                                if parts.len() != 3{log::error!("Invalid Bomb note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Bomb note");
                                Note::Bomb(Bomb::new(parts[1].parse().unwrap(), parts[2].parse().unwrap()))
                            },
                            11 => {
                                if parts.len() != 6{log::error!("Invalid Trail note line: {}, Skipping", line); continue;}
                                // log::info!("Parsing Trail note");
                                Note::Trail(Trail::new(parts[1].parse().unwrap(), parts[2].parse().unwrap(), parts[3].parse().unwrap(), parts[4].parse().unwrap(), parts[5].parse().unwrap()))
                            },
                            _ => {
                                log::warn!("Unknown note type: {}, Skipping", note_type);
                                continue;
                            },
                        };
                        chart.note.push(note);
                    }
                    Section::None => (),
                },
            }
        }
        chart.update_hitsound();
        chart.distance_preprocessing();
        Ok(chart)
    }

    pub fn load_chart_from_json(path: &str) -> io::Result<Self> {
        log::info!("Loading chart from json format: {}", path);
        let mut file: File = File::open(path)?;
        let mut data: String = String::new();
        file.read_to_string(&mut data)?;
        let json: Value = serde_json::from_str(&data)?;
        let mut chart: Chart = Chart::create_empty_chart();
        chart.version = json["version"].as_i64().unwrap() as i32;
        chart.offset = json["offset"].as_f64().unwrap() as f32;
        for bpm in json["bpm"].as_array().unwrap() {
            chart.bpm.push(serde_json::from_value(bpm.clone()).unwrap());
        }
        for note in json["note"].as_array().unwrap() {
            let note_type: i32 = note["type"].as_i64().unwrap() as i32;
            let note = match note_type {
                0 => Note::Tap(serde_json::from_value(note.clone()).unwrap()),
                1 => Note::Flick(serde_json::from_value(note.clone()).unwrap()),
                2 => Note::Slide(serde_json::from_value(note.clone()).unwrap()),
                4 => Note::Rotate(serde_json::from_value(note.clone()).unwrap()),
                5 => Note::Catch(serde_json::from_value(note.clone()).unwrap()),
                6 => Note::Bomb(serde_json::from_value(note.clone()).unwrap()),
                11 => Note::Trail(serde_json::from_value(note.clone()).unwrap()),
                _ => {
                    log::warn!("Unknown note type: {}, Skipping", note_type);
                    continue;
                },
            };
            chart.note.push(note);
        }
        chart.distance_preprocessing();
        chart.update_hitsound();
        Ok(chart)
    }

    // 这个函数的作用是对bpm、speed、note进行排序
    pub fn sort_chart(&mut self) {
        self.bpm.sort_by(|a: &BPM, b: &BPM| a.time.partial_cmp(&b.time).unwrap());
        self.speed.sort_by(|a: &Speed, b: &Speed| a.time.partial_cmp(&b.time).unwrap());
        self.note.sort_by(|a: &Note, b: &Note| a.get_time().partial_cmp(&b.get_time()).unwrap());

        self.speed_distance.sort_by(|a: &SpeedDistance, b: &SpeedDistance| a.time.partial_cmp(&b.time).unwrap());
        self.trail_distance.sort_by(|a: &TrailDistance, b: &TrailDistance| {
            if a.distance < b.distance {
                std::cmp::Ordering::Less
            } else if a.distance == b.distance {
                a.time.partial_cmp(&b.time).unwrap()
            } else {
                std::cmp::Ordering::Greater
            }
        });

        self.phone_trail_distance.sort_by(|a: &TrailDistance, b: &TrailDistance| {
            if a.distance < b.distance {
                std::cmp::Ordering::Less
            } else if a.distance == b.distance {
                a.time.partial_cmp(&b.time).unwrap()
            } else {
                std::cmp::Ordering::Greater
            }
        });
    }

    // 这个函数的作用是根据实际时间计算谱面时间
    pub fn real_time_to_chart_time(&self, real_time: f32) -> f32 {
        let chart_time: f32 = real_time * 1000.0;
        chart_time
    }

    // 这个函数的作用是根据谱面时间计算实际时间
    pub fn chart_time_to_real_time(&self, chart_time: f32) -> f32 {
        let real_time: f32 = chart_time / (1000.0);
        real_time
    }

    // 这个函数的作用是根据实际时间计算小节数
    pub fn real_time_to_beat(&self, real_time: f32) -> f32 {
        let mut beat = 0.0;
        let mut last_time = 0.0;
        let mut last_bpm = 0.0;

        for bpm in &self.bpm {
            if real_time < bpm.time {
                break;
            }
            beat += (bpm.time - last_time) / 60.0 * last_bpm;
            last_time = bpm.time;
            last_bpm = bpm.bpm;
        }

        // 如果 real_time 超过了最后一个 BPM 段的时间
        if real_time > last_time {
            beat += (real_time - last_time) / 60.0 * last_bpm;
        }

        beat
    }
    
    // 这个函数的作用是根据小节数计算实际时间
    pub fn beat_to_real_time(&self, beat: f32) -> f32 {
        let mut real_time = 0.0;
        let mut last_time = 0.0;
        let mut last_bpm = 0.0;
        let mut accumulated_beat = 0.0;

        for bpm in &self.bpm {
            let segment_beat = (bpm.time - last_time) / 60.0 * last_bpm;
            if accumulated_beat + segment_beat > beat {
                break;
            }
            accumulated_beat += segment_beat;
            real_time = bpm.time;
            last_time = bpm.time;
            last_bpm = bpm.bpm;
        }

        // 如果 beat 超过了最后一个 BPM 段的节拍
        if beat > accumulated_beat {
            real_time += (beat - accumulated_beat) * 60.0 / last_bpm;
        }

        real_time
    }

    // 这个函数的作用是根据谱面时间计算小节数
    pub fn chart_time_to_beat(&self, chart_time: f32) -> f32 {
        let real_time: f32 = self.chart_time_to_real_time(chart_time);
        self.real_time_to_beat(real_time)
    }

    // 这个函数的作用是根据小节数计算谱面时间
    pub fn beat_to_chart_time(&self, beats: f32) -> f32 {
        let real_time: f32 = self.beat_to_real_time(beats);
        self.real_time_to_chart_time(real_time)
    }

    // 这个函数的作用是根据时间找到对应的数组下标，其中mode的值有四种情况
    // 0: 找到小于等于time的最大值
    // 1: 找到大于等于time的最小值
    // 2: 找到小于time的最大值
    // 3: 找到大于time的最小值
    pub fn find_pos_by_time<T: TimeGetter>(&self, input: &Vec<T>, time: f32, mode: i32) -> usize {   
        let (mut start, mut end): (usize, usize) = (0, input.len());
        let mut i: usize;

        if input.is_empty() {
            return usize::MAX;
        }

        loop {
            i = (start + end) / 2;
            if i >= input.len() - 1 {
                return input.len() - 1;
            } else if i == 0 {
                return 0;
            } else if input[i].get_time() <= time && input[i + 1].get_time() >= time {
                break;
            } else if input[i].get_time() < time {
                start = i;
            } else if input[i].get_time() > time {
                end = i;
            }
        }

        if input[i].get_time() < time && input[i + 1].get_time() == time {
            i += 1;
        }

        let mut new_time = time;
        match mode {
            0 => {
                new_time = input[i].get_time();
                while i > 0 && input[i - 1].get_time() == new_time {
                    i -= 1;
                }
            }
            1 => {
                new_time = input[i].get_time();
                while i < input.len() - 1 && input[i + 1].get_time() == new_time {
                    i += 1;
                }
            }
            2 => {
                if i < input.len() - 1 && input[i].get_time() != new_time {
                    new_time = input[i + 1].get_time();
                }
                while i > 0 && input[i - 1].get_time() == new_time {
                    i -= 1;
                }
            }
            3 => {
                if i < input.len() - 1 && input[i].get_time() != new_time {
                    new_time = input[i + 1].get_time();
                }
                while i < input.len() - 1 && input[i + 1].get_time() == new_time {
                    i += 1;
                }
            }
            _ => {}
        }

        i
    }

    // 这个函数的作用是根据时间找到对应的速度，然后计算出积分
    pub fn find_distance_by_time(&self, time: f32) -> f32 {
        let (mut p1, mut p2): (usize, usize);
        let (mut d1, mut t1, mut t2, mut v1, mut v2): (f32, f32, f32, f32, f32);

        p1 = self.find_pos_by_time(&self.speed_distance, time, 1);
        t1 = self.speed_distance[p1].time;
        v1 = self.speed_distance[p1].speed;
        d1 = self.speed_distance[p1].distance;
        p2 = p1 + 1;

        if p2 < self.speed_distance.len() && self.speed_distance[p2].smooth == 1 {
            t2 = self.speed_distance[p2].time;
            v2 = self.speed_distance[p2].speed;
            d1 + v1 * (time - t1) + (time - t1).powi(2) * (v2 - v1) / (t2 - t1) / 2.0
        } else {
            d1 + (time - t1) * v1
        }
    }

    pub fn find_bpm_by_time(&self, chart_time: f32) -> f32 {
        let index = self.find_pos_by_time(&self.bpm, chart_time, 0);
        return self.bpm[index].bpm;
    }

    // 这个函数的作用是根据积分找到对应的时间
    pub fn find_time_by_distance(&self, distance: f32) -> f32 {
        let (mut p1, mut p2): (usize, usize);
        let (mut d1, mut d2, mut t1, mut t2, mut v1, mut v2): (f32, f32, f32, f32, f32, f32);
    
        p1 = self.find_pos_by_distance(&self.speed_distance, distance, 1);

        while p1 < self.speed_distance.len() && self.speed_distance[p1].speed == 0.0 {
            p1 += 1;
        }

        d1 = self.speed_distance[p1].distance;
        t1 = self.speed_distance[p1].time;
        v1 = self.speed_distance[p1].speed;
        p2 = p1 + 1;

        while p2 < self.speed_distance.len() && self.speed_distance[p2].time == self.speed_distance[p1].time {
            p2 += 1;  
        }
    
        if p2 < self.speed_distance.len() && self.speed_distance[p2].smooth == 1 {
            d2 = self.speed_distance[p2].distance;
            t2 = self.speed_distance[p2].time;
            v2 = self.speed_distance[p2].speed;
            t1 + ((distance - d1) / v1) + ((distance - d1) * (v2 - v1) / (d2 - d1) / 2.0)
        } else {
            t1 + ((distance - d1) / v1)
        }
    }

    pub fn find_speed_by_time(&self, time: f32) -> f32 {
        let mut p1: usize = self.find_pos_by_time(&self.speed, time, 1);
        self.speed[p1].speed
    }

    // 这个函数的作用是根据积分找到对应的数组下标，其中mode的值有四种情况
    // 0: 找到小于等于pos的最大值
    // 1: 找到大于等于pos的最小值
    // 2: 找到小于pos的最大值
    // 3: 找到大于pos的最小值
    pub fn find_pos_by_distance<T: DistanceGetter>(&self, input: &Vec<T>, pos: f32, mode: i32) -> usize {
        let mut start: usize = 0;
        let mut end: usize = input.len();
        let mut i: usize;
        if input.is_empty() {
            return usize::MAX;
        }
        loop {
            i = (start + end) / 2;
            if i >= input.len() - 1 {
                return input.len() - 1;
            } else if i == 0 {
                return 0;
            } else if input[i].get_distance() <= pos && input[i + 1].get_distance() >= pos {
                break;
            } else if input[i].get_distance() < pos {
                start = i;
            } else if input[i].get_distance() > pos {
                end = i;
            }
        }
    
        if input[i].get_distance() < pos && input[i + 1].get_distance() == pos {
            i += 1;
        }
    
        let mut new_time = pos;
        match mode {
            0 => {
                new_time = input[i].get_distance();
                while i > 0 && input[i - 1].get_distance() == new_time {
                    i -= 1;
                }
            }
            1 => {
                new_time = input[i].get_distance();
                while i < input.len() - 1 && input[i + 1].get_distance() == new_time {
                    i += 1;
                }
            }
            2 => {
                if i < input.len() - 1 && input[i].get_distance() != new_time {
                    new_time = input[i + 1].get_distance();
                }
                while i > 0 && input[i - 1].get_distance() == new_time {
                    i -= 1;
                }
            }
            3 => {
                if i < input.len() - 1 && input[i].get_distance() != new_time {
                    new_time = input[i + 1].get_distance();
                }
                while i < input.len() - 1 && input[i + 1].get_distance() == new_time {
                    i += 1;
                }
            }
            _ => {}
        }
    
        i
    }

    // 这个函数的作用是先计算谱面速度对时间的积分，然后再计算每个trail的位置
    pub fn distance_preprocessing(&mut self){     
        let mut time_start: f32 = 0.0;
        let mut time_end: f32 = 0.0;
        let mut speed_start: f32 = 0.0;
        let mut speed_end: f32 = 0.0;
        let mut smooth: i32 = 0;
        let mut distance_start: f32 = 0.0;

        let cal  = |speed_start: f32, speed_end: f32, time_start: f32, time_end: f32, smooth: i32| {
            if smooth == 1 {
                (speed_start + speed_end) * (time_end - time_start) / 2.0
            } else {
                speed_start * (time_end - time_start)
            }
        };

        if !self.speed.is_empty() {
            for i in 0..self.speed.len() {
                time_end = self.speed[i].time;
                speed_end = self.speed[i].speed;
                smooth = self.speed[i].smooth;

                if i == 0 {
                    time_start = 0.0;
                    if time_end != 0.0 {
                        if smooth == 1 {
                            speed_start = speed_end;
                            smooth = 0;
                        } else {
                            speed_start = 1.0;
                        }
                        let speed_distance: SpeedDistance = speeddistance::SpeedDistance::new(0.0, speed_start, 0, 0.0);
                        self.speed_distance.push(speed_distance);
                    }
                } else {
                    time_start = self.speed[i - 1].time;
                    speed_start = self.speed[i - 1].speed;
                }

                distance_start += cal(speed_start, speed_end, time_start, time_end, smooth);
                self.speed_distance.push(speeddistance::SpeedDistance::new(time_end, speed_end, smooth, distance_start));
            }
        } else {
            self.speed_distance.push(speeddistance::SpeedDistance::new(0.0, 1.0, 0, 0.0));
        }


        for i in 0..self.note.len(){
            //如果是Trail类型的note就处理trail_distance
            if let Note::Trail(trail) = &self.note[i]{
                let distance: f32 = self.find_distance_by_time(trail.time);
                let trail_distance: TrailDistance = TrailDistance::new(trail.time, trail.degree, trail.delta, trail.prev_curv, trail.next_curv, distance);
                self.trail_distance.push(trail_distance);
                continue;
            }

            if let Note::Rotate(rotate) = &self.note[i]{
                let distance: f32 = self.find_distance_by_time(rotate.time);
                let trail_distance: TrailDistance = TrailDistance::new(rotate.time, rotate.degree, rotate.delta, rotate.prev_curv, rotate.next_curv, distance);
                self.trail_distance.push(trail_distance);
                continue;
            }

        }

        if self.trail_distance[0].time != 0.0 {
            let start = self.trail_distance.first().unwrap();
            let trail_distance: TrailDistance = TrailDistance::new(0.0, start.degree, 0.0, start.prev_curv, start.next_curv, 0.0);
            self.trail_distance.insert(0, trail_distance);
        }

        let last = self.trail_distance.last().unwrap();
        let distance: f32 = self.find_distance_by_time(last.time + 50000.0);
        let trail_distance: TrailDistance = TrailDistance::new(last.time + 50000.0, last.degree + last.delta, 0.0, last.prev_curv, last.next_curv, distance);
        self.trail_distance.push(trail_distance);

        //计算手机怎么转 还有点瑕疵
        let mut flag: bool = false;
        for i in 0..self.note.len(){
            flag = false;
            let note = &self.note[i];
            let time = note.get_time();
            for j in 0..3{
                if i + j >= self.note.len(){
                    break;
                }
                let temp = &self.note[j + i];
                if temp.get_time() - time > 100.0{
                    break;
                }
                if let Note::Catch(catch) = temp{
                    let distance: f32 = self.find_distance_by_time(catch.time);
                    let degree = self.find_degree_by_time(catch.time);
                    let trail_distance: TrailDistance = TrailDistance::new(catch.time, degree, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                    flag = true;
                    break;
                }
            }
            if flag{
                continue;
            }
            match note{
                Note::Catch(catch) => {
                    let distance: f32 = self.find_distance_by_time(catch.time);
                    let degree = self.find_degree_by_time(catch.time);
                    let trail_distance: TrailDistance = TrailDistance::new(catch.time, degree, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                }
                Note::Tap(tap) => {
                    let distance: f32 = self.find_distance_by_time(tap.time);
                    let trail_distance: TrailDistance = TrailDistance::new(tap.time, tap.degree % 180.0, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                }
                Note::Flick(flick) => {
                    let distance: f32 = self.find_distance_by_time(flick.time);
                    let trail_distance: TrailDistance = TrailDistance::new(flick.time, flick.degree % 180.0, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                }
                Note::Slide(slide) => {
                    let distance: f32 = self.find_distance_by_time(slide.time);
                    let trail_distance: TrailDistance = TrailDistance::new(slide.time, slide.degree % 180.0, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                }
                Note::Bomb(bomb) => {
                    let distance: f32 = self.find_distance_by_time(bomb.time);
                    let degree = self.find_degree_by_time(bomb.time);
                    let trail_distance: TrailDistance = TrailDistance::new(bomb.time, degree % 180.0, 0.0, 0.0, 0.0, distance);
                    self.phone_trail_distance.push(trail_distance);
                }
                Note::Rotate(rotate) => {
                    let distance1: f32 = self.find_distance_by_time(rotate.time - 30.0);
                    let distancemid: f32 = self.find_distance_by_time(rotate.time);
                    let distance2: f32 = self.find_distance_by_time(rotate.time + 30.0);   
                    let trail_distance1: TrailDistance = TrailDistance::new(rotate.time - 30.0, rotate.degree % 180.0, 0.0, rotate.prev_curv, rotate.next_curv, distance1);
                    self.phone_trail_distance.push(trail_distance1);
                    let trail_distance_mid: TrailDistance = TrailDistance::new(rotate.time, rotate.degree % 180.0 + rotate.delta / 2.0, 0.0, rotate.prev_curv, rotate.next_curv, distancemid);
                    self.phone_trail_distance.push(trail_distance_mid);
                    let traildistance2: TrailDistance = TrailDistance::new(rotate.time + 30.0, rotate.degree % 180.0 + rotate.delta, 0.0, rotate.prev_curv, rotate.next_curv, distance2);
                    self.phone_trail_distance.push(traildistance2);
                }
                _ => {}
            }
        }
        let trail_distance: TrailDistance = TrailDistance::new(0.0, 90.0, 0.0, 0.0,0.0, 0.0);
        self.phone_trail_distance.insert(0, trail_distance);

        let last = self.phone_trail_distance.last().unwrap();
        let distance: f32 = self.find_distance_by_time(last.time + 200.0);
        let trail_distance: TrailDistance = TrailDistance::new(last.time + 200.0, 90.0, 0.0, last.prev_curv, last.next_curv, distance);
        self.phone_trail_distance.push(trail_distance);

        self.sort_chart();
    }

    pub fn get_y_from_x(&self, start: f32, end: f32, control_a: f32, control_b: f32, x: f32) -> f32 {
        // find cubic x value based on control point a and b.
        let calculate_x = |a: f32, b: f32, time: f32| {
            time * (3.0 * a - 6.0 * a * time + 3.0 * b * time + time * time + 3.0 * a * time * time - 3.0 * b * time * time)
        };
    
        // find cubic y value based on t
        let calculate_y = |time: f32| {
            -2.0 * time * time * time + 3.0 * time * time
        };
    
        // Approximated t value based on x
        // binary search
        let mut upper_time: f32 = 1.0; // tail 
        let mut lower_time: f32 = 0.0; // head
        let mut time: f32 = 0.0; // compare time
    
        // Do a binary search 10 time to approximate the target time
        // For each loop, the compare time = (head + tail) / 2
        // And then compare the X at compare time
        for _ in 0..20 {
            time = (upper_time + lower_time) / 2.0;
            let search_x: f32 = calculate_x(control_a, 1.0 - control_b, time);
            if search_x < x {
                lower_time = time;
            } else {
                upper_time = time;
            }
        }
    
        calculate_y(time) * (end - start) + start
    }

    pub fn find_degree_by_2_trails(&self, mut trail1: TrailDistance, mut trail2: TrailDistance, progress: f32) -> f32 {
        let calculate_delta = |degree1: f32, degree2: f32| {
            let mut delta = (degree1 - degree2).abs() % 360.0;
            if delta > 180.0 {
                delta = 360.0 - delta;
            }
            delta
        };

        let mut time1: f32;
        let mut time2: f32;
        let mut degree1: f32;
        let mut degree2: f32;
        let mut curvature1: f32;
        let mut curvature2: f32;
        let mut result: f32;
        // 聊天记录：这个我知道是什么原因了 应该是if abs(正常点-上一个点) < abs(对称点-上一个点)  走正常点 else 走对称点
        // 首先，找到两个trail相距角度最小的一边
        degree1 = trail1.degree + trail1.delta;
        degree2 = trail2.degree;
        if calculate_delta(degree2, degree1) < calculate_delta(degree2 + 180.0, degree1){
            degree2 += 180.0;
            degree2 = degree2 % 360.0;
        }
        time1 = trail1.time;
        time2 = trail2.time;
        curvature1 = trail1.next_curv;
        curvature2 = trail2.prev_curv;
        // 获取基础信息
        // 处理角度
        degree1 = degree1 % 360.0;
        if degree1 < 0.0 {
            degree1 += 360.0;
        }
        degree2 = degree2 % 360.0;
        if degree2 < 0.0 {
            degree2 += 360.0;
        }

        if degree1 >= 180.0 {
            if (degree1 - (degree2 + 360.0)).abs() <= 90.0 {
                degree2 += 360.0;
            } else if (degree1 - (degree2 + 180.0)).abs() <= 90.0 {
                degree2 += 180.0;
            } else if (degree1 - (degree2 - 180.0)).abs() <= 90.0 {
                degree2 -= 180.0;
            }
        } else {
            if (degree1 - (degree2 - 360.0)).abs() < 90.0 {
                degree2 -= 360.0;
            } else if (degree1 - (degree2 - 180.0)).abs() < 90.0 {
                degree2 -= 180.0;
            } else if (degree1 - (degree2 + 180.0)).abs() < 90.0 {
                degree2 += 180.0;
            }
        }

        result = self.get_y_from_x(degree1, degree2, curvature1 / 100.0, curvature2 / 100.0, progress);
        return result;
    }

    pub fn find_degree_by_time(&self, time: f32) -> f32 {
        let mut position1: usize;
        let mut position2: usize;
        let mut time1: f32;
        let mut time2: f32;
        let mut distance1: f32;
        let mut distance: f32;
        let mut distance2: f32;
        let mut degree1: f32;
        let mut degree2: f32;
        let mut curvature1: f32;
        let mut curvature2: f32;
        let mut result: f32;
        distance = self.find_distance_by_time(time);
        position1 = self.find_pos_by_distance(&self.trail_distance, distance, 1);
        position2 = position1 + 1;
        
        time1 = self.trail_distance[position1].time;
        if time <= time1{
            return self.trail_distance[position1].degree;
        }
        distance1 = self.trail_distance[position1].distance;
        degree1 = self.trail_distance[position1].degree + self.trail_distance[position1].delta;
        curvature1 = self.trail_distance[position1].next_curv;

        if position2 < self.trail_distance.len() {
            time2 = self.trail_distance[position2].time;
            distance2 = self.trail_distance[position2].distance;
            degree2 = self.trail_distance[position2].degree;
            curvature2 = self.trail_distance[position2].prev_curv;
            
            degree1 = degree1 % 360.0;
            if degree1 < 0.0 {
                degree1 += 360.0;
            }
            degree2 = degree2 % 360.0;
            if degree2 < 0.0 {
                degree2 += 360.0;
            }

            if degree1 >= 180.0 {
                if (degree1 - (degree2 + 360.0)).abs() <= 90.0 {
                    degree2 += 360.0;
                } else if (degree1 - (degree2 + 180.0)).abs() <= 90.0 {
                    degree2 += 180.0;
                } else if (degree1 - (degree2 - 180.0)).abs() <= 90.0 {
                    degree2 -= 180.0;
                }
            } else {
                if (degree1 - (degree2 - 360.0)).abs() < 90.0 {
                    degree2 -= 360.0;
                } else if (degree1 - (degree2 - 180.0)).abs() < 90.0 {
                    degree2 -= 180.0;
                } else if (degree1 - (degree2 + 180.0)).abs() < 90.0 {
                    degree2 += 180.0;
                }
            }

            result = self.get_y_from_x(degree1, degree2, curvature1 / 100.0, curvature2 / 100.0, (distance - distance1) / (distance2 - distance1));
            // result = result % 180.0;
        } else {
            result = degree1;
        }
        if result < 0.0 {
            result += 360.0;
        }
        result = result % 180.0;
        result
    }

    pub fn find_phone_degree_by_time(&self, time: f32) -> f32 {
        let mut position1: usize;
        let mut position2: usize;
        let mut time1: f32;
        let mut time2: f32;
        let mut distance1: f32;
        let mut distance: f32;
        let mut distance2: f32;
        let mut degree1: f32;
        let mut degree2: f32;
        let mut curvature1: f32;
        let mut curvature2: f32;
        let mut result: f32;
        distance = self.find_distance_by_time(time);
        position1 = self.find_pos_by_distance(&self.phone_trail_distance, distance, 1);
        position2 = position1 + 1;
        
        time1 = self.phone_trail_distance[position1].time;
        if time <= time1{
            return self.phone_trail_distance[position1].degree;
        }
        distance1 = self.phone_trail_distance[position1].distance;
        degree1 = self.phone_trail_distance[position1].degree + self.phone_trail_distance[position1].delta;
        curvature1 = self.phone_trail_distance[position1].next_curv;

        if position2 < self.phone_trail_distance.len() {
            time2 = self.phone_trail_distance[position2].time;
            distance2 = self.phone_trail_distance[position2].distance;
            degree2 = self.phone_trail_distance[position2].degree;
            curvature2 = self.phone_trail_distance[position2].prev_curv;
            
            degree1 = degree1 % 360.0;
            if degree1 < 0.0 {
                degree1 += 360.0;
            }
            degree2 = degree2 % 360.0;
            if degree2 < 0.0 {
                degree2 += 360.0;
            }

            if degree1 >= 180.0 {
                if (degree1 - (degree2 + 360.0)).abs() <= 90.0 {
                    degree2 += 360.0;
                } else if (degree1 - (degree2 + 180.0)).abs() <= 90.0 {
                    degree2 += 180.0;
                } else if (degree1 - (degree2 - 180.0)).abs() <= 90.0 {
                    degree2 -= 180.0;
                }
            } else {
                if (degree1 - (degree2 - 360.0)).abs() < 90.0 {
                    degree2 -= 360.0;
                } else if (degree1 - (degree2 - 180.0)).abs() < 90.0 {
                    degree2 -= 180.0;
                } else if (degree1 - (degree2 + 180.0)).abs() < 90.0 {
                    degree2 += 180.0;
                }
            }

            result = self.get_y_from_x(degree1, degree2, curvature1 / 100.0, curvature2 / 100.0, (distance - distance1) / (distance2 - distance1));
            // result = result % 180.0;
        } else {
            result = degree1;
        }
        if result < 0.0 {
            result += 360.0;
        }
        result = result % 180.0;
        result
    }

    pub fn find_degree_by_distance(&self, distance: f32) -> f32 {
        let time: f32 = self.find_time_by_distance(distance);
        self.find_degree_by_time(time)
    }

    pub fn export_to_txt(&self, path: &str) -> IoResult<()>{
        log::info!("Exporting chart to txt format: {}", path);
        let mut file: File = File::create(path)?;
        writeln!(file, "# Version {}", self.version)?;
        writeln!(file, "\n# BPM")?;
        for bpm in &self.bpm {
            writeln!(file, "{},{}", bpm.time, bpm.bpm)?;
        }
        writeln!(file, "\n# Speed")?;
        for speed in &self.speed {
            writeln!(file, "{},{},{}", speed.time, speed.speed, speed.smooth)?;
        }
        writeln!(file, "\n# Note")?;
        for note in &self.note {
           match note {
               Note::Tap(tap) => {
                   writeln!(file, "0,{},{}", tap.time, tap.degree)?;
               },
               Note::Flick(flick) => {
                   writeln!(file, "1,{},{}", flick.time, flick.degree)?;
               },
               Note::Slide(slide) => {
                   writeln!(file, "2,{},{},{},{},{},{},{},{}", slide.time, slide.degree, slide.slidetype, slide.end_degree, slide.snap, slide.amount, slide.prev_curv, slide.next_curv)?;
               },
               Note::Rotate(rotate) => {
                   writeln!(file, "4,{},{},{},{},{}", rotate.time, rotate.degree, rotate.delta, rotate.prev_curv, rotate.next_curv)?;
               },
               Note::Catch(catch) => {
                   writeln!(file, "5,{},{}", catch.time, catch.degree)?;
               },
               Note::Trail(trail) => {
                   writeln!(file, "11,{},{},{},{},{}", trail.time, trail.degree, trail.delta, trail.prev_curv, trail.next_curv)?;
               },
               Note::Bomb(bomb) => {
                   writeln!(file, "6,{},{}", bomb.time, bomb.degree)?;
               },
               _ => {}
           }
        }
        Ok(())
    }

    pub fn export_to_json(&self, path: &str) -> IoResult<()> {
        log::info!("Exporting chart to json format: {}", path);
        let mut bpm: Vec<Value> = Vec::new();
        for b in &self.bpm {
            bpm.push(serde_json::to_value(b).unwrap());
        }
        let mut speeddistance: Vec<Value> = Vec::new();
        for s in &self.speed_distance {
            speeddistance.push(serde_json::to_value(s).unwrap());
        }
        let mut traildistance: Vec<Value> = Vec::new();
        for t in &self.trail_distance {
            traildistance.push(serde_json::to_value(t).unwrap());
        }

        let mut note: Vec<Value> = Vec::new();
        for n in &self.note {
            // 根据不同的note类型，将note转换为json
            match n {
                Note::Tap(tap) => {
                    let dict: Value = json!({
                        "type": 0,
                        "typename": "Tap",
                        "time": tap.time,
                        "degree": tap.degree,
                    });
                    note.push(dict);
                },
                Note::Flick(flick) => {
                    let dict: Value = json!({
                        "type": 1,
                        "typename": "Flick",
                        "time": flick.time,
                        "degree": flick.degree,
                    });
                    note.push(dict);
                },
                Note::Slide(slide) => {
                    let dict: Value = json!({
                        "type": 2,
                        "typename": "Slide",
                        "slidetype": slide.slidetype,   
                        "time": slide.time,
                        "degree": slide.degree,
                        "end_degree": slide.end_degree,
                        "snap": slide.snap,
                        "amount": slide.amount,
                        "prev_curv": slide.prev_curv,
                        "next_curv": slide.next_curv,
                    });
                    note.push(dict);
                },
                Note::Rotate(rotate) => {
                    let dict: Value = json!({
                        "type": 4,
                        "typename": "Rotate",
                        "time": rotate.time,
                        "degree": rotate.degree,
                        "delta": rotate.delta,
                        "prev_curv": rotate.prev_curv,
                        "next_curv": rotate.next_curv,
                    });
                    note.push(dict);
                },
                Note::Catch(catch) => {
                    let dict: Value = json!({
                        "type": 5,
                        "typename": "Catch",
                        "time": catch.time,
                        "degree": catch.degree,
                        "truedegree": self.find_degree_by_time(catch.time),
                    });
                    // print!("Catch: {:?} Time: {:?}\n", dict.get("truedegree").unwrap(), catch.time);
                    note.push(dict);
                },
                Note::Trail(trail) => {
                    let dict: Value = json!({
                        "type": 11,
                        "typename": "Trail",
                        "time": trail.time,
                        "degree": trail.degree,
                        "delta": trail.delta,
                        "prev_curv": trail.prev_curv,
                        "next_curv": trail.next_curv,
                    });
                    note.push(dict);
                },
                Note::Bomb(bomb) => {
                    let dict: Value = json!({
                        "type": 6,
                        "typename": "Bomb",
                        "time": bomb.time,
                        "degree": bomb.degree,
                    });
                    note.push(dict);
                },
                _ => {}
                
            }
        }
        
        // let mut deg: Vec<f32> = Vec::new();
        // for i in 0..100000 {
        //     deg.push(self.find_degree_by_time(i as f32));
        // }
        let json: Value = json!({
            "version": self.version,
            "offset": self.offset,
            "bpm": bpm,
            "speeddistance": speeddistance,
            "traildistance": traildistance,
            "note": note,
            // "deg": deg,
        });
    
        let mut json_str: Vec<u8> = Vec::new();
        let formatter: PrettyFormatter = PrettyFormatter::with_indent(b"    ");
        let mut serializer: Serializer<&mut Vec<u8>, PrettyFormatter> = Serializer::with_formatter(&mut json_str, formatter);
        json.serialize(&mut serializer).unwrap();
    
        let mut file: File = File::create(path)?;
        file.write_all(&json_str)?;
    
        Ok(())
    }

}