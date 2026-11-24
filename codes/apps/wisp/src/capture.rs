//! Full-scene offscreen capture, independent of DRM direct scanout.
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Color32F, ExportMem, Frame, Offscreen, Renderer,
            element::RenderElement,
            gles::{GlesRenderbuffer, GlesRenderer},
            utils::draw_render_elements,
        },
    },
    utils::{Rectangle, Transform},
};

pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn capture<E: RenderElement<GlesRenderer>>(
    renderer: &mut GlesRenderer,
    elements: &[E],
    size: [u32; 2],
    scale: f64,
) -> Result<Pixels, String> {
    let run = || -> Result<Pixels, Box<dyn std::error::Error>> {
        let dimensions = (size[0] as i32, size[1] as i32).into();
        let mut buffer: GlesRenderbuffer = renderer.create_buffer(Fourcc::Abgr8888, dimensions)?;
        let mut target = renderer.bind(&mut buffer)?;
        let physical = (size[0] as i32, size[1] as i32).into();
        let rect = Rectangle::from_size(physical);
        let mut frame = renderer.render(&mut target, physical, Transform::Normal)?;
        frame.clear(Color32F::new(0.05, 0.06, 0.08, 1.0), &[rect])?;
        draw_render_elements::<GlesRenderer, _, _>(&mut frame, scale, elements, &[rect])?;
        frame.finish()?.wait()?;
        let mapping = renderer.copy_framebuffer(
            &target,
            Rectangle::from_size(dimensions),
            Fourcc::Abgr8888,
        )?;
        // Wisp renders with Transform::Normal into scanout-style FBOs: GL row
        // zero is logical row zero for both Smithay surfaces and Bhoo chrome.
        // GlesMapping::flipped() describes GL storage, not this output transform.
        let rgba = renderer.map_texture(&mapping)?.to_vec();
        Ok(Pixels {
            width: size[0],
            height: size[1],
            rgba,
        })
    };
    let mut run = run;
    run().map_err(|e| e.to_string())
}

#[cfg(feature = "remote_control")]
pub fn encode(p: Pixels) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut data, p.width, p.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&p.rgba)
            .map_err(|e| e.to_string())?;
    }
    Ok(data)
}
