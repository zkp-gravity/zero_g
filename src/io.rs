use std::path::Path;

use hdf5::{File, Result};
use image::ImageError;
use ndarray::{s, Array, Array2, Array3};
use ndarray::{Ix1, Ix3};

use crate::wnn::Wnn;

pub fn load_image(img_path: &Path) -> Result<Array2<u8>, ImageError> {
    let image = image::open(img_path)?.to_rgb8();
    let array: Array3<u8> = Array::from_shape_vec(
        (image.height() as usize, image.width() as usize, 3),
        image.into_raw(),
    )
    .expect("Error converting image to ndarray");

    let array = array.slice_move(s![.., .., 0]);

    Ok(array)
}

pub fn load_wnn(path: &Path) -> Result<Wnn> {
    let file = File::open(path)?;

    let num_classes = file.attr("num_classes")?.read_scalar::<i64>()? as usize;
    let num_inputs = file.attr("num_inputs")?.read_scalar::<i64>()? as usize;
    let bits_per_input = file.attr("bits_per_input")?.read_scalar::<i64>()? as usize;
    let num_filter_inputs = file.attr("num_filter_inputs")?.read_scalar::<i64>()? as usize;
    let num_filter_entries = file.attr("num_filter_entries")?.read_scalar::<i64>()? as usize;
    let num_filter_hashes = file.attr("num_filter_hashes")?.read_scalar::<i64>()? as usize;
    let p = file.attr("p")?.read_scalar::<i64>()? as u64;

    let expected_shape = [
        num_classes,
        num_inputs * bits_per_input / num_filter_inputs,
        num_filter_entries,
    ];
    let bloom_filters = file.dataset("bloom_filters")?;
    let bloom_filters = bloom_filters.read::<bool, Ix3>()?;
    assert_eq!(bloom_filters.shape(), expected_shape);

    let width = (num_inputs as f32).sqrt() as usize;
    let expected_shape = [width, width, bits_per_input];
    let binarization_thresholds = file.dataset("binarization_thresholds")?;
    let binarization_thresholds = binarization_thresholds.read::<f32, Ix3>()?;
    let binarization_thresholds = binarization_thresholds * 255.0;
    assert_eq!(binarization_thresholds.shape(), expected_shape);

    let input_order = file.dataset("input_order")?;
    let input_order = input_order.read::<u64, Ix1>()?;
    let num_input_bits = num_inputs * bits_per_input;
    assert_eq!(input_order.shape(), [num_input_bits]);

    Ok(Wnn::new(
        num_classes,
        num_filter_entries,
        num_filter_hashes,
        num_filter_inputs,
        p,
        bloom_filters,
        input_order,
        binarization_thresholds,
    ))
}
