use super::*;
use core::fmt::{Display, Formatter, Result as FmtResult};
use log::{warn, info, debug, trace, error};


/// Move a bit from one tile to another in the given bitboard.
fn move_bit(board: u64, from: Tile, to: Tile) -> u64 {
    let from_bit = from.to_bit();
    let to_bit = to.to_bit();
    if board & from_bit == 0 {
        return board;
    }
    (board & !from_bit) | to_bit
}

/// Return a bitboard with all the bits in the same sector as the given bit
/// set to 1.
fn sector_bits(board: u64, sector: Sector) -> u64 {
    // Get the sector bits
    let mut result = 0;
    let sector = sector.get_index();
    let rank = sector / 4;
    let file = sector % 4;
    for i in 0..2 {
        for j in 0..2 {
            result |= 1 << ((rank * 2 + i) * 8 + file * 2 + j);
        }
    }

    // Remove the bits that are not on the board
    result & board
}

/// Is the path from a source tile to a target tile blocked?
fn is_blocked(board: u64, from: Tile, to: Tile) -> bool {
    let mut result = false;

    // Check if the path is blocked
    
    let mut tile = from;
    for _ in 0..8 {
        tile.step_towards(to);
        if tile == from {
            continue;
        }
        if tile == to {
            break;
        }
        if board & tile.to_bit() != 0 {
            result = true;
            break;
        }
    }

    result
}

// Restrict an attack to visible squares only
fn visible_pieces(all_pieces: u64, origin: Tile, vision: u64) -> u64 {
    // Get the rank and file of the origin
    // Check all the bits in the attack and remove the ones that are not visible
    let mut result = 0;
    let mut visible_attack_bits = vision;

    // For each attack bit, check if its path is blocked
    while visible_attack_bits != 0 {
        // Get the next attack bit
        let attack_bit = visible_attack_bits & visible_attack_bits.wrapping_neg();
        // Remove the attack bit from the attack bits
        visible_attack_bits ^= attack_bit;

        // Get the attack tile
        let attack_tile = Tile::from_bit(attack_bit);

        // Check if the path is blocked
        if is_blocked(all_pieces, origin, attack_tile) {
            continue;
        }

        // Add the attack bit to the visible attack
        result |= attack_bit;
    }
    result
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Board {
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    white_queens: u64,
    white_king: u64,
    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    black_queens: u64,
    black_king: u64,
    en_passant: Option<Tile>,
    castling_rights: CastlingRights,
    current_turn: Color,
    winner: Option<Color>,
}

impl Default for Board {
    fn default() -> Self {
        let mut result = Self {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,
            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,
            en_passant: None,
            castling_rights: CastlingRights::default(),
            current_turn: Color::default(),
            winner: None,
        };

        // Spawn the white pieces
        for file in 'a'..='h' {
            result.spawn_white_pawn(Tile::new(Rank::PAWN_STARTER_WHITE, File::from_char(file)));
        }
        result.spawn_white_knight(Tile::new(Rank::BACK_RANK_WHITE, File::B));
        result.spawn_white_knight(Tile::new(Rank::BACK_RANK_WHITE, File::G));
        result.spawn_white_bishop(Tile::new(Rank::BACK_RANK_WHITE, File::C));
        result.spawn_white_bishop(Tile::new(Rank::BACK_RANK_WHITE, File::F));
        result.spawn_white_rook(Tile::new(Rank::BACK_RANK_WHITE, File::A));
        result.spawn_white_rook(Tile::new(Rank::BACK_RANK_WHITE, File::H));
        result.spawn_white_queen(Tile::new(Rank::BACK_RANK_WHITE, File::D));
        result.spawn_white_king(Tile::new(Rank::BACK_RANK_WHITE, File::E));

        // Spawn the black pieces
        for file in 'a'..='h' {
            result.spawn_black_pawn(Tile::new(Rank::PAWN_STARTER_BLACK, File::from_char(file)));
        }
        result.spawn_black_knight(Tile::new(Rank::BACK_RANK_BLACK, File::B));
        result.spawn_black_knight(Tile::new(Rank::BACK_RANK_BLACK, File::G));
        result.spawn_black_bishop(Tile::new(Rank::BACK_RANK_BLACK, File::C));
        result.spawn_black_bishop(Tile::new(Rank::BACK_RANK_BLACK, File::F));
        result.spawn_black_rook(Tile::new(Rank::BACK_RANK_BLACK, File::A));
        result.spawn_black_rook(Tile::new(Rank::BACK_RANK_BLACK, File::H));
        result.spawn_black_queen(Tile::new(Rank::BACK_RANK_BLACK, File::D));
        result.spawn_black_king(Tile::new(Rank::BACK_RANK_BLACK, File::E));

        result
    }
}

impl Board {
    /// An empty board with no pieces on it.
    pub fn empty() -> Self {
        Self {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,
            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,
            en_passant: None,
            castling_rights: CastlingRights::none(),
            current_turn: Color::default(),
            winner: None,
        }
    }

    /// Set the turn of who's allowed to play the next move.
    pub fn set_turn(&mut self, color: Color) {
        self.current_turn = color;
    }

    /// Perform a sanity check on the board.
    /// Confirm there are no overlapping pieces.
    pub fn sanity_check(&self) -> Result<(), ()> {
        info!("Performing sanity check on board");
        let mut bits = 0;
        bits |= self.white_pawns;
        if bits & self.white_knights != 0 {
            error!("White knights overlap with other white pieces");
            return Err(());
        }
        bits |= self.white_knights;
        if bits & self.white_bishops != 0 {
            error!("White bishops overlap with other white pieces");
            return Err(());
        }
        bits |= self.white_bishops;
        if bits & self.white_rooks != 0 {
            error!("White rooks overlap with other white pieces");
            return Err(());
        }
        bits |= self.white_rooks;
        if bits & self.white_queens != 0 {
            error!("White queens overlap with other white pieces");
            return Err(());
        }
        bits |= self.white_queens;
        if bits & self.white_king != 0 {
            error!("White king overlaps with other white pieces");
            return Err(());
        }
        bits |= self.white_king;
        if bits & self.black_pawns != 0 {
            error!("Black pawns overlap with other black pieces");
            return Err(());
        }
        bits |= self.black_pawns;
        if bits & self.black_knights != 0 {
            error!("Black knights overlap with other black pieces");
            return Err(());
        }
        bits |= self.black_knights;
        if bits & self.black_bishops != 0 {
            debug!("Black bishops overlap with other black pieces");
            return Err(());
        }
        bits |= self.black_bishops;
        if bits & self.black_rooks != 0 {
            error!("Black rooks overlap with other black pieces");
            return Err(());
        }
        bits |= self.black_rooks;
        if bits & self.black_queens != 0 {
            error!("Black queens overlap with other black pieces");
            return Err(());
        }
        bits |= self.black_queens;
        if bits & self.black_king != 0 {
            error!("Black king overlaps with other black pieces");
            return Err(());
        }

        // Check if king is off square, and if we still have castling rights
        if self.white_king != Tile::king_start_position(Color::White).to_bit() {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::White), Tile::new(Rank::BACK_RANK_WHITE, File::H)) {
                error!("White king is off square, but still has castling rights");
                return Err(());
            }
            if self.castling_rights.can_castle(Tile::king_start_position(Color::White), Tile::new(Rank::BACK_RANK_WHITE, File::A)) {
                error!("White king is off square, but still has castling rights");
                return Err(());
            }
        }
        if self.black_king != Tile::king_start_position(Color::Black).to_bit() {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::Black), Tile::new(Rank::BACK_RANK_BLACK, File::H)) {
                error!("Black king is off square, but still has castling rights");
                return Err(());
            }
            if self.castling_rights.can_castle(Tile::king_start_position(Color::Black), Tile::new(Rank::BACK_RANK_BLACK, File::A)) {
                error!("Black king is off square, but still has castling rights");
                return Err(());
            }
        }
        // Check if rook is off square, and if we still have castling rights
        if self.white_rooks & Tile::new(Rank::BACK_RANK_WHITE, File::H).to_bit() == 0 {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::White), Tile::new(Rank::BACK_RANK_WHITE, File::H)) {
                error!("White rook is off square, but still has castling rights");
                return Err(());
            }
        }
        if self.white_rooks & Tile::new(Rank::BACK_RANK_WHITE, File::A).to_bit() == 0 {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::White), Tile::new(Rank::BACK_RANK_WHITE, File::A)) {
                error!("White rook is off square, but still has castling rights");
                return Err(());
            }
        }
        if self.black_rooks & Tile::new(Rank::BACK_RANK_BLACK, File::H).to_bit() == 0 {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::Black), Tile::new(Rank::BACK_RANK_BLACK, File::H)) {
                error!("Black rook is off square, but still has castling rights");
                return Err(());
            }
        }
        if self.black_rooks & Tile::new(Rank::BACK_RANK_BLACK, File::A).to_bit() == 0 {
            if self.castling_rights.can_castle(Tile::king_start_position(Color::Black), Tile::new(Rank::BACK_RANK_BLACK, File::A)) {
                error!("Black rook is off square, but still has castling rights");
                return Err(());
            }
        }

        // Check if en passant is on a valid square
        // it must be on the 3rd or 6th rank
        if let Some(en_passant) = self.en_passant {
            if en_passant.get_rank() != Rank::PAWN_STARTER_WHITE.advance(Color::White, 1) && en_passant.get_rank() != Rank::PAWN_STARTER_BLACK.advance(Color::Black, 1) {
                error!("En passant is on an invalid square at {:?}", en_passant);
                return Err(());
            }

            let color = en_passant.get_player_side();

            // Confirm there's a pawn right past the en passant square
            let pawn_tile = en_passant.advance(color, 1);

            if self.get_piece(pawn_tile) != Some(Piece::pawn(color)) {
                error!("There is no {:?} pawn right at {pawn_tile:?} past the en passant square at {en_passant:?}", color);
                return Err(());
            }
        }

        /*
        // UNNECESSARY CHECK,
        // since players can purchase pieces on their back rank

        // Confirm there are no pawns on the back ranks
        for i in 0..File::RIGHTMOST.get_index() {
            if self.white_pawns & Tile::new(Rank::BACK_RANK_WHITE, File::from_index(i)).to_bit() != 0 {
                error!("White pawn on back rank, it should be promoted");
                return Err(());
            }
            if self.black_pawns & Tile::new(Rank::BACK_RANK_BLACK, File::from_index(i)).to_bit() != 0 {
                error!("Black pawn on back rank, it should be promoted");
                return Err(());
            }
        }
         */

        Ok(())
    }

    /// Whose turn is it?
    #[inline]
    pub fn whose_turn(&self) -> Color {
        self.current_turn
    }

    /// Get the castling rights of the board
    #[inline]
    pub fn get_castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Is there a black or white piece on the given tile?
    #[inline]
    pub fn has_piece_on(&self, location: Tile) -> bool {
        self.has_white_piece_on(location) || self.has_black_piece_on(location)
    }
    
    /// Can the given king and rook castle?
    /// This will check for the legality of the move, and if the castling rights
    /// allow the move.
    pub fn can_castle(&self, king: Tile, rook: Tile) -> bool {
        debug!("Checking if {:?} can castle with {:?}", king, rook);

        // Check if the castling rights haven't been disabled
        if !self.castling_rights.can_castle(king, rook) {
            debug!("Castling rights have been disabled");
            return false;
        }

        // Check if the king is in check
        if self.is_in_check(king.get_player_side()) {
            debug!("King is in check");
            return false;
        }
        
        if let Some(king_piece) = self.get_piece(king) {
            let color = king_piece.get_color();

            // Check if the path is blocked
            if is_blocked(self.all_pieces_as_bits() | self.get_attacking_bits(!color), king, rook) {
                debug!("Path is blocked");
                return false;
            }

            if let Some(rook_piece) = self.get_piece(rook) {
                if king_piece.get_type() == PieceType::King && rook_piece.get_type() == PieceType::Rook {
                    debug!("Castling is legal");
                    return true;
                }
            }
        }

        false
    }

    /// For all the sectors on the board, return true if the given color controls
    /// the sector. Controlling a sector is determined by having the most points
    /// in the sector.
    #[inline]
    pub(crate) fn get_controlled_sectors(&self, color: Color) -> [bool; Sector::NUM_SECTORS] {
        // The result of who controls what sector
        let mut result = [false; Sector::NUM_SECTORS];
        // For each sector, check if the given color controls it
        for sector in 0..Sector::NUM_SECTORS {
            // If the given color controls the sector, set the result to trueS
            if self.controls_sector(Sector::from_index(sector), color) {
                info!("{:?} controls sector {}", color, sector);
                result[sector] = true;
            }
        }
        result
    }

    /// Does the given color control the given sector?
    #[inline]
    pub fn controls_sector(&self, sector: Sector, color: Color) -> bool {
        self.who_controls_sector(sector) == Some(color)
    }

    /// Which color controls the given sector?
    pub fn who_controls_sector(&self, sector: Sector) -> Option<Color> {
        // Whoa has the majority point value in the sector?
        let (white_sector_value, black_sector_value) = self.get_sector_values(sector);

        if white_sector_value > black_sector_value {
            debug!("White controls sector {}", sector);
            Some(Color::White)
        } else if black_sector_value > white_sector_value {
            debug!("Black controls sector {}", sector);
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Get the value for a given player's sector on the board.
    fn get_sector_values(&self, sector: Sector) -> (Currency, Currency) {
        // Create a new board, where all the pieces that aren't in the sector are masked out
        let mut board = *self;
        let sector_bits = sector_bits(self.all_pieces_as_bits(), sector);
        let other_sector_bits = !sector_bits;
        board.white_pawns &= sector_bits;
        board.white_knights &= sector_bits;
        board.white_bishops &= sector_bits;
        board.white_rooks &= sector_bits;
        board.white_queens &= sector_bits;
        board.white_king &= sector_bits;
        board.black_pawns &= sector_bits;
        board.black_knights &= sector_bits;
        board.black_bishops &= sector_bits;
        board.black_rooks &= sector_bits;
        board.black_queens &= sector_bits;
        board.black_king &= sector_bits;

        // Get the values of the sector, for white and black.
        let mut white_value = Currency::zero();
        let mut black_value = Currency::zero();
        for tile in Tile::all() {
            if other_sector_bits & tile.to_bit() != 0 {
                continue;
            }
            if let Some(piece) = board.get_piece(tile) {
                let value = Currency::penny() * piece.get_type().get_value();
                match piece.get_color() {
                    Color::White => white_value += value,
                    Color::Black => black_value += value,
                } 
            }
        }
        debug!("Sector {} has a value of {} for white and {} for black", sector, white_value, black_value);
        (white_value, black_value)
    }

    /// Get the king bits of the given color
    #[inline]
    fn get_king_bits(&self, color: Color) -> u64 {
        match color {
            Color::White => self.white_king,
            Color::Black => self.black_king,
        }
    }

    /// Return 1 for all the bits of the board that white is attacking.
    fn white_attacking_bits(&self) -> u64 {
        let all_pieces_as_bits = self.all_pieces_as_bits();

        // Get the white pawn attacking bits
        let mut white_pawn_attacking_bits = 0;
        let mut white_pawn_bits = self.white_pawns;
        while white_pawn_bits != 0 {
            // Get the next white pawn bit
            let white_pawn_bit = white_pawn_bits & white_pawn_bits.wrapping_neg();
            // Get the tile of the white pawn
            let white_pawn_tile = Tile::from_bit(white_pawn_bit);
            // Add the white pawn attacking bits to the result
            // white_pawn_attacking_bits |= white_pawn_tile.pawn_attacking_bits(Color::White);
            white_pawn_attacking_bits |= visible_pieces(all_pieces_as_bits, white_pawn_tile, white_pawn_tile.pawn_attacking_bits(Color::White));
            // Remove the white pawn bit from the white pawn bits
            white_pawn_bits ^= white_pawn_bit;
        }

        // Get the white knight attacking bits
        let mut white_knight_attacking_bits = 0;
        let mut white_knight_bits = self.white_knights;
        while white_knight_bits != 0 {
            // Get the next white knight bit
            let white_knight_bit = white_knight_bits & white_knight_bits.wrapping_neg();
            // Get the tile of the white knight
            let white_knight_tile = Tile::from_bit(white_knight_bit);
            // Add the white knight attacking bits to the result
            // white_knight_attacking_bits |= white_knight_tile.knight_attacking_bits();
            white_knight_attacking_bits |= white_knight_tile.knight_attacking_bits();
            // Remove the white knight bit from the white knight bits
            white_knight_bits ^= white_knight_bit;
        }

        // Get the white bishop attacking bits
        let mut white_bishop_attacking_bits = 0;
        let mut white_bishop_bits = self.white_bishops;
        while white_bishop_bits != 0 {
            // Get the next white bishop bit
            let white_bishop_bit = white_bishop_bits & white_bishop_bits.wrapping_neg();
            // Get the tile of the white bishop
            let white_bishop_tile = Tile::from_bit(white_bishop_bit);
            // Add the white bishop attacking bits to the result
            // white_bishop_attacking_bits |= white_bishop_tile.bishop_attacking_bits();
            white_bishop_attacking_bits |= visible_pieces(all_pieces_as_bits, white_bishop_tile, white_bishop_tile.bishop_attacking_bits());
            // Remove the white bishop bit from the white bishop bits
            white_bishop_bits ^= white_bishop_bit;
        }

        // Get the white rook attacking bits
        let mut white_rook_attacking_bits = 0;
        let mut white_rook_bits = self.white_rooks;
        while white_rook_bits != 0 {
            // Get the next white rook bit
            let white_rook_bit = white_rook_bits & white_rook_bits.wrapping_neg();
            // Get the tile of the white rook
            let white_rook_tile = Tile::from_bit(white_rook_bit);
            // Add the white rook attacking bits to the result
            // white_rook_attacking_bits |= white_rook_tile.rook_attacking_bits();
            white_rook_attacking_bits |= visible_pieces(all_pieces_as_bits, white_rook_tile, white_rook_tile.rook_attacking_bits());
            // Remove the white rook bit from the white rook bits
            white_rook_bits ^= white_rook_bit;
        }

        // Get the white queen attacking bits
        let mut white_queen_attacking_bits = 0;
        let mut white_queen_bits = self.white_queens;
        while white_queen_bits != 0 {
            // Get the next white queen bit
            let white_queen_bit = white_queen_bits & white_queen_bits.wrapping_neg();
            // Get the tile of the white queen
            let white_queen_tile = Tile::from_bit(white_queen_bit);
            // Add the white queen attacking bits to the result
            // white_queen_attacking_bits |= white_queen_tile.queen_attacking_bits();
            white_queen_attacking_bits |= visible_pieces(all_pieces_as_bits, white_queen_tile, white_queen_tile.queen_attacking_bits());
            // Remove the white queen bit from the white queen bits
            white_queen_bits ^= white_queen_bit;
        }

        // Get the white king attacking bits
        let mut white_king_attacking_bits = 0;
        let mut white_king_bits = self.white_king;
        while white_king_bits != 0 {
            // Get the next white king bit
            let white_king_bit = white_king_bits & white_king_bits.wrapping_neg();
            // Get the tile of the white king
            let white_king_tile = Tile::from_bit(white_king_bit);
            // Add the white king attacking bits to the result
            // white_king_attacking_bits |= white_king_tile.king_attacking_bits();
            white_king_attacking_bits |= visible_pieces(all_pieces_as_bits, white_king_tile, white_king_tile.king_attacking_bits());
            // Remove the white king bit from the white king bits
            white_king_bits ^= white_king_bit;
        }

        // Return the white attacking bits
        white_pawn_attacking_bits
            | white_knight_attacking_bits
            | white_bishop_attacking_bits
            | white_rook_attacking_bits
            | white_queen_attacking_bits
            | white_king_attacking_bits
    }

    /// Return 1 for all the bits of the board that black is attacking.
    /// This is the same as white_attacking_bits, but for black.
    pub fn black_attacking_bits(&self) -> u64 {
        // Get the black pawn attacking bits
        let mut black_pawn_attacking_bits = 0;
        let mut black_pawn_bits = self.black_pawns;
        while black_pawn_bits != 0 {
            // Get the next black pawn bit
            let black_pawn_bit = black_pawn_bits & black_pawn_bits.wrapping_neg();
            // Get the tile of the black pawn
            let black_pawn_tile = Tile::from_bit(black_pawn_bit);
            // Add the black pawn attacking bits to the result
            // black_pawn_attacking_bits |= black_pawn_tile.pawn_attacking_bits(Color::Black);
            black_pawn_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_pawn_tile, black_pawn_tile.pawn_attacking_bits(Color::Black));
            // Remove the black pawn bit from the black pawn bits
            black_pawn_bits ^= black_pawn_bit;
        }

        // Get the black knight attacking bits
        let mut black_knight_attacking_bits = 0;
        let mut black_knight_bits = self.black_knights;
        while black_knight_bits != 0 {
            // Get the next black knight bit
            let black_knight_bit = black_knight_bits & black_knight_bits.wrapping_neg();
            // Get the tile of the black knight
            let black_knight_tile = Tile::from_bit(black_knight_bit);
            // Add the black knight attacking bits to the result
            black_knight_attacking_bits |= black_knight_tile.knight_attacking_bits();
            // black_knight_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_knight_tile, black_knight_tile.knight_attacking_bits());
            // Remove the black knight bit from the black knight bits
            black_knight_bits ^= black_knight_bit;
        }

        // Get the black bishop attacking bits
        let mut black_bishop_attacking_bits = 0;
        let mut black_bishop_bits = self.black_bishops;
        while black_bishop_bits != 0 {
            // Get the next black bishop bit
            let black_bishop_bit = black_bishop_bits & black_bishop_bits.wrapping_neg();
            // Get the tile of the black bishop
            let black_bishop_tile = Tile::from_bit(black_bishop_bit);
            // Add the black bishop attacking bits to the result
            // black_bishop_attacking_bits |= black_bishop_tile.bishop_attacking_bits();
            black_bishop_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_bishop_tile, black_bishop_tile.bishop_attacking_bits());
            // Remove the black bishop bit from the black bishop bits
            black_bishop_bits ^= black_bishop_bit;
        }

        // Get the black rook attacking bits
        let mut black_rook_attacking_bits = 0;
        let mut black_rook_bits = self.black_rooks;
        while black_rook_bits != 0 {
            // Get the next black rook bit
            let black_rook_bit = black_rook_bits & black_rook_bits.wrapping_neg();
            // Get the tile of the black rook
            let black_rook_tile = Tile::from_bit(black_rook_bit);
            // Add the black rook attacking bits to the result
            // black_rook_attacking_bits |= black_rook_tile.rook_attacking_bits();
            black_rook_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_rook_tile, black_rook_tile.rook_attacking_bits());
            // Remove the black rook bit from the black rook bits
            black_rook_bits ^= black_rook_bit;
        }

        // Get the black queen attacking bits
        let mut black_queen_attacking_bits = 0;
        let mut black_queen_bits = self.black_queens;
        while black_queen_bits != 0 {
            // Get the next black queen bit
            let black_queen_bit = black_queen_bits & black_queen_bits.wrapping_neg();
            // Get the tile of the black queen
            let black_queen_tile = Tile::from_bit(black_queen_bit);
            // Add the black queen attacking bits to the result
            // black_queen_attacking_bits |= black_queen_tile.queen_attacking_bits();
            black_queen_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_queen_tile, black_queen_tile.queen_attacking_bits());
            // Remove the black queen bit from the black queen bits
            black_queen_bits ^= black_queen_bit;
        }

        // Get the black king attacking bits
        let mut black_king_attacking_bits = 0;
        let mut black_king_bits = self.black_king;
        while black_king_bits != 0 {
            // Get the next black king bit
            let black_king_bit = black_king_bits & black_king_bits.wrapping_neg();
            // Get the tile of the black king
            let black_king_tile = Tile::from_bit(black_king_bit);
            // Add the black king attacking bits to the result
            // black_king_attacking_bits |= black_king_tile.king_attacking_bits();
            black_king_attacking_bits |= visible_pieces(self.all_pieces_as_bits(), black_king_tile, black_king_tile.king_attacking_bits());

            // Remove the black king bit from the black king bits
            black_king_bits ^= black_king_bit;
        }

        // Return the black attacking bits
        black_pawn_attacking_bits
            | black_knight_attacking_bits
            | black_bishop_attacking_bits
            | black_rook_attacking_bits
            | black_queen_attacking_bits
            | black_king_attacking_bits
    }

    #[inline]
    fn get_attacking_bits(&self, color: Color) -> u64 {
        match color {
            Color::White => self.white_attacking_bits(),
            Color::Black => self.black_attacking_bits(),
        }
    }

    /// Is this move a castling?
    fn is_castling_move(&self, from: Tile, to: Tile) -> bool {
        let src_piece = self.get_piece(from);

        if let Some(src_piece) = src_piece {
            if src_piece.get_type() == PieceType::King {
                return self.get_castling_rights().is_castling_move(from, to);
            }
        }

        false
    }

    /// Are any of the player's kings in check?
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_bits = self.get_king_bits(color);

        // Get the enemy attacking bits
        let enemy_attacking_bits = self.get_attacking_bits(!color);

        // Check if the king is in check
        (king_bits & enemy_attacking_bits) != 0
    }

    /// Is the player in checkmate?
    pub fn is_in_checkmate(&self, color: Color) -> bool {
        info!("Checking if {:?} is in checkmate", color);
        // Check if the player is in check
        if !self.is_in_check(color) {
            info!("{:?} is not in check", color);
            return false;
        }

        // Check if the player can move out of check
        for tile in Tile::all() {
            if let Some(piece) = self.get_piece(tile) {
                if piece.get_color() == color {
                    for to in tile.get_moves(piece) {
                        if self.is_legal_piece_move(tile, to) {
                            info!("{:?} can move from {:?} to {:?} to get out of check", color, tile, to);
                            return false;
                        }
                    }
                }
            }
        }

        info!("{:?} is in checkmate", color);
        true
    }

    /// Is the board in a state of stalemate?
    pub fn is_stalemate(&self) -> bool {
        info!("Checking if the board is in stalemate");
        // Check if the player is in check
        if self.is_in_check(self.current_turn) {
            info!("The board is not in stalemate because {:?} is in check", self.current_turn);
            return false;
        }

        // Check if the player can move out of check
        for tile in Tile::all() {
            if let Some(piece) = self.get_piece(tile) {
                if piece.get_color() == self.current_turn {
                    for to in tile.get_moves(piece) {
                        if self.is_legal_piece_move(tile, to) {
                            info!("The board is not in stalemate because {:?} can move from {:?} to {:?} to get out of check", self.current_turn, tile, to);
                            return false;
                        }
                    }
                }
            }
        }

        info!("The board is in stalemate");
        true
    }

    /// Is a move legal? This will return if the move can be played.
    /// 
    /// This is the public interface used to check if a move can be applied to the board.
    pub fn is_legal_move(&self, player_move: &Move) -> bool {
        trace!("Checking if move {:?} is legal for player {:?}", player_move, self.whose_turn());
        // Check if the move is a castling move
        match player_move {
            Move::Castling(side) => {
                // Get the king and rook tiles
                let king = Tile::king_start_position(self.current_turn);
                let rook = Tile::rook_start_position(self.current_turn, *side);

                // Check if the castling move is legal
                self.can_castle(king, rook)
            }

            Move::FromTo { from, to, .. } => {
                self.is_legal_piece_move(*from, *to)
            }

            Move::PieceTo { piece, to, .. } => {
                if let Some(from) = self.get_eligible_piece(*piece, *to) {
                    // Get the eligible piece
                    trace!("Eligible piece found for {:?} at {}", piece, from);
                    self.is_legal_piece_move(from, *to)
                } else {
                    trace!("No eligible piece found for {:?}", piece);
                    false
                }
            }

            // Assume that all purchase moves are illegal
            Move::Pass => false,
            Move::Resign => true,

            Move::Many(moves) if !moves.is_empty() => {
                let mut copy = self.clone();
                for player_move in moves {
                    copy.current_turn = self.whose_turn();
                    if !copy.is_legal_move(player_move) {
                        trace!("Illegal move {:?}", player_move);
                        return false;
                    }

                    copy.current_turn = self.whose_turn();
                    if copy.apply(player_move.clone()).is_err() {
                        trace!("Failed to apply move {:?}", player_move);
                        return false;
                    }

                    copy.current_turn = self.whose_turn();
                }
                info!("All moves are legal");
                true
            }
            Move::Many(_) => false,
            Move::Purchase { to, .. } => {
                !self.has_piece_on(*to) && !self.is_in_check(self.whose_turn())
            }
        }
    }

    /// Would the player be in check after moving a piece from one tile to another?
    fn is_in_check_after_move(&self, color: Color, from: Tile, to: Tile) -> bool {
        // Move the piece
        let mut copy = *self;
        copy.move_piece(from, to);
        copy.current_turn = self.current_turn;
        copy.is_in_check(color)
    }

    /// Is a piece move legal? This is a private interface used to check internally
    /// if a move of a piece from one tile to another is legal.
    /// 
    /// Castling is encoded by passing the king's tile as the `from` tile, and the
    /// rook's tile as the `to` tile.
    pub fn is_legal_piece_move(&self, from: Tile, to: Tile) -> bool {
        info!("Checking if piece move from {from} to {to} is legal");
        if INSERT_SANITY_CHECKS {
            if self.sanity_check().is_err() {
                error!("{self}");
                panic!("Board is in an invalid state");
            }
        }
        // Get piece at source and destination
        let src_piece = self.get_piece(from);
        let dst_piece = self.get_piece(to);

        match (src_piece, dst_piece) {
            // Handle capture move
            (Some(src_piece), Some(dst_piece)) => {
                // Check if the piece is moving to a square occupied by a friendly piece
                if src_piece.get_color() == dst_piece.get_color() {
                    // Check if is castling
                    if self.is_castling_move(from, to) && self.can_castle(from, to) {
                        // Can we castle?
                        return true;
                    }
                    debug!("Piece {:?} is moving to a square occupied by a friendly piece", src_piece);
                    return false;
                }
                
                // Check if we control the source piece being moved
                if src_piece.get_color() != self.current_turn {
                    debug!("Piece {:?} is not owned by player", src_piece);
                    return false;
                }

                // Check if the piece type can move to the destination
                if !src_piece.can_move(from, to, true, self.en_passant) {
                    debug!("Piece {:?} cannot move from {:?} to {:?}", src_piece, from, to);
                    return false;
                }

                if self.is_blocked(from, to) && src_piece.get_type() != PieceType::Knight {
                    debug!("Path from {:?} to {:?} is blocked", from, to);
                    return false;
                }

                if self.is_in_check_after_move(self.current_turn, from, to) {
                    debug!("Move from {:?} to {:?} would put player in check", from, to);
                    return false;
                }

                info!("Moving from {from} to {to} is legal");
                return true;
            },
            // Handle no capture move
            (Some(src_piece), None) => {
                trace!("Moving from {:?} to {:?}", from, to);
                if src_piece.get_color() != self.current_turn {
                    debug!("Piece {:?} is not owned by player", src_piece);
                    return false;
                }

                // Check if is castling
                if self.is_castling_move(from, to)  {
                    // Can we castle?
                    if self.can_castle(from, to) {
                        trace!("Castling from {:?} to {:?} is legal", from, to);
                        return true;
                    }
                    debug!("Castling from {:?} to {:?} is illegal", from, to);
                    return false;
                }

                // Check if the piece type can move to the destination
                if !src_piece.can_move(from, to, false, self.en_passant) {
                    debug!("Piece {:?} cannot move from {:?} to {:?}", src_piece, from, to);
                    return false;
                }

                if self.is_blocked(from, to) && src_piece.get_type() != PieceType::Knight {
                    debug!("Path from {:?} to {:?} is blocked", from, to);
                    return false;
                }

                // Check if will be in check after the move
                if self.is_in_check_after_move(self.current_turn, from, to) {
                    debug!("Move from {:?} to {:?} would put player in check", from, to);
                    return false;
                }

                info!("Moving from {from} to {to} is legal");
                return true;
            }

            (None, _) => {
                // There is no piece at the source
                trace!("No piece at {:?}", from);
                return false;
            }
        }
    }
    
    /// Is this a pawn capturing to the en passant tile?
    fn is_en_passant_capture(&self, from: Tile, to: Tile) -> bool {
        // Check if the move is an en passant (only possibility since no capture)
        if let Some(en_passant) = self.en_passant {
            // If the move is an en passant, then the destination must be the en passant tile
            if en_passant == to {
                // Check if the attacking piece is a pawn
                if let Some(piece) = self.get_piece(from) {
                    // If the attacking piece is a pawn, then perform the en passant
                    if piece.get_type() == PieceType::Pawn {
                        info!("En-passant capture detected");
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Perform an en passant capture using a pawn at a given tile. This abstracts
    /// the need to know the attacked pawn's tile is.
    fn capture_en_passant(&mut self, from: Tile) {
        info!("Capturing en passant from {:?}", from);
        // Get the en passant tile
        let en_passant = self.en_passant.unwrap();

        // Get the en passant pawn tile
        let en_passant_pawn = en_passant.advance(self.current_turn, -1);

        // Remove the en passant pawn
        self.remove_piece(en_passant_pawn);

        // Move the attacking pawn
        self.move_piece(from, en_passant);

        // Remove the en passant tile
        self.en_passant = None;
    }

    fn detect_possible_en_passant(&mut self, from: Tile, to: Tile) {
        info!("Checking for possible en-passant next turn...");
        self.en_passant = None;
        // Check if the move is a pawn double move
        if let Some(piece) = self.get_piece(from) {
            if piece.get_type() == PieceType::Pawn {
                if from.get_rank() == Rank::PAWN_STARTER_WHITE || from.get_rank() == Rank::PAWN_STARTER_BLACK {
                    if to.get_rank() == Rank::PAWN_STARTER_WHITE + 2 || to.get_rank() == Rank::PAWN_STARTER_BLACK - 2 {
                        info!("Possible next-turn-en-passant detected, marking the tile behind the pawn as capturable");
                        // Set the en passant tile
                        self.en_passant = Some(from.advance(self.current_turn, 1));
                        return;
                    }
                }
            }
        }
        info!("No possible next-turn-en-passant detected");
    }

    /// Perform a move on the board.
    pub fn apply(&mut self, player_move: Move) -> Result<(), ()> {
        info!("Applying move {:?}", player_move);
        if INSERT_SANITY_CHECKS {
            assert!(self.sanity_check().is_ok());
        }
        match player_move {
            Move::FromTo { from, to, promotion } => {
                self.perform_move_from_to(from, to, promotion)
            }
            Move::PieceTo { piece, to, promotion } => {
                let from = self.get_eligible_piece(piece, to).ok_or(())?;
                self.perform_move_from_to(from, to, promotion)
            }
            Move::Castling(side) => {
                let king = Tile::king_start_position(self.current_turn);
                let rook = Tile::rook_start_position(self.current_turn, side);
                self.perform_move_from_to(king, rook, None)
            }
            Move::Many(moves) if !moves.is_empty() => {
                let turn = self.current_turn;
                for player_move in moves {
                    self.current_turn = turn;
                    self.apply(player_move)?
                }
                self.current_turn = !turn;
                Ok(())
            }
            Move::Many(_) => {
                Ok(())
            }
            Move::Resign => {
                self.set_winner(!self.current_turn);
                self.current_turn = !self.current_turn;
                Ok(())
            }
            Move::Pass => {
                self.current_turn = !self.current_turn;
                Ok(())
            }
            Move::Purchase { piece, to } => {
                self.spawn(piece, to);
                self.current_turn = !self.current_turn;
                Ok(())
            }
        }
    }

    /// Mark a given player as the winner.
    fn set_winner(&mut self, winner: Color) {
        info!("Setting winner to {:?}", winner);
        self.winner = Some(winner);
    }

    /// Given a move of a piece type and a destination tile, return the eligible
    /// piece that can move to the destination tile. Return None if no piece can
    /// move to the destination tile, or if there are multiple pieces that can
    /// move to the destination tile.
    fn get_eligible_piece(&self, piece: PieceType, to: Tile) -> Option<Tile> {
        info!("Getting eligible piece of type {:?} to move to {:?}", piece, to);
        let is_attack = self.has_piece_on(to);
        for tile in Tile::all() {
            if let Some(src_piece) = self.get_piece(tile) {
                if src_piece.get_type() == piece && src_piece.get_color() == self.current_turn {
                    if src_piece.can_move(tile, to, is_attack, self.en_passant) {
                        info!("Found eligible piece at {:?}", tile);
                        return Some(tile);
                    }
                }
            }
        }
        warn!("No eligible piece found");
        None
    }

    /// Perform castling with a given king and rook tile.
    /// This will move the king to the proper, predefined square for the
    /// king after castling, and it will move the rook to the square on
    /// the opposite side of the king.
    fn perform_castling(&mut self, king_tile: Tile, rook_tile: Tile) {
        info!("Performing castling on king at {king_tile} and rook at {rook_tile}");
        if !self.is_castling_move(king_tile, rook_tile) {
            warn!("Called perform_castling to perform castling on invalid king and rook tiles");
            return;
        }

        // Remove the castling rights
        self.castling_rights.disable_castling(king_tile, rook_tile);
        let side = rook_tile.get_castling_side();

        // Move the king to the castling tile
        let new_king_tile = Tile::castling_destination_for_king(self.current_turn, side);
        self.move_piece(king_tile, new_king_tile);
        let new_rook_tile = Tile::castling_destination_for_rook(self.current_turn, side);
        self.move_piece(rook_tile, new_rook_tile);

        info!("Castling performed");
    }

    /// Given a piece move from one tile to another, detect if the move disables
    /// any castling rights.
    fn detect_disabled_castling_rights(&mut self, from: Tile, to: Tile) {
        info!("Checking disabled castling rights...");
        if self.is_castling_move(from, to) {
            warn!("Called detect_disabled_castling_rights on a move that performs castling - rights should already be revoked");
            let color = self.current_turn;
            info!("Disabling castling rights for {color:?}");
            self.castling_rights.disable_castling_color(color);
            return;
        }

        // Check if the move is a rook move
        if from == Tile::WHITE_KINGSIDE_ROOK_START || from == Tile::WHITE_QUEENSIDE_ROOK_START {
            // Check if the move is a kingside rook move
            if from == Tile::WHITE_KINGSIDE_ROOK_START {
                // Disable the white kingside castling rights
                info!("White kingside rook moved, disabling white kingside castling rights");
                self.castling_rights.disable_castling_color_and_side(Color::White, CastlingSide::King);
            }

            // Check if the move is a queenside rook move
            if from == Tile::WHITE_QUEENSIDE_ROOK_START {
                // Disable the white queenside castling rights
                info!("White queenside rook moved, disabling white queenside castling rights");
                self.castling_rights.disable_castling_color_and_side(Color::White, CastlingSide::Queen);
            }
        } else if from == Tile::BLACK_KINGSIDE_ROOK_START || from == Tile::BLACK_QUEENSIDE_ROOK_START {
            // Check if the move is a kingside rook move
            if from == Tile::BLACK_KINGSIDE_ROOK_START {
                // Disable the black kingside castling rights
                info!("Black kingside rook moved, disabling black kingside castling rights");
                self.castling_rights.disable_castling_color_and_side(Color::Black, CastlingSide::King);
            }

            // Check if the move is a queenside rook move
            if from == Tile::BLACK_QUEENSIDE_ROOK_START {
                // Disable the black queenside castling rights
                info!("Black queenside rook moved, disabling black queenside castling rights");
                self.castling_rights.disable_castling_color_and_side(Color::Black, CastlingSide::Queen);
            }
        } else if from == Tile::WHITE_KING_START {
            // Disable the white castling rights
            info!("White king moved, disabling white castling rights");
            self.castling_rights.disable_castling_color(Color::White);
        } else if from == Tile::BLACK_KING_START {
            // Disable the black castling rights
            info!("Black king moved, disabling black castling rights");
            self.castling_rights.disable_castling_color(Color::Black);
        }
    }

    /// Perform a move from one tile to another.
    /// Returns true if the move was successful, false otherwise.
    /// 
    /// This is the function to use to move pieces on the board.
    /// it will perform the validation and the move, and change
    /// the state accordingly.
    fn perform_move_from_to(&mut self, from: Tile, to: Tile, promotion: Option<PieceType>) -> Result<(), ()> {
        if !self.is_legal_piece_move(from, to) {
            // debug!("Tried to perform illegal move from {from:?} to {to:?}");
            return Err(())
        }
        
        self.current_turn = self.get_piece(from).ok_or(())?.get_color();
        
        // Check if the move is a castling
        if self.is_castling_move(from, to) {
            // Perform the castling
            self.perform_castling(from, to);

            self.current_turn = !self.current_turn;
            return Ok(());
        }
        
        // Check if any castling rights are disabled by this move
        self.detect_disabled_castling_rights(from, to);

        // Check if move is an en passant capture
        if self.is_en_passant_capture(from, to) {
            // Perform the en passant capture
            self.capture_en_passant(from);
        } else {
            self.remove_piece(to);
            // Check if this is a promotion
            if self.is_valid_promotion(from, to) {
                let promotion = promotion.unwrap_or(PieceType::Queen);
                info!("Promoting pawn at {to:?} to {promotion:?}");
                self.remove_piece(from);
                self.spawn(promotion, to)
            } else {
                // Remove the en passant tile
                self.detect_possible_en_passant(from, to);
                self.move_piece(from, to);
            }
        }

        self.current_turn = !self.current_turn;
        return Ok(());
    }

    /// Spawn a piece of a given type on a given tile.
    /// This will remove any piece that was previously on the tile.
    pub fn spawn(&mut self, piece: PieceType, to: Tile) {
        self.remove_piece(to);
        let color = self.current_turn;
        match color {
            Color::White => {
                match piece {
                    PieceType::Pawn => self.spawn_white_pawn(to),
                    PieceType::Knight => self.spawn_white_knight(to),
                    PieceType::Bishop => self.spawn_white_bishop(to),
                    PieceType::Rook => self.spawn_white_rook(to),
                    PieceType::Queen => self.spawn_white_queen(to),
                    PieceType::King => self.spawn_white_king(to),
                }
            },
            Color::Black => {
                match piece {
                    PieceType::Pawn => self.spawn_black_pawn(to),
                    PieceType::Knight => self.spawn_black_knight(to),
                    PieceType::Bishop => self.spawn_black_bishop(to),
                    PieceType::Rook => self.spawn_black_rook(to),
                    PieceType::Queen => self.spawn_black_queen(to),
                    PieceType::King => self.spawn_black_king(to),
                }
            }
        }
    }

    /// Is the move from one tile to another a valid promotion?
    /// 
    /// This will return true if the move is a pawn moving to the last rank of either player.
    pub(crate) fn is_valid_promotion(&self, from: Tile, to: Tile) -> bool {
        // Check that the piece is a pawn
        if let Some(piece) = self.get_piece(from) {
            if piece.get_type() == PieceType::Pawn {
                if to.get_rank() == Rank::BACK_RANK_BLACK && piece.get_color() == Color::White {
                    return true;
                }

                if to.get_rank() == Rank::BACK_RANK_WHITE && piece.get_color() == Color::Black {
                    return true;
                }
            }
        }

        false
    }
    
    /// Is the path from one tile to another blocked?
    #[inline]
    fn is_blocked(&self, from: Tile, to: Tile) -> bool {
        is_blocked(self.all_pieces_as_bits(), from, to)
    }

    #[inline]
    fn all_pieces_as_bits(&self) -> u64 {
        self.white_pieces_as_bits() | self.black_pieces_as_bits()
    }

    #[inline]
    fn white_pieces_as_bits(&self) -> u64 {
        self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king
    }

    #[inline]
    fn black_pieces_as_bits(&self) -> u64 {
        self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king
    }

    /// Returns the piece on the given location
    /// Returns None if there is no piece on the location
    #[inline]
    pub fn get_piece(&self, location: Tile) -> Option<Piece> {
        // Check if there is a white piece on the location
        let bit = location.to_bit();
        if (self.white_pawns & bit) != 0 {
            return Some(Piece(PieceType::Pawn, Color::White));
        }
        if (self.white_knights & bit) != 0 {
            return Some(Piece(PieceType::Knight, Color::White));
        }
        if (self.white_bishops & bit) != 0 {
            return Some(Piece(PieceType::Bishop, Color::White));
        }
        if (self.white_rooks & bit) != 0 {
            return Some(Piece(PieceType::Rook, Color::White));
        }
        if (self.white_queens & bit) != 0 {
            return Some(Piece(PieceType::Queen, Color::White));
        }
        if (self.white_king & bit) != 0 {
            return Some(Piece(PieceType::King, Color::White));
        }

        // Check if there is a black piece on the location
        if (self.black_pawns & bit) != 0 {
            return Some(Piece(PieceType::Pawn, Color::Black));
        }
        if (self.black_knights & bit) != 0 {
            return Some(Piece(PieceType::Knight, Color::Black));
        }
        if (self.black_bishops & bit) != 0 {
            return Some(Piece(PieceType::Bishop, Color::Black));
        }
        if (self.black_rooks & bit) != 0 {
            return Some(Piece(PieceType::Rook, Color::Black));
        }
        if (self.black_queens & bit) != 0 {
            return Some(Piece(PieceType::Queen, Color::Black));
        }
        if (self.black_king & bit) != 0 {
            return Some(Piece(PieceType::King, Color::Black));
        }


        // There is no piece on the location
        None
    }

    /// Remove a piece from the board
    #[inline]
    pub fn remove_piece(&mut self, location: Tile) {
        let bit = location.to_bit();
        self.white_pawns &= !bit;
        self.white_knights &= !bit;
        self.white_bishops &= !bit;
        self.white_rooks &= !bit;
        self.white_queens &= !bit;
        self.white_king &= !bit;

        self.black_pawns &= !bit;
        self.black_knights &= !bit;
        self.black_bishops &= !bit;
        self.black_rooks &= !bit;
        self.black_queens &= !bit;
        self.black_king &= !bit;
    }

    /// Move a piece from one location to another
    #[inline]
    fn move_piece(&mut self, from: Tile, to: Tile) {
        self.remove_piece(to);

        // Move the piece in all the bitboards
        self.white_pawns = move_bit(self.white_pawns, from, to);
        self.white_knights = move_bit(self.white_knights, from, to);
        self.white_bishops = move_bit(self.white_bishops, from, to);
        self.white_rooks = move_bit(self.white_rooks, from, to);
        self.white_queens = move_bit(self.white_queens, from, to);
        self.white_king = move_bit(self.white_king, from, to);
        self.black_pawns = move_bit(self.black_pawns, from, to);
        self.black_knights = move_bit(self.black_knights, from, to);
        self.black_bishops = move_bit(self.black_bishops, from, to);
        self.black_rooks = move_bit(self.black_rooks, from, to);
        self.black_queens = move_bit(self.black_queens, from, to);
        self.black_king = move_bit(self.black_king, from, to);
    }

    #[inline]
    pub fn spawn_white_pawn(&mut self, location: Tile) {
        self.white_pawns |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_pawn(&mut self, location: Tile) {
        self.black_pawns |= location.to_bit();
    }

    #[inline]
    pub fn spawn_white_knight(&mut self, location: Tile) {
        self.white_knights |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_knight(&mut self, location: Tile) {
        self.black_knights |= location.to_bit();
    }

    #[inline]
    pub fn spawn_white_bishop(&mut self, location: Tile) {
        self.white_bishops |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_bishop(&mut self, location: Tile) {
        self.black_bishops |= location.to_bit();
    }

    #[inline]
    pub fn spawn_white_rook(&mut self, location: Tile) {
        self.white_rooks |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_rook(&mut self, location: Tile) {
        self.black_rooks |= location.to_bit();
    }

    #[inline]
    pub fn spawn_white_queen(&mut self, location: Tile) {
        self.white_queens |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_queen(&mut self, location: Tile) {
        self.black_queens |= location.to_bit();
    }

    #[inline]
    pub fn spawn_white_king(&mut self, location: Tile) {
        self.white_king |= location.to_bit();
    }

    #[inline]
    pub fn spawn_black_king(&mut self, location: Tile) {
        self.black_king |= location.to_bit();
    }

    /// Returns the number of white pieces on the board
    #[inline]
    pub fn white_piece_count(&self) -> u64 {
        self.white_pieces_as_bits().count_ones() as u64
    }

    /// Returns the number of black pieces on the board
    #[inline]
    pub fn black_piece_count(&self) -> u64 {
        self.black_pieces_as_bits().count_ones() as u64
    }

    #[inline]
    pub fn white_pawn_count(&self) -> u64 {
        self.white_pawns.count_ones() as u64
    }

    #[inline]
    pub fn black_pawn_count(&self) -> u64 {
        self.black_pawns.count_ones() as u64
    }

    #[inline]
    pub fn white_knight_count(&self) -> u64 {
        self.white_knights.count_ones() as u64
    }

    #[inline]
    pub fn black_knight_count(&self) -> u64 {
        self.black_knights.count_ones() as u64
    }

    #[inline]
    pub fn white_bishop_count(&self) -> u64 {
        self.white_bishops.count_ones() as u64
    }

    #[inline]
    pub fn black_bishop_count(&self) -> u64 {
        self.black_bishops.count_ones() as u64
    }

    #[inline]
    pub fn white_rook_count(&self) -> u64 {
        self.white_rooks.count_ones() as u64
    }

    #[inline]
    pub fn black_rook_count(&self) -> u64 {
        self.black_rooks.count_ones() as u64
    }

    #[inline]
    pub fn white_queen_count(&self) -> u64 {
        self.white_queens.count_ones() as u64
    }

    #[inline]
    pub fn black_queen_count(&self) -> u64 {
        self.black_queens.count_ones() as u64
    }

    #[inline]
    pub fn white_king_count(&self) -> u64 {
        self.white_king.count_ones() as u64
    }

    #[inline]
    pub fn black_king_count(&self) -> u64 {
        self.black_king.count_ones() as u64
    }

    /// Returns the total value of the white pieces on the board
    /// The value of a piece is as follows:
    /// - Pawn: 1
    /// - Knight: 3
    /// - Bishop: 3
    /// - Rook: 5
    /// - Queen: 9
    /// - King: 1000
    #[inline]
    pub fn total_white_piece_value(&self) -> u64 {
        (self.white_pawn_count() as f64 * PieceType::Pawn.get_value()
            + self.white_knight_count() as f64 * PieceType::Knight.get_value()
            + self.white_bishop_count() as f64 * PieceType::Bishop.get_value()
            + self.white_rook_count() as f64 * PieceType::Rook.get_value()
            + self.white_queen_count() as f64 * PieceType::Queen.get_value()
            + self.white_king_count() as f64 * PieceType::King.get_value()) as u64
    }

    /// Returns the total value of the black pieces on the board
    /// The value of a piece is as follows:
    /// - Pawn: 1
    /// - Knight: 3
    /// - Bishop: 3
    /// - Rook: 5
    /// - Queen: 9
    /// - King: 100
    #[inline]
    pub fn total_black_piece_value(&self) -> u64 {
        (self.black_pawn_count() as f64 * PieceType::Pawn.get_value()
            + self.black_knight_count() as f64 * PieceType::Knight.get_value()
            + self.black_bishop_count() as f64 * PieceType::Bishop.get_value()
            + self.black_rook_count() as f64 * PieceType::Rook.get_value()
            + self.black_queen_count() as f64 * PieceType::Queen.get_value()
            + self.black_king_count() as f64 * PieceType::King.get_value()) as u64
    }

    #[inline]
    pub fn has_white_piece_on(&self, location: Tile) -> bool {
        let bit = location.to_bit();
        (self.white_pawns & bit) != 0
            || (self.white_knights & bit) != 0
            || (self.white_bishops & bit) != 0
            || (self.white_rooks & bit) != 0
            || (self.white_queens & bit) != 0
            || (self.white_king & bit) != 0
    }

    #[inline]
    pub fn has_black_piece_on(&self, location: Tile) -> bool {
        let bit = location.to_bit();
        (self.black_pawns & bit) != 0
            || (self.black_knights & bit) != 0
            || (self.black_bishops & bit) != 0
            || (self.black_rooks & bit) != 0
            || (self.black_queens & bit) != 0
            || (self.black_king & bit) != 0
    }

    #[inline]
    pub fn next_white_piece(&self, last: Tile) -> Tile {
        // Perform a bit scan on the bit-board to find the next white piece
        let mut white_pieces = self.white_pieces_as_bits();
        white_pieces &= !(1 << (last.get_rank().get_index() * 8 + last.get_file().get_index()) - 1);
        Tile::from_bit(white_pieces)
    }

    #[inline]
    pub fn next_black_piece(&self, last: Tile) -> Tile {
        // Perform a bit scan on the bit-board to find the next black piece
        let mut black_pieces = self.black_pieces_as_bits();
        black_pieces &= !(1 << (last.get_rank().get_index() * 8 + last.get_file().get_index()) - 1);
        Tile::from_bit(black_pieces)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Magenta
        let primary_color = "\x1b[0;45m";
        // Cyan
        let secondary_color = "\x1b[0;46m";
        // Red
        let alt_primary_color = "\x1b[0;41m";
        // Blue
        let alt_secondary_color = "\x1b[0;44m";
        write!(f, " ")?;
        for file in 0..8 {
            write!(f, " {}", File::from_index(file))?;
        }
        write!(f, "\n")?;
        for rank in (0..8).rev() {
            // Store the pieces in the rank in the result.
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                // is en-passant square?
                let tile = Tile::new(Rank::from_index(rank), File::from_index(file));

                let (primary, secondary) = if self.controls_sector(tile.get_sector(), !self.current_turn) {
                    (alt_primary_color, alt_secondary_color)
                } else {
                    (primary_color, secondary_color)
                };

                // Color the square with ansi code
                if (rank + file) % 2 == 0 {
                    // White square (magenta background)
                    write!(f, "{primary}")?;
                } else {
                    // Black square (cyan background)
                    write!(f, "{secondary}")?;
                }
                // Foreground color (black)
                write!(f, "\x1b[30m")?;

                let location = Tile::new(Rank::from_index(rank), File::from_index(file));

                write!(f, "{}", match self.get_piece(location) {
                    Some(piece) => piece.into(),
                    None => ' ',
                })?;

                // let sector = location.get_sector();
                // Push as hex character
                // if sector.is_center() {
                //     write!(f, "!")?;
                // } else {
                //     write!(f, ".")?;
                // }
                write!(f, " ")?;

                // Reset the color
                write!(f, "\x1b[0m")?;
            }
            write!(f, " {}\n", rank + 1)?;
        }
        write!(f, " ")?;
        for file in 0..8 {
            write!(f, " {}", File::from_index(file))?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

// pub(crate) fn display_bitboard(bitboard: u64) -> String {
//     let mut result = String::new();
//     for rank in (0..8).rev() {
//         // Store the pieces in the rank in the result.
//         for file in 0..8 {
//             let location = Tile::new(Rank::from_index(rank), File::from_index(file));
//             if (bitboard & location.to_bit()) != 0 {
//                 result.push('1');
//             } else {
//                 result.push('0');
//             }
//         }
//         result.push('\n');
//     }
//     result
// }


/// A struct that represents the castling rights of a board
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CastlingRights {
    white_king_side: bool,
    white_queen_side: bool,
    black_king_side: bool,
    black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

impl CastlingRights {
    /// Returns a new castling rights struct with no castling rights
    pub fn none() -> Self {
        Self {
            white_king_side: false,
            white_queen_side: false,
            black_king_side: false,
            black_queen_side: false,
        }
    }

    /// Sets the given color to not be able to castle on the given side
    pub fn disable_castling(&mut self, king: Tile, rook: Tile) {
        if !self.is_castling_move(king, rook) {
            return;
        }
        let color = king.get_player_side();
        let side = rook.get_castling_side();
        self.disable_castling_color_and_side(color, side);
    }

    /// Returns true if the given color can castle on the king side
    pub fn can_castle(&self, king: Tile, rook: Tile) -> bool {
        if !self.is_castling_move(king, rook) {
            return false;
        }
        let color = king.get_player_side();
        let side = rook.get_castling_side();
        self.can_castle_color_and_side(color, side)
    }

    /// Disable castling for the given color
    pub fn disable_castling_color(&mut self, color: Color) {
        match color {
            Color::White => {
                self.white_king_side = false;
                self.white_queen_side = false;
            },
            Color::Black => {
                self.black_king_side = false;
                self.black_queen_side = false;
            },
        }
    }

    /// Is this a castling move?
    #[inline]
    fn is_castling_move(&self, king: Tile, rook: Tile) -> bool {
        let color = king.get_player_side();
        king == Tile::king_start_position(color)
            && (rook.is_castling_destination_for_king(color)
            || rook.is_rook_square(color)) && self.can_castle_color_and_side(color, rook.get_castling_side())
    }

    /// Remove the given color's ability to castle on the given side
    fn disable_castling_color_and_side(&mut self, color: Color, side: CastlingSide) {
        match color {
            Color::White => {
                match side {
                    CastlingSide::King => self.white_king_side = false,
                    CastlingSide::Queen => self.white_queen_side = false,
                }
            },
            Color::Black => {
                match side {
                    CastlingSide::King => self.black_king_side = false,
                    CastlingSide::Queen => self.black_queen_side = false,
                }
            },
        }
    }

    /// Returns true if the given color can castle on the given side
    fn can_castle_color_and_side(&self, color: Color, side: CastlingSide) -> bool {
        match color {
            Color::White => {
                match side {
                    CastlingSide::King => self.white_king_side,
                    CastlingSide::Queen => self.white_queen_side,
                }
            },
            Color::Black => {
                match side {
                    CastlingSide::King => self.black_king_side,
                    CastlingSide::Queen => self.black_queen_side,
                }
            },
        }
    }
}