use minifb::{Key, Window, WindowOptions};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};
use std::time::Instant;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const RUNNER_COUNT: usize = 20_000;
const RUNNER_RADIUS: usize = 1;
const ALIGNED_RUNNERS: usize = 40; // Number of runners in each row
const RUNNER_START_DISTANCE: usize = 2;

const T_MAX: f64 = 600.0; // Maximum simulation time in seconds
const TIME_FACTOR: f64 = 10.0;

const V_MEAN: f64 = 12.0;
const V_STANDARD_DEVIATION: f64 = 2.0;

const BACKGROUND_COLOR: u32 = 0x000000;

/// Represents a single runner in the race
struct Runner {
    x0: f64,           // Starting x-coordinate
    y0: f64,           // Starting y-coordinate
    v: f64,            // Velocity
    wave_index: usize, // Index to the wave the runner belongs to
}

/// Represents a wave with specific velocity ranges and color
struct Wave {
    v_min: f64,
    v_max: f64,
    color: u32,
}

/// Manages waves and assigns runners to the appropriate wave based on their velocity
struct WaveManager {
    waves: Vec<Wave>,
}

impl WaveManager {
    /// Creates a new `WaveManager` with predefined waves
    fn new() -> Self {
        Self {
            waves: vec![
                Wave {
                    v_min: 0.0,
                    v_max: 10.0,
                    color: 0xff0000,
                },
                Wave {
                    v_min: 10.0,
                    v_max: 12.0,
                    color: 0x0000ff,
                },
                Wave {
                    v_min: 12.0,
                    v_max: 15.0,
                    color: 0x00ff00,
                },
                Wave {
                    v_min: 15.0,
                    v_max: f64::INFINITY,
                    color: 0xffffff,
                },
            ],
        }
    }

    /// Assigns a wave index to a runner based on their velocity
    fn assign_wave(&self, velocity: f64) -> usize {
        for (i, wave) in self.waves.iter().enumerate() {
            if velocity >= wave.v_min && velocity < wave.v_max {
                return i;
            }
        }
        self.waves.len() - 1 // Default to the last wave if no match
    }

    /// Returns a reference to the wave by its index
    fn get_wave(&self, index: usize) -> &Wave {
        &self.waves[index]
    }
}

/// Represents a race with multiple runners
struct Race {
    runners: Vec<Runner>,
    wave_manager: WaveManager,
}

impl Race {
    /// Create and initialize runners in the race
    fn new(runner_count: usize) -> Self {
        let wave_manager = WaveManager::new();

        let mut runners = Vec::with_capacity(runner_count);

        let mut rng = thread_rng();
        let normal_distribution = Normal::new(V_MEAN, V_STANDARD_DEVIATION).unwrap();

        for _i in 0..runner_count {
            let velocity = normal_distribution.sample(&mut rng); // Tirage selon la loi normale

            // Assign the runner to the correct wave based on velocity
            let wave_index = wave_manager.assign_wave(velocity);

            runners.push(Runner {
                x0: 0.0,
                y0: 0.0,
                v: velocity,
                wave_index,
            });
        }

        let mut next_i0: usize = 0;
        let mut next_j0: usize = 0;

        for wave_index in 0..wave_manager.waves.len() {
            for runner in runners.iter_mut() {
                if runner.wave_index == wave_index {
                    let next_x0 = (next_j0 * RUNNER_START_DISTANCE) as f64;
                    let next_y0 = (next_i0 * RUNNER_START_DISTANCE) as f64;

                    runner.x0 = next_x0;
                    runner.y0 = next_y0;

                    if next_i0 == ALIGNED_RUNNERS {
                        next_i0 = 0;
                        next_j0 += 1;
                    } else {
                        next_i0 += 1;
                    }
                }
            }

            if next_i0 != 0 {
                next_i0 = 0;
                next_j0 += 1;
            }
        }

        Self {
            runners,
            wave_manager,
        }
    }

    /// Draw all runners at the given time on the buffer
    fn draw(&self, t: f64, buffer: &mut [u32]) {
        for runner in &self.runners {
            runner.draw(t, buffer, &self.wave_manager);
        }
    }
}

impl Runner {
    /// Draw the runner at time `t` on the buffer
    fn draw(&self, t: f64, buffer: &mut [u32], wave_manager: &WaveManager) {
        let new_x = self.x0 + self.v * t;
        let new_y = self.y0;

        // Get the wave properties for the runner
        let wave = wave_manager.get_wave(self.wave_index);

        // Draw runner with specific wave color
        for dx in -(RUNNER_RADIUS as isize + 1)..RUNNER_RADIUS as isize {
            for dy in -(RUNNER_RADIUS as isize + 1)..RUNNER_RADIUS as isize {
                if dx * dx + dy * dy <= (RUNNER_RADIUS * RUNNER_RADIUS) as isize {
                    draw_dot(new_x + dx as f64, new_y + dy as f64, buffer, wave.color);
                }
            }
        }
    }
}

/// Draws a dot at position (x, y) on the buffer with the specified color
fn draw_dot(x: f64, y: f64, buffer: &mut [u32], color: u32) {
    let x_mod = (x as usize) % WIDTH;
    let y_mod = y as usize + 2 * ALIGNED_RUNNERS * RUNNER_START_DISTANCE * (x as usize / WIDTH);

    if x_mod < WIDTH && y_mod < HEIGHT {
        let index = (y_mod as usize) * WIDTH + x_mod;
        buffer[index] = color;
    }
}

fn main() {
    let mut buffer = vec![BACKGROUND_COLOR; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Runners Simulation - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Unable to open window");

    let race = Race::new(RUNNER_COUNT);
    let start_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update elapsed time
        let elapsed_secs = start_time.elapsed().as_secs_f64() * TIME_FACTOR;
        if elapsed_secs >= T_MAX {
            break;
        }

        // Clear the buffer
        buffer.fill(BACKGROUND_COLOR);

        // Draw the race state
        race.draw(elapsed_secs, &mut buffer);

        // Render the buffer to the window
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update buffer");
    }
}
