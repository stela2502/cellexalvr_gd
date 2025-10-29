extends Node3D

@export var left_hand: XRController3D
@export var right_hand: XRController3D2

var grabbed := false
var two_handed := false
var initial_distance := 0.0
var initial_scale := Vector3.ONE
var initial_transform := Transform3D.IDENTITY

func _process(delta: float) -> void:
	
	if not left_hand or not right_hand:
		return

	var left_grip := left_hand.get_float("trigger")
	var right_grip := right_hand.get_float("trigger")

	# --- Two-handed scale/rotate ---
	if left_grip > 0.7 and right_grip > 0.7:
		if not two_handed:
			two_handed = true
			initial_distance = left_hand.global_position.distance_to(right_hand.global_position)
			initial_scale = scale
		var new_distance = left_hand.global_position.distance_to(right_hand.global_position)
		if initial_distance > 0.001:
			var scale_factor = new_distance / initial_distance
			scale = initial_scale * scale_factor
		look_at((left_hand.global_position + right_hand.global_position) / 2.0)
		global_position = (left_hand.global_position + right_hand.global_position) / 2.0

	# --- Single-hand grab & move ---
	elif left_grip > 0.7 or right_grip > 0.7:
		var hand = left_hand if left_grip > 0.7 else right_hand
		if not grabbed:
			grabbed = true
			initial_transform = hand.global_transform.affine_inverse() * global_transform
		global_transform = hand.global_transform * initial_transform

	else:
		grabbed = false
		two_handed = false
