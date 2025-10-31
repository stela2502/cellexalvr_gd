use godot::prelude::*;
use godot::classes::{
    Engine, StandardMaterial3D, MeshInstance3D, SphereMesh, 
    XrController3D, XrServer, XrInterface, XrOrigin3D, XrCamera3D, 
    Area3D, CollisionShape3D, SphereShape3D, Node3D,
};
use crate::utils::color_to_id;
use crate::PrintForgeCore;

#[derive(GodotClass)]
#[class(base = Node3D, init)]
pub struct XrUser {
    #[base]
    base: Base<Node3D>,
    left: Option<Gd<XrController3D>>,
    right: Option<Gd<XrController3D>>,
    left_area: Option<Gd<Area3D>>,
    right_area: Option<Gd<Area3D>>,
    grouping_mode: bool,
    left_trigger: f32,
    right_trigger: f32,
    color_index: usize,
    colors: Vec<Color>,  // ✅ Now stored as Color, not String
}

#[godot_api]
impl XrUser {
    #[func]
    fn _ready(&mut self) {
        godot_print!("🧠 Initializing XR user (Rust)…");

        // --- check interface ---
        let xr_server = XrServer::singleton();
        if let Some(iface) = xr_server.find_interface("OpenXR") {
            let iface_ref = iface.cast::<XrInterface>();
            if iface_ref.is_initialized() {
                godot_print!("✅ OpenXR initialized: {}", iface_ref.get_name());
            } else {
                godot_error!("❌ OpenXR interface not initialized!");
                return;
            }
        } else {
            godot_error!("❌ OpenXR interface not found!");
            return;
        }

        // --- origin ---
        let mut origin = XrOrigin3D::new_alloc();
        origin.set_name("XrOrigin3D");
        self.base_mut().add_child(&origin);
        godot_print!("🧩 Spawned XrOrigin3D");

        // --- camera ---
        let mut cam = XrCamera3D::new_alloc();
        cam.set_name("XrCamera3D");
        origin.add_child(&cam);

        // --- left hand ---
        let mut left = XrController3D::new_alloc();
        left.set_name("LeftHand");
        left.set_tracker("left_hand");
        origin.add_child(&left);

        // --- right hand ---
        let mut right = XrController3D::new_alloc();
        right.set_name("RightHand");
        right.set_tracker("right_hand");
        origin.add_child(&right);
        self.colors = vec![
            Color::from_rgb(1.0, 0.0, 0.0),   // red
            Color::from_rgb(1.0, 0.5, 0.0),   // orange
            Color::from_rgb(1.0, 1.0, 0.0),   // yellow
            Color::from_rgb(0.5, 1.0, 0.0),   // yellow-green
            Color::from_rgb(0.0, 1.0, 0.0),   // green
            Color::from_rgb(0.0, 1.0, 0.5),   // spring green
            Color::from_rgb(0.0, 1.0, 1.0),   // cyan
            Color::from_rgb(0.0, 0.5, 1.0),   // sky blue
            Color::from_rgb(0.0, 0.0, 1.0),   // blue
            Color::from_rgb(0.5, 0.0, 1.0),   // violet
            Color::from_rgb(1.0, 0.0, 1.0),   // magenta
            Color::from_rgb(1.0, 0.0, 0.5),   // pink
            Color::from_rgb(0.7, 0.1, 0.1),   // dark red
            Color::from_rgb(1.0, 0.1, 0.6),   // deep pink
            Color::from_rgb(1.0, 0.84, 0.0),  // gold
            Color::from_rgb(0.0, 0.8, 0.8),   // turquoise
        ];
        self.color_index = 4;

        self.left_area = Some(Self::attach_selection_area(&mut left, &self.colors[self.color_index] ));
        self.right_area = Some(Self::attach_selection_area(&mut right, &self.colors[self.color_index]));

        self.left = Some(left);
        self.right = Some(right);

        self.grouping_mode = false;
        self.left_trigger = 0.0;
        self.right_trigger = 0.0;
        

        godot_print!("✅ XR origin + hands spawned");
    }

    fn attach_selection_area(controller: &mut Gd<XrController3D>, color:&Color ) -> Gd<Area3D> {
        let mut area = Area3D::new_alloc();
        area.set_name("SelectionArea");

        let mut shape_node = CollisionShape3D::new_alloc();
        shape_node.set_name("SelectionShape");
        let mut sphere = SphereShape3D::new_gd();
        sphere.set_radius(0.05);
        shape_node.set_shape(&sphere);
        area.add_child(&shape_node);

        let mut vis = MeshInstance3D::new_alloc();
        let mut mesh = SphereMesh::new_gd();
        mesh.set_radius(0.05);
        vis.set_mesh(&mesh);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo( *color );
        vis.set_material_override(&mat);
        area.add_child(&vis);

        controller.add_child(&area);
        godot_print!("🔹 Added SelectionArea to {}", controller.get_name());
        area
    }

    fn handle_main_hand_stick_input(&mut self, hand_name: &str ) {

        let (axis, area_opt) = match hand_name {
            "LeftHand" => {
                let Some(left) = self.left.as_ref() else {
                    return;
                };
                let Some(area) = self.left_area.as_ref() else {
                    return;
                };
                (left.get_vector2("thumbstick"), Some(area))
            }
            "RightHand" => {
                let Some(right) = self.right.as_ref() else {
                    return;
                };
                let Some(area) = self.right_area.as_ref() else {
                    return;
                };
                (right.get_vector2("thumbstick"), Some(area))
            }
            _ => unreachable!(),
        };
        if let Some(area) = area_opt {

            // Y-axis scaling
            if axis.y.round() != 0.0 {
                let factor = if axis.y > 0.0 { 1.05 } else { 0.95 };
                self.scale_area(&area, factor);
            };

            // X-axis change color
            let dir = axis.x.round() as i32;
            if dir != 0 {
                let len = self.colors.len() as i32;
                self.color_index = ((self.color_index as i32 + dir).rem_euclid(len)) as usize;
                let new_color = self.colors[self.color_index];
                if let Some(mut mesh) = area.try_get_node_as::<MeshInstance3D>("SelectionArea/MeshInstance3D") {
                    let mut mat = mesh.get_material_override().unwrap().cast::<StandardMaterial3D>();
                    mat.set_albedo(new_color);
                    mesh.set_material_override(&mat);
                }
                godot_print!("🎨 Switched to color {} -> {:?}", self.color_index, new_color);
            }
        }
    }


    fn scale_area(&self, area: &Gd<Area3D>, factor: f32) {
        let mut shape_node = area.get_node_as::<CollisionShape3D>("SelectionShape");
        let mut shape = shape_node.get_shape().unwrap().cast::<SphereShape3D>();
        let old = shape.get_radius();
        shape.set_radius(old * factor);
    }


    #[func]
    fn _physics_process(&mut self, _delta: f64) {
        // --- Read both hands' trigger and grip values ---
        if let (Some(left), Some(right)) = (&self.left, &self.right) {
            let lt = left.get_float("trigger");
            let lg = left.get_float("grip_strength");
            let rt = right.get_float("trigger");
            let rg = right.get_float("grip_strength");



            // optional debug
            godot_print!("L:{lt:.2}/{lg:.2}  R:{rt:.2}/{rg:.2}");

            // --- joystick actions ---
            self.handle_main_hand_stick_input( "LeftHand" );
            self.handle_main_hand_stick_input( "RightHand");


            // check trigger press (e.g. value > 0.8 threshold)
            let left_trigger_pressed = lt > 0.8;
            let right_trigger_pressed = rt > 0.8;

            // only proceed if grouping mode is on
            if self.grouping_mode {
                // Left-hand selection check
                if left_trigger_pressed {
                    self.handle_selection_hand( "LeftHand");
                }

                // Right-hand selection check
                if right_trigger_pressed {
                    self.handle_selection_hand("RightHand");
                }
            }
        }
    }

    fn handle_selection_hand(&mut self, hand_name: &str) {
        // 1️⃣ Selection must be active
        if !self.grouping_mode {
            return;
        }

        // 2️⃣ Check which hand & trigger value
        let (trigger_value, area_opt) = match hand_name {
            "LeftHand" => (self.left_trigger, self.left_area.as_ref()),
            "RightHand" => (self.right_trigger, self.right_area.as_ref()),
            _ => return,
        };

        // 3️⃣ Require trigger pressed
        if trigger_value < 0.6 {
            return;
        }

        // 4️⃣ Need a valid Area3D node
        let Some(area) = area_opt else {
            godot_warn!("⚠️ {hand_name}: missing Area3D");
            return;
        };

        let center = area.get_global_position();

        // 5️⃣ Try to get its CollisionShape3D child safely
        let Some(shape_node) = area.try_get_node_as::<CollisionShape3D>("SelectionShape") else {
            godot_warn!("⚠️ {hand_name}: missing SelectionShape");
            return;
        };

        // 6️⃣ Get its assigned Shape (SphereShape3D expected)
        let Some(shape_base) = shape_node.get_shape() else {
            godot_warn!("⚠️ {hand_name}: shape missing on SelectionShape");
            return;
        };

        // Cast the shape to SphereShape3D
        let Ok(shape) = shape_base.try_cast::<SphereShape3D>() else {
            godot_warn!("⚠️ {hand_name}: shape is not SphereShape3D");
            return;
        };

        let radius = shape.get_radius();

        // 7️⃣ Find the PrintForgeCore node via its group
        let mut tree = Engine::singleton()
            .get_main_loop()
            .unwrap()
            .cast::<SceneTree>();

        let cores = tree.get_nodes_in_group("PrintForgeCore");
        if cores.is_empty() {
            godot_error!("❌ No PrintForgeCore found in SceneTree");
            return;
        }

        // 8️⃣ Use the first core node found
        let Ok(mut core) = cores.at(0).try_cast::<PrintForgeCore>() else {
            godot_error!("❌ Could not cast node to PrintForgeCore");
            return;
        };

        // 9️⃣ Perform the selection call
        let color = self.colors[self.color_index];
        core.bind_mut().handle_selection(center, radius, color);

        godot_print!(
            "🎯 {hand_name} triggered selection at {:?} (r={:.3}) with color {:?}",
            center,
            radius,
            color
        );
    }

}
