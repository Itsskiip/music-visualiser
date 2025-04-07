#![feature(thread_sleep_until)]
#![feature(iter_collect_into)]
#![feature(iter_array_chunks)]
#![feature(duration_constructors)]
#![feature(isqrt)]

use glium::winit;

mod graphics;
mod processing;
mod audio;

fn main() {
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();

    let mut app = graphics::App::new(
        "Nyoom",
        800, 
        600, 
        120., 
        "music.mp3", 
        8192,
        100);

    event_loop.run_app(&mut app).unwrap();
}