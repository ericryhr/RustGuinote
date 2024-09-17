use crate::{bot_behaviour::Behaviour, game::{Board, Card, GameState, Hand, Pal}};

pub struct SmartBot {
    pub name: String
}

impl SmartBot {
    pub fn new(name: String) -> Self {
        SmartBot {
            name
        }
    }
}

impl Behaviour for SmartBot {
    fn play_card(&self, board: &mut Board) -> Result<GameState, String> {
        let hand: Hand = board.get_current_player_hand();
        let legal_cards: Vec<Card> = board.get_legal_cards();
        let trumfo: Pal = board.current_trumfo.pal;

        let best_card: &Card = legal_cards.iter().reduce(|a, b| if a.is_better_than(*b, trumfo) {a} else {b}).unwrap(); 
        let card_index: usize = hand.get_index(best_card).unwrap();
        
        board.play_card(card_index)
    }
    
    fn post_baza_actions(&self, board: &mut Board, player: usize) {
        let available_pals: Vec<Pal> = board.get_available_cantes(player);
        for pal in available_pals {
            board.cantar(player, pal).unwrap();
        }
    }
}