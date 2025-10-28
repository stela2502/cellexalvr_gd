extends Camera3D

@export var orbit_target := Vector3.ZERO
@export var distance := 10.0
@export var zoom_speed := 1.0
@export var rotation_speed := 0.01

var rot_x := 0.0
var rot_y := 0.0
var vr_mode := false
var last_click_time := 0.0
const DOUBLE_CLICK_DELAY := 0.3   # seconds

func _ready():
	Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
	update_camera()

func _input(event):
	# --- Detect double click ---
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var now = Time.get_ticks_msec() / 1000.0
		if now - last_click_time < DOUBLE_CLICK_DELAY:
			toggle_vr_mode()
		last_click_time = now

	# --- ESC to leave VR ---
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE and vr_mode:
		toggle_vr_mode()

	# --- Zoom (always works, even outside VR) ---
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			distance = max(1.0, distance - zoom_speed)
			update_camera()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			distance += zoom_speed
			update_camera()

	# --- Orbit (only if in VR mode) ---
	if vr_mode and event is InputEventMouseMotion and Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT):
		rot_x -= event.relative.y * rotation_speed
		rot_y -= event.relative.x * rotation_speed
		rot_x = clamp(rot_x, -PI / 2, PI / 2)
		update_camera()

func toggle_vr_mode():
	vr_mode = !vr_mode
	if vr_mode:
		Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
		print("🎮 Entering VR / mouse-capture mode")
	else:
		Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
		print("🖱️ Returning to orbit / editor mode")

func update_camera():
	var t := Transform3D.IDENTITY
	t.origin = orbit_target
	t.basis = Basis(Vector3(0,1,0), rot_y).rotated(Vector3(1,0,0), rot_x)
	global_transform = t.translated(Vector3(0, 0, distance))
