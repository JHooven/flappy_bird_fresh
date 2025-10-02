use core::ffi;


use crate::color;
use crate::config::PLAYER_Y_MAX;
use crate::config::PLAYER_Y_MIN;
use crate::config::{
    self,
    Coord
};
use crate::display;
use crate::obstacle;
use crate::player;
use crate::draw;
use crate::assets;

extern "C" {
    fn HAL_GetTick() -> u32;
}


pub enum GameState {
    Start,
    Running,
    End,
    Halt,
}

pub trait InputDevice {
    type Error;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn log_data(&mut self) {

    }
    fn is_tap(&mut self, y_min: Coord, y_max: Coord) -> Result<(Coord, bool), Self::Error>;
}

pub struct Game<T: InputDevice> {
    state: GameState,
    score: u32,
    countdown_start_time: u32,
    obstacle: obstacle::Obstacle,
    player: player::Player,
    pub input_device: T,
}

impl<T: InputDevice> Game<T> {
    pub fn init(mut input_device: T) -> Result<Self, T::Error> {

        input_device.init()?;

        let game = Game {
            state: GameState::Start,
            score: 0,
            countdown_start_time: 0,
            obstacle: obstacle::Obstacle::init(),
            player: player::Player::init(),
            input_device,
            
        };

        Ok(game)
    }


    pub fn draw_start_screen() {
        display::draw_image(40, 160, 40, 80, &assets::GAME_NAME_IMG_DATA);
    }
}