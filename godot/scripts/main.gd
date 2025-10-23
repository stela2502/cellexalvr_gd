extends Node3D

@onready var viewer := preload("res://extensions/printforge3d/umap_viewer.gdns").new()
@onready var shader_mat := load("res://materials/umap_instanced.tres")

# A simple “paddle” box that we’ll move with keys to test selection
var paddle_pos := Vector3(0, 0, 0)
var paddle_size := Vector3(0.5, 0.5, 0.5)

func _ready():
	add_child(viewer)

	# Generate some test data (replace this with your UMAP import later)
	var count := 200_000
	var flat := PackedFloat32Array()
	flat.resize(count * 3)
	for i in count:
		flat[i*3 + 0] = randf_range(-5.0, 5.0)
		flat[i*3 + 1] = randf_range(-5.0, 5.0)
		flat[i*3 + 2] = randf_range(-5.0, 5.0)

	viewer.load_umap(flat)

	# Apply the shader material to the viewer’s MultiMeshInstance3D via override.
	# (We set a StandardMaterial in Rust, but override beats it.)
	# Find the MM instance at runtime:
	for c in viewer.get_children():
		if c is MultiMeshInstance3D:
			c.material_override = shader_mat

func _process(delta):
	# Quick keyboard control for the paddle box (WASD + RF)
	var s := 3.0 * delta
	if Input.is_action_pressed("ui_up"):    paddle_pos.z -= s
	if Input.is_action_pressed("ui_down"):  paddle_pos.z += s
	if Input.is_action_pressed("ui_left"):  paddle_pos.x -= s
	if Input.is_action_pressed("ui_right"): paddle_pos.x += s
	if Input.is_key_pressed(KEY_R):         paddle_pos.y += s
	if Input.is_key_pressed(KEY_F):         paddle_pos.y -= s

	viewer.update_selection(paddle_pos, paddle_size)

