use super::{PieceType, Sector, Currency, Move};

/// This contains all the configuration data for the banks, and purchase values for pieces
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Market {
    /// The value of a pawn
    pawn_value: Currency,
    /// The value of a knight
    knight_value: Currency,
    /// The value of a bishop
    bishop_value: Currency,
    /// The value of a rook
    rook_value: Currency,
    /// The value of a queen
    queen_value: Currency,
    /// The value of a king
    king_value: Currency,
    /// The cost of a move
    base_move_cost: Currency,

    /// The value of castling
    castling_value: Currency,
    /// The cost of passing a turn (the tax for not moving)
    pass_value: Currency,

    ///  of the center sectors
    center_sector_income_value: Currency,
    /// Value of the outer sectors
    outer_sector_income_value: Currency,

    /// The compounding interest rate of performing additional moves
    move_interest_rate: f64,
}

impl Default for Market {
    fn default() -> Self {
        Self {
            pawn_value: Currency::doubloon() * PieceType::Pawn.get_value() * 2,
            knight_value: Currency::doubloon() * PieceType::Knight.get_value() * 2,
            bishop_value: Currency::doubloon() * PieceType::Bishop.get_value() * 2,
            rook_value: Currency::doubloon() * PieceType::Rook.get_value() * 2,
            queen_value: Currency::doubloon() * PieceType::Queen.get_value() * 2,
            king_value: Currency::doubloon() * PieceType::King.get_value() * 2,
            base_move_cost: Currency::doubloon(),

            castling_value: Currency::doubloon() * 2,
            pass_value: Currency::zero(),

            center_sector_income_value: Currency::doubloon() * 2,
            outer_sector_income_value: Currency::doubloon(),

            move_interest_rate: 2.0,
        }
    }
}

impl Market {
    /// Set the value of a pawn
    pub fn with_pawn_value(mut self, pawn_value: Currency) -> Self {
        self.pawn_value = pawn_value;
        self
    }

    /// Set the value of a knight
    pub fn with_knight_value(mut self, knight_value: Currency) -> Self {
        self.knight_value = knight_value;
        self
    }
    
    /// Set the value of a bishop
    pub fn with_bishop_value(mut self, bishop_value: Currency) -> Self {
        self.bishop_value = bishop_value;
        self
    }

    /// Set the value of a rook
    pub fn with_rook_value(mut self, rook_value: Currency) -> Self {
        self.rook_value = rook_value;
        self
    }

    /// Set the value of a queen
    pub fn with_queen_value(mut self, queen_value: Currency) -> Self {
        self.queen_value = queen_value;
        self
    }

    /// Set the value of a king
    pub fn with_king_value(mut self, king_value: Currency) -> Self {
        self.king_value = king_value;
        self
    }

    /// Set the value of all pieces
    pub fn with_piece_values(mut self, pawn_value: Currency, knight_value: Currency, bishop_value: Currency, rook_value: Currency, queen_value: Currency, king_value: Currency) -> Self {
        self.pawn_value = pawn_value;
        self.knight_value = knight_value;
        self.bishop_value = bishop_value;
        self.rook_value = rook_value;
        self.queen_value = queen_value;
        self.king_value = king_value;
        self
    }

    /// Set the value of castling
    pub fn with_castling_value(mut self, castling_value: Currency) -> Self {
        self.castling_value = castling_value;
        self
    }

    /// Set the income value of the center sectors
    pub fn with_center_sector_income_value(mut self, center_sector_income_value: Currency) -> Self {
        self.center_sector_income_value = center_sector_income_value;
        self
    }

    /// Set the income value of the outer sectors
    pub fn with_outer_sector_income_value(mut self, outer_sector_income_value: Currency) -> Self {
        self.outer_sector_income_value = outer_sector_income_value;
        self
    }

    /// Set the compounding interest rate of performing additional moves
    pub fn with_interest_rate(mut self, move_interest_rate: f64) -> Self {
        self.move_interest_rate = move_interest_rate;
        self
    }

    /// Set the base cost of a move
    pub fn with_base_move_cost(mut self, base_move_cost: Currency) -> Self {
        self.base_move_cost = base_move_cost;
        self
    }

    /// Get the base cost of a move
    pub fn get_base_move_cost(&self) -> Currency {
        self.base_move_cost
    }

    /// Get the value of a piece in the market
    pub fn get_piece_value(&self, piece: PieceType) -> Currency {
        match piece {
            PieceType::Pawn => self.pawn_value,
            PieceType::Knight => self.knight_value,
            PieceType::Bishop => self.bishop_value,
            PieceType::Rook => self.rook_value,
            PieceType::Queen => self.queen_value,
            PieceType::King => self.king_value,
        }
    }

    /// Get the value of a move in the market.
    pub fn get_move_value(&self, player_move: &Move) -> Currency {
        match player_move {
            Move::FromTo { .. } | Move::PieceTo { .. } => self.base_move_cost,
            Move::Purchase { piece, to: _ } => self.get_piece_value(*piece),
            Move::Castling { .. } => self.castling_value,
            Move::Many(moves) => {
                let mut total = Currency::zero();
                for (i, player_move) in moves.iter().enumerate() {
                    total += self.get_move_value(player_move) * self.move_interest_rate.powi(i as i32);
                }
                total
            },
            Move::Pass => self.pass_value,
            Move::Resign => Currency::zero(),
        }
    }

    /// Get income value of a sector
    #[inline]
    pub fn get_sector_value(&self, sector: Sector) -> Currency {
        if sector.is_center() {
            self.center_sector_income_value
        } else {
            self.outer_sector_income_value
        }
    }
}