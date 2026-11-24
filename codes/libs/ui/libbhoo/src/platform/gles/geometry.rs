use super::Renderer;
use crate::{Color, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Affine2D {
    // x' = a*x + c*y + e
    // y' = b*x + d*y + f
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine2D {
    pub(super) const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub(super) fn is_identity(self) -> bool {
        self == Self::IDENTITY
    }

    pub(super) fn from_parts(
        origin: [f32; 2],
        translate: [f32; 2],
        scale: [f32; 2],
        rotation_rad: f32,
    ) -> Self {
        let (sin, cos) = rotation_rad.sin_cos();
        let a = scale[0] * cos;
        let b = scale[0] * sin;
        let c = -scale[1] * sin;
        let d = scale[1] * cos;
        let e = origin[0] + translate[0] - (a * origin[0] + c * origin[1]);
        let f = origin[1] + translate[1] - (b * origin[0] + d * origin[1]);
        Self { a, b, c, d, e, f }
    }

    pub(super) fn scale_y(origin: f32, scale: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: scale,
            e: 0.0,
            f: origin * (1.0 - scale),
        }
    }

    pub(super) fn then(self, local: Self) -> Self {
        Self {
            a: self.a * local.a + self.c * local.b,
            b: self.b * local.a + self.d * local.b,
            c: self.a * local.c + self.c * local.d,
            d: self.b * local.c + self.d * local.d,
            e: self.a * local.e + self.c * local.f + self.e,
            f: self.b * local.e + self.d * local.f + self.f,
        }
    }

    pub(super) fn apply(self, p: [f32; 2]) -> [f32; 2] {
        [
            self.a * p[0] + self.c * p[1] + self.e,
            self.b * p[0] + self.d * p[1] + self.f,
        ]
    }

    pub(super) fn transform_rect_aabb(self, rect: Rect) -> Rect {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return Rect::default();
        }
        if self.is_identity() {
            return rect;
        }
        let corners = [
            self.apply([rect.x, rect.y]),
            self.apply([rect.x + rect.width, rect.y]),
            self.apply([rect.x + rect.width, rect.y + rect.height]),
            self.apply([rect.x, rect.y + rect.height]),
        ];
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for [x, y] in corners {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        Rect {
            x: min_x,
            y: min_y,
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        }
    }
}

impl Renderer {
    pub(super) fn glyph_quad(&mut self, rect: Rect, uv: [f32; 4], color: Color, baseline: f32, shear: f32) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let half = [rect.width * 0.5, rect.height * 0.5];
        for [corner_x, corner_y] in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            let py = rect.y + corner_y * rect.height;
            let px = rect.x + corner_x * rect.width + (baseline - py) * shear;
            let pos = self.transform.apply([px, py]);
            self.vertices.extend_from_slice(&[
                pos[0],
                pos[1],
                (corner_x * 2.0 - 1.0) * half[0],
                (corner_y * 2.0 - 1.0) * half[1],
                uv[0] + corner_x * (uv[2] - uv[0]),
                uv[1] + corner_y * (uv[3] - uv[1]),
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0,
                half[0],
                half[1],
                0.0,
                1.0,
            ]);
        }
    }

    pub(super) fn quad(&mut self, rect: Rect, uv: [f32; 4], color: Color, radius: f32, mode: f32) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let clipped = match self.active_local_clip() {
            Some(clip) => rect.intersect(clip),
            None => rect,
        };
        if clipped.width <= 0.0 || clipped.height <= 0.0 {
            return;
        }
        if !self.transform.is_identity() {
            let screen = self
                .transform
                .transform_rect_aabb(clipped)
                .intersect(self.screen_clip);
            if screen.width <= 0.0 || screen.height <= 0.0 {
                return;
            }
        }
        let half = [rect.width * 0.5, rect.height * 0.5];
        for [corner_x, corner_y] in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            let u = (clipped.x + corner_x * clipped.width - rect.x) / rect.width;
            let v = (clipped.y + corner_y * clipped.height - rect.y) / rect.height;
            let pos = self
                .transform
                .apply([rect.x + u * rect.width, rect.y + v * rect.height]);
            self.vertices.extend_from_slice(&[
                pos[0],
                pos[1],
                (u * 2.0 - 1.0) * half[0],
                (v * 2.0 - 1.0) * half[1],
                uv[0] + u * (uv[2] - uv[0]),
                uv[1] + v * (uv[3] - uv[1]),
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0,
                half[0],
                half[1],
                radius.min(half[0]).min(half[1]).max(0.0),
                mode,
            ]);
        }
    }

    pub(super) fn static_shape(&mut self, bounds: Rect, index: usize, color: Color, rotation: f32) {
        let Some(shape) = crate::shape_data(index) else {
            return;
        };
        let rotated_clip = crate::shape_clip_bounds(bounds, rotation);
        let screen = self
            .transform
            .transform_rect_aabb(rotated_clip)
            .intersect(self.screen_clip);
        if screen.width <= 0.0 || screen.height <= 0.0 {
            return;
        }
        let previous_screen_clip = self.screen_clip;
        let previous_local_clip = self.local_clip;
        if self.transform.is_identity() {
            self.screen_clip = self.screen_clip.intersect(rotated_clip);
        } else {
            self.local_clip = Some(
                self.local_clip
                    .map_or(rotated_clip, |c| c.intersect(rotated_clip)),
            );
        }
        let size = bounds.width.min(bounds.height);
        let x = bounds.x + (bounds.width - size) / 2.0;
        let y = bounds.y + (bounds.height - size) / 2.0;
        let (sin, cos) = rotation.sin_cos();
        let transform = |p: [f32; 2]| {
            let (px, py) = (p[0] - 0.5, p[1] - 0.5);
            [
                x + (0.5 + px * cos - py * sin) * size,
                y + (0.5 + px * sin + py * cos) * size,
            ]
        };
        let point = |i: u16| {
            let p = transform(shape.points[i as usize]);
            [p[0], p[1], 1.0]
        };
        for tri in shape.triangles.as_chunks::<3>().0 {
            self.triangle([point(tri[0]), point(tri[1]), point(tri[2])], color);
        }
        for contour in shape.contours.windows(2) {
            let points = &shape.points[contour[0] as usize..contour[1] as usize];
            let area: f32 = points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .take(points.len())
                .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
                .sum();
            let winding = if area >= 0.0 { 1.0 } else { -1.0 };
            for (a, b) in points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .take(points.len())
            {
                let (a, b) = (transform(*a), transform(*b));
                let d = [b[0] - a[0], b[1] - a[1]];
                let length = d[0].hypot(d[1]);
                if length > 0.00001 {
                    let n = [
                        winding * d[1] / length / self.scale,
                        -winding * d[0] / length / self.scale,
                    ];
                    self.mesh_quad(
                        [
                            [a[0], a[1], 1.0],
                            [b[0], b[1], 1.0],
                            [b[0] + n[0], b[1] + n[1], 0.0],
                            [a[0] + n[0], a[1] + n[1], 0.0],
                        ],
                        color,
                    );
                }
            }
        }
        self.screen_clip = previous_screen_clip;
        self.local_clip = previous_local_clip;
    }

    pub(super) fn loading(&mut self, bounds: Rect, sample: crate::loading::LoadingSample, color: Color) {
        let screen = self
            .transform
            .transform_rect_aabb(bounds)
            .intersect(self.screen_clip);
        if screen.width <= 0.0 || screen.height <= 0.0 {
            return;
        }
        let mut points = std::mem::take(&mut self.path_points);
        points.clear();
        crate::loading::loading_cubics(bounds, sample, |c| {
            if points.is_empty() {
                points.push([c[0], c[1]]);
            }
            for step in 1..=12 {
                let t = step as f32 / 12.0;
                let u = 1.0 - t;
                points.push(std::array::from_fn(|axis| {
                    u * u * u * c[axis]
                        + 3.0 * u * u * t * c[axis + 2]
                        + 3.0 * u * t * t * c[axis + 4]
                        + t * t * t * c[axis + 6]
                }));
            }
        });
        let center = [
            bounds.x + bounds.width * 0.5,
            bounds.y + bounds.height * 0.5,
            1.0,
        ];
        // Pair adjacent fan triangles into quads; no degenerate fill geometry on
        // the common unclipped path, and half the previous upload size.
        for i in (1..points.len()).step_by(2) {
            let a = points[i - 1];
            let b = points[i];
            let c = points[(i + 1).min(points.len() - 1)];
            self.mesh_quad(
                [
                    center,
                    [a[0], a[1], 1.0],
                    [b[0], b[1], 1.0],
                    [c[0], c[1], 1.0],
                ],
                color,
            );
        }
        for edge in points.windows(2) {
            let (a, b) = (edge[0], edge[1]);
            // One physical pixel of exterior coverage, with reused mesh buffers.
            let delta = [b[0] - a[0], b[1] - a[1]];
            let length = delta[0].hypot(delta[1]);
            if length > 0.00001 {
                let n = [
                    delta[1] / length / self.scale,
                    -delta[0] / length / self.scale,
                ];
                let a0 = [a[0], a[1], 1.0];
                let b0 = [b[0], b[1], 1.0];
                let a1 = [a[0] + n[0], a[1] + n[1], 0.0];
                let b1 = [b[0] + n[0], b[1] + n[1], 0.0];
                self.mesh_quad([a0, b0, b1, a1], color);
            }
        }
        self.path_points = points;
    }

    pub(super) fn emit_quad(&mut self, points: [[f32; 3]; 4], color: Color) {
        for p in points {
            let pos = self.transform.apply([p[0], p[1]]);
            self.vertices.extend_from_slice(&[
                pos[0],
                pos[1],
                0.0,
                0.0,
                0.0,
                0.0,
                color.0 as f32 / 255.0,
                color.1 as f32 / 255.0,
                color.2 as f32 / 255.0,
                color.3 as f32 / 255.0 * p[2],
                1.0,
                1.0,
                0.0,
                -1.0,
            ]);
        }
    }

    pub(super) fn mesh_quad(&mut self, points: [[f32; 3]; 4], color: Color) {
        match self.active_local_clip() {
            None => self.emit_quad(points, color),
            Some(clip) => {
                if points.iter().all(|p| clip.contains([p[0], p[1]])) {
                    self.emit_quad(points, color);
                } else {
                    self.triangle([points[0], points[1], points[2]], color);
                    self.triangle([points[0], points[2], points[3]], color);
                }
            }
        }
    }

    pub(super) fn polyline(&mut self, points: [[f32; 2]; 3], count: u8, width: f32, color: Color) {
        let n = usize::from(count).clamp(2, 3);
        let normal = |a: [f32; 2], b: [f32; 2]| {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len = dx.hypot(dy).max(0.00001);
            [-dy / len, dx / len]
        };
        let first = normal(points[0], points[1]);
        let last = if n == 3 {
            normal(points[1], points[2])
        } else {
            first
        };
        let half = width * 0.5;
        let middle = if n == 3 {
            let sum = [first[0] + last[0], first[1] + last[1]];
            let dot = (sum[0] * last[0] + sum[1] * last[1]).max(0.01);
            [sum[0] * half / dot, sum[1] * half / dot]
        } else {
            [first[0] * half, first[1] * half]
        };
        let offsets = [
            [first[0] * half, first[1] * half],
            middle,
            [last[0] * half, last[1] * half],
        ];
        for i in 0..n - 1 {
            let p = points[i];
            let q = points[i + 1];
            let a = offsets[i];
            let b = offsets[i + 1];
            self.mesh_quad(
                [
                    [p[0] + a[0], p[1] + a[1], 1.0],
                    [q[0] + b[0], q[1] + b[1], 1.0],
                    [q[0] - b[0], q[1] - b[1], 1.0],
                    [p[0] - a[0], p[1] - a[1], 1.0],
                ],
                color,
            );
        }
        // Fade a one-physical-pixel fringe along the outer perimeter. This is
        // clipped by mesh_quad just like the filled stroke, including at joins.
        let mut perimeter = [[0.0; 2]; 6];
        for i in 0..n {
            perimeter[i] = [points[i][0] + offsets[i][0], points[i][1] + offsets[i][1]];
            perimeter[2 * n - 1 - i] = [points[i][0] - offsets[i][0], points[i][1] - offsets[i][1]];
        }
        let perimeter = &perimeter[..2 * n];
        let area: f32 = perimeter
            .iter()
            .zip(perimeter.iter().cycle().skip(1))
            .take(perimeter.len())
            .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
            .sum();
        let winding = if area >= 0.0 { 1.0 } else { -1.0 };
        for (&a, &b) in perimeter
            .iter()
            .zip(perimeter.iter().cycle().skip(1))
            .take(perimeter.len())
        {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = dx.hypot(dy);
            if length > 0.00001 {
                let offset = [
                    winding * dy / length / self.scale,
                    -winding * dx / length / self.scale,
                ];
                self.mesh_quad(
                    [
                        [a[0], a[1], 1.0],
                        [b[0], b[1], 1.0],
                        [b[0] + offset[0], b[1] + offset[1], 0.0],
                        [a[0] + offset[0], a[1] + offset[1], 0.0],
                    ],
                    color,
                );
            }
        }
    }

    pub(super) fn triangle(&mut self, triangle: [[f32; 3]; 3], color: Color) {
        let Some(clip) = self.active_local_clip() else {
            self.emit_quad([triangle[0], triangle[1], triangle[2], triangle[0]], color);
            return;
        };
        // Convex triangle/rect clipping, including fringe coverage, on the stack.
        let mut a = [[0.0; 3]; 12];
        let mut b = [[0.0; 3]; 12];
        a[..3].copy_from_slice(&triangle);
        let mut count = 3;
        for (axis, edge, greater) in [
            (0, clip.x, true),
            (0, clip.x + clip.width, false),
            (1, clip.y, true),
            (1, clip.y + clip.height, false),
        ] {
            if count == 0 {
                return;
            }
            let mut next = 0;
            for i in 0..count {
                let p = a[i];
                let q = a[(i + 1) % count];
                let inside = |v: [f32; 3]| {
                    if greater {
                        v[axis] >= edge
                    } else {
                        v[axis] <= edge
                    }
                };
                let pi = inside(p);
                let qi = inside(q);
                if pi {
                    b[next] = p;
                    next += 1;
                }
                if pi != qi {
                    let t = (edge - p[axis]) / (q[axis] - p[axis]);
                    b[next] = std::array::from_fn(|j| p[j] + t * (q[j] - p[j]));
                    next += 1;
                }
            }
            count = next;
            std::mem::swap(&mut a, &mut b);
        }
        if count < 3 {
            return;
        }
        for i in 1..count - 1 {
            self.emit_quad([a[0], a[i], a[i + 1], a[0]], color);
        }
    }
}
