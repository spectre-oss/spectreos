pub(super) const VERTEX: &str = r#"#version 300 es
precision highp float;
layout(location=0) in vec2 position;
layout(location=1) in vec2 local;
layout(location=2) in vec2 uv;
layout(location=3) in vec4 color;
layout(location=4) in vec2 half_size;
layout(location=5) in float radius;
layout(location=6) in float mode;
uniform vec2 viewport;
out vec2 v_local;
out vec2 v_uv;
out vec4 v_color;
flat out vec2 v_half;
flat out float v_radius;
flat out float v_mode;
void main() {
    float vh = abs(viewport.y);
    float ny = 1.0 - position.y / vh * 2.0;
    gl_Position = vec4(position.x / viewport.x * 2.0 - 1.0, viewport.y < 0.0 ? -ny : ny, 0.0, 1.0);
    v_local = local; v_uv = uv; v_color = color;
    v_half = half_size; v_radius = radius; v_mode = mode;
}
"#;
pub(super) const FRAGMENT: &str = r#"#version 300 es
precision highp float;
uniform sampler2D atlas;
uniform sampler2D image_atlas;
in vec2 v_local;
in vec2 v_uv;
in vec4 v_color;
flat in vec2 v_half;
flat in float v_radius;
flat in float v_mode;
out vec4 output_color;
void main() {
    // Hot path 1: solid axis-aligned rectangle (zero SDF / fwidth ALU cost)
    if (v_mode == 0.0 && v_radius <= 0.0) {
        float alpha = v_color.a;
        output_color = vec4(v_color.rgb * alpha, alpha);
        return;
    }
    // Hot path 2: glyph coverage from R8 font atlas with sRGB luminance-aware contrast tuning
    // (stem darkening for dark ink on light surfaces; crisp light ink on dark surfaces).
    if (v_mode > 0.85 && v_mode < 1.15) {
        float cov = texture(atlas, v_uv).r;
        if (cov > 0.0 && cov < 1.0) {
            float lum = dot(v_color.rgb, vec3(0.2126, 0.7152, 0.0722));
            cov = pow(cov, mix(0.76, 1.08, lum));
        }
        float alpha = v_color.a * cov;
        output_color = vec4(v_color.rgb * alpha, alpha);
        return;
    }
    // Hot path 3: offscreen layer composite (premultiplied RGBA)
    if (v_mode > 0.05 && v_mode < 0.15) {
        output_color = texture(image_atlas, v_uv) * v_color.a;
        return;
    }
    // Hot path 4b: MSDF (Multi-Channel Signed Distance Field) resolution-independent vector icon
    if (v_mode > -0.65 && v_mode < -0.50) {
        vec3 msd = textureLod(image_atlas, v_uv, 0.0).rgb;
        float sd = max(min(msd.r, msd.g), min(max(msd.r, msd.g), msd.b)) - 0.5;
        vec2 unit_range = vec2(v_radius) / vec2(textureSize(image_atlas, 0));
        vec2 screen_tex_size = vec2(1.0) / max(fwidth(v_uv), vec2(0.00001));
        float screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
        float cov = clamp(screen_px_range * sd + 0.5, 0.0, 1.0);
        if (cov > 0.0 && cov < 1.0) {
            float lum = dot(v_color.rgb, vec3(0.2126, 0.7152, 0.0722));
            cov = pow(cov, mix(0.84, 1.04, lum));
        }
        float alpha = v_color.a * cov;
        output_color = vec4(v_color.rgb * alpha, alpha);
        return;
    }
    // Hot path 4: premultiplied RGBA image quad (skip SDF when unrounded)
    if (v_mode > -0.5 && v_mode < -0.1) {
        float coverage = 1.0;
        if (v_radius > 0.0) {
            vec2 q = abs(v_local) - v_half + v_radius;
            float distance = min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - v_radius;
            float aa = max(fwidth(distance), 0.001);
            coverage = 1.0 - smoothstep(-aa * 0.5, aa * 0.5, distance);
        }
        vec4 image = texture(image_atlas, v_uv);
        float tint_a = v_color.a * coverage;
        output_color = vec4(image.rgb * v_color.rgb * tint_a, image.a * tint_a);
        return;
    }
    // Hot path 5: rounded rectangle
    if (v_mode == 0.0) {
        vec2 q = abs(v_local) - v_half + v_radius;
        float distance = min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - v_radius;
        float aa = max(fwidth(distance), 0.001);
        float alpha = v_color.a * (1.0 - smoothstep(-aa * 0.5, aa * 0.5, distance));
        output_color = vec4(v_color.rgb * alpha, alpha);
        return;
    }
    // Hot path 6: single-pass analytical soft rounded-box shadow
    if (v_mode > -0.85 && v_mode < -0.65) {
        float blur = max(v_uv.x, 0.5);
        vec2 box_half = max(v_half - vec2(blur), vec2(0.0));
        float r = min(max(v_uv.y, 0.0), min(box_half.x, box_half.y));
        vec2 q = abs(v_local) - box_half + r;
        float d = min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
        float u = 1.0 - clamp(d / blur, 0.0, 1.0);
        float coverage = u * u * (3.0 - 2.0 * u);
        float alpha = v_color.a * coverage;
        output_color = vec4(v_color.rgb * alpha, alpha);
        return;
    }
    float coverage;
    if (v_mode < -1.5) {
        bool rounded=v_mode>-10001.0;
        float radius = -v_mode - (rounded?2.0:10002.0);
        float angle = atan(v_local.y, v_local.x);
        float delta = mod(angle - v_uv.x, 6.28318530718);
        float sweep = min(v_uv.y, 6.28318530718);
        float d = abs(length(v_local)-radius);
        if (!rounded && delta>sweep) {discard;}
        if (delta > sweep) {
            vec2 a=radius*vec2(cos(v_uv.x),sin(v_uv.x));
            vec2 b=radius*vec2(cos(v_uv.x+sweep),sin(v_uv.x+sweep));
            d=min(length(v_local-a),length(v_local-b));
        }
        d-=v_radius*0.5;
        float aa=max(fwidth(d),0.001);
        coverage=1.0-smoothstep(-aa*0.5,aa*0.5,d);
    } else if (v_mode < -1.1) {
        vec2 delta=v_uv*2.0;
        float t=clamp(dot(v_local+v_uv,delta)/max(dot(delta,delta),0.000001),0.0,1.0);
        float d=length(v_local+v_uv-t*delta)-v_radius;
        float aa=max(fwidth(d),0.001);
        coverage=1.0-smoothstep(-aa*0.5,aa*0.5,d);
    } else if (v_mode < -0.9) {
        coverage = 1.0;
    } else if (v_mode > 0.2 && v_mode < 0.3) {
        float d = length(v_local / max(v_half, vec2(0.001))) - 1.0;
        float aa = max(fwidth(d), 0.001);
        coverage = 1.0 - smoothstep(-aa * 0.5, aa * 0.5, d);
    } else if (v_mode > 0.45 && v_mode < 0.5) {
        vec2 h=max(v_half,vec2(0.001));
        vec2 normalized=v_local/h;
        float length_normalized=length(normalized);
        float gradient=length(normalized/h)/max(length_normalized,0.001);
        float outer=(length_normalized-1.0)/max(gradient,0.001);
        float width=min(max(v_uv.x,0.0),min(h.x,h.y));
        float d=max(outer,-outer-width);
        float aa=max(fwidth(d),0.001);
        coverage=1.0-smoothstep(-aa*0.5,aa*0.5,d);
    } else if (v_mode > 0.35 && v_mode < 0.45) {
        vec2 q=abs(v_local)-v_half+v_radius;
        float outer=min(max(q.x,q.y),0.0)+length(max(q,0.0))-v_radius;
        float width=min(max(v_uv.x,0.0),min(v_half.x,v_half.y));
        float d=max(outer,-outer-width);
        float aa=max(fwidth(d),0.001);
        coverage=1.0-smoothstep(-aa*0.5,aa*0.5,d);
    } else {
        vec2 q = abs(v_local) - v_half + v_radius;
        float distance = min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - v_radius;
        float aa = max(fwidth(distance), 0.001);
        coverage = 1.0 - smoothstep(-aa * 0.5, aa * 0.5, distance);
        if (v_mode > 1.5) {
            float ink_radius = max(v_mode - (v_mode>10001.0?10002.0:2.0), 0.001);
            float initial_radius = max(floor(max(v_half.x, v_half.y) * 0.4) * 0.5, 0.5);
            float solid = max(1.0 - 70.0 / initial_radius, 0.65);
            float radial = length(v_local - v_uv) / ink_radius;
            if (v_mode>10001.0) {float edge=length(v_local-v_uv)-ink_radius;float ink_aa=max(fwidth(edge),0.001);coverage*=1.0-smoothstep(-ink_aa*0.5,ink_aa*0.5,edge);}else {coverage *= clamp((1.0 - radial) / (1.0 - solid), 0.0, 1.0);}
        }
    }
    float alpha = v_color.a * coverage;
    output_color = vec4(v_color.rgb * alpha, alpha);
}
"#;

