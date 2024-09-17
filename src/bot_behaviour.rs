use crate::{game::GameState, Board};

pub trait Behaviour {
    fn play_card(&self, board: &mut Board) -> Result<GameState, String>;
    fn post_baza_actions(&self, board: &mut Board, player: usize);
}