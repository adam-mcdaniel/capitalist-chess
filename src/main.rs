use capitalist_chess::*;
use std::{str::FromStr, io::{stdin, stdout, Write}};

fn main() {
    env_logger::init();

    // Set the thread stack size with rayon
    rayon::ThreadPoolBuilder::new().stack_size(1024 * 1024 * 1024).build_global().unwrap();

    let mut board = StateCapitalistBoard::default();

    // Loop and read moves from stdin
    loop {
        let legal_moves = SimpleEngine.legal_moves(&board);
        for (i, legal_move) in legal_moves.iter().enumerate() {
            let cost = board.get_bank(board.whose_turn()).get_market().get_move_value(legal_move);
            println!("{i}. {legal_move:?} ({cost})", i=i+1);
        }
        println!("{board}");

        if board.whose_turn() == Color::Black {
            eprintln!("Engine is thinking...");
            let result = SimpleEngine.best_move(&board).unwrap();
            println!("Engine move: {result:?}");
            board.apply(result).unwrap();
            continue;
        }

        print!("Enter a move:\n> ");
        // Flush stdout
        stdout().flush().unwrap();
        
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }


        if let Ok(player_move) = Move::from_str(input) {
            println!("{player_move:?}");
            if board.is_legal_move(&player_move) {
                println!("Legal move!");
            } else {
                println!("Illegal move!");
                continue;
            }

            if board.apply(player_move).is_err() {
                println!("Illegal move!");
            }
        } else {
            println!("Invalid move!");
        }
    }
}