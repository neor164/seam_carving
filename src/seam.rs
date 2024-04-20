use crate::{
    cost::{build_cost_matrix, find_shortest_path, Direction},
    sobel::{Kernel, Sobel},
};
use image::{DynamicImage, GrayImage, RgbImage};

fn remove_path_from_image<I>(
    img: &mut Vec<I>,
    path: Vec<usize>,
    no_channels: usize,
    dir: Direction,
    width: usize,
) where
    I: Copy + Default,
{
    match dir {
        Direction::Row => remove_path_from_image_dir_row(img, path, no_channels),
        Direction::Column => remove_path_from_image_dir_col(img, path, no_channels, width),
    }
}

fn remove_path_from_image_dir_col<I>(
    img: &mut Vec<I>,
    path: Vec<usize>,
    no_channels: usize,
    width: usize,
) where
    I: Copy + Default,
{
    let new_len = img.len() - path.len() * no_channels;
    for idx in path.iter().copied() {
        let mut idx = idx * no_channels;
        while idx + width * no_channels < img.len() {
            for i in 0..no_channels {
                let cur = idx + i;
                let from = idx + width * no_channels + i;
                img[cur] = img[from];
            }
            idx += width * no_channels;
        }
    }
    img.resize(new_len, Default::default());
}

fn remove_path_from_image_dir_row<I>(img: &mut Vec<I>, mut path: Vec<usize>, no_channels: usize)
where
    I: Copy + Default,
{
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

    img.resize(new_len, Default::default());
}
#[derive(Debug, Clone, Copy)]

pub struct Dims {
    width: usize,
    height: usize,
}

impl Dims {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

pub struct SeamCarver {
    orig: Dims,
    desired: Dims,
    img: Option<DynamicImage>,
    gray_buf: Vec<u8>,
    energy_buf: Vec<f32>,
}

impl SeamCarver {
    pub fn new(img: DynamicImage, new_width: usize, new_height: usize) -> Self {
        let width = img.width() as usize;
        let height = img.height() as usize;
        if new_height > height || new_width > width {
            panic!("Can only reduce img in size");
        }
        let orig = Dims::new(width, height);
        let desired = Dims::new(new_width, new_height);

        let gray_buf = img.grayscale().into_luma8().into_vec();
        let sobel = Sobel::new().kernel(Kernel::X3);
        let energy_buf = sobel.apply(&gray_buf, width, height);
        Self {
            orig,
            desired,
            img: Some(img),
            gray_buf,
            energy_buf,
        }
    }

    pub fn apply(mut self) -> DynamicImage {
        let w_diff = self.orig.width - self.desired.width;
        let h_diff = self.orig.height - self.desired.height;
        for _ in 0..h_diff {
            self.remove_seam(Direction::Column);
        }
        for _ in 0..w_diff {
            self.remove_seam(Direction::Row);
        }
        self.img.unwrap()
    }

    fn remove_seam(&mut self, dir: Direction) {
        let img = self.img.take().unwrap();
        let width = img.width() as usize;
        let height = img.height() as usize;
        let cost_mat = build_cost_matrix(&self.energy_buf, width, height, dir);
        let path = find_shortest_path(&cost_mat, width, height, dir);
        let (mut buf, no_channels) = match img {
            DynamicImage::ImageLuma8(img) => (img.into_vec(), 1),
            DynamicImage::ImageRgb8(img) => (img.into_vec(), 3),
            _ => panic!("unsupported image format"),
        };
        remove_path_from_image(&mut self.gray_buf, path.clone(), 1, dir, width);
        remove_path_from_image(&mut self.energy_buf, path.clone(), 1, dir, width);
        remove_path_from_image(&mut buf, path, no_channels, dir, width);

        let (width, height) = match dir {
            Direction::Row => (width - 1, height),
            Direction::Column => (width, height - 1),
        };
        let width = width as u32;
        let height = height as u32;
        let new_img = match no_channels {
            1 => GrayImage::from_vec(width, height, buf).unwrap().into(),
            _ => RgbImage::from_vec(width, height, buf).unwrap().into(),
        };
        self.img = Some(new_img);
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
        let len = img.len();
        remove_path_from_image(&mut img, path, no_channels, Direction::Row, len);
        let expected = vec![0, 1, 3, 5, 6, 8, 10];
        assert_eq!(expected.len(), img.len());
        assert_eq!(expected, img);
    }

    #[test]
    fn test_basic_remove_02() {
        let mut img = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let path = vec![0, 2, 4, 7, 9, 10];
        let no_channels = 1;
        let len = img.len();
        remove_path_from_image(&mut img, path, no_channels, Direction::Row, len);
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
        let len = img.len();
        remove_path_from_image(&mut img, path, no_channels, Direction::Row, len);
        let expected = vec![1, 1, 3, 3, 5, 5, 6, 6, 8, 8];
        assert_eq!(expected.len(), img.len());
        assert_eq!(expected, img);
    }

    #[test]
    fn test_remove_top_row() {
        let w = 5;
        let h = 4;
        #[rustfmt::skip]
        let mut energy = vec![
            0., 0., 0., 0., 0.,
            1., 1., 1., 1., 1.,
            2., 2., 2., 2., 2.,
            3., 3., 3., 3., 3.,
        ];
        let path = find_shortest_path(&energy, w, h, Direction::Column);
        let expected_path = vec![0, 1, 2, 3, 4];
        assert_eq!(path, expected_path);
        remove_path_from_image(&mut energy, path, 1, Direction::Column, w);
        #[rustfmt::skip]
        let expected = vec![
            1., 1., 1., 1., 1.,
            2., 2., 2., 2., 2.,
            3., 3., 3., 3., 3.,
        ];
        assert_eq!(expected, energy);
    }

    #[test]
    fn test_remove_top_row_with_channels() {
        let w = 5;
        let h = 4;
        #[rustfmt::skip]
        let energy = vec![
            0., 0., 0., 0., 0.,
            1., 1., 1., 1., 1.,
            2., 2., 2., 2., 2.,
            3., 3., 3., 3., 3.,
        ];
        #[rustfmt::skip]
        let mut img = vec![
            0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0.,
            1., 1., 1., 1., 1.,
            1., 1., 1., 1., 1.,
            1., 1., 1., 1., 1.,
            2., 2., 2., 2., 2.,
            2., 2., 2., 2., 2.,
            2., 2., 2., 2., 2.,
            3., 3., 3., 3., 3.,
            3., 3., 3., 3., 3.,
            3., 3., 3., 3., 3.,
        ];
        let path = find_shortest_path(&energy, w, h, Direction::Column);
        let expected_path = vec![0, 1, 2, 3, 4];
        assert_eq!(path, expected_path);
        remove_path_from_image(&mut img, path, 3, Direction::Column, w);
        #[rustfmt::skip]
        let expected = vec![
            1., 1., 1., 1., 1.,
            1., 1., 1., 1., 1.,
            1., 1., 1., 1., 1.,
            2., 2., 2., 2., 2.,
            2., 2., 2., 2., 2.,
            2., 2., 2., 2., 2.,
            3., 3., 3., 3., 3.,
            3., 3., 3., 3., 3.,
            3., 3., 3., 3., 3.,
        ];
        assert_eq!(expected, img);
    }

    #[test]
    fn test_full_cycle() {
        let src_path = format!("./test_data/src/broadway_tower.jpg");
        let img = ImageReader::open(src_path).unwrap().decode().unwrap();
        let width = img.width() as usize;
        let height = img.height() as usize;
        let new_img = SeamCarver::new(img, width / 2, height).apply();
        let fname = format!("./test_data/outputs/broadway_tower-sc-2.png");
        new_img.save(fname).unwrap();
    }
}
