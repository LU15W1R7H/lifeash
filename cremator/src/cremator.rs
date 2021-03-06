#[allow(unused)]
use color_eyre::{Help, Report, Result};
#[allow(unused)]
use eyre::{eyre, WrapErr};

#[allow(unused)]
pub use tracing::{
    debug, debug_span, error, error_span, info, info_span, instrument, trace, trace_span, warn,
    warn_span,
};

use std::{io::prelude::*, time::Instant};

use glium::{
    glutin::{
        self,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    Display,
};

use la::{Cell, Universe};

use crate::graphics::renderer::Renderer;

pub struct Cremator {
    display: Display,
    event_loop: EventLoop<()>,
    renderer: Renderer,

    universe: Universe,

    tick_count: u64,
    _last_tick: Instant,
}

impl Cremator {
    pub fn new() -> Cremator {
        // graphics context creation
        let event_loop = EventLoop::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let builder = WindowBuilder::new()
            .with_title(env!("CARGO_PKG_NAME"))
            .with_inner_size(glutin::dpi::LogicalSize::new(1600f64, 1200f64));
        let display =
            Display::new(builder, context, &event_loop).expect("Failed to create display");
        let renderer = Renderer::init(&display);

        // universe creation
        let mut universe = Universe::new();
        universe.initialize();

        Cremator {
            display,
            event_loop,
            renderer,
            universe,
            tick_count: 0,
            _last_tick: Instant::now(),
        }
    }

    pub fn run(self) {
        let Cremator {
            display,
            event_loop,
            mut renderer,
            mut universe,
            mut tick_count,
            mut _last_tick,
        } = self;
        event_loop.run(move |event, _, control_flow| match event {
            // beginning
            Event::NewEvents(_) => {
                _last_tick = Instant::now();
                tick_count = tick_count.wrapping_add(1);
            }
            // updating
            Event::MainEventsCleared => Self::update(&mut universe, &mut renderer, &tick_count),
            // rendering
            Event::RedrawRequested(_) => Self::render(&mut renderer, &universe, &display),
            Event::RedrawEventsCleared => display.gl_window().window().request_redraw(),
            // window events
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter('x'),
                ..
            } => *control_flow = ControlFlow::Exit,
            // hand over any left over events
            event => renderer.handle_event(event, &display), // TODO: handle any other event
        })
    }

    fn update(universe: &mut Universe, renderer: &mut Renderer, tick_count: &u64) {
        if tick_count % 10 == 0 {
            universe.evolve();
        }
        renderer.update();
    }

    pub fn render(renderer: &mut Renderer, universe: &Universe, display: &Display) {
        renderer.render(universe, display);
    }

    pub fn read_rls(&mut self, pattern: &str) {
        let (mut x, mut y) = (0i64, 0i64);
        let mut argument: u32 = 0;

        for line in pattern.lines() {
            if line.starts_with('x') || line.starts_with('#') {
                continue;
            }
            let line = line.trim();
            for c in line.chars() {
                let parameter: u32 = if argument == 0 { 1 } else { argument };

                match c {
                    'b' => {
                        x += parameter as i64;
                        argument = 0;
                    }
                    'o' => {
                        for _ in 0..parameter {
                            self.universe.set_cell((x, y), Cell::Alive);
                            x += 1;
                        }
                        argument = 0
                    }
                    '$' => {
                        y += parameter as i64;
                        x = 0;
                        argument = 0;
                    }
                    '!' => return,
                    _ if c.is_digit(10) => {
                        argument = 10 * argument + c.to_digit(10).unwrap();
                    }
                    _ => panic!("invalid char"),
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn read_rls_from_stdin(&mut self) -> Result<()> {
        let mut string = String::new();

        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();

        while handle.read_line(&mut line)? != 0 {
            string.push_str(&line);
            if line.contains('!') {
                break;
            }
        }

        self.read_rls(&string);

        Ok(())
    }
}
