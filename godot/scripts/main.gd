extends Node3D

@onready var core := PrintForgeCore.new()

# A simple “paddle” box that we’ll move with keys to test selection
var paddle_pos := Vector3(0, 0, 0)
var paddle_size := Vector3(0.5, 0.5, 0.5)

func _ready():
	var xr_interface = XRServer.find_interface("OpenXR")
	if xr_interface and xr_interface.initialize():
		get_viewport().use_xr = true
		print("✅ OpenXR initialized")
	else:
		print("❌ OpenXR not found")
	var path := "res://data/pbmc3k/"
	var dataset_path = ProjectSettings.globalize_path(path)
	print("📂 Checking files in:", path)

	core.load_dataset_and_projections("pbmc3k", dataset_path )
	print("✅ Finished loading, creating 3D objects");
	add_child(core)
