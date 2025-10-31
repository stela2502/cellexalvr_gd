use godot::prelude::*;

mod printforge_core;
mod data_store;
mod umap_graph_3d;
mod xr_user;
mod utils;

use printforge_core::PrintForgeCore;
use xr_user::XrUser;

#[gdextension]
unsafe impl ExtensionLibrary for PrintForgeCore {}