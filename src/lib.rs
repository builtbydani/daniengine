pub mod math;
pub mod physics;
pub mod render;
pub mod input;
pub mod scene;

// use daniengine::prelude::*;
pub mod prelude {
    pub use crate::math::*;
    pub use crate::physics::*;
    pub use crate::render::canvas::*;
}
