use core::{str::FromStr, fmt::{Debug, Display, Formatter, Result as FmtResult}};
use alloc::{vec::Vec, vec};

use super::{Tile, Board, Bank, CastlingSide, PieceType};
// pub struct Turn {
//     white_move: Move,
//     black_move: Move,
// }

#[derive(Clone, PartialEq)]
pub enum Move {
    FromTo {
        from: Tile,
        to: Tile,
        promotion: Option<PieceType>,
    },
    PieceTo {
        piece: PieceType,
        to: Tile,
        promotion: Option<PieceType>,
    },
    Purchase {
        piece: PieceType,
        to: Tile,
    },
    Castling(CastlingSide),
    Resign,
    Pass,
    Many(Vec<Move>),
}

impl Move {
    /// Create a new move from a tile to a tile
    pub fn new(from: Tile, to: Tile, promotion: Option<PieceType>) -> Self {
        Self::FromTo {
            from,
            to,
            promotion,
        }
    }

    /// Generate all the legal moves for a given player on the board
    pub fn legal_moves(board: &Board) -> Vec<Move> {
        let mut result = vec![];

        let turn = board.whose_turn();

        for tile in Tile::all() {
            if let Some(piece) = board.get_piece(tile) {
                if piece.get_color() == turn {
                    for to in tile.get_moves(piece) {
                        if board.is_legal_piece_move(tile, to) {
                            if board.is_valid_promotion(tile, to) {
                                for piece_type in PieceType::PROMOTIONS {
                                    result.push(Move::new(tile, to, Some(piece_type)));
                                }
                            } else {
                                result.push(Move::new(tile, to, None));
                            }
                        }
                    }
                }
            }
        }

        let king_tile = Tile::king_start_position(turn);

        // Check castling moves
        if board.can_castle(king_tile, Tile::rook_start_position(turn, CastlingSide::King)) {
            result.push(Move::Castling(CastlingSide::King));
        }

        if board.can_castle(king_tile, Tile::rook_start_position(turn, CastlingSide::Queen)) {
            result.push(Move::Castling(CastlingSide::Queen));
        }

        result
    }

    /// Generate all the legal purchases for a given player on the board
    pub fn legal_purchases(board: &Board, bank: &Bank) -> Vec<Move> {
        let mut result = vec![];

        for to in Tile::all() {
            if !board.has_piece_on(to) {
                for piece in PieceType::PURCHASES {
                    let player_move = Move::Purchase {piece, to};
                    if to.get_sector().is_home_for(bank.get_color()) && bank.can_afford(&player_move) && !board.is_in_check(board.whose_turn()) {
                        result.push(player_move);
                    }
                }
            }
        }
        
        result
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::FromTo { from, to, promotion } => {
                write!(f, "{}{}{}", from, to, promotion.map(|p| char::from(p)).unwrap_or_default())
            },
            Self::PieceTo { piece, to, promotion } => {
                write!(f, "{}{}{}", char::from(*piece), to, promotion.map(|p| char::from(p)).unwrap_or_default())
            },
            Self::Purchase { piece, to } => {
                write!(f, "{}{}", char::from(*piece), to)
            },
            Self::Castling(side) => {
                write!(f, "{}", side)
            },
            Self::Resign => {
                write!(f, "resign")
            },
            Self::Pass => {
                write!(f, "pass")
            },
            Self::Many(moves) => {
                for (i, m) in moves.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", m)?;
                }
                Ok(())
            },
        }
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::FromTo { from, to, promotion } => {
                write!(f, "move {from} to {to}")?;
                if let Some(promotion) = promotion {
                    write!(f, " and promote to {}", char::from(*promotion))?;
                }
            },

            Self::PieceTo { piece, to, promotion } => {
                write!(f, "move {} to {to}", char::from(*piece))?;
                if let Some(promotion) = promotion {
                    write!(f, " and promote to {}", char::from(*promotion))?;
                }
            },

            Self::Purchase { piece, to } => {
                write!(f, "purchase {} at {to}", char::from(*piece))?
            },

            Self::Castling(side) => {
                write!(f, "castling {side}")?
            },

            Self::Resign => {
                write!(f, "resign")?
            },

            Self::Pass => {
                write!(f, "pass")?
            },

            Self::Many(moves) => {
                for (i, m) in moves.iter().enumerate() {
                    write!(f, "{:?}", m)?;
                    if i < moves.len() {
                        write!(f, ", ")?;
                    }
                }
            },
        }
        Ok(())
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut moves = Vec::new();
        let words = s.split_whitespace();
        for word in words {
            if word == "O-O" || word == "O-O-O" {
                return Ok(Move::Castling(word.parse().unwrap()));
            }

            if word == "resign" {
                return Ok(Move::Resign);
            }

            if word == "pass" {
                return Ok(Move::Pass);
            }

            if word.starts_with("$") {
                let piece = PieceType::from_str(&word[1..2]).unwrap();
                let to = Tile::from_str(&word[2..4]).unwrap();
                moves.push(Move::Purchase {piece, to});
                continue;
            }

            if word.len() == 4 {
                let from = Tile::from_str(&word[0..2]).unwrap();
                let to = Tile::from_str(&word[2..4]).unwrap();
                moves.push(Move::FromTo {
                    from,
                    to,
                    promotion: None,
                });
                continue;
            }

            if word.len() == 3 {
                let piece = PieceType::from_str(&word[0..1]).unwrap();
                let to = Tile::from_str(&word[1..3]).unwrap();
                moves.push(Move::PieceTo {
                    piece,
                    to,
                    promotion: None,
                });
                continue;
            }

            if word.len() == 2 {
                let piece = PieceType::Pawn;
                let to = Tile::from_str(&word[0..2]).unwrap();
                moves.push(Move::PieceTo {
                    piece,
                    to,
                    promotion: None,
                });
                continue;
            }

            return Err(());
        }

        if moves.len() == 1 {
            return Ok(moves[0].clone());
        }

        Ok(Move::Many(moves))
    }
}
