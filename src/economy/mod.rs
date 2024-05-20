use log::{info, error};

mod currency;
pub use currency::Currency;

mod bank;
pub use bank::Bank;

mod market;
pub use market::Market;

use core::fmt::{Display, Formatter, Result as FmtResult};
use alloc::vec::Vec;

use super::*;

/// A board for a game of State Capitalist Chess.
/// 
/// This board is used to keep track of the game state.
/// It is also used to validate moves.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StateCapitalistBoard {
    /// The market for the game.
    market: Market,
    /// The white bank.
    white_bank: Bank,
    /// The black bank.
    black_bank: Bank,
    /// The current board state.
    board: Board,
}

impl Default for StateCapitalistBoard {
    fn default() -> Self {
        Self::new(Market::default())
    }
}

impl StateCapitalistBoard {
    /// Create a new board.
    pub fn new(market: Market) -> Self {
        let mut result = Self {
            market,
            white_bank: Bank::new(Color::White, market),
            black_bank: Bank::new(Color::Black, market),
            board: Board::default(),
        };
        result.perform_census_for_color(Color::White);
        result
    }

    pub fn get_market(&self) -> &Market {
        &self.market
    }

    pub fn get_balance(&self, color: Color) -> Currency {
        self.get_bank(color).get_balance()
    }

    /// Get a piece at the given tile.
    #[inline]
    pub fn get_piece(&self, tile: Tile) -> Option<Piece> {
        self.board.get_piece(tile)
    }

    /// Whose turn is it?
    #[inline]
    pub fn whose_turn(&self) -> Color {
        self.board.whose_turn()
    }
    
    /// Get the bank for the given color.
    #[inline]
    pub fn get_bank(&self, color: Color) -> &Bank {
        match color {
            Color::White => &self.white_bank,
            Color::Black => &self.black_bank,
        }
    }

    /// Get the bank for the given color.
    #[inline]
    fn get_bank_mut(&mut self, color: Color) -> &mut Bank {
        match color {
            Color::White => &mut self.white_bank,
            Color::Black => &mut self.black_bank,
        }
    }

    /// Is the given move legal?
    pub fn is_legal_move(&self, player_move: &Move) -> bool {
        let whose_turn = self.whose_turn();

        match player_move {
            Move::Purchase { to, .. } => {
                // First, confirm the "to" tile is empty
                if self.board.has_piece_on(*to) {
                    error!("Tile is not empty!");
                    return false;
                }

                if !to.get_sector().is_home_for(whose_turn) {
                    error!("Tile is not in the home sector!");
                    return false;
                }

                // Next, confirm the player can afford the piece
                let result = self.get_bank(whose_turn).can_afford(player_move) && self.board.is_legal_move(player_move);
                if !result {
                    error!("Player cannot afford to purchase!");
                }
                result
            },
            Move::Pass => {
                // Confirm the player can afford to pass
                let result = self.get_bank(whose_turn).can_afford(player_move);
                if !result {
                    error!("Player cannot afford to pass!");
                }
                result
            }
            Move::Many(moves) => {
                let mut copy = self.clone();
                for (i, player_move) in moves.iter().enumerate() {
                    copy.board.set_turn(self.whose_turn());
                    if !copy.is_legal_move(player_move) {
                        error!("Illegal move #{i} {player_move:?} move!");
                        return false;
                    }
                    copy.board.set_turn(self.whose_turn());
                    copy.apply_without_census(player_move.clone()).unwrap();
                }
                true
            },
            _ => self.board.is_legal_move(player_move),
        }
    }

    /// Perform a census for the given color.
    fn perform_census_for_color(&mut self, color: Color) {
        info!("Performing census for {color:?}");
        let board = self.board;
        let bank = self.get_bank_mut(color);
        bank.perform_census(&board);
    }

    /// Apply the move to the board.
    pub fn apply(&mut self, player_move: Move) -> Result<(), ()> {
        if !self.is_legal_move(&player_move) {
            eprintln!("Illegal move!!!!");
            return Err(())
        }
        let whose_turn = self.whose_turn();
        // Purchase the move
        self.get_bank_mut(whose_turn).purchase(&player_move)?;

        self.board.apply(player_move)?;
        self.perform_census_for_color(!whose_turn);
        Ok(())
    }

    /// This applies a move without performing a census.
    /// This is used to perform partial moves, without updating the bank.
    fn apply_without_census(&mut self, player_move: Move) -> Result<(), ()> {
        if !self.is_legal_move(&player_move) {
            eprintln!("Illegal move!!!!");
            return Err(())
        }
        let whose_turn = self.whose_turn();
        // Purchase the move
        self.get_bank_mut(whose_turn).purchase(&player_move)?;

        self.board.apply(player_move)?;
        Ok(())
    }

    /// Get the legal moves for the current player.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut result = vec![];

        let whose_turn = self.whose_turn();

        // Add purchase moves
        for player_move in Move::legal_purchases(&self.board, self.get_bank(whose_turn)) {
            assert!(self.is_legal_move(&player_move));
            result.push(player_move);
        }

        // Add board moves
        for player_move in Move::legal_moves(&self.board) {
            assert!(self.is_legal_move(&player_move));
            result.push(player_move);
        }

        result
    }
}

impl From<StateCapitalistBoard> for Board {
    fn from(board: StateCapitalistBoard) -> Self {
        board.board
    }
}

impl Display for StateCapitalistBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Print black bank
        write!(f, "{}", self.black_bank)?;
        // Print board
        write!(f, "{}", self.board)?;
        // Print white bank
        write!(f, "{}", self.white_bank)?;
        Ok(())
    }
}