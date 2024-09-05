mod utils;

use rand::thread_rng;
use rand::seq::SliceRandom;
use std::array;
use std::fmt;
use std::io::{self, Write};
use std::mem::swap;
use std::str::FromStr;
use crate::utils::intersect;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Pal {
    Orus, Copes, Espases, Bastos
}

impl FromStr for Pal {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Orus" => Ok(Pal::Orus),
            "Copes" => Ok(Pal::Copes),
            "Espases" => Ok(Pal::Espases),
            "Bastos" => Ok(Pal::Bastos),
            _ => Err("Invalid Pal".to_string())
        }
    }
    
    type Err = String;
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Card {
    pal: Pal,
    number: u32
}

impl Card {
    const NULL_CARD: u32 = 0;

    fn default() -> Self {
        Card {
            pal: Pal::Orus,
            number: Card::NULL_CARD
        }
    }

    fn value(&self) -> u32 {
        match self.number {
            1 => 11,
            3 => 10,
            12 => 4,
            10 => 3,
            11 => 2,
            _ => 0
        }
    }

    fn is_better_than(&self, other_card: Card, trumfo: Pal) -> bool {
        if self.pal == other_card.pal {
            if self.value() != other_card.value() {
                self.value() > other_card.value()
            } else {
                self.number > other_card.number
            }
        } else if other_card.pal != trumfo {
            true
        } else {
            false
        }
    }
}


struct Deck {
    cards: Vec<Card>
}

impl Deck {
    fn new() -> Self {
        Deck {
            cards: Vec::new(),
        }
    }

    fn fill(mut self) -> Self {
        self.cards.clear();

        for pal in [Pal::Orus, Pal::Copes, Pal::Espases, Pal::Bastos] {
            for number in [1, 2, 3, 4, 5, 6, 7, 10, 11, 12] {
                self.cards.push(Card { pal: pal.clone(), number });
            }
        }

        self
    }

    fn scramble(mut self) -> Self {
        self.cards.shuffle(&mut thread_rng());

        self
    }

    fn draw_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}


struct Hand {
    cards: Vec<Card>
}

impl Hand {
    fn new() -> Self {
        Hand {
            cards: Vec::new()
        }
    }

    fn clone(&self) -> Self {
        Hand { 
            cards: self.cards.clone() 
        }
    }
}


struct Player {
    player_id: usize,
    team_id: usize,
    hand: Hand
}

impl Player {
    fn new(player_id: usize, team_id: usize) -> Self {
        Player {
            player_id,
            team_id,
            hand: Hand::new()
        }
    }

    fn play_card(&mut self, index: usize) -> Card {
        self.hand.cards.remove(index)
    }

    fn give_card(&mut self, card: Card) {
        self.hand.cards.push(card);
    }
}


struct Board {
    players: [Player; 4],
    deck: Deck,

    bazas: [Vec<Card>; 2],
    cantes: Vec<Pal>,
    points: [u32; 2],

    current_player: usize,
    current_trumfo: Card,
    current_baza: [Card; 4]
}

impl Board {
    fn new() -> Self {
        let mut players: [Player; 4] = [
            Player::new(0, 0),
            Player::new(1, 1),
            Player::new(2, 0),
            Player::new(3, 1),
        ];

        let mut deck: Deck = Deck::new().fill().scramble();
        
        // Robar cartes inicials (es fa a la manera guiñote, de 3 en 3 fins a 6 per jugador)
        // PD: totalment inutil ja que es un random, pero es gracios
        for i in 0..8 {
            for _ in 0..3 {
                let player_to_draw: &mut Player = &mut players[i % 4];
                let card: Card = deck.draw_card().unwrap();     // Can't panic if deck is filled
                player_to_draw.give_card(card);
            }
        }
        
        let current_trumfo: Card = deck.draw_card().unwrap();

        // Tauler inicial
        Board {
            players,
            deck,
            bazas: array::from_fn(|_| Vec::new()),
            cantes: Vec::new(),
            points: [0; 2],
            current_player: 0,
            current_trumfo,
            current_baza: [Card::default(); 4]
        }
    }

    /////////// GETTERS //////////
    
    fn get_current_player_hand(&self) -> Hand {
        self.players[self.current_player].hand.clone()
    }

    ////////// CARD PLAYS //////////

    fn play_card(&mut self, index: usize) -> Result<(), String> {
        match self.is_legal_movement(index) {
            Ok(()) => (),
            Err(error) => return Err(error)
        }
        
        // Play move
        let player: &mut Player = &mut self.players[self.current_player];
        let played_card: Card = player.play_card(index);
        self.current_baza[self.current_player] = played_card;
        self.current_player = Board::next_player(self.current_player);

        if Board::is_baza_complete(self.current_baza) {
            // Complete baza
            self.current_player = Board::determine_baza_winner(self.current_player, self.current_trumfo.pal, self.current_baza);
            let team_id: usize = self.players[self.current_player].team_id;
            self.bazas[team_id].extend(self.current_baza.iter());
            
            // Reset baza
            self.current_baza = [Card::default(); 4];

            // Draw cards
            if self.deck.cards.len() > 0 {
                let mut draw_player: usize = self.current_player;
                for _ in 0..4 {
                    match self.deck.draw_card() {
                        Some(card) => self.players[draw_player].give_card(card),
                        // Si no queden cartes al mazo es canvia el trumfo
                        None => self.players[draw_player].give_card(self.current_trumfo)
                    }

                    draw_player = Board::next_player(draw_player);
                }
            }

            // Check for game end
            if self.players[0].hand.cards.len() == 0 {
                // bazas hauria de ser una immutable reference pero no es pot pk self es &mut
                self.points = Board::count_points(self.bazas.clone());

                // 10 de ultimas
                self.points[self.players[self.current_player].team_id] += 10;
            }
        }

        Ok(())
    }

    fn cantar(&mut self, player: usize, pal: Pal) -> Result<(), String> {
        match self.is_legal_cante(player, pal) {
            Ok(()) => (),
            Err(error) => return Err(error)
        }

        self.cantes.push(pal);

        let player_team: usize = self.players[player].team_id;
        if pal == self.current_trumfo.pal {
            self.points[player_team] += 40
        } else {
            self.points[player_team] += 20
        }

        Ok(())
    }

    fn change_trumfo_card(&mut self, player: usize) -> Result<(), String> {
        match self.is_legal_canvi_trumfo(player) {
            Ok(()) => (),
            Err(error) => return Err(error)
        }

        let mut seven_card: &Card = self.players[player].hand.cards
                                                    .iter()
                                                    .find(|&card| card == &Card { pal: self.current_trumfo.pal, number: 7 })
                                                    .unwrap();
        let mut trumfo_card: &Card = &self.current_trumfo;

        swap(&mut seven_card, &mut trumfo_card);

        Ok(())
    }

    /////////// PRIVATE METHODS //////////

    fn is_legal_movement(&self, index: usize) -> Result<(), String> {
        // Check for player hand size
        if index >= self.players[self.current_player].hand.cards.len() {
            return Err("Invalid card index.".to_string());
        }

        let card_played: Card = self.players[self.current_player].hand.cards[index];
        if !self.legal_cards().contains(&card_played) {
            return Err("Renuncio. Carta invàlida.".to_string());
        }
        
        Ok(())
    }

    fn is_legal_cante(&self, player: usize, pal: Pal) -> Result<(), String> {
        if player >= 4 {
            return Err("Invalid player index.".to_string());
        }

        // Nomes es pot cantar si s'ha guanyat l'ultima baza
        if !self.player_team_won_last_baza(player) {
            return Err("No es pot cantar si no és començament de baza o no ha guanyat la última baza.".to_string());
        }

        // No es pot cantar si ja s'ha cantat en aquest pal
        if self.cantes.contains(&pal) {
            return Err("No es pot cantar si ja s'ha cantat en aquest pal.".to_string());
        }

        // Comprovem que tingui les cartes valides a la ma
        let sota: Card = Card { pal, number: 10 };
        let rey: Card = Card { pal, number: 12 };
        if  !self.players[player].hand.cards.contains(&sota) ||
            !self.players[player].hand.cards.contains(&rey) {
            return Err("No es pot cantar sense la sota i el rey a la mà.".to_string());
        }

        Ok(())
    }

    // TODO: No s'ha de poder canviar trumfo si s'acaba de robar a l'ultima baza abans de l'arrastre
    fn is_legal_canvi_trumfo(&self, player: usize) -> Result<(), String> {
        if player >= 4 {
            return Err("Invalid player index.".to_string());
        }

        // Nomes es pot canviar si s'ha guanyat l'ultima baza
        if !self.player_team_won_last_baza(player) {
            return Err("No es pot canviar trumfo si no és començament de baza o no ha guanyat la última baza.".to_string());
        }

        // No es pot fer en arrastre (be nomes abans de començarlo, per tant es pot saber pel nombre de cartes del jugador)
        if self.players[player].hand.cards.len() < 6 {
            return Err("No es pot canviar trumfo durant l'arrastre.".to_string());
        }

        // Comprovem que tingui la carta valida a la ma
        let seven_card: Card = Card { pal: self.current_trumfo.pal, number: 7 };
        if  !self.players[player].hand.cards.contains(&seven_card) {
            return Err("No es pot canviar trumfo sense el 7 de trumfo a la mà.".to_string());
        }

        Ok(())
    }

    // Returns legal cards for the current player
    fn legal_cards(&self) -> Vec<Card> {
        let player_hand: Vec<Card> = self.players[self.current_player].hand.cards.clone();

        // Si no es arrastre tots els moviments son legals
        if self.deck.cards.len() > 0 {
            return player_hand;
        }

        // Si es la primera carta d'una baza es legal
        if Board::is_baza_empty(self.current_baza) {
            return player_hand;
        }

        // ARRASTRE
        // 1. Obtenir la carta que comença la baza
        let starting_card_index: usize = self.current_baza.iter()
                                                    .position(|&card| card.number != Card::NULL_CARD)
                                                    .unwrap();
        let starting_card: Card = self.current_baza[starting_card_index];

        // 2. Obtenir la carta que va guanyant la baza
        let winning_card_index: usize = Board::determine_baza_winner(starting_card_index, self.current_trumfo.pal, self.current_baza);
        let winning_card: Card = self.current_baza[winning_card_index];

        // 3. ??!?
        let current_player_team_winning: bool = self.players[winning_card_index].team_id == self.players[self.current_player].team_id;
        let cards_with_baza_pal: Vec<Card> = player_hand.iter()
                                                        .filter(|&card| card.pal == starting_card.pal)
                                                        .cloned()
                                                        .collect();
        let cards_better_than_winning_card: Vec<Card> = player_hand.iter()
                                                                    .filter(|&card| card.is_better_than(winning_card, self.current_trumfo.pal))
                                                                    .cloned()
                                                                    .collect();
        let cards_with_baza_pal_better_than_winning_card: Vec<Card> = intersect(&cards_with_baza_pal, &cards_better_than_winning_card);

        if current_player_team_winning {
            if cards_with_baza_pal.len() > 0 {
                return cards_with_baza_pal;
            }
        } else {
            if cards_with_baza_pal_better_than_winning_card.len() > 0 {
                return cards_with_baza_pal_better_than_winning_card;
            } else if cards_with_baza_pal.len() > 0 {
                return cards_with_baza_pal;
            } else if cards_better_than_winning_card.len() > 0 {
                return cards_better_than_winning_card;
            }
        }

        // 4. Profit
        player_hand
    }

    fn player_team_won_last_baza(&self, player: usize) -> bool {
        // Invalid player index
        if player >= self.players.len() {
            return false
        }
        
        // Accio nomes disponible al principi de baza
        if !Board::is_baza_empty(self.current_baza) {
            return false;
        }

        // Nomes pot cantar o canviar trumfo un jugador de l'equip guanyador de l'ultima baza, es a dir, al que li toca jugar
        let cante_player_team: usize = self.players[player].team_id;
        let current_player_team: usize = self.players[self.current_player].team_id;
        if cante_player_team != current_player_team {
            return false
        }

        true
    }

    /////////// STATIC HELPER METHODS //////////

    fn count_points(bazas: [Vec<Card>; 2]) -> [u32; 2] {
        let mut points: [u32; 2] = [0; 2];
        
        for (team_id, team_bazas) in bazas.iter().enumerate() {
            for card in team_bazas {
                points[team_id] += card.value();
            }
        }

        points
    }

    fn is_baza_empty(baza: [Card; 4]) -> bool {
        for card in baza {
            if card.number != Card::NULL_CARD {
                return false;
            }
        }

        true
    }

    fn is_baza_complete(baza: [Card; 4]) -> bool {
        for card in baza {
            if card.number == Card::NULL_CARD {
                return false;
            }
        }

        true
    }

    fn determine_baza_winner(starting_player: usize, current_trumfo: Pal, baza: [Card; 4]) -> usize {
        let mut winner: usize = starting_player;
        let mut player_to_check: usize = Board::next_player(starting_player);

        for _ in 0..3 {
            let winner_card: Card = baza[winner];
            let current_card: Card = baza[player_to_check];

            if current_card.is_better_than(winner_card, current_trumfo) {
                winner = player_to_check;
            }

            player_to_check = Board::next_player(player_to_check);
        }

        winner
    }

    fn next_player(current_player: usize) -> usize {
        (current_player + 1) % 4
    }

}


fn main() {
    let mut board = Board::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "board" => println!("{}", board),
            "hand" => println!("{}", board.get_current_player_hand()),
            "quit" | "exit" => break,
            _ => {
                if input.starts_with("play ") {
                    if let Ok(card_index) = input[5..].parse::<usize>() {
                        match board.play_card(card_index) {
                            Ok(()) => println!("Card played successfully."),
                            Err(error) => println!("Error: {}", error),
                        }
                    } else {
                        println!("Invalid card index. Usage: play <card_index>");
                    }
                } else if input.starts_with("cantar ") {
                    if let Ok(player_index) = input[7..8].parse::<usize>() {
                        if let Ok(pal) = input[9..].parse::<Pal>() {
                            match board.cantar(player_index, pal) {
                                Ok(()) => println!("Successful cante."),
                                Err(error) => println!("Error: {}", error),
                            }
                        } else {
                            println!("Invalid pal. Usage: cantar <player_index> <pal>");
                        }
                    } else {
                        println!("Invalid player index. Usage: cantar <player_index> <pal>");
                    }
                } else if input.starts_with("trumfo ") {
                    if let Ok(player_index) = input[7..].parse::<usize>() {
                        match board.change_trumfo_card(player_index) {
                            Ok(()) => println!("Trumfo changed successfully."),
                            Err(error) => println!("Error: {}", error),
                        }
                    } else {
                        println!("Invalid player index. Usage: trumfo <player_index>");
                    }
                } else {
                    println!("Unknown command. Available commands:  board, 
                                        hand, 
                                        play <card_index>, 
                                        cantar <player_index> <pal>, 
                                        trumfo <player_index>, 
                                        quit");
                }
            }
        }
    }
}

/////////////////////// FORMATTERS ///////////////////////

impl fmt::Display for Pal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pal::Orus => write!(f, "Orus")?,
            Pal::Copes => write!(f, "Copes")?,
            Pal::Espases => write!(f, "Espases")?,
            Pal::Bastos => write!(f, "Bastos")?,
        }
        Ok(())
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.pal, self.number)?;
        Ok(())
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &card in &self.cards {
            write!(f, "{}, ", card)?;
        }
        Ok(())
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let card_strings: Vec<String> = self.cards.iter().map(|card: &Card| {
            card.to_string()
        }).collect();
    
        write!(f, "Deck: [{}]", card_strings.join(", "))?;
        Ok(())
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let card_strings: Vec<String> = self.hand.cards.iter().map(|card: &Card| {
            card.to_string()
        }).collect();

        write!(f, "Player {}, Team {}: [{}]", self.player_id, self.team_id, card_strings.join(", "))?;
        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board State:")?;
        
        writeln!(f, "Players:")?;
        for player in self.players.iter() {
            writeln!(f, "  {}", player)?;
        }
        
        writeln!(f, "\n{}", self.deck)?;
        
        writeln!(f, "\nBazas:")?;
        for (i, baza) in self.bazas.iter().enumerate() {
            write!(f, "  Team {}: [", i)?;
            for (j, card) in baza.iter().enumerate() {
                if j > 0 { write!(f, ", ")? }
                write!(f, "{}", card)?;
            }
            writeln!(f, "]")?;
        }
        
        writeln!(f, "\nPoints: {:?}", self.points)?;
        
        writeln!(f, "Current Player: {}", self.current_player)?;
        writeln!(f, "Current Trumfo: {}", self.current_trumfo)?;
        
        write!(f, "Current Baza: [")?;
        for (i, card) in self.current_baza.iter().enumerate() {
            if i > 0 { write!(f, ", ")? }
            write!(f, "{}", card)?;
        }
        writeln!(f, "]")?;
        
        Ok(())
    }
}