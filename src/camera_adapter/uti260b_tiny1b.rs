use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    NokhwaError,
};

use crate::{temperature::Temp, thermal_data::ThermalData};

use super::CameraAdapter;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 192;
const COMBINED_IMAGE_HEIGHT: u32 = 386;
const THERMAL_OFFSET_ROWS: u32 = 192;

pub struct Uti260bTiny1bAdapter {}

//
// Adapter for UTi260B/Tiny1B-style USB thermal cameras.
//
// The tested device enumerates as Realtek USB bridge 0bda:3901 and advertises
// a 256x386 YUYV stream. Rows 0..191 contain a greyscale preview. Rows
// 192..383 contain 256x192 uint16 temperature data. Rows 384..385 are trailing
// metadata. The temperature samples are encoded in 1/16 K on the tested unit.
//
impl CameraAdapter for Uti260bTiny1bAdapter {
    fn name(&self) -> String {
        "UTi260B / Tiny1B UVC".to_string()
    }

    fn short_name(&self) -> String {
        "UTi260B".to_string()
    }

    fn requested_format(&self) -> RequestedFormat<'static> {
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(IMAGE_WIDTH, COMBINED_IMAGE_HEIGHT),
            FrameFormat::YUYV,
            25,
        )))
    }

    fn temperature_range(&self) -> (f32, f32) {
        (253.15, 823.15)
    }

    fn capture_thermal_data(&self, cam: &mut nokhwa::Camera) -> Result<ThermalData, NokhwaError> {
        let frame_data: std::borrow::Cow<'_, [u8]> = cam.frame_raw()?;

        let thermal_data_buf = &frame_data[((IMAGE_WIDTH * THERMAL_OFFSET_ROWS) * 2) as usize..]
            [..(IMAGE_WIDTH * IMAGE_HEIGHT * 2) as usize];

        let u16_temperature_data = unsafe {
            std::slice::from_raw_parts(
                thermal_data_buf.as_ptr() as *const u16,
                (IMAGE_WIDTH * IMAGE_HEIGHT) as usize,
            )
        };

        Ok::<ThermalData, NokhwaError>(ThermalData::new(
            IMAGE_WIDTH as usize,
            IMAGE_HEIGHT as usize,
            u16_temperature_data
                .iter()
                .map(|&x| Temp::new(x as f32 / 16.0))
                .collect(),
        ))
    }

    fn usb_vid_pids(&self) -> Vec<(u16, u16)> {
        vec![(0x0bda, 0x3901)]
    }
}
