// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

fn main() -> bake::Result<()> {
    bake::Registry::discover().and_then(|registry| registry.run())
}

mod registry;

#[path = "bake_generated_tasks/mod.rs"]
mod bake_generated_tasks;
