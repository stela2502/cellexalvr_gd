shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_opaque, blend_mix;

uniform float global_size : hint_range(1.0, 20.0) = 4.0;
uniform vec3 light_dir = vec3(0.4, 0.7, 0.5);   // fake light direction
uniform float rim_strength : hint_range(0.0, 2.0) = 0.6;
uniform float ambient : hint_range(0.0, 1.0) = 0.25;

void vertex() {
    POINT_SIZE = global_size;
}

void fragment() {
    // Map from [0,1] UV to [-1,1] space
    vec2 uv = (POINT_COORD - 0.5) * 2.0;
    float r2 = dot(uv, uv);
    if (r2 > 1.0) discard;

    // Fake sphere normal
    vec3 n = normalize(vec3(uv, sqrt(max(0.0, 1.0 - r2))));

    // Simple diffuse + rim lighting
    float diff = max(dot(n, normalize(light_dir)), 0.0);
    float rim  = pow(1.0 - diff, 2.0) * rim_strength;

    vec3 base = COLOR.rgb;
    vec3 shaded = base * (ambient + 0.75 * diff) + base * rim;

    ALBEDO = shaded;
    EMISSION = shaded;
    EMISSION = 0.6;
    ALPHA = 1.0;
}
