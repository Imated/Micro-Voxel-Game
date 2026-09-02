use std::{fmt::Debug, process::abort};

use glam::UVec3;
use tracing::error;

pub mod constants;
pub mod free_list;
pub mod pipeline;

#[must_use]
pub const fn flatten(x: u32, y: u32, z: u32, size: UVec3) -> u32 {
    z * size.y * size.x + y * size.x + x
}

pub trait ResultExt<T> {
    fn unwrap_or_abort<S: Into<String> + Debug>(self, msg: S) -> T;
}

impl<T, E: Debug> ResultExt<T> for Result<T, E> {
    fn unwrap_or_abort<S: Into<String> + Debug>(self, msg: S) -> T {
        self.unwrap_or_else(|err| {
            error!("{msg:?} Error: {err:?}");
            abort()
        })
    }
}
