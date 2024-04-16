pub enum Direction {
    Row,
    Column,
}

struct MapState {
    outer: usize,
    inner: usize,
    stride: usize,
    offset: usize,
}

impl MapState {
    fn from_dir(width: usize, height: usize, dir: Direction) -> Self {
        let (outer, inner, offset, stride) = match dir {
            Direction::Column => (width, height, width, 1),
            Direction::Row => (height, width, 1, width),
        };
        Self {
            outer,
            inner,
            stride,
            offset,
        }
    }
}

pub fn build_cost_matrix(energy: &[f32], width: usize, height: usize, dir: Direction) -> Vec<f32> {
    let mut res = vec![0.0; energy.len()];
    let state = MapState::from_dir(width, height, dir);
    // Copy the last row from the energy matrix
    let mut idx = (state.outer - 1) * state.stride;
    for _ in 0..state.inner {
        res[idx] = energy[idx];
        idx += state.offset;
    }

    // Loop over one col/row
    idx -= state.offset;
    for _ in 0..(state.outer - 1) {
        idx = idx - (state.inner - 1) * state.offset - state.stride;
        let mut e_idx = idx + state.stride;
        res[idx] = energy[idx] + min2(res[e_idx], res[e_idx + state.offset]);

        idx += state.offset;
        for _ in 1..(state.inner - 1) {
            res[idx] = energy[idx]
                + min3(
                    res[e_idx],
                    res[e_idx + state.offset],
                    res[e_idx + 2 * state.offset],
                );
            e_idx += state.offset;
            idx += state.offset;
        }

        res[idx] = energy[idx] + min2(res[e_idx], res[e_idx + state.offset]);
    }

    res
}

pub fn find_shortest_path(cost: &[f32], width: usize, height: usize, dir: Direction) -> Vec<usize> {
    let state = MapState::from_dir(width, height, dir);
    let mut res = Vec::with_capacity(state.outer);

    let mut idx = 0;
    let mut cur_min = f32::MAX;
    let mut min_idx = 0;
    for _ in 0..state.inner {
        let val = cost[idx];
        if cur_min > val {
            min_idx = idx;
            cur_min = val;
        }
        idx += state.offset;
    }
    res.push(min_idx);

    for _ in 0..state.outer - 1 {
        idx = min_idx + state.stride;
        min_idx = idx;
        cur_min = cost[idx];
        if (idx / state.offset) % state.inner != 0 {
            let o_idx = idx - state.offset;
            let val = cost[o_idx];
            if cur_min > val {
                min_idx = o_idx;
                cur_min = val;
            }
        }
        if (idx / state.offset) % state.inner != state.inner - 1 {
            let o_idx = idx + state.offset;
            let val = cost[o_idx];
            if cur_min > val {
                min_idx = o_idx;
            }
        }
        res.push(min_idx);
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
    fn test_build_cost_0_case(
        #[values(Direction::Row, Direction::Column)] dir: Direction,
        #[values(3, 4, 5)] width: usize,
        #[values(3, 4, 5)] height: usize,
    ) {
        let energy = vec![0.0; width * height];
        let costs = build_cost_matrix(&energy, width, height, dir);
        assert_eq!(energy, costs);
    }

    #[test]
    fn test_build_cost_01_row() {
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
    fn test_build_cost_01_col() {
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
    fn test_build_cost_02() {
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

    #[test]
    fn test_test_find_path_01() {
        let w = 5;
        let h = 4;
        #[rustfmt::skip]
        let energy = vec![
            7., 2., 3., 4., 5.,
            6., 9., 4., 2., 6.,
            5., 2., 5., 5., 1.,
            1., 3., 9., 8., 7.,
        ];
        let row_path = vec![1, 7, 11, 15];
        let col_path = vec![15, 11, 7, 8, 14];
        let path = find_shortest_path(&energy, w, h, Direction::Row);
        assert_eq!(row_path, path);
        let path = find_shortest_path(&energy, w, h, Direction::Column);
        assert_eq!(col_path, path);
    }

    #[test]
    fn test_test_find_path_02() {
        let w = 5;
        let h = 5;
        #[rustfmt::skip]
        let energy = vec![
            1., 8., 3., 4., 7.,
            6., 2., 8., 12., 6.,
            5., 7., 2., 13., 11.,
            4., 5., 9., 1., 7.,
            5., 4., 6., 7., 2.,
        ];
        let expected_path = vec![0, 6, 12, 18, 24];
        let path = find_shortest_path(&energy, w, h, Direction::Row);
        assert_eq!(expected_path, path);
        let path = find_shortest_path(&energy, w, h, Direction::Column);
        assert_eq!(expected_path, path);
    }

    #[test]
    fn test_test_find_path_03() {
        let w = 5;
        let h = 5;
        #[rustfmt::skip]
        let energy = vec![
            7., 8., 3., 4., 1.,
            6., 9., 8., 2., 6.,
            5., 2., 2., 5., 11.,
            4., 2., 9., 8., 7.,
            1., 4., 6., 7., 9.,
        ];
        let row_path = vec![4, 8, 12, 16, 20];
        let mut col_path = row_path.clone();
        col_path.reverse();
        let path = find_shortest_path(&energy, w, h, Direction::Row);
        assert_eq!(row_path, path);
        let path = find_shortest_path(&energy, w, h, Direction::Column);
        assert_eq!(col_path, path);
    }
}
