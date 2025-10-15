#![allow(dead_code)]

pub type Coord = i32;

pub const LCD_WIDTH: u32 = 240; // Physical LCD width
pub const LCD_HEIGHT: u32 = 320; // Physical LCD height

// Game logical dimensions (landscape orientation through software rotation)
pub const GAME_WIDTH: u32 = 320; // Logical game width (rotated)
pub const GAME_HEIGHT: u32 = 240; // Logical game height (rotated)
pub const OBSTACLE_WIDTH: u32 = 30;
pub const OBSTACLE_GAP: u32 = 80;

pub const SCORE_BOARD_HEIGHT: u32 = 30;
pub const PLANTS_HEIGHT: u32 = 30;

pub const LCD_BIGIN: Coord = 0;
pub const LCD_END: Coord = GAME_WIDTH as Coord;

pub const INIT_PLAYER_POS_X: Coord = 60;
pub const INIT_PLAYER_POS_Y: Coord = (SCORE_BOARD_HEIGHT + 10) as Coord;
pub const PLAYER_WIDTH: u32 = 30;
pub const PLAYER_HEIGHT: u32 = 30;

pub const GRAVITY: i32 = 0;

pub const GROUND_Y_POS: Coord = (GAME_HEIGHT - PLANTS_HEIGHT - 10) as Coord; // Adjusted for landscape game

pub const MPU6050_DEV_ADDR: u8 = 0x68;

pub const PLAYER_Y_MIN: Coord = SCORE_BOARD_HEIGHT as Coord;
pub const PLAYER_Y_MAX: Coord = (GAME_HEIGHT - PLANTS_HEIGHT - PLAYER_HEIGHT) as Coord;

pub const SPEED: u32 = 2;
