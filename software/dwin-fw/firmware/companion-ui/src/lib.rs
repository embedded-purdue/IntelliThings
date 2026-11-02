//! companion display crate

#![cfg_attr(not(test), no_std)]

pub mod air;
pub mod canvas;
pub mod clock;
pub mod demo;
pub mod dgus;
pub mod director;
#[rustfmt::skip]
pub mod generated;
pub mod lightbar;
pub mod model;
pub mod screens;
pub mod text;
pub mod ui;
pub mod vp;
#[rustfmt::skip]
pub mod vp_map;

/// node vp map
#[cfg(test)]
#[path = "../../../handoff/dwin_vp.rs"]
#[allow(clippy::all)]
mod dwin_vp;
