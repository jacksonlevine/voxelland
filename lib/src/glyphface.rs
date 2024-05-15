pub struct GlyphFace {
    pub tlx: f32,
    pub tly: f32, 
    pub blx: f32,
    pub bly: f32,
    pub brx: f32,
    pub bry: f32,
    pub trx: f32,
    pub tr_y: f32
}

impl GlyphFace {
    pub fn new(code: u8) -> GlyphFace {
        let mut gf = GlyphFace{
            tlx:0.0,
            tly:0.0,
            blx:0.0,
            bly:0.0,
            brx:0.0,
            bry:0.0,
            trx:0.0,
            tr_y:0.0,
        };
        gf.set_char(code);
        return gf;
    }
    pub fn set_char(&mut self, code: u8) {
        static FATLX: f32 = 288.0/544.0;
        static FATLY: f32 = 0.0;
        static GLYPH_WIDTH: f32 = 16.0/544.0;
    
        let offset: i32 = code as i32 - 32;
        let x_offset: f32 = (offset % 16) as f32 * GLYPH_WIDTH;
        let y_offset = ((offset/16) as u8) as f32 * GLYPH_WIDTH;
        
        self.tlx = FATLX + x_offset;
        self.tly = FATLY + y_offset;
        self.trx = self.tlx + GLYPH_WIDTH;
        self.tr_y = self.tly;
        self.brx = self.trx;
        self.bry = self.tr_y + GLYPH_WIDTH;
        self.blx = self.tlx;
        self.bly = self.tly + GLYPH_WIDTH;
    }
}