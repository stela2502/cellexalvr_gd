shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_opaque, blend_mix;

uniform float global_size : hint_range(1.0, 20.0) = 4.0;
uniform vec3 light_dir = vec3(0.4, 0.7, 0.5);
uniform float ambient = 0.25;
uniform float specular_strength = 0.8;

void vertex() {
    POINT_SIZE = global_size;
}

void fragment() {
    vec2 uv = (POINT_COORD - 0.5) * 2.0;
    float r2 = dot(uv, uv);
    if (r2 > 1.0) discard;

    // Fake sphere normal
    vec3 n = normalize(vec3(uv, sqrt(1.0 - r2)));

    // Lighting
    vec3 L = normalize(light_dir);
    float diff = max(dot(n, L), 0.0);
    vec3 base = COLOR.rgb;
    vec3 diffuse = base * diff;
    vec3 ambient_col = base * ambient;

    // Simple specular
    vec3 view_dir = vec3(0.0, 0.0, 1.0);
    vec3 half_dir = normalize(L + view_dir);
    float spec = pow(max(dot(n, half_dir), 0.0), 16.0) * specular_strength;

    vec3 shaded = ambient_col + diffuse + spec;
    COLOR = vec4(shaded, 1.0);
}