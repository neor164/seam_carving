use crate::{
    cost::{build_cost_matrix, find_shortest_path, Direction},
    sobel::{KernelType, Sobel},
};
use image::{DynamicImage, GrayImage, RgbImage};

fn remove_path_from_image(img: &mut Vec<u8>, mut path: Vec<usize>, no_channels: usize) {
    // We want the lowest index as the first item
    path.sort();
    path.reverse();

    let new_len = img.len() - path.len() * no_channels;
    let mut idx = path.pop().unwrap() * no_channels;
    let mut inc = no_channels;
    while let Some(next_idx) = path.pop().map(|i| i * no_channels) {
        for i in idx..(next_idx - inc) {
            img[i] = img[i + inc];
        }
        idx = next_idx - inc;
        inc += no_channels;
    }
    for i in idx..new_len {
        img[i] = img[i + inc];
    }

    img.resize(new_len, 0);
}

pub fn seam_carving(mut img: DynamicImage, new_width: usize, new_height: usize) -> DynamicImage {
    let width = img.width() as usize;
    let height = img.height() as usize;
    if new_height > height || new_width > width {
        panic!("Can only reduce img in size");
    }
    let w_diff = width - new_width;
    let h_diff = height - new_height;
    for _ in 0..w_diff {
        img = remove_seam(img, Direction::Column);
    }
    for _ in 0..h_diff {
        img = remove_seam(img, Direction::Row);
    }
    img
}

fn remove_seam(img: DynamicImage, dir: Direction) -> DynamicImage {
    let gray_img: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = img.grayscale().into_luma8();
    let sobel = Sobel::new().kernel(KernelType::X3);
    let width = gray_img.width() as usize;
    let height = gray_img.height() as usize;
    let energy = sobel.apply(&gray_img.as_raw(), width, height);
    let cost_mat = build_cost_matrix(&energy, width, height, dir);
    let path = find_shortest_path(&cost_mat, width, height, dir);
    let (mut buf, no_channels) = match img {
        DynamicImage::ImageLuma8(img) => (img.into_vec(), 1),
        DynamicImage::ImageRgb8(img) => (img.into_vec(), 3),
        _ => panic!("unsupported image format"),
    };
    remove_path_from_image(&mut buf, path, no_channels);
    let (width, height) = match dir {
        Direction::Row => (width - 1, height),
        Direction::Column => (width, height - 1),
    };
    let width = width as u32;
    let height = height as u32;
    match no_channels {
        1 => GrayImage::from_vec(width, height, buf).unwrap().into(),
        _ => RgbImage::from_vec(width, height, buf).unwrap().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::io::Reader as ImageReader;

    #[test]
    fn test_basic_remove_01() {
        let mut img = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let path = vec![2, 4, 7, 9];
        let no_channels = 1;
        remove_path_from_image(&mut img, path, no_channels);
        let expected = vec![0, 1, 3, 5, 6, 8, 10];
        assert_eq!(expected.len(), img.len());
        assert_eq!(expected, img);
    }

    #[test]
    fn test_basic_remove_02() {
        let mut img = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let path = vec![0, 2, 4, 7, 9, 10];
        let no_channels = 1;
        remove_path_from_image(&mut img, path, no_channels);
        let expected = vec![1, 3, 5, 6, 8];
        assert_eq!(expected.len(), img.len());
        assert_eq!(expected, img);
    }

    #[test]
    fn test_basic_remove_03() {
        let mut img = vec![
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10,
        ];
        let path = vec![0, 2, 4, 7, 9, 10];
        let no_channels = 2;
        remove_path_from_image(&mut img, path, no_channels);
        let expected = vec![1, 1, 3, 3, 5, 5, 6, 6, 8, 8];
        assert_eq!(expected.len(), img.len());
        assert_eq!(expected, img);
    }

    #[test]
    fn test_full_cycle() {
        let src_path = format!("./test_data/src/saturn.jpg");
        let img = ImageReader::open(src_path).unwrap().decode().unwrap();
        let width = img.width() as usize;
        let height = img.height() as usize;
        let new_img = seam_carving(img, width - 50, height - 50);
        let fname = format!("./test_data/outputs/saturn-sc-1.png");
        new_img.save(fname).unwrap();
    }
}
