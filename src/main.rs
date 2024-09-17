mod game;
mod bot_behaviour;
mod utils;
mod bot_behaviours {
    pub mod random_bot;
    pub mod smart_bot;
}

use std::io::{self, Write};
use bot_behaviours::{random_bot::RandomBot, smart_bot::SmartBot};
use game::{Board, GameState, Pal};
use crate::bot_behaviour::Behaviour;
use rand::{thread_rng, Rng};


fn main() {
    let bot_0: RandomBot = RandomBot::new("random".to_string());
    let bot_1: SmartBot = SmartBot::new("smart1".to_string());
    let bot_2: RandomBot = RandomBot::new("random2".to_string());
    let bot_3: SmartBot = SmartBot::new("smart3".to_string());
    
    let bots: Vec<Box<dyn Behaviour>> = vec![
        Box::new(bot_0), 
        Box::new(bot_1), 
        Box::new(bot_2), 
        Box::new(bot_3)
    ];

    let mut games: [GameState; 1000] = [const { GameState::None }; 1000];

    for i in 0..games.len() {
        let game_state: GameState = play_game(&bots);
        games[i] = game_state;
    }

    println!("Team 0 won: {}", games.iter().filter(|&g| g == &GameState::Team0Won).count());
    println!("Team 1 won: {}", games.iter().filter(|&g| g == &GameState::Team1Won).count());

    // Player driver
    /* loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input: String = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input: &str = input.trim();

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
    } */
}

fn play_game(bots: &Vec<Box<dyn Behaviour>>) -> GameState {
    let starting_player: usize = thread_rng().gen_range(0..=3);
    let mut board: Board = Board::new(starting_player);

    loop {
        match board.current_player {
            0..=3 => match bots[board.current_player].play_card(&mut board) {
                Ok(game_state) => match game_state {
                    GameState::BazaEnded => post_baza_actions(&bots, &mut board),
                    GameState::Continuation => (),
                    GameState::Team0Won | GameState::Team1Won => {
                        return game_state;
                    },
                    _ => ()
                },
                Err(error) => println!("Error: {}", error)
            }
            _ => {
                println!("Wrong player");
                return GameState::None;
            }
        }
    }
}

fn post_baza_actions(bots: &Vec<Box<dyn Behaviour>>, board: &mut Board) {
    for (player, bot) in bots.iter().enumerate() {
        bot.post_baza_actions(board, player);
    }
}