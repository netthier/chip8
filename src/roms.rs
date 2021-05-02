#[macro_export]
macro_rules! impl_get_bytes {
    ($($file:literal),*) => {
        pub static ROMS: [&'static str; 23] = [$($file),*];

        pub fn get_bytes(file: &str) -> Vec<u8> {
            match file {
                $(
                    $file => include_bytes!(concat!("../roms/", $file)).to_vec(),
                )*
                _ => panic!(),
            }
        }
    }
}

impl_get_bytes!(
    "15PUZZLE", "BLINKY", "BLITZ", "BRIX", "CONNECT4", "GUESS", "HIDDEN", "INVADERS", "KALEID",
    "MAZE", "MERLIN", "MISSILE", "PONG", "PONG2", "PUZZLE", "SYZYGY", "TANK", "TETRIS", "TICTAC",
    "UFO", "VBRIX", "VERS", "WIPEOFF"
);
