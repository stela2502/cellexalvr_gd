shader_type spatial;
render_mode blend_mix, depth_draw_opaque, depth_test_default, cull_back, diffuse_lambert, specular_schlick_ggx;

uniform float brightness : hint_range(0.0, 2.0) = 1.0;

#ifndef INSTANCE_CUSTOM
    #define INSTANCE_CUSTOM vec4(1.0)
#endif

void fragment() {
    vec3 color = INSTANCE_CUSTOM.rgb * brightness;
    COLOR = vec4(color, 1.0);
}

