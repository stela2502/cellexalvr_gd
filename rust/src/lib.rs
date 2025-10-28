use godot::prelude::*;

mod printforge_core;
mod data_store;
mod umap_graph_3d;
mod cell_u_vec4;

use printforge_core::PrintForgeCore;


#[gdextension]
unsafe impl ExtensionLibrary for PrintForgeCore {}