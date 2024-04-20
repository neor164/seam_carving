use std::cmp::{max, min};

#[derive(Debug, Clone, Copy)]
pub enum Kernel {
    X5,
    X3,
}

impl Kernel {
    #[inline]
    fn size(self) -> usize {
        match self {
            Self::X5 => 5,
            Self::X3 => 3,
        }
    }

    fn x(self) -> Vec<i32> {
        match self {
            Self::X5 => vec![
                2, 2, 4, 2, 2, 1, 1, 2, 1, 1, 0, 0, 0, 0, 0, -1, -1, -2, -1, -1, -2, -2, -4, -2, -2,
            ],
            Self::X3 => vec![-1, -2, -1, 0, 0, 0, 1, 2, 1],
        }
    }

    fn y(self) -> Vec<i32> {
        match self {
            Self::X5 => vec![
                2, 1, 0, -1, -2, 2, 1, 0, -1, -2, 4, 2, 0, -2, -4, 2, 1, 0, -1, -2, 2, 1, 0, -1, -2,
            ],

            Self::X3 => vec![-1, 0, 1, -2, 0, 2, -1, 0, 1],
        }
    }
}

#[derive(Debug)]
pub struct Sobel {
    kernel: Kernel,
}

impl Sobel {
    pub fn new() -> Self {
        Self { kernel: Kernel::X3 }
    }

    pub fn kernel(mut self, kernel: Kernel) -> Self {
        self.kernel = kernel;
        self
    }

    pub fn apply(&self, image: &[u8], width: usize, height: usize) -> Vec<f32> {
        let kernel_x = self.kernel.x();
        let kernel_y = self.kernel.y();
        let mut buf = vec![0.0; image.len()];
        let ksize = self.kernel.size();
        let b = (ksize / 2) as isize;
        for r in 0..height {
            for c in 0..width {
                let start_x = max(c as isize - b, 0) as usize;
                let stop_x = min(c as isize + b, width as isize - 1) as usize;
                let start_y = max(r as isize - b, 0) as usize;
                let stop_y = min(r as isize + b, height as isize - 1) as usize;
                let mut val_kx = 0;
                let mut val_ky = 0;
                for ix in start_x..=stop_x {
                    for iy in start_y..=stop_y {
                        let kx = ix + b as usize - c;
                        let ky = iy + b as usize - r;
                        let val_i = image[ix + iy * width] as i32;
                        val_kx += val_i * kernel_x[kx + ky * ksize];
                        val_ky += val_i * kernel_y[kx + ky * ksize];
                    }
                }
                let mag = (val_kx.pow(2) + val_ky.pow(2)) as f32;
                buf[c + r * width] = mag.powf(0.5);
            }
        }
        buf
    }
}

impl Default for Sobel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::io::Reader as ImageReader;
    use image::GrayImage;
    use rstest::rstest;

    #[rstest]
    #[case(Kernel::X3, "saturn.jpg")]
    #[case(Kernel::X3, "valve.png")]
    #[case(Kernel::X5, "saturn.jpg")]
    #[case(Kernel::X5, "valve.png")]
    #[case(Kernel::X5, "broadway_tower.jpg")]
    #[case(Kernel::X3, "broadway_tower.jpg")]
    #[case(Kernel::X3, "broadway_tower_flilpped.png")]

    fn test_sobel(#[case] kernel_type: Kernel, #[case] img_name: &str) {
        let src_path = format!("./test_data/src/{img_name}");
        let img = ImageReader::open(src_path).unwrap().decode().unwrap();
        let img = img.grayscale().into_luma8();
        let sobel = Sobel::new().kernel(kernel_type);
        let result = sobel.apply(&img.as_raw(), img.width() as usize, img.height() as usize);
        let width = img.width();
        let height = img.height();
        let result = result.iter().map(|&x| x as u8).collect();
        let result = GrayImage::from_raw(width as u32, height as u32, result).unwrap();
        let ksize = kernel_type.size();
        let fname = format!("./test_data/outputs/{img_name}_edges_kernel{ksize}x{ksize}.png");
        result.save(fname).unwrap();
    }
}
