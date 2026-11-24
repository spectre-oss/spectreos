precision mediump float;
varying vec2 v_coords;
uniform float progress;
uniform float alpha;

void main() {
    // The visible area contracts into a soft ellipse at the bottom center.
    vec2 offset = vec2((v_coords.x - 0.5) * 1.12, 1.0 - v_coords.y);
    float distance_from_bottom = length(offset);
    float time = clamp(progress, 0.0, 1.0);
    float radius = mix(1.65, -0.24, time);
    float feather = mix(0.49, 0.20, time);
    float visible = 1.0 - smoothstep(radius - feather, radius + feather, distance_from_bottom);
    visible *= 1.0 - 0.24 * time;
    gl_FragColor = vec4(0.0, 0.0, 0.0, (1.0 - visible) * alpha);
}
