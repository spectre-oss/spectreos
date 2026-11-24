//! Native ImageView matrix reflection fixture for the GLES framebuffer test.
use bhoo::{android_xml::LayoutState, window::{self, FrameAction, WindowOptions}, *};
#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_imageview_matrix.rs"]
mod example;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke=std::env::args().any(|x|x=="--smoke-test");
    let mut state=LayoutState::default();
    state.images.insert("quadrants".into(),Image::rgba(2,2,vec![255,0,0,255,0,255,0,255,0,0,255,255,255,255,0,255]).unwrap());
    state.image_scales.insert("quadrants".into(),40.0);
    state.image_sources.insert("matrix_example".into(),"quadrants".into());
    state.image_matrices.insert("matrix_example".into(),[-1.0,1.0,80.0,0.0]);
    let mut frames=0;
    window::run(WindowOptions{title:"Bhoo / image reflection".into(),size:[300,180],capture:std::env::var_os("BHOO_CAPTURE").map(Into::into),..WindowOptions::default()},move |f|{
        frames+=1;
        example::LAYOUT.render(f,Rect{x:0.0,y:0.0,width:144.0,height:100.0},&mut state);
        if smoke&&frames>=4 {FrameAction::Exit} else if smoke {FrameAction::Redraw} else {FrameAction::Wait}
    })
}
