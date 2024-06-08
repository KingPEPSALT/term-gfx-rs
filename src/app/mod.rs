pub mod state;
// TODO: move App into app.rs 
// TODO: make benchmark_app.rs - BenchmarkApp implements Application
use std::{
    cmp::Ordering,
    error::Error,
    io::{self, BufWriter, StdoutLock, Write},
    time::{Duration, Instant},
};

use crate::graphics::{
    BufferedCanvas, Camera, Canvas, Colour, DirectionalLight, LightColour, LightingContribution,
    Material, PointLight, Sphere, World, WorldVector,
};
use crossterm::{
    cursor,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use mouse_position::mouse_position::Mouse;
use nalgebra::{Vector2, Vector3};
use num_traits::{clamp, Zero};
use state::{Event, State};
fn set_mouse_pos(x: i32, y: i32) {
    unsafe {
        if winapi::um::winuser::SetCursorPos(x, y) == 0 {
            panic!("SetCursorPos failed");
        }
    }
}
#[derive(PartialEq, Debug)]
pub struct Calibration {
    top_left: Vector2<u16>,
    bottom_right: Vector2<u16>,
    cell_size: Vector2<u16>,
}

pub struct FrameTime {
    start: Instant,
    render: f64,
    update: f64,
    total: f64,
}

impl Calibration {
    /// Converts a pixel posiution to a cell position
    pub fn pixel_to_cell(&self, pixel_position: Vector2<u16>) -> Vector2<usize> {
        let adjusted_pos = Vector2::new(
            pixel_position.x - self.top_left.x,
            pixel_position.y - self.top_left.y,
        );
        Vector2::new(
            (adjusted_pos.x / (self.cell_size.x + 1)) as usize,
            (adjusted_pos.y / (self.cell_size.y + 1)) as usize,
        )
    }

    //  ---------------- MOUSE INPUT ----------------

    pub fn mouse_position() -> Vector2<u16> {
        match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => Vector2::new(x as u16, y as u16),
            Mouse::Error => Vector2::new(0, 0),
        }
    }
    /// Gets the hovered cell, `Calibration::pixel_to_cell()`, clamps
    /// the mouse position to within the calibrated area
    pub fn mouse_cell(&self) -> Vector2<usize> {
        let mut position = Self::mouse_position();
        position.x = clamp(position.x, self.top_left.x, self.bottom_right.x);
        position.y = clamp(position.y, self.top_left.y, self.bottom_right.y);
        self.pixel_to_cell(position)
    }
}

pub trait Application<'a> {
    
    type Error: Error;

    /// Returns a new and prepared application
    /// ready for use
    fn fresh(title: &'a str) -> Self;

    /// Initialises the application ready
    /// for rendering
    fn initialise(&mut self) -> Result<(), Self::Error>;
    
    /// Clears the canvas and resets the cursor
    fn clear(&mut self) -> Result<(), Self::Error>;
    
    /// Called at the start of the frame, after Self::clear
    fn begin_frame(&mut self) -> Result<(), Self::Error>;

    /// Handles input, before Self::update
    fn input(&mut self) -> Result<(), Self::Error>;
    
    /// Update logic for the application, runs before Self::render
    fn update(&mut self) -> Result<(), Self::Error>;
    
    /// Renders the application
    fn render(&mut self) -> Result<(), Self::Error>;
    
    /// Called at the end of the frame, after Self::render
    fn end_frame(&mut self) -> Result<(), Self::Error>;

    /// Runs after the application exits the main loop
    fn end(&mut self) -> Result<(), Self::Error>;

    /// Whether the application is running
    fn is_running(&self) -> bool;
}

/// Writes to StdoutLock with a BufWriter
/// Uses a lock to avoid locking and unlocking,
/// is a BufWriter to allow faster prints, avoiding
/// buffering on new-lines (we print a lot of those).
pub type StdoutWriter<'a> = BufWriter<StdoutLock<'a>>;

pub struct App<'a> {
    /// Used for writing to Stdout
    _buf_writer: StdoutWriter<'a>,
    canvas: BufferedCanvas<'a, 200, 100, 1>,
    state: State,
    world: World,
    /// "Window" title
    title: &'a str,

    /// Calibration information, used for mouse position
    calibration: Calibration,

    /// Timing statistics: total, render and update time
    frame_time: FrameTime,

    mouse_pos_last_frame: Vector2<u16>,

    light_at: LightingContribution,
    fps_limit: Option<f64>,
}

impl<'a> App<'a> {
    // ---------------- STATE HANDLING ----------------

    /// Processes the current state (update logic for states)
    pub fn process(&mut self) -> Result<Option<Event>, io::Error> {
        match self.state {
            State::Running { start } => {
                self.world.camera.update(self.frame_time.total);
                let mouse_cell = self.calibration.mouse_cell();
                self.light_at = match self.world.trace_ray(
                    self.world
                        .camera
                        .from_canvas(mouse_cell, self.canvas.size()),
                    1f64,
                    10000f64,
                ) {
                    Some(hit) => self.world.get_lighting(&hit),
                    None => LightingContribution::default(),
                };
                for y in 0..self.canvas.size().y {
                    for x in 0..self.canvas.size().x {
                        let canvas_position = Vector2::new(x, y);
                        let colour = match self.world.trace_ray(
                            self.world
                                .camera
                                .from_canvas(canvas_position, self.canvas.size()),
                            1f64,
                            10000f64,
                        ) {
                            Some(hit) => self.world.compute_lighting(&hit),
                            None => Colour::zeros(),
                        };
                        self.canvas.put_pixel(colour, canvas_position);
                    }
                }
                self.canvas
                    .put_pixel(Colour::new(250, 160, 180), self.canvas.size() / 2);
                let fmt = |label: &str, max_integrals: usize, max_decimals: usize, val: f64| {
                    format!(
                        " {0:}: {1:>2$}ms ",
                        label,
                        format!("{0:.1$}", val, max_decimals),
                        max_integrals + max_decimals
                    )
                };

                // --- MOUSE & HOVER DEBUG ---
                self.canvas.write(
                    format!(
                        "    MOUSE: ({:>4},{:>4}) ",
                        self.mouse_pos_last_frame.x, self.mouse_pos_last_frame.y
                    ),
                    Colour::from_element(40),
                    Vector2::zero(),
                );

                self.canvas.write(
                    format!(
                        "  DIFFUSE: ({:>1.4}, {:>1.4}, {:>1.4}) ",
                        self.light_at.diffuse.x, self.light_at.diffuse.y, self.light_at.diffuse.y
                    ),
                    Colour::from_element(40),
                    Vector2::new(0, 1),
                );
                self.canvas.write(
                    format!(
                        " SPECULAR: ({:>1.4}, {:>1.4}, {:>1.4}) ",
                        self.light_at.specular.x,
                        self.light_at.specular.y,
                        self.light_at.specular.y
                    ),
                    Colour::from_element(40),
                    Vector2::new(0, 2),
                );
                self.canvas.write(
                    format!(
                        "  AMBIENT: ({:>1.4}, {:>1.4}, {:>1.4}) ",
                        self.light_at.ambient.x, self.light_at.ambient.y, self.light_at.ambient.y
                    ),
                    Colour::from_element(40),
                    Vector2::new(0, 3),
                );

                // ---- CAMERA DEBUG ----

                self.canvas.write(
                    format!(
                        " CAMERA: ({:>2.5}, {:>2.5}, {:>2.5}) ({:>2.5}, {:>2.5}) ({:>2.5}*{:>2.5})",
                        self.world.camera.position.x,
                        self.world.camera.position.y,
                        self.world.camera.position.z,
                        self.world.camera.yaw,
                        self.world.camera.pitch,
                        -self.world.camera.yaw.cos(),
                        self.world.camera.pitch.cos()
                    ),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 4),
                );

                self.canvas.write(
                    "   BASIS {                            ".to_string(),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 5),
                );
                self.canvas.write(
                    format!(
                        "     FORWARD: ({:>2.5}, {:>2.5}, {:>2.5})",
                        self.world.camera.basis.forward.x,
                        self.world.camera.basis.forward.y,
                        self.world.camera.basis.forward.z
                    ),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 6),
                );
                self.canvas.write(
                    format!(
                        "   RIGHTWARD: ({:>2.5}, {:>2.5}, {:>2.5})",
                        self.world.camera.basis.right.x,
                        self.world.camera.basis.right.y,
                        self.world.camera.basis.right.z
                    ),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 7),
                );
                self.canvas.write(
                    format!(
                        "      UPWARD: ({:>2.5}, {:>2.5}, {:>2.5})",
                        self.world.camera.basis.up.x,
                        self.world.camera.basis.up.y,
                        self.world.camera.basis.up.z
                    ),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 8),
                );
                self.canvas.write(
                    "   }                                  ".to_string(),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 9),
                );
                let (max_index, max_val) = self
                    .world
                    .camera
                    .basis
                    .forward
                    .iter()
                    .enumerate()
                    .max_by(|(_, &here), (_, &there)| {
                        if here < there {
                            Ordering::Less
                        } else if here == there {
                            Ordering::Equal
                        } else {
                            Ordering::Greater
                        }
                    })
                    .unwrap();

                let (min_index, min_val) = self
                    .world
                    .camera
                    .basis
                    .forward
                    .iter()
                    .enumerate()
                    .min_by(|&(_, &here), &(_, &there)| {
                        if here < there {
                            Ordering::Less
                        } else if here == there {
                            Ordering::Equal
                        } else {
                            Ordering::Greater
                        }
                    })
                    .unwrap();

                let suffix = if *max_val > min_val.abs() { "+" } else { "-" };
                let prefix = ["X", "Y", "Z"][if *max_val > min_val.abs() {
                    max_index
                } else {
                    min_index
                }];

                self.canvas.write(
                    format!("FACING: {}{}", prefix, suffix),
                    Colour::new(50, 100, 50),
                    Vector2::new(0, 10),
                );

                // ---- FRAME TIME DEBUG ----

                let elapsed = start.elapsed().as_secs();
                self.canvas.write(
                    format!(" T({:0>2}:{:0>2}) ", elapsed / 60, elapsed % 60),
                    Colour::from_element(40),
                    Vector2::new((self.canvas.size().x - 10) / 2, 0),
                );

                self.canvas.write(
                    fmt("UPDATE", 3, 4, self.frame_time.update),
                    Colour::new(150, 50, 50),
                    Vector2::new(0, self.canvas.size().y - 3),
                );
                self.canvas.write(
                    fmt("RENDER", 3, 4, self.frame_time.render),
                    Colour::new(50, 150, 50),
                    Vector2::new(0, self.canvas.size().y - 2),
                );
                self.canvas.write(
                    fmt(" FRAME", 3, 4, self.frame_time.total),
                    Colour::new(50, 50, 150),
                    Vector2::new(0, self.canvas.size().y - 1),
                );

                self.canvas.swap();

                Ok(None)
            }
            State::Calibrating { second_stage } => {

                self.canvas.put_pixel(
                    Colour::new(0, 255, 0),
                    if second_stage {
                        Vector2::new(self.canvas.size().x - 1, self.canvas.size().y - 1)
                    } else {
                        Vector2::zero()
                    },
                );

                self.canvas.full_swap();
                self.canvas.write(
                    if second_stage {
                        "Click on the bottom-left of the highlighted square"
                    } else {
                        "Click on the top-left of the highlighted square"
                    }
                    .to_string(),
                    Colour::new(100, 0, 0),
                    Vector2::new(self.canvas.size().x / 2 - 47 / 4, self.canvas.size().y / 2),
                );
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Handles inputs, uses state logic
    pub fn state_input(&mut self) -> Option<Event> {
        if inputbot::KeybdKey::QKey.is_pressed() {
            return Some(Event::Exited);
        }

        match &mut self.state {
            State::Calibrating { second_stage } => {
                if inputbot::MouseButton::LeftButton.is_pressed() {
                    if *second_stage {
                        self.calibration.bottom_right = Calibration::mouse_position();
                        self.calibration.cell_size =
                            self.calibration.bottom_right - self.calibration.top_left;
                        self.calibration.cell_size.x /= self.canvas.size().x as u16;
                        self.calibration.cell_size.y /= self.canvas.size().y as u16;
                        inputbot::MouseButton::LeftButton.release();
                        Some(Event::Calibrated)
                    } else {
                        *second_stage = true;
                        self.calibration.top_left = Calibration::mouse_position();
                        None
                    }
                } else {
                    None
                }
            }
            State::Running { start: _ } => {
                self.world.camera.input();
                let mouse_pos_this_frame = Calibration::mouse_position();

                self.world.camera.process_mouse_motion(
                    mouse_pos_this_frame.cast::<i16>() - Vector2::new(1000, 500),
                    self.frame_time.total,
                );
                self.mouse_pos_last_frame = mouse_pos_this_frame;

                set_mouse_pos(1000, 500);
                None
            }
            _ => None,
        }
    }

    /// Defines what each state should do on enter
    pub fn enter_state(&mut self) -> Result<(), io::Error> {
        match &mut self.state {
            _ => Ok(()),
        }
    }
    /// Defines what each state should do on exit
    pub fn exit_state(&mut self) -> Result<(), io::Error> {
        match &mut self.state {
            State::Calibrating { second_stage: true } => Ok(()),
            _ => Ok(()),
        }
    }

    /// Returns state to transition to if one exists
    pub fn next(&mut self, event: Event) -> Option<State> {
        match (self.state, event) {
            (_, Event::Exited) => Some(State::Exiting),
            (State::Initialising, Event::Initialised) => Some(State::Calibrating {
                second_stage: false,
            }),
            (State::Calibrating { second_stage: true }, Event::Calibrated) => {
                Some(State::Running {
                    start: Instant::now(),
                })
            }
            _ => None,
        }
    }

    /// Transitions between states,
    /// next defines the state-to-state transitions
    pub fn transit(&mut self, event: Event) -> Result<bool, io::Error> {
        Ok(match self.next(event) {
            Some(state) => {
                self.exit_state()?;
                self.state = state;
                self.enter_state()?;
                true
            }
            None => false,
        })
    }

    /// Current state
    #[inline]
    pub fn state(&self) -> &State {
        &self.state
    }
}

impl<'a> Application<'a> for App<'a> {

    type Error = io::Error;

    fn fresh(title: &'a str) -> Self {
        let canvas = BufferedCanvas::new();
        let size = canvas.size();
        let this = Self {
            _buf_writer: StdoutWriter::new(io::stdout().lock()),
            canvas,
            state: State::Initialising,
            world: World {
                ambient: LightColour::from_element(0.3),
                spheres: vec![
                    Sphere {
                        center: Vector3::new(0.0, 6.0, 10.0),
                        radius: 5.0,
                        material: Material {
                            colour: LightColour::x(),
                            specular: Some(500.0),
                        },
                    },
                    Sphere {
                        center: Vector3::new(10.0, 6.0, 0.0),
                        radius: 5.0,
                        material: Material {
                            colour: LightColour::z(),
                            specular: Some(500.0),
                        },
                    },
                    Sphere {
                        center: Vector3::new(0.0, 3.0, -10.0),
                        radius: 5.0,
                        material: Material {
                            colour: LightColour::y(),
                            specular: Some(10.0),
                        },
                    },
                    Sphere {
                        center: Vector3::new(-10.0, 3.0, 0.0),
                        radius: 5.0,
                        material: Material {
                            colour: LightColour::new(1.0, 1.0, 0.0),
                            specular: Some(1000.0),
                        },
                    },
                ],
                light_sources: vec![
                    Box::new(PointLight {
                        position: Vector3::zeros(),
                        colour: LightColour::from_element(0.5),
                    }),
                    Box::new(DirectionalLight {
                        direction: WorldVector::new(-1.0, -1.0, 1.0).normalize(),
                        colour: LightColour::from_element(1.0),
                    }),
                ],
                camera: Camera::new(size),
            },
            title, 
            calibration: Calibration {
                top_left: Vector2::zero(),
                bottom_right: Vector2::zero(),
                cell_size: Vector2::zero(),
            },
            frame_time: FrameTime {
                start: Instant::now(),
                render: 0.0,
                update: 0.0,
                total: 0.0,
            },
            light_at: LightingContribution::default(),
            mouse_pos_last_frame: Vector2::zero(),
            fps_limit: Some(144f64),
        };

        this
    }

    fn initialise(&mut self) -> Result<(), Self::Error> {
        self._buf_writer.execute(EnterAlternateScreen)?;
        self._buf_writer.execute(cursor::Hide)?;
        self._buf_writer.execute(SetTitle(self.title))?;
        enable_raw_mode()?;
        self.transit(Event::Initialised)?;

        Ok(())
    }

    fn input(&mut self) -> Result<(), Self::Error> {
        if let Some(event) = self.state_input() {
            self.transit(event)?;
        }
        Ok(())
    }

    fn update(&mut self) -> Result<(), Self::Error> {
        let now = Instant::now();
        {
            if let Some(event) = self.process()? {
                self.transit(event)?;
            };
        }
        self.frame_time.update = now.elapsed().as_millis_f64();

        Ok(())
    }

    fn render(&mut self) -> Result<(), Self::Error> {
        let now = Instant::now();
        let res = self.canvas.display(&mut self._buf_writer);
        self.frame_time.render = now.elapsed().as_millis_f64();
        res

    }
    fn begin_frame(&mut self) -> Result<(), Self::Error> {
        self.frame_time.start = Instant::now();
        Ok(())
    }
    fn end_frame(&mut self) -> Result<(), Self::Error> {
        self.frame_time.total = self.frame_time.start.elapsed().as_millis_f64();
        

        // Wait if we need to
        if let Some(fps) = self.fps_limit {
            let diff = 1000.0/fps - self.frame_time.total;
            if diff > 0.0 {
                std::thread::sleep(Duration::from_micros((diff*1000.0) as u64));
            }
        }

        Ok(())

    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self._buf_writer.execute(cursor::MoveTo(0, 0))?;
        self.canvas.clear();
        Ok(())
    }
    
    fn end(&mut self) -> Result<(), Self::Error> {
        self._buf_writer.execute(Clear(ClearType::All))?;
        self._buf_writer.flush()?;
        self._buf_writer.execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.state != State::Exiting
    }

}
