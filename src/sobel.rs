use image::GrayImage;

#[derive(Clone, Copy)]
pub enum KernelType {
    X5,
    X3,
}

impl KernelType {
    #[inline]
    fn size(self) -> usize {
        match self {
            KernelType::X5 => 5,
            KernelType::X3 => 3,
        }
    }
}

struct Kernel {
    kernel_type: KernelType,
    patch: [u8; 25],
    kernel_x: [i8; 25],
    kernel_y: [i8; 25],
}

impl Kernel {
    fn new(kernel_type: KernelType) -> Self {
        let (kernel_x, kernel_y) = match kernel_type {
            KernelType::X5 => (
                [
                    2, 2, 4, 2, 2, 1, 1, 2, 1, 1, 0, 0, 0, 0, 0, -1, -1, -2, -1, -1, -2, -2, -4,
                    -2, -2,
                ],
                [
                    2, 1, 0, -1, -2, 2, 1, 0, -1, -2, 4, 2, 0, -2, -4, 2, 1, 0, -1, -2, 2, 1, 0,
                    -1, -2,
                ],
            ),
            KernelType::X3 => (
                [
                    -1, -2, -1, 0, 0, 0, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                [
                    -1, 0, 1, -2, 0, 2, -1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            ),
        };
        Self {
            kernel_type,
            patch: [0; 25],
            kernel_x,
            kernel_y,
        }
    }

    #[inline]
    fn calculate(&self) -> f32 {
        let klength = self.kernel_type.size().pow(2);
        let patch = &self.patch[..klength];
        let kernel_x = &self.kernel_x[..klength];
        let kernel_y = &self.kernel_y[..klength];

        let mut gx = 0;
        let mut gy = 0;

        for ((v, kx), ky) in patch
            .iter()
            .copied()
            .zip(kernel_x.iter().copied())
            .zip(kernel_y.iter().copied())
        {
            let vx = v as i16 * kx as i16;
            gx += vx as i32;
            let vy = v as i16 * ky as i16;
            gy += vy as i32;
        }
        let res = (gx.pow(2) + gy.pow(2)) as f32;
        let res = res.powf(0.5).round();

        res
    }
}

pub struct Sobel {
    kernel_type: KernelType,
}

impl Sobel {
    pub fn new() -> Self {
        Self {
            kernel_type: KernelType::X5,
        }
    }

    pub fn kernel(mut self, kernel: KernelType) -> Self {
        self.kernel_type = kernel;
        self
    }

    pub fn apply(&self, image: &GrayImage) -> Vec<f32> {
        let mut kernel = Kernel::new(self.kernel_type);
        let width = image.width() as usize;
        let height = image.height() as usize;
        let image = image.as_raw();
        let mut buf = vec![0.0; image.len()];
        let ksize = self.kernel_type.size();
        let b = ksize / 2;
        for x in b..width - b {
            for y in b..height - b {
                for i in 0..ksize {
                    let j = i as isize - b as isize;
                    let yi = (y as isize - j) as usize;
                    let src_slice = &image[x - b + (yi * width)..x + b + 1 + yi * width];
                    let dst_slice = &mut kernel.patch[i * ksize..(i + 1) * ksize];
                    dst_slice.copy_from_slice(src_slice);
                }
                let v = kernel.calculate();

                buf[x + y * width] = v;
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
    use rstest::rstest;

    #[rstest]
    #[case(KernelType::X3, "saturn.jpg")]
    #[case(KernelType::X3, "valve.png")]
    #[case(KernelType::X5, "saturn.jpg")]
    #[case(KernelType::X5, "valve.png")]

    fn test_sobel(#[case] kernel_type: KernelType, #[case] img_name: &str) {
        let src_path = format!("./test_data/src/{img_name}");
        let img = ImageReader::open(src_path).unwrap().decode().unwrap();
        let img = img.grayscale().into_luma8();
        let sobel = Sobel::new().kernel(kernel_type);
        let result = sobel.apply(&img);
        let width = img.width();
        let height = img.height();
        let result = result.iter().map(|&x| x as u8).collect();
        let result = GrayImage::from_raw(width as u32, height as u32, result).unwrap();
        let ksize = kernel_type.size();
        let fname = format!("./test_data/outputs/{img_name}_edges_kernel{ksize}x{ksize}.png");
        result.save(fname).unwrap();
    }
}
