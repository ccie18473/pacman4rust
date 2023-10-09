#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate sdl2;

use sdl2::AudioSubsystem;
use sdl2::audio::AudioQueue;
use sdl2::event::*;
use sdl2::keyboard::*;
use sdl2::pixels::*;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::Sdl;
use sdl2::TimerSubsystem;

use std::fs::File;
use std::io::Read;
use std::ptr;

pub mod pac;
pub mod wsg;
pub mod z80;

pub use pac::*;
pub use wsg::*;
pub use z80::*;

pub struct userdata<'a> {
    pub game_ptr: *mut game<'a>,
}
impl<'a> userdata<'a> {
    pub fn new() -> Self {
        Self {
            game_ptr: ptr::null_mut(),
        }
    }
}

pub struct game<'a> {
    pub should_quit: bool,
    pub has_focus: bool,
    pub is_paused: bool,
    pub speed: i32,
    pub sdl_context: Sdl,
    pub timer: TimerSubsystem,
    pub renderer: Canvas<Window>,
    pub audio: AudioSubsystem,
    pub audio_device: AudioQueue<i16>,
    pub p: pac::pac<'a>,
    pub current_time: u32,
    pub last_time: u32,
    pub dt: u32,
}

impl<'a> game<'a> {
    pub fn new() -> Self {
        // SDL init
        let sdl_context = sdl2::init().unwrap();
        // timer
        let timer = sdl_context.timer().unwrap();
        // create SDL window
        let video_subsystem = sdl_context.video().unwrap();
        let title: String;
        title = "pacman4rust".to_string();
        let window = video_subsystem
            .window(
                &title,
                PAC_SCREEN_WIDTH as u32 * 2,
                PAC_SCREEN_HEIGHT as u32 * 2,
            )
            .resizable()
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        // create renderer
        let renderer = window
            .into_canvas()
            .accelerated()
            .target_texture()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        // audio
        let audio = sdl_context.audio().unwrap();
        let audio_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: Some(1024),
        };
        let audio_device = audio.open_queue::<i16, _>(None, &audio_spec).unwrap();
        Self {
            should_quit: false,
            has_focus: true,
            is_paused: false,
            speed: 1,
            sdl_context,
            timer,
            renderer,
            audio,
            audio_device,
            p: pac::pac::new(),
            current_time: 0,
            last_time: 0,
            dt: 0,
        }
    }
}

pub fn update_screen(g: &mut game) {
    //println!("update_screen");

    let texture_creator = g.renderer.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            PAC_SCREEN_WIDTH as u32,
            PAC_SCREEN_HEIGHT as u32,
        )
        .map_err(|e| e.to_string())
        .unwrap();

    let pixels = &mut g.p.screen_buffer;
    let pitch = 3 * PAC_SCREEN_WIDTH;

    texture.update(None, pixels, pitch as usize).unwrap();
    g.renderer.clear();
    g.renderer.copy(&texture, None, None).unwrap();
    g.renderer.present();
}

pub fn push_sample(g: &mut game, sample: i16) {
    //println!("push_sample");

    g.audio_device.queue_audio(&[sample]).unwrap();
}

pub fn send_quit_event(g: &mut game) {
    //println!("send_quit_event");

    g.should_quit = true;
}

pub fn screenshot(_g: &mut game) {
    //println!("screenshot");
}

pub fn mainloop(g: &mut game) {
    //println!("mainloop");

    g.current_time = g.timer.ticks();
    g.dt = g.current_time - g.last_time;

    let mut event_pump = g.sdl_context.event_pump().unwrap();

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => g.should_quit = true,
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::FocusGained => {
                    g.has_focus = true;
                }
                WindowEvent::FocusLost => {
                    g.has_focus = false;
                }
                _ => {}
            },
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                match scancode {
                    Scancode::Return | Scancode::Num1 => {
                        g.p.p1_start = 1; // start (1p)
                    }
                    Scancode::Num2 => {
                        g.p.p2_start = 1; // start (2p)
                    }
                    Scancode::Up => {
                        g.p.p1_up = 1; // up
                    }
                    Scancode::Down => {
                        g.p.p1_down = 1; // down
                    }
                    Scancode::Left => {
                        g.p.p1_left = 1; // left
                    }
                    Scancode::Right => {
                        g.p.p1_right = 1; // right
                    }
                    Scancode::C | Scancode::Num5 => {
                        g.p.coin_s1 = 1; // coin
                    }
                    Scancode::V => {
                        g.p.coin_s2 = 1; // coin (slot 2)
                    }
                    Scancode::T => {
                        g.p.board_test = 1; // board test
                    }
                    Scancode::M => {
                        g.p.mute_audio = !g.p.mute_audio;
                    }
                    Scancode::P => {
                        g.is_paused = !g.is_paused;
                    }
                    Scancode::S => {
                        screenshot(g);
                    }
                    Scancode::I => {
                        pac_cheat_invincibility(g);
                    }
                    Scancode::Tab => {
                        g.speed = 5;
                    }
                    _ => {}
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                match scancode {
                    Scancode::Return | Scancode::Num1 => {
                        g.p.p1_start = 0; // start (1p)
                    }
                    Scancode::Num2 => {
                        g.p.p2_start = 0; // start (2p)
                    }
                    Scancode::Up => {
                        g.p.p1_up = 0; // up
                    }
                    Scancode::Down => {
                        g.p.p1_down = 0; // down
                    }
                    Scancode::Left => {
                        g.p.p1_left = 0; // left
                    }
                    Scancode::Right => {
                        g.p.p1_right = 0; // right
                    }
                    Scancode::C | Scancode::Num5 => {
                        g.p.coin_s1 = 0; // coin
                    }
                    Scancode::V => {
                        g.p.coin_s2 = 0; // coin (slot 2)
                    }
                    Scancode::T => {
                        g.p.board_test = 0; // board test
                    }
                    Scancode::Tab => {
                        g.speed = 1;
                        // clear the queued audio to avoid audio delays
                        g.audio_device.clear();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if !g.is_paused && g.has_focus {
        pac_update(g, g.dt * g.speed as u32);
    }

    g.last_time = g.current_time;
}

fn main() {
    let mut g = game::new();

    // print info on renderer:
    let renderer_info = g.renderer.info();
    println!("INFO: Using renderer {}",renderer_info.name);

    // audio init

    let driver_name = g.audio.current_audio_driver();
    println!("INFO: audio device has been opened ({})", driver_name);

    g.audio_device.resume(); // start playing

    // pac init
    pac_init(&mut g);

    g.p.sample_rate = 44100;
    g.p.push_sample = push_sample;
    g.p.update_screen = update_screen;
    update_screen(&mut g);

    // main loop
    g.current_time = g.timer.ticks();
    g.last_time = g.timer.ticks();

    while !g.should_quit {
        mainloop(&mut g);
    }

    pac_quit(&mut g);
}
