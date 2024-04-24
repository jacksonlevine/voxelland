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

    assert_eq!((0b0000_0000_0000_0000_0000_0000_0000_0000, 0b0000_0000), PackedVertex::pack(0, 0, 0, 0, 0, 0, 0, 0));
    assert_eq!((0b0001_0000_0001_0001_0000_0000_0000_0000, 0b0000_0000), PackedVertex::pack(1, 1, 1, 0, 0, 0, 0, 0));
    assert_eq!((0b0010_1000_0000_0010_0010_1110_0001_0000, 0b0000_0000), PackedVertex::pack(2, 128, 2, 2, 14, 1, 0, 0));
    assert_eq!((0b0011_1000_0001_0010_0010_1110_0001_0000, 0b0100_0000), PackedVertex::pack(3, 129, 2, 2, 14, 1, 4, 0));
}
