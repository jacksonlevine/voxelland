

pub fn rotate_coordinates_around_y_negative_90(coords: &[f32], num_rotations: i32) -> Vec<f32> {
    if coords.len() % 5 != 0 {
        //Do nothing for now
    }
    let mut rotatedcoords = Vec::new();
    rotatedcoords.extend_from_slice(coords);
    for t in 0..num_rotations {
        for i in (0..rotatedcoords.len()).step_by(5) {
            let x = rotatedcoords[i];
            let z = rotatedcoords[i+2];

            rotatedcoords[i] = -z;
            rotatedcoords[i+2] = x;
        }
    }
    rotatedcoords
}