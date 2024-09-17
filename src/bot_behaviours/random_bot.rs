use rand::{seq::SliceRandom, thread_rng};
use crate::{bot_behaviour::Behaviour, game::{Board, Card, GameState, Hand, Pal}};

pub struct RandomBot {
    pub name: String
}

impl RandomBot {
    pub fn new(name: String) -> Self {
        RandomBot {
            name
        }
    }
}

impl Behaviour for RandomBot {
    fn play_card(&self, board: &mut Board) -> Result<GameState, String> {
        let hand: Hand = board.get_current_player_hand();
        let mut legal_cards: Vec<Card> = board.get_legal_cards();
        legal_cards.shuffle(&mut thread_rng());
        let random_card: Card = legal_cards[0];
        let card_index: usize = hand.get_index(&random_card).unwrap();
        
        board.play_card(card_index)
    }
    
    fn post_baza_actions(&self, board: &mut Board, player: usize) {
        let available_pals: Vec<Pal> = board.get_available_cantes(player);
        for pal in available_pals {
            board.cantar(player, pal).unwrap();
        }
    }
}