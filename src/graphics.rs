use std::time::{Duration, Instant};

use crate::{
    audio, processing::{self, ProcessorOutput}
};

use glium::{winit::{self, window::Window}, Surface};

pub type Display = glium::Display<glium::glutin::surface::WindowSurface>;

#[derive(Clone)]
pub struct WindowSettings {
    title: String,
    width: u32,
    height: u32,

    last_refresh: Instant,
    frametime: Duration,
}

impl WindowSettings {
    pub fn new(title: &str, width: u32, height:u32, max_framerate: f32) -> Self {
        let frametime = Duration::from_secs_f32(1.0 / max_framerate);

        Self {
            title: title.to_string(),
            width,
            height,

            last_refresh: Instant::now(),
            frametime,
        }
    }
}

pub struct Renderer {
    window: Window,
    display: Display,
    
    left_fft: programs::fftprogram::FFTProgram,
    right_fft: programs::fftprogram::FFTProgram,
    left_phase: programs::phaseprogram::PhaseProgram,
}

impl Renderer {
    pub fn new(display: &Display, window: Window, fft_bins: usize, phase_len: usize) -> Self {
        Self {
            window,
            display: display.clone(),
            
            left_fft: programs::fftprogram::FFTProgram::new(fft_bins, &display,[0.0, 0.0, 0.0]),
            right_fft: programs::fftprogram::FFTProgram::new(fft_bins, &display, [1.0, 0.0, 0.0]),
            left_phase: programs::phaseprogram::PhaseProgram::new(phase_len, &display, [0.0, 0.0, 0.0]),
        }
    }

    pub fn render(&mut self, values: &ProcessorOutput) {
        let mut target = self.display.draw();
        target.clear_color(0., 0., 0., 1.);

        self.left_fft.render(&mut target, &values.left_fft);
        self.right_fft.render(&mut target, &values.right_fft);

        self.left_phase.render(&mut target, &values.phase_left);

        target.finish().unwrap();
    }
}

mod programs;

pub struct App<'a> {
    window_settings: WindowSettings,
    audio: audio::Audio<'a>,
    processor: processing::Processor,
    renderer: Option<Renderer>,
    counter: usize,
}

impl App<'_> {
    pub fn new(
        title: &str, 
        width: u32, 
        height: u32, 
        max_framerate: f32, 
        audio_file: &str, 
        sample_window: usize, 
        fft_output_bins: usize,
        phase_pts: usize,
    ) -> Self {
        let window_settings= WindowSettings::new(title, width, height, max_framerate);
        let audio = audio::Audio::new(audio_file, sample_window);
        let processor = processing::Processor::new(sample_window, fft_output_bins, phase_pts);
        
        Self {
            window_settings,
            audio,
            processor,
            renderer: None,
            counter: 0,
        }
    }

    fn start(&mut self, display: &Display, window: Window) {
        let render_data = Renderer::new(
            &display,
            window,
            self.processor.fft_output_bins,
            self.processor.phase_pts,
        );
        
        self.renderer = Some(render_data);
        self.audio.play();
    }

    fn render(&mut self) {
        self.audio.get_samples((&mut self.processor.audio_buffer.0, &mut self.processor.audio_buffer.1));

        let bars = self.processor.process_samples();
        self.renderer.as_mut().unwrap().render(&bars);
    }
}

impl winit::application::ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title(&self.window_settings.title)
            .with_inner_size(self.window_settings.width, self.window_settings.height).build(event_loop);

        self.start(&display, window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                self.render();
                let elapsed = Instant::now().duration_since(self.window_settings.last_refresh);
                self.counter += 1;
                if self.counter > 30 {
                    println!("{:#?}", 1. / (elapsed).as_secs_f32());
                    self.counter = 0;
                }
                let delay = self.window_settings.frametime.saturating_sub(elapsed);
                std::thread::sleep(delay);
                self.window_settings.last_refresh = Instant::now();

                self.renderer.as_ref().unwrap().window.request_redraw();
            },
            _ => ()
        }
    }
}