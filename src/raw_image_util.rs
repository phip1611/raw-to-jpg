use crate::util::ReadableByteSlice;
use rawloader::RawImageData;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct RgbImage {
    rgb_data: Vec<(u8, u8, u8)>,
    height: usize,
    width: usize,
}

/// Loads a raw image and transforms it to a `RGB` format.
pub fn load_raw_image_as_0rgb<P: AsRef<Path>>(path: P) -> RgbImage {
    let decoded_raw = rawloader::decode_file(path).unwrap();
    let decoded_image_u16 = match &decoded_raw.data {
        RawImageData::Integer(data) => data,
        RawImageData::Float(_) => panic!("not supported yet"),
    };
    assert_eq!(
        decoded_image_u16.len(),
        decoded_raw.height * decoded_raw.width,
        "decoded image length must match height * width"
    );
    let mut decoded_image_u8 = decoded_image_u16
        .iter()
        .map(|val| {
            // todo find out how to interpret the u16!
            let val_f32 = *val as f32;
            // let u16_max_f32 = u16::MAX as f32;
            let u16_max_f32 = (2.0_f32.powf(14.0)) as f32;
            let u8_max_f32 = u8::MAX as f32;
            (val_f32 / u16_max_f32 * u8_max_f32) as u8
        })
        .collect::<Vec<u8>>();

    // prepare final RGB buffer
    let bytes_per_pixel = 3; // RGB
    let mut demosaic_buf = vec![0; bytes_per_pixel * decoded_raw.width * decoded_raw.height];
    let mut dst = bayer::RasterMut::new(
        decoded_raw.width,
        decoded_raw.height,
        bayer::RasterDepth::Depth8,
        &mut demosaic_buf,
    );

    // adapter so that `bayer::run_demosaic` can read from the Vec
    let mut decoded_image_u8 = ReadableByteSlice::new(decoded_image_u8.as_slice());

    bayer::run_demosaic(
        &mut decoded_image_u8,
        bayer::BayerDepth::Depth8,
        bayer::CFA::RGGB,
        bayer::Demosaic::Linear,
        &mut dst,
    )
    .unwrap();

    // "buf" contains the demosaiced data here because it belongs to the `bayer::RusterMut`
    // structure that was passed to `bayer::run_demosaic` as destination buffer.
    let rgb_buf = demosaic_buf
        .chunks(3)
        .map(|rgb| (rgb[0], rgb[1], rgb[2]))
        .collect();

    RgbImage {
        rgb_data: rgb_buf,
        height: decoded_raw.height,
        width: decoded_raw.width,
    }
}

#[cfg(test)]
mod tests {
    use crate::raw_image_util::load_raw_image_as_0rgb;
    use std::fs::File;

    #[test]
    fn test_raw_to_rgb_to_png() {
        let rgb = load_raw_image_as_0rgb("res/DSC08894.ARW");

        let out_file = File::create("DSC08894.png").unwrap();
        let mut w = std::io::BufWriter::new(out_file);
        let mut encoder = png::Encoder::new(w, rgb.width as u32, rgb.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let rgb_as_slice = rgb
            .rgb_data
            .iter()
            .map(|(r, g, b)| [*r, *g, *b].into_iter())
            .flatten()
            .collect::<Vec<_>>();

        writer.write_image_data(&rgb_as_slice).unwrap();
    }
}
