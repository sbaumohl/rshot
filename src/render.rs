use image::{imageops::crop_imm, RgbaImage};
use wayland_client::protocol::wl_shm::Format;

use crate::{argparser::RegionSelect, state::ImageDims};

pub fn get_rgba(dims: &ImageDims, buf: &memmap2::Mmap) -> Vec<u8> {
    match dims.format {
        Format::Xrgb8888 | Format::Argb8888 => {
            let mut rgba_data = Vec::with_capacity(dims.total_size());

            for idx in (0..(dims.height * dims.stride) as usize).step_by(4) {
                let r = buf[idx + 2];
                let g = buf[idx + 1];
                let b = buf[idx];

                let a = buf[idx + 3];
                rgba_data.extend_from_slice(&[r, g, b, a]);
            }
            rgba_data
        }
        _ => {
            panic!("Format is not currently supported! ({:?})", dims.format);
        }
    }
}
pub fn crop_to_region(image: &RgbaImage, region: &RegionSelect) -> Result<RgbaImage, String> {
    if image.width() < region.top_left_origin.x || image.height() < region.top_left_origin.y {
        return Err("Selected region is empty!".to_string());
    }
    Ok(crop_imm(
        image,
        region.top_left_origin.x,
        region.top_left_origin.y,
        region.size.x,
        region.size.y,
    )
    .to_image())
}
