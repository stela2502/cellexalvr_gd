use godot::prelude::*;

mod printforge_core;
use printforge_core::PrintForgeCore;


#[gdextension]
unsafe impl ExtensionLibrary for PrintForgeCore {}