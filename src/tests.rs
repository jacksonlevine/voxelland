use crate::packedvertex::PackedVertex;

#[test]
fn test_coord_packing() {
    /*Supposed to be: */
    /*In (u32, u8) tuple: */

    /*u32*/
    /*0000 0000 0000 0000 0000 0000 0000 0000  */
    /*x    y         z    cor  amb  bl   empty */

    /*u8*/
    /*0000 0000 */
    /*u    v    */

    assert_eq!(
        (0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000),
        PackedVertex::pack(0, 0, 0, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        (0b0001_0000_0001_0001_0000_0000_0000_0000, 0b0000_0000),
        PackedVertex::pack(1, 1, 1, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        (0b0010_1000_0000_0010_0010_1110_0001_0000, 0b0000_0000),
        PackedVertex::pack(2, 128, 2, 2, 14, 1, 0, 0)
    );
    assert_eq!(
        (0b0011_1000_0001_0010_0010_1110_0001_0000, 0b0100_0000),
        PackedVertex::pack(3, 129, 2, 2, 14, 1, 4, 0)
    );
}

#[test]
fn test_coord_unpacking() {
    let x = 10;
    let y = 50;
    let z = 14;
    let corn = 4;
    let amb = 12;
    let bl = 10;

    let (packed32, _packed8) = PackedVertex::pack(x, y, z, corn, amb, bl, 0, 0);

    let unpk_x = (packed32 >> 28) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_y = (packed32 >> 20) & 0b0000_0000_0000_0000_0000_0000_1111_1111;
    let unpk_z = (packed32 >> 16) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_corn = (packed32 >> 12) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_amb = (packed32 >> 8) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_bl = (packed32 >> 4) & 0b0000_0000_0000_0000_0000_0000_0000_1111;

    assert_eq!(x, unpk_x as u8);
    assert_eq!(y, unpk_y as u8);
    assert_eq!(z, unpk_z as u8);
    assert_eq!(corn, unpk_corn as u8);
    assert_eq!(amb, unpk_amb as u8);
    assert_eq!(unpk_bl as u8, bl);

    let x = 14;
    let y = 100;
    let z = 9;
    let corn = 2;
    let amb = 9;
    let bl = 0;

    let (packed32, _packed8) = PackedVertex::pack(x, y, z, corn, amb, bl, 0, 0);

    let unpk_x = (packed32 >> 28) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_y = (packed32 >> 20) & 0b0000_0000_0000_0000_0000_0000_1111_1111;
    let unpk_z = (packed32 >> 16) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_corn = (packed32 >> 12) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_amb = (packed32 >> 8) & 0b0000_0000_0000_0000_0000_0000_0000_1111;
    let unpk_bl = (packed32 >> 4) & 0b0000_0000_0000_0000_0000_0000_0000_1111;

    assert_eq!(x, unpk_x as u8);
    assert_eq!(y, unpk_y as u8);
    assert_eq!(z, unpk_z as u8);
    assert_eq!(corn, unpk_corn as u8);
    assert_eq!(amb, unpk_amb as u8);
    assert_eq!(unpk_bl as u8, bl);
}
