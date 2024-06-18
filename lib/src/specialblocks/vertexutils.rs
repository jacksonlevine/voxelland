

pub fn rotate_coordinates_around_y_negative_90(coords: &[f32], num_rotations: i32) -> Vec<f32> {
    if coords.len() % 5 != 0 {
        // Do nothing for now
    }

    let mut rotatedcoords = Vec::new();
    rotatedcoords.extend_from_slice(coords);

    for _ in 0..num_rotations {
        for i in (0..rotatedcoords.len()).step_by(5) {
            let x = rotatedcoords[i];
            let y = rotatedcoords[i + 1];
            let z = rotatedcoords[i + 2];
            let light_block = rotatedcoords[i + 3];
            let ambient_light = rotatedcoords[i + 4];

            // Translate coordinates to rotate around (0.5, 0.5, 0.5)
            let translated_x = x - 0.5;
            let translated_z = z - 0.5;

            // Perform the rotation
            let rotated_x = -translated_z;
            let rotated_z = translated_x;

            // Translate coordinates back
            rotatedcoords[i] = rotated_x + 0.5;
            rotatedcoords[i + 1] = y; // y remains the same
            rotatedcoords[i + 2] = rotated_z + 0.5;
            rotatedcoords[i + 3] = light_block; // Keep the light block value unchanged
            rotatedcoords[i + 4] = ambient_light; // Keep the ambient light value unchanged
        }
    }

    rotatedcoords
}