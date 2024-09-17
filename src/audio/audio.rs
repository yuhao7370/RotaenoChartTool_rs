use camera::mouse;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::io::{BufReader, Cursor};
use std::fs::File;
use std::{
    path::Path,
    sync::{Arc, RwLock, Mutex},
};
use macroquad::prelude::*;
use std::thread;

use super::buffer_player::SamplesBuffer;
use super::buffer_player::AudioController;

pub struct HitSoundPlayer {
    pub audio_data: Arc<Vec<u8>>,
    pub stream_handle: OutputStreamHandle,
}

impl HitSoundPlayer {
    pub fn new(file_path: &str, stream_handle: OutputStreamHandle) -> Self {
        // 打开音频文件并读取到内存中
        let file = File::open(file_path).unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        std::io::copy(&mut reader, &mut buffer).unwrap();

        HitSoundPlayer {
            audio_data: Arc::new(buffer),
            stream_handle,
        }
    }

    pub fn play(&self, volume: f32) {
        let audio_data = self.audio_data.clone();
        let stream_handle = self.stream_handle.clone();

        thread::spawn(move || {
            let cursor = Cursor::new(audio_data.as_ref().clone());
            let source = Decoder::new(cursor).unwrap();

            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.set_volume(volume); // 设置音量
            sink.append(source);
            sink.play();
            sink.sleep_until_end();
        });
    }
}

pub struct HitSoundManager {
    pub tap_hitsound: HitSoundPlayer,
    pub flick_hitsound: HitSoundPlayer,
    pub slide_hitsound: HitSoundPlayer,
    pub catch_hitsound: HitSoundPlayer,
    pub rotate_L_hitsound: HitSoundPlayer,
    pub rotate_R_hitsound: HitSoundPlayer,
    pub active_sinks: Arc<Mutex<Vec<Arc<Sink>>>>, // 活跃的音效播放
    pub volume: Arc<Mutex<f32>>, // 全局音量
}

impl HitSoundManager {
    pub async fn init(
        tap_hitsound_path: &str,
        flick_hitsound_path: &str,
        slide_hitsound_path: &str,
        catch_hitsound_path: &str,
        rotate_L_hitsound_path: &str,
        rotate_R_hitsound_path: &str,
        stream_handle: OutputStreamHandle,
    ) -> Self {
        let tap_hitsound = HitSoundPlayer::new(tap_hitsound_path, stream_handle.clone());
        let flick_hitsound = HitSoundPlayer::new(flick_hitsound_path, stream_handle.clone());
        let slide_hitsound = HitSoundPlayer::new(slide_hitsound_path, stream_handle.clone());
        let catch_hitsound = HitSoundPlayer::new(catch_hitsound_path, stream_handle.clone());
        let rotate_L_hitsound = HitSoundPlayer::new(rotate_L_hitsound_path, stream_handle.clone());
        let rotate_R_hitsound = HitSoundPlayer::new(rotate_R_hitsound_path, stream_handle.clone());

        Self {
            tap_hitsound,
            flick_hitsound,
            slide_hitsound,
            catch_hitsound,
            rotate_L_hitsound,
            rotate_R_hitsound,
            active_sinks: Arc::new(Mutex::new(Vec::new())),
            volume: Arc::new(Mutex::new(0.5)), // 默认音量为 0.5
        }
    }

    pub fn set_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().unwrap();
        *vol = volume;
    }

    pub fn play(&self, hitsound: &HitSoundPlayer) {
        let volume = *self.volume.lock().unwrap();
        let sink = Arc::new(Sink::try_new(&self.tap_hitsound.stream_handle).unwrap());
        sink.set_volume(volume);
        let audio_data = hitsound.audio_data.clone();
        let cursor = Cursor::new(audio_data.as_ref().clone());
        let source = Decoder::new(cursor).unwrap();
        sink.append(source);
        sink.play();

        let mut active_sinks = self.active_sinks.lock().unwrap();
        active_sinks.push(sink.clone());

        // 移除已经播放完的音效
        active_sinks.retain(|s| !s.empty());
    }

    pub fn play_tap(&self) {
        self.play(&self.tap_hitsound);
    }

    pub fn play_flick(&self) {
        self.play(&self.flick_hitsound);
    }

    pub fn play_slide(&self) {
        self.play(&self.slide_hitsound);
    }

    pub fn play_catch(&self) {
        self.play(&self.catch_hitsound);
    }

    pub fn play_rotate_L(&self) {
        self.play(&self.rotate_L_hitsound);
    }

    pub fn play_rotate_R(&self) {
        self.play(&self.rotate_R_hitsound);
    }
}

pub struct AudioManager {
    pub controller: Arc<AudioController<i16>>,
    speed: f32,
    duration: f32, // in seconds
    is_playing: bool,
}

//params = (600.0, 20.0, 600.0, 20.0, 0.0, duration)
impl AudioManager {
    pub fn new<P: AsRef<Path>>(path: P, audio_device: &OutputStreamHandle) -> Self {
        let stop_loading: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
        let progress: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
        let music: Arc<SamplesBuffer<i16>> = SamplesBuffer::load_from_file_async_stoppable(path, stop_loading, progress).unwrap().into();
        let controller: Arc<AudioController<i16>> = AudioController::new_with_buffer(&audio_device, music).into();
        let duration = controller.get_target_buffer().get_duration().as_secs_f32();
        // println!("Duration: {}", duration);
        controller.set_loop_mode(false);
        controller.set_speed(0.0);

        Self{
            controller: controller,
            speed: 1.0,
            duration: duration,
            is_playing: false,
        }
    }

    pub fn set_loop_mode(&mut self, loop_mode: bool) {  //设置循环模式
        self.controller.set_loop_mode(loop_mode);
    }

    pub fn set_speed(&mut self, speed: f32) {  //设置播放速度
        self.controller.set_speed(speed);
        self.speed = speed;
    }

    pub fn get_speed(&self) -> f32 {  //获取播放速度
        self.speed
    }

    pub fn start(&mut self) {  //开始播放
        self.controller.set_speed(self.speed);
        self.is_playing = true;
    }

    pub fn pause(&mut self) {  //暂停
        self.controller.set_speed(0.0);
        self.is_playing = false;
    }

    pub fn toggle(&mut self) {  //切换播放状态
        if self.is_playing {
            self.pause();
        } else {
            self.start();
        }
    }

    pub fn is_playing(&self) -> bool {  //是否正在播放
        self.is_playing
    }

    pub fn set_volume(&mut self, volume: f32) {  //设置音量，范围0-1
        self.controller.set_volume(volume);  
    }

    pub fn set_time(&mut self, time: f32) {  //设置播放时间，单位秒
        self.controller.set_time(time);
    }

    pub fn get_time(&self) -> f32 {  //获取当前播放时间，单位秒
        self.controller.get_time()
    }

    pub fn get_duration(&self) -> f32 { //获取音频时长，单位秒
        self.duration
    }
}

pub struct AudioProgressBar {
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
    progress: f32,
    musiclen_s: f32,
    mouse_dragging: bool,
}

impl AudioProgressBar {
    pub fn new(center_x: f32, center_y: f32, width: f32, height: f32, progress: f32, musiclen_s: f32) -> Self {
        Self {
            center_x,
            center_y,
            width,
            height,
            progress,
            musiclen_s,
            mouse_dragging: false,
        }
    }

    pub fn set_musiclen_s(&mut self, musiclen_s: f32) {
        self.musiclen_s = musiclen_s;
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn update(&mut self, playing: bool) -> bool{
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            if (mouse_x >= self.center_x - self.width / 2.0 && mouse_x <= self.center_x + self.width / 2.0 && mouse_y >= self.center_y - self.height / 2.0 && mouse_y <= self.center_y + self.height / 2.0){
                self.mouse_dragging = true;
                self.set_progress((mouse_x - (self.center_x - self.width / 2.0)) / self.width);
                return true;
            }
        }
        if is_mouse_button_down(MouseButton::Left) && !playing {
            let (mouse_x, mouse_y) = mouse_position();
            if (mouse_x >= self.center_x - self.width / 2.0 && mouse_x <= self.center_x + self.width / 2.0 && mouse_y >= self.center_y - self.height / 2.0 && mouse_y <= self.center_y + self.height / 2.0){
                self.mouse_dragging = true;
                // 如果在进度条范围内先更新是否拖动状态，再更新进度
            }
            if self.mouse_dragging{
                // 限制鼠标拖动范围
                let mouse_x: f32 = clamp(mouse_x, self.center_x - self.width / 2.0, self.center_x + self.width / 2.0);
                self.set_progress((mouse_x - (self.center_x - self.width / 2.0)) / self.width);
                return true;
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            // 鼠标释放时更新拖动状态
            self.mouse_dragging = false;
        }
        return false;
    }

    pub fn draw(&self) {
        let diamond_size = self.height / 3.0;
        let diamond_x = self.center_x - self.width / 2.0 + self.progress * self.width;
        let diamond_y = self.center_y;

        draw_line(self.center_x - self.width / 2.0, self.center_y + self.height / 2.0, self.center_x + self.width / 2.0, self.center_y + self.height / 2.0, 2.0, WHITE);
        draw_line(self.center_x - self.width / 2.0, self.center_y - self.height / 2.0, self.center_x + self.width / 2.0, self.center_y - self.height / 2.0, 2.0, WHITE);

        draw_poly(diamond_x, diamond_y, 4, diamond_size, 0.0, WHITE);

        let minutes = (self.progress * self.musiclen_s / 60.0) as i32;
        let seconds = (self.progress * self.musiclen_s % 60.0) as i32;
        let milliseconds = ((self.progress * self.musiclen_s * 1000.0) % 1000.0) as i32;

        let time_str = format!("{:02}:{:02}.{:03}", minutes, seconds, milliseconds);
        draw_text(&time_str, self.center_x + self.width / 2.0, self.center_y + 25.0, 14.0, WHITE);
    }
}