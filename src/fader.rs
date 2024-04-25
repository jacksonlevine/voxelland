






pub struct Fader {
    pub value: f32,
    pub speed: f32,
    pub top: f32,
    pub bottom: f32,
    pub mode: bool
}

impl Fader {
    pub fn new(top: f32, bottom: f32, speed: f32, mode: bool) -> Fader {
        Fader {
            value: if mode { top } else { bottom },
            speed,
            top,
            bottom,
            mode
        }
    }
    pub fn up(&mut self) {
        self.mode = true;
    }

    pub fn down(&mut self) {
        self.mode = false;
    }
    pub fn tick(&mut self, delta_time: f32) -> bool {
        if self.mode {
            if self.value < self.top {
                self.value = (self.value + (delta_time * self.speed)).clamp(self.bottom, self.top);
                return true;
            } else {
                return false
            }
        } else {
            if self.value > self.bottom {
                self.value = (self.value - (delta_time * self.speed)).clamp(self.bottom, self.top);
                return true;
            } else {
                return false;
            }
        }

    }
}