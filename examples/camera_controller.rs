use std::f32::consts::PI;

use cam::Camera;
use piston::input::mouse::MouseButton;
use piston::input::{Button, GenericEvent};

// Many of these constants only make sense based on values specified in `demo.rs`.

/// Determines for how long height changes after a scroll event. If set to 0.01, then height
/// velocity will decrease by 99% every second.
const HEIGHT_VELOCITY_AFTER_SECOND: f32 = 0.01;

/// Determines how quickly height changes the instant after a scroll event.
const INITIAL_HEIGHT_VELOCITY: f32 = 0.05;

/// When the user drags the mouse for one pixel with the camera at minimum height, this is the
/// resulting change in `look_at`.
const DRAG_DISTANCE_PER_PIXEL_MIN_HEIGHT: f32 = 0.0001;

/// Same as `DRAG_DISTANCE_PER_PIXEL_MIN_HEIGHT`, but when the camera is at maximum height.
const DRAG_DISTANCE_PER_PIXEL_MAX_HEIGHT: f32 = 0.002;

/// The lowest the camera can go.
const MIN_HEIGHT: f32 = 0.05;

/// The highest the camera can go.
const MAX_HEIGHT: f32 = 1.0;

/// The height above which the viewing angle will always be `MAX_ANGLE`.
const MAX_ANGLE_HEIGHT: f32 = 0.1;

/// The viewing angle when at `MIN_HEIGHT`.
const MIN_ANGLE: f32 = PI * 0.3;

/// The viewing angle when at `MAX_ANGLE_HEIGHT` or above.
const MAX_ANGLE: f32 = PI * 0.5;

/// The furthest "up" (north) the camera can be looking at.
const MAX_Y: f32 = 1.0;

/// The furthest "down" (south) the camera can be looking at.
const MIN_Y: f32 = 0.0;

fn clamp(min: f32, max: f32, n: f32) -> f32 {
    min.max(max.min(n))
}

fn linear_interpolate(min: f32, max: f32, t: f32) -> f32 {
    min + t * (max - min)
}

#[derive(Debug)]
pub struct CameraController {
    look_at: [f32; 2],
    height: f32,
    velocity: [f32; 3],
    dragging: bool,
}

impl CameraController {
    pub fn new() -> CameraController {
        CameraController {
            look_at: [0.0, 0.0],
            height: MAX_HEIGHT,
            velocity: [0.0, 0.0, 0.0],
            dragging: false,
        }
    }

    pub fn event<E>(&mut self, e: &E)
    where
        E: GenericEvent,
    {
        e.update(|args| {
            let dt = args.dt as f32;
            let velocity_loss_factor = HEIGHT_VELOCITY_AFTER_SECOND.powf(dt);

            let new_height = self.height + self.velocity[2] * dt;
            self.height = clamp(MIN_HEIGHT, MAX_HEIGHT, new_height);
            self.velocity[2] *= velocity_loss_factor;
        });

        e.mouse_scroll(|_scroll_x, scroll_y| {
            let scroll = -(scroll_y as f32);
            self.velocity[2] += scroll * INITIAL_HEIGHT_VELOCITY;
        });

        e.press(|button| { self.set_drag_if_middle(button, true); });

        e.release(|button| { self.set_drag_if_middle(button, false); });

        e.mouse_relative(|x, y| if self.dragging {
            let t = (self.height - MIN_HEIGHT) / (MAX_HEIGHT - MIN_HEIGHT);
            let drag_distance_per_pixel = linear_interpolate(
                DRAG_DISTANCE_PER_PIXEL_MIN_HEIGHT,
                DRAG_DISTANCE_PER_PIXEL_MAX_HEIGHT,
                t,
            );

            self.look_at[0] -= x as f32 * drag_distance_per_pixel;
            self.look_at[1] += y as f32 * drag_distance_per_pixel;

            self.look_at[1] = clamp(MIN_Y, MAX_Y, self.look_at[1]);
        });
    }

    /// If `button` is the middle mouse button, set dragging state to `dragging`.
    fn set_drag_if_middle(&mut self, button: Button, dragging: bool) {
        match button {
            Button::Mouse(mouse_button) => {
                match mouse_button {
                    MouseButton::Middle => {
                        self.dragging = dragging;
                    }
                    _ => {}
                };
            }
            _ => {}
        };
    }

    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let camera_look_at = [self.look_at[0], self.look_at[1], 0.0];

        let mut camera = Camera::new(self.camera_position());
        camera.look_at(camera_look_at);
        camera.orthogonal()
    }

    pub fn camera_position(&self) -> [f32; 3] {
        let angle = self.viewing_angle();
        let offset_y = self.height * (1.0 / angle.tan());

        [self.look_at[0], self.look_at[1] - offset_y, self.height]
    }

    pub fn camera_height(&self) -> f32 {
        self.height
    }

    pub fn look_at(&self) -> [f32; 2] {
        self.look_at.clone()
    }

    fn viewing_angle(&self) -> f32 {
        let h = MAX_ANGLE_HEIGHT.min(self.height);
        let t = (h - MIN_HEIGHT) / (MAX_ANGLE_HEIGHT - MIN_HEIGHT);

        linear_interpolate(MIN_ANGLE, MAX_ANGLE, t)
    }
}
