





pub struct Monsters {

}



impl Monsters {



  pub fn get_aggro_sound(model_index: usize) -> &'static str {
    match model_index {
      0 => {
        "assets/sfx/monster1.mp3"
      }
      1 => {
        "assets/sfx/monster1.mp3"
      }
      2 => {
        "assets/sfx/monster1.mp3"
      }
      3 => {
        "assets/sfx/monster2.mp3"
      }
      _ => {
        "assets/sfx/monster2.mp3"
      }
    }
  }
}