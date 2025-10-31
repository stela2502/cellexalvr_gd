# res://scripts/controller_input.gd
extends XRController3D

signal trigger(value: float)
signal grip(value: float)

func _on_input_float_changed(action_name: String, value: float) -> void:
	if action_name == "trigger":
		trigger.emit(value)
	elif action_name == "grip" or action_name == "grip_strength":
		grip.emit(value)
