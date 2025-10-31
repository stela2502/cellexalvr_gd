use godot::prelude::Color;

/// Converts a Godot Color into a compact hex string ("#RRGGBB" or "#RRGGBBAA").
pub fn color_to_id(color: &Color) -> String {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u8;

    if a == 255 {
        // no transparency -> #RRGGBB
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        // keep alpha -> #RRGGBBAA
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }
}


pub fn id_to_color(id: &str) -> Color {
    let s = id.trim_start_matches('#');

    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
            Color::from_rgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            )
        }
        _ => Color::from_rgb(1.0, 1.0, 1.0), // fallback white
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use godot::prelude::Color;

    #[test]
    fn color_to_id_and_back_roundtrip() {
        let colors = vec![
            Color::from_rgb(0.0, 0.0, 0.0),
            Color::from_rgb(1.0, 1.0, 1.0),
            Color::from_rgb(0.5, 0.25, 0.75),
            Color::from_rgba(0.9, 0.1, 0.2, 0.5),
            Color::from_rgba(0.2, 0.9, 0.4, 0.9),
        ];

        for c in colors {
            let id = color_to_id(&c);
            let c2 = id_to_color(&id);

            assert!(
                (c.r - c2.r).abs() < 1.0 / 255.0,
                "r mismatch: {:?} -> {} -> {:?}",
                c, id, c2
            );
            assert!(
                (c.g - c2.g).abs() < 1.0 / 255.0,
                "g mismatch: {:?} -> {} -> {:?}",
                c, id, c2
            );
            assert!(
                (c.b - c2.b).abs() < 1.0 / 255.0,
                "b mismatch: {:?} -> {} -> {:?}",
                c, id, c2
            );
            assert!(
                (c.a - c2.a).abs() < 1.0 / 255.0,
                "a mismatch: {:?} -> {} -> {:?}",
                c, id, c2
            );
        }
    }

    #[test]
    fn color_to_u32_and_back_roundtrip() {
        let colors = vec![
            Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            Color::from_rgba(0.3, 0.6, 0.9, 1.0),
            Color::from_rgba(0.5, 0.2, 0.1, 0.3),
        ];

        for c in colors {
            let code = color_to_u32(&c);
            let c2 = u32_to_color(code);

            assert!(
                (c.r - c2.r).abs() < 1.0 / 255.0,
                "r mismatch: {:?} -> {:#X} -> {:?}",
                c, code, c2
            );
            assert!(
                (c.g - c2.g).abs() < 1.0 / 255.0,
                "g mismatch: {:?} -> {:#X} -> {:?}",
                c, code, c2
            );
            assert!(
                (c.b - c2.b).abs() < 1.0 / 255.0,
                "b mismatch: {:?} -> {:#X} -> {:?}",
                c, code, c2
            );
            assert!(
                (c.a - c2.a).abs() < 1.0 / 255.0,
                "a mismatch: {:?} -> {:#X} -> {:?}",
                c, code, c2
            );
        }
    }

    #[test]
    fn id_to_color_handles_edge_cases() {
        // incomplete or invalid inputs
        let invalids = vec!["", "#", "#GGHHII", "#12345", "nonsense"];

        for s in invalids {
            let c = id_to_color(s);
            // Should not panic, should return white fallback
            assert!(
                (c.r - 1.0).abs() < 1e-6 && (c.g - 1.0).abs() < 1e-6 && (c.b - 1.0).abs() < 1e-6,
                "Unexpected color for '{}': {:?}",
                s,
                c
            );
        }

        // valid but edge cases
        let c1 = id_to_color("#000000");
        let c2 = id_to_color("#FFFFFF");
        let c3 = id_to_color("#FF000080"); // red, half transparent

        assert!((c1.r - 0.0).abs() < 1e-6 && (c1.b - 0.0).abs() < 1e-6);
        assert!((c2.r - 1.0).abs() < 1e-6 && (c2.g - 1.0).abs() < 1e-6);
        assert!((c3.a - 0.5).abs() < 0.01, "alpha should be ≈0.5, got {:?}", c3.a);
    }
}