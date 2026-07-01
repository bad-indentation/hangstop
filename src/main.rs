//! # HangStop
//! (c) 2026 by bad_indentation
//!
//! Program entry point

use std::error::Error;

use hangstop::{Config, run};
use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
   run(Config::parse()) 
}
