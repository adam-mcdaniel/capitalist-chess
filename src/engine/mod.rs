use super::{StateCapitalistBoard, Color, Bank, Move, Tile};
use alloc::vec::Vec;
use itertools::Itertools;
use log::{debug, info};
use rayon::prelude::*;

/// Return all the combinations of moves where the total cost of the moves
/// is affordable to the given bank.
#[allow(dead_code)]
fn affordable_combinations(moves: Vec<Move>, bank: &Bank, board: &StateCapitalistBoard) -> Vec<Move> {
    let max_moves = bank.get_balance() / bank.get_market().get_base_move_cost();
    let mut result = Vec::new();

    for moves in moves.into_iter().combinations((max_moves.round() as usize).max(1).min(3)) {
        let m = Move::Many(moves);
        if board.is_legal_move(&m) {
            debug!("Can afford: {:?}", m);
            result.push(m);
        }
    }

    result
}


/// An engine evaluates a chess board and returns a score.
pub trait Engine: Send + Sync {
    /// Get the name of the engine.
    fn name(&self) -> &str;

    /// Evaluate the given board.
    fn evaluate(&self, board: &StateCapitalistBoard, color: Color) -> f64;

    /// Get the legal moves for the given board.
    fn legal_moves(&self, board: &StateCapitalistBoard) -> Vec<Move> {
        // Move::legal_moves(&Board::from(*board))
        let result = board.legal_moves();
        // result.extend(affordable_combinations(result.clone(), board.get_bank(board.whose_turn()), board));
        info!("Legal moves: {:?}", result);
        // info!("Legal moves: {:?}", affordable_combinations(result.clone(), board.get_bank(board.whose_turn()), &board));

        // // result
        // let mut new_result = affordable_combinations(result.clone(), board.get_bank(board.whose_turn()), &board);
        // result.append(&mut new_result);
        result
    }

    /// Return the best move for the given board.
    fn best_move(&self, board: &StateCapitalistBoard) -> Option<Move> {
        let (score, best_move) = self.minimax(board, 4, board.whose_turn(), None);
        eprintln!("Score: {}", score);
        Some(best_move)
    }

    /// Perform a minimax search on the given board.
    /// This function returns a tuple of the score and the best move.
    fn minimax(&self, board: &StateCapitalistBoard, depth: u32, color: Color, original_move: Option<Move>) -> (f64, Move) {
        if depth == 0 {
            return (self.evaluate(board, color), original_move.unwrap());
        }

        info!("Checking minimax at depth {}", depth);
        // let mut best_score = f64::NEG_INFINITY;
        // let mut best_move = None;

        
        let all_scores_and_moves = self.legal_moves(board).par_iter().map(|legal_move| {
            let mut board_copy = board.clone();
            if board_copy.apply(legal_move.clone()).is_err() {
                eprintln!("Illegal move: {:?}", legal_move);
                return (f64::NEG_INFINITY, legal_move.clone());
            }
            
            let score = -self.minimax(&board_copy, depth - 1, color, Some(original_move.clone().unwrap_or(legal_move.clone()))).0;
            // eprintln!("Score: {}", score);
            // if score > best_score {
            //     best_score = score;
            //     best_move = Some(legal_move);
            // }

            return (score, legal_move.clone());
        }).collect::<Vec<_>>();

        if all_scores_and_moves.is_empty() {
            return (f64::NEG_INFINITY, Move::Pass);
        }

        let (best_score, best_move) = all_scores_and_moves.into_iter().max_by(|(score1, _), (score2, _)| score1.partial_cmp(score2).unwrap()).unwrap();

        // for legal_move in self.legal_moves(board) {
        //     let mut board_copy = board.clone();
        //     if board_copy.apply(legal_move.clone()).is_err() {
        //         eprintln!("Illegal move: {:?}", legal_move);
        //         continue;
        //     }
            
        //     let score = -self.minimax(&board_copy, depth - 1, color, Some(original_move.clone().unwrap_or(legal_move.clone()))).0;
        //     // eprintln!("Score: {}", score);
        //     if score > best_score {
        //         best_score = score;
        //         best_move = Some(legal_move);
        //     }
        // }

        (best_score, best_move)
    }
}

/// A random engine.
pub struct RandomEngine;

impl Engine for RandomEngine {
    fn name(&self) -> &str {
        "Random Engine"
    }

    fn evaluate(&self, _board: &StateCapitalistBoard, _color: Color) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen()
    }
}

/// A simple engine that evaluates the board based on the number of pieces.
pub struct SimpleEngine;

impl Engine for SimpleEngine {
    fn name(&self) -> &str {
        "Simple Engine"
    }

    fn evaluate(&self, board: &StateCapitalistBoard, color: Color) -> f64 {
        let mut score = 0.0;
        let market = board.get_market();

        for tile in Tile::all() {
            if let Some(piece) = board.get_piece(tile) {
                if piece.get_color() == color {
                    score += (market.get_piece_value(piece.get_type()).get_amount() * 2) as f64;
                } else {
                    score -= (market.get_piece_value(piece.get_type()).get_amount() * 2) as f64;
                }
            }
        }

        score + board.get_balance(color).get_amount() as f64 / 2.0 - board.get_balance(!color).get_amount() as f64 / 2.0
    }
}