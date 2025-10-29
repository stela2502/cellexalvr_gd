extends XRController3D

@onready var ray: RayCast3D = $RayCast3D
var grabbed_object: Node3D = null
var is_grabbing := false
const GRIP_THRESHOLD := 0.6

func _ready():
	ray.enabled = true
	ray.collide_with_areas = true
	ray.collide_with_bodies = true

func _physics_process(_delta):
	# --- detect grip strength ---
	var grip_value := 0.0
	grip_value = get_float("squeeze")
	if grip_value == 0.0:
		grip_value = get_float("grip_force")
	if grip_value == 0.0:
		grip_value = get_float("trigger")
	var grip_pressed := grip_value > GRIP_THRESHOLD
	# --- raycast ---
	#print("Gripped? ", grip_value)
	var target: Node3D = null
	if ray.is_colliding():
		var collider = ray.get_collider()
		if collider is Node3D:
			target = collider

	# --- grab logic ---
	if grip_pressed and not is_grabbing and target:
		grabbed_object = target
		is_grabbing = true
		print("✋ Grab:", grabbed_object.name)
	elif not grip_pressed and is_grabbing:
		print("👐 Release:", grabbed_object.name)
		is_grabbing = false
		grabbed_object = null

	if is_grabbing and grabbed_object:
		grabbed_object.global_transform = self.global_transform
