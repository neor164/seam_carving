pub enum Direction {
    Row,
    Column,
}

pub fn build_cost_matrix(energy: &[f32], width: usize, height: usize, dir: Direction) -> Vec<f32> {
    let mut res = vec![0.0; energy.len()];
    let (outer, inner, offset, stride) = match dir {
        Direction::Column => (width, height, width, 1),
        Direction::Row => (height, width, 1, width),
    };
    // Copy the last row from the energy matrix
    let mut idx = (outer - 1) * stride;
    for _ in 0..inner {
        res[idx] = energy[idx];
        idx += offset;
    }

    // Loop over one col/row
    idx -= offset;
    for _ in 0..(outer - 1) {
        idx = idx - (inner - 1) * offset - stride;
        let mut e_idx = idx + stride;
        res[idx] = energy[idx] + min2(res[e_idx], res[e_idx + offset]);

        idx += offset;
        for _ in 1..(inner - 1) {
            res[idx] = energy[idx] + min3(res[e_idx], res[e_idx + offset], res[e_idx + 2 * offset]);
            e_idx += offset;
            idx += offset;
        }

        res[idx] = energy[idx] + min2(res[e_idx], res[e_idx + offset]);
    }

    res
}

#[inline]
fn min2(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        v1
    } else {
        v2
    }
}

#[inline]
fn min3(v1: f32, v2: f32, v3: f32) -> f32 {
    min2(v1, min2(v2, v3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_zero_case(
        #[values(Direction::Row, Direction::Column)] dir: Direction,
        #[values(3, 4, 5)] width: usize,
        #[values(3, 4, 5)] height: usize,
    ) {
        let energy = vec![0.0; width * height];
        let costs = build_cost_matrix(&energy, width, height, dir);
        assert_eq!(energy, costs);
    }

    #[test]
    fn test_simple_case_01_row() {
        let w = 3;
        let h = 4;
        #[rustfmt::skip]
        let energy = vec![
            1., 0., 0.,
            0., 1., 0.,
            0., 0., 1.,
            2., 1., 3.
        ];
        #[rustfmt::skip]
        let expected = vec![
            2., 1., 1.,
            1., 2., 1.,
            1., 1., 2.,
            2., 1., 3.
        ];
        let dir = Direction::Row;
        let costs = build_cost_matrix(&energy, w, h, dir);
        assert_eq!(expected, costs);
    }

    #[test]
    fn test_simple_case_01_col() {
        let w = 3;
        let h = 4;
        #[rustfmt::skip]
        let energy = vec![
            1., 0., 0.,
            0., 1., 0.,
            0., 0., 1.,
            2., 1., 3.
        ];
        #[rustfmt::skip]
        let expected = vec![
            1., 0., 0.,
            0., 1., 0.,
            0., 0., 1.,
            2., 2., 3.
        ];
        let dir = Direction::Column;
        let costs = build_cost_matrix(&energy, w, h, dir);
        assert_eq!(expected, costs);
    }

    #[test]
    fn test_simple_case_02() {
        let w = 5;
        let h = 2;
        #[rustfmt::skip]
        let energy = vec![
            1., 2., 3., 4., 5.,
            10., 9., 8., 7., 6.
        ];
        #[rustfmt::skip]
        let expected = vec![
            10., 10., 10., 10., 11.0,
            10., 9., 8., 7., 6.
        ];
        let dir = Direction::Row;
        let costs = build_cost_matrix(&energy, w, h, dir);
        assert_eq!(expected, costs);
    }
}
