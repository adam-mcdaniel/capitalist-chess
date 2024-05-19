// #![no_std]
extern crate alloc;

mod board;
pub use board::*;

mod economy;
pub use economy::*;

mod turn;
pub use turn::*;

mod engine;
pub use engine::*;

use core::{str::FromStr, fmt::{Display, Debug, Formatter, Result as FmtResult}, ops::{Add, Sub, Not}};
use alloc::{boxed::Box, vec};

/// Indicates whether we should insert sanity checks into
/// all the board operations.
pub const INSERT_SANITY_CHECKS: bool = cfg!(debug_assertions);

/// The type of a piece.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    /// All the piece types.
    pub const ALL: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Possible promotions for a pawn.
    pub const PROMOTIONS: [Self; 4] = [
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
    ];

    pub const PURCHASES: [Self; 6] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Get the base value of the piece type.
    pub const fn get_value(&self) -> f64 {
        match self {
            Self::Pawn => 1.0,
            Self::Knight => 3.0,
            Self::Bishop => 3.15,
            Self::Rook => 5.0,
            Self::Queen => 9.0,
            Self::King => 100.0,
        }
    }
}

impl Display for PieceType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", char::from(*self))
    }
}

impl From<PieceType> for char {
    fn from(piece_type: PieceType) -> char {
        match piece_type {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }
}

impl FromStr for PieceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(Self::Pawn),
            "N" => Ok(Self::Knight),
            "B" => Ok(Self::Bishop),
            "R" => Ok(Self::Rook),
            "Q" => Ok(Self::Queen),
            "K" => Ok(Self::King),
            _ => Err(()),
        }
    }
}

/// A color is either white or black.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Color {
    #[default]
    White,
    Black,
}

impl Color {
    /// Get the opposite color.
    #[inline]
    pub fn enemy(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.enemy()
    }
}

/// A castling side is either the king side or the queen side.
#[derive(Copy, Clone, PartialEq)]
pub enum CastlingSide {
    King,
    Queen,
}

impl FromStr for CastlingSide {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "O-O" => Ok(CastlingSide::King),
            "O-O-O" => Ok(CastlingSide::Queen),
            _ => Err(()),
        }
    }
}

impl Display for CastlingSide {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            CastlingSide::King => write!(f, "O-O"),
            CastlingSide::Queen => write!(f, "O-O-O"),
        }
    }
}

impl Debug for CastlingSide {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            CastlingSide::King => write!(f, "kingside"),
            CastlingSide::Queen => write!(f, "queenside"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Piece(PieceType, Color);

impl Piece {
    /// Create a new piece with the given type and color.
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self(piece_type, color)
    }

    /// Get the type of the piece.
    pub fn get_type(&self) -> PieceType {
        self.0
    }

    /// Get the color of the piece.
    pub fn get_color(&self) -> Color {
        self.1
    }

    /// Create a pawn with the given color.
    pub fn pawn(color: Color) -> Self {
        Self::new(PieceType::Pawn, color)
    }

    /// Create a knight with the given color.
    pub fn knight(color: Color) -> Self {
        Self::new(PieceType::Knight, color)
    }

    /// Create a bishop with the given color.
    pub fn bishop(color: Color) -> Self {
        Self::new(PieceType::Bishop, color)
    }

    /// Create a rook with the given color.
    pub fn rook(color: Color) -> Self {
        Self::new(PieceType::Rook, color)
    }

    /// Create a queen with the given color.
    pub fn queen(color: Color) -> Self {
        Self::new(PieceType::Queen, color)
    }

    /// Create a king with the given color.
    pub fn king(color: Color) -> Self {
        Self::new(PieceType::King, color)
    }

    /// Get the value of the piece.
    pub fn get_value(&self) -> f64 {
        self.0.get_value()
    }

    /// Can this piece type move from one tile to another?
    pub fn can_move(&self, from: Tile, to: Tile, is_attack: bool, en_passant_tile: Option<Tile>) -> bool {
        match self.get_type() {
            PieceType::Pawn => from.is_pawn_move_away(to, self.get_color(), is_attack, en_passant_tile),
            PieceType::Knight => from.is_knight_move_away(to),
            PieceType::Bishop => from.is_bishop_move_away(to),
            PieceType::Rook => from.is_rook_move_away(to),
            PieceType::Queen => from.is_queen_move_away(to),
            PieceType::King => from.is_king_move_away(to),
        }
    }
}

impl From<Piece> for char {
    fn from(piece: Piece) -> Self {
        match piece {
            Piece(PieceType::Pawn, Color::White) => '♙',
            Piece(PieceType::Knight, Color::White) => '♘',
            Piece(PieceType::Bishop, Color::White) => '♗',
            Piece(PieceType::Rook, Color::White) => '♖',
            Piece(PieceType::Queen, Color::White) => '♕',
            Piece(PieceType::King, Color::White) => '♔',
            Piece(PieceType::Pawn, Color::Black) => '♟',
            Piece(PieceType::Knight, Color::Black) => '♞',
            Piece(PieceType::Bishop, Color::Black) => '♝',
            Piece(PieceType::Rook, Color::Black) => '♜',
            Piece(PieceType::Queen, Color::Black) => '♛',
            Piece(PieceType::King, Color::Black) => '♚'
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", char::from(*self))
    }
}

/// A chessboard is a 8x8 grid of squares.
/// The rank is the horizontal row of squares, numbered 0 to 7 from the bottom up.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Rank(u8);

impl Rank {
    /// The 0th rank is the bottom rank.
    pub const BOTTOM: Self = Self(0);
    /// The 7th rank is the top rank.
    pub const TOP: Self = Self(7);

    /// The minimum white rank.
    pub const MIN_WHITE: Self = Self(0);
    /// The maximum white rank.
    pub const MAX_WHITE: Self = Self(3);
    /// The minimum black rank.
    pub const MIN_BLACK: Self = Self(4);
    /// The maximum black rank.
    pub const MAX_BLACK: Self = Self(7);

    /// White back rank.
    /// The white back rank is the rank that white pieces start on.
    pub const BACK_RANK_WHITE: Self = Self(0);
    /// Black back rank.
    /// The black back rank is the rank that black pieces start on.
    pub const BACK_RANK_BLACK: Self = Self(7);

    /// White pawn starter rank.
    /// The white pawn starter rank is the rank that white pawns start on.
    pub const PAWN_STARTER_WHITE: Self = Self(1);
    /// Black pawn starter rank.
    /// The black pawn starter rank is the rank that black pawns start on.
    pub const PAWN_STARTER_BLACK: Self = Self(6);

    /// Direction of white pawns.
    pub const WHITE_DIRECTION: i8 = 1;
    /// Direction of black pawns.
    pub const BLACK_DIRECTION: i8 = -1;

    /// Advance the rank according to the pawn color
    #[inline]
    pub fn advance(&self, color: Color, count: i8) -> Self {
        let result =if color == Color::White {
            self.0 as i8 + Self::WHITE_DIRECTION * count
        } else {
            self.0 as i8 + Self::BLACK_DIRECTION * count
        };
        if result < 0 || result > 7 {
            panic!("Cannot advance rank {} by {}", self, count);
        }
        Self::from_index(result as u8)
    }

    /// Get the index of the rank.
    /// The index is the number of the rank, from 0 to 7.
    #[inline]
    fn get_index(&self) -> u8 {
        self.0
    }

    /// Is this rank within N ranks of the other rank?
    #[inline]
    pub fn is_within(&self, other: Self, n: u8) -> bool {
        (self.0 as i8 - other.0 as i8).abs() <= n as i8
    }

    /// Get the player side of the rank.
    /// The player side is the side of the board that the player is on.
    #[inline]
    pub fn get_player_side(&self) -> Color {
        if self.0 <= 3 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Create a new rank from a character.
    /// The character must be a digit from 1 to 8.
    #[inline]
    pub const fn from_char(c: char) -> Self {
        assert!(c >= '1' && c <= '8');
        Self(c as u8 - b'1')
    }

    /// Create a new rank from a number.
    /// The number must be from 0 to 7.
    #[inline]
    const fn from_index(n: u8) -> Self {
        assert!(n < 8);
        Self(n)
    }
}

impl Add<i8> for Rank {
    type Output = Self;

    fn add(self, rhs: i8) -> Self::Output {
        Self::from_index((self.0 as i8 + rhs) as u8)
    }
}

impl Add for Rank {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_index(self.0 + rhs.0)
    }
}

impl Sub<i8> for Rank {
    type Output = Self;

    fn sub(self, rhs: i8) -> Self::Output {
        Self::from_index((self.0 as i8 - rhs) as u8)
    }
}

impl Sub for Rank {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_index(self.0 - rhs.0)
    }
}

impl PartialEq<File> for Rank {
    fn eq(&self, other: &File) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<u8> for Rank {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0 + 1)
    }
}

/// A chessboard is a 8x8 grid of squares.
/// The file is the vertical column of squares, numbered 0 to 7 from the left.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct File(u8);

impl File {
    /// The 0th file is the left file.
    pub const LEFTMOST: Self = Self(0);
    /// The 7th file is the right file.
    pub const RIGHTMOST: Self = Self(7);

    /// The king file is the file that the king is on.
    pub const KING: Self = Self(4);
    /// The queen file is the file that the queen is on.
    pub const QUEEN: Self = Self(3);

    /// The king side castle destination is the file that the king moves to when castling kingside.
    const KINGSIDE_CASTLE_DESTINATION: Self = Self(6);
    /// The queen side castle destination is the file that the king moves to when castling queenside.
    const QUEENSIDE_CASTLE_DESTINATION: Self = Self(2);

    /// The white king side rook file is the file that the white king side rook is on.
    pub const KINGSIDE_ROOK: Self = Self(7);
    /// The white queen side rook file is the file that the white queen side rook is on.
    pub const QUEENSIDE_ROOK: Self = Self(0);

    pub const A: Self = Self(0);
    pub const B: Self = Self(1);
    pub const C: Self = Self(2);
    pub const D: Self = Self(3);
    pub const E: Self = Self(4);
    pub const F: Self = Self(5);
    pub const G: Self = Self(6);
    pub const H: Self = Self(7);

    /// Get the index of the file.
    /// The index is the number of the rank, from 0 to 7.
    #[inline]
    fn get_index(&self) -> u8 {
        self.0
    }

    /// Is this file within N files of the other file?
    #[inline]
    pub fn is_within(&self, other: Self, n: u8) -> bool {
        (self.0 as i8 - other.0 as i8).abs() <= n as i8
    }

    /// Get the direction of castling, given that this is the file of the rook
    #[inline]
    pub fn get_castling_side(&self) -> CastlingSide {
        if self.0 < 4 {
            CastlingSide::Queen
        } else {
            CastlingSide::King
        }
    }

    /// Create a new file from a character.
    /// The character must be a lowercase letter from a to h.
    #[inline]
    pub const fn from_char(mut c: char) -> Self {
        c = c.to_ascii_lowercase();
        assert!(c >= 'a' && c <= 'h');
        Self(c as u8 - b'a')
    }

    /// Create a new file from a number.
    /// The number must be from 0 to 7.
    #[inline]
    const fn from_index(n: u8) -> Self {
        assert!(n < 8);
        Self(n)
    }
}

impl Add<i8> for File {
    type Output = Self;

    fn add(self, rhs: i8) -> Self::Output {
        Self::from_index((self.0 as i8 + rhs) as u8)
    }
}

impl Add for File {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_index(self.0 + rhs.0)
    }
}

impl Sub<i8> for File {
    type Output = Self;

    fn sub(self, rhs: i8) -> Self::Output {
        Self::from_index((self.0 as i8 - rhs) as u8)
    }
}

impl Sub for File {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_index(self.0 - rhs.0)
    }
}

impl PartialEq<Rank> for File {
    fn eq(&self, other: &Rank) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<u8> for File {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", (self.0 + b'a') as char)
    }
}

/// A sector is a 2x2 square of squares.
/// The board is divided into 16 sectors, numbered 0 to 15 from the bottom left to the top right.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sector(u8);

impl Sector {
    /// The bottom left sector is the 0th sector.
    pub const BOTTOM_LEFT: Self = Self(0);
    /// The bottom right sector is the 3rd sector.
    pub const BOTTOM_RIGHT: Self = Self(0x3);
    /// The top left sector is the 12th sector.
    pub const TOP_LEFT: Self = Self(0xC);
    /// The top right sector is the 15th sector.
    pub const TOP_RIGHT: Self = Self(0xF);

    /// This is the largest number that a sector can be.
    pub const MAX_SECTOR_NUMBER: u8 = 0xF;
    /// This is the smallest number that a sector can be.
    pub const MIN_SECTOR_NUMBER: u8 = 0x0;

    /// The number of sectors
    pub const NUM_SECTORS: usize = 16;

    /// Get a sector number from its index.
    /// The index is the number of the sector, from 0 to 15.
    #[inline]
    pub fn from_index(index: usize) -> Self {
        let index = index as u8;
        assert!(Self::MIN_SECTOR_NUMBER <= index && index <= Self::MAX_SECTOR_NUMBER);
        Self(index)
    }

    /// Get the index of the sector.
    #[inline]
    pub fn get_index(&self) -> usize {
        self.0 as usize
    }

    /// Get a new sector from a tile on the board
    #[inline]
    pub fn new(tile: Tile) -> Self {
        tile.get_sector()
    }

    /// Is this sector one of the 4 center sectors?
    /// The center sectors are the 4 sectors in the center of the board.
    #[inline]
    pub fn is_center(&self) -> bool {
        self.0 == 5 || self.0 == 6 || self.0 == 9 || self.0 == 10
    }

    /// Is this sector one of the 12 outer sectors?
    /// The outer sectors are the 12 sectors on the outside of the board.
    #[inline]
    pub fn is_outer(&self) -> bool {
        !self.is_center()
    }

    /// Get an iterator over all the sectors.
    #[inline]
    pub fn all() -> impl Iterator<Item = Self> {
        (0..16).map(Self::from_index)
    }

    /// Is this sector one of the home sectors for the given color?
    pub fn is_home_for(&self, color: Color) -> bool {
        match color {
            Color::White => self.0 <= 3,
            Color::Black => self.0 >= 12,
        }
    }
}

impl Display for Sector {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<Tile> for Sector {
    fn from(location: Tile) -> Self {
        Self::new(location)
    }
}


/// A tile set is a set of tiles.
/// 
/// This is used to do bitwise operations on tiles.
pub struct TileSet(u64);

impl TileSet {
    pub fn insert(&mut self, tile: Tile) {
        self.0 |= tile.to_bit();
    }

    pub fn remove(&mut self, tile: Tile) {
        self.0 &= !tile.to_bit();
    }

    pub fn contains(&self, tile: Tile) -> bool {
        self.0 & tile.to_bit() != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn into_iter(self) -> impl Iterator<Item = Tile> {
        (0..64).filter_map(move |i| {
            if self.0 & (1 << i) != 0 {
                Some(Tile::from_nth(i as u8))
            } else {
                None
            }
        })
    }
}


/// A location is a square on the chessboard.
/// The location is represented by a rank and a file.
/// The rank is the horizontal row of squares, numbered 0 to 7 from the bottom up.
/// The file is the vertical column of squares, numbered 0 to 7 from the left.
#[derive(Copy, Clone, PartialEq)]
pub struct Tile(Rank, File);

impl Tile {
    /// The tile of the white queenside rook.
    pub const WHITE_QUEENSIDE_ROOK_START: Self = Self(Rank::BACK_RANK_WHITE, File::LEFTMOST);
    /// The tile of the white kingside rook.
    pub const WHITE_KINGSIDE_ROOK_START: Self = Self(Rank::BACK_RANK_WHITE, File::RIGHTMOST);
    /// The tile of the black queenside rook.
    pub const BLACK_QUEENSIDE_ROOK_START: Self = Self(Rank::BACK_RANK_BLACK, File::LEFTMOST);
    /// The tile of the black kingside rook.
    pub const BLACK_KINGSIDE_ROOK_START: Self = Self(Rank::BACK_RANK_BLACK, File::RIGHTMOST);

    /// The starting square of white king
    pub const WHITE_KING_START: Self = Self(Rank::BACK_RANK_WHITE, File::KING);
    /// The starting square of black king
    pub const BLACK_KING_START: Self = Self(Rank::BACK_RANK_BLACK, File::KING);

    /// The destination of the white king when castling kingside
    const WHITE_KINGSIDE_CASTLE_DESTINATION: Self = Self(Rank::BACK_RANK_WHITE, File::KINGSIDE_CASTLE_DESTINATION);
    /// The destination of the white king when castling queenside
    const WHITE_QUEENSIDE_CASTLE_DESTINATION: Self = Self(Rank::BACK_RANK_WHITE, File::QUEENSIDE_CASTLE_DESTINATION);
    /// The destination of the black king when castling kingside
    const BLACK_KINGSIDE_CASTLE_DESTINATION: Self = Self(Rank::BACK_RANK_BLACK, File::KINGSIDE_CASTLE_DESTINATION);
    /// The destination of the black king when castling queenside
    const BLACK_QUEENSIDE_CASTLE_DESTINATION: Self = Self(Rank::BACK_RANK_BLACK, File::QUEENSIDE_CASTLE_DESTINATION);
    
    #[inline]
    pub fn king_start_position(color: Color) -> Self {
        match color {
            Color::White => Self::WHITE_KING_START,
            Color::Black => Self::BLACK_KING_START,
        }
    }

    #[inline]
    pub fn rook_start_position(color: Color, side: CastlingSide) -> Self {
        match color {
            Color::White => match side {
                CastlingSide::King => Self::WHITE_KINGSIDE_ROOK_START,
                CastlingSide::Queen => Self::WHITE_QUEENSIDE_ROOK_START,
            },
            Color::Black => match side {
                CastlingSide::King => Self::BLACK_KINGSIDE_ROOK_START,
                CastlingSide::Queen => Self::BLACK_QUEENSIDE_ROOK_START,
            },
        }
    }

    /// Step this tile diagonally or orthogonally towards the target tile.
    #[inline]
    pub fn step_towards(&mut self, target: Tile) {
        let my_rank = self.get_rank();
        let my_file = self.get_file();
        let target_rank = target.get_rank();
        let target_file = target.get_file();

        self.0 = self.0 + if my_rank < target_rank {
            1
        } else if my_rank > target_rank {
            -1
        } else {
            0
        };

        self.1 = self.1 + if my_file < target_file {
            1
        } else if my_file > target_file {
            -1
        } else {
            0
        };
    }

    /// Get an iterator over all the tiles.
    #[inline]
    pub fn all() -> impl Iterator<Item = Self> {
        (0..64).map(Self::from_nth)
    }

    /// Advance the tile a pawn's move for a given color and count of tiles.
    #[inline]
    pub fn advance(&self, color: Color, count: i8) -> Self {
        Self::new(self.get_rank().advance(color, count), self.get_file())
    }

    /// Move the tile by a rank and a file.
    #[inline]
    pub fn move_by<A, B>(&self, rank: A, file: B) -> Option<Self> where A: Into<i8>, B: Into<i8> {
        let rank = rank.into();
        let file = file.into();
        let new_rank = self.get_rank().get_index() as i8 + rank;
        let new_file = self.get_file().get_index() as i8 + file;
        if new_rank < 8 && new_file < 8 && new_rank >= 0 && new_file >= 0 {
            Some(Self::new(Rank::from_index(new_rank as u8), File::from_index(new_file as u8)))
        } else {
            None
        }
    }

    /// Returns the square where the king ends up after castling.
    #[inline]
    pub fn castling_destination_for_king(color: Color, side: CastlingSide) -> Self {
        match color {
            Color::White => match side {
                CastlingSide::King => Self::WHITE_KING_START.move_by(0, 2).unwrap(),
                CastlingSide::Queen => Self::WHITE_KING_START.move_by(0, -2).unwrap(),
            },
            Color::Black => match side {
                CastlingSide::King => Self::BLACK_KING_START.move_by(0, 2).unwrap(),
                CastlingSide::Queen => Self::BLACK_KING_START.move_by(0, -2).unwrap(),
            },
        }
    }

    /// Returns the square where the rook ends up after castling.
    #[inline]
    pub fn castling_destination_for_rook(color: Color, side: CastlingSide) -> Self {
        Self::castling_destination_for_king(color, side).move_by(0, match side {
            CastlingSide::King => -1,
            CastlingSide::Queen => 1,
        }).unwrap()
    }

    #[inline]
    pub fn from_bit(bit: u64) -> Self {
        let index = bit.trailing_zeros() as u8;
        Self::from_nth(index)
    }

    #[inline]
    pub fn from_nth(n: u8) -> Self {
        Self::new(Rank::from_index(n / 8), File::from_index(n % 8))
    }

    /// Create a new location from a rank and a file.
    /// The rank is the horizontal row of squares, numbered 0 to 7 from the bottom up.
    /// The file is the vertical column of squares, numbered 0 to 7 from the left.
    #[inline]
    pub fn new(rank: Rank, file: File) -> Self {
        Self(rank, file)
    }

    /// The en passant square is the square that a pawn can move to when it performs an en passant capture.
    /// This function takes that tile, and returns the tile that the attacked pawn is on.
    #[inline]
    pub fn pawn_from_en_passant_square(en_passant_square: Self) -> Self {
        // The attacked pawn is on the rank above the en passant square.
        // On white's side, the attacked pawn is on the rank below the en passant square.
        let rank = if en_passant_square.get_player_side() == Color::White {
            en_passant_square.get_rank() - 1
        } else {
            en_passant_square.get_rank() + 1
        };

        // The attacked pawn is on the same file as the en passant square.
        let file = en_passant_square.get_file();

        Self::new(rank, file)
    }

    /// Get the rank of the location.
    #[inline]
    pub fn get_rank(&self) -> Rank {
        self.0
    }

    /// Get the file of the location.
    #[inline]
    pub fn get_file(&self) -> File {
        self.1
    }

    /// Get the player side of the location.
    #[inline]
    pub fn get_player_side(&self) -> Color {
        self.get_rank().get_player_side()
    }

    /// Get the castling side of the location.
    /// The castling side is the side of the board that the rook is on.
    /// The king side is the right side of the board, and the queen side is the left side of the board.
    #[inline]
    pub fn get_castling_side(&self) -> CastlingSide {
        self.get_file().get_castling_side()
    }
    
    /// Get the sector of the location.
    /// The sector is the 2x2 square of squares that the location is in.
    /// The board is divided into 16 sectors, numbered 0 to 15 from the bottom left.
    /// The 0th sector is the bottom left 2x2 square of squares, and the 15th sector
    /// is the top right 2x2 square of squares.
    #[inline]
    fn get_sector_number(&self) -> u8 {
        (self.get_rank().get_index() / 2) * 4 + (self.get_file().get_index() / 2)
    }

    /// Get the sector of the location.
    /// The sector is the 2x2 square of squares that the location is in.
    /// The board is divided into 16 sectors, numbered 0 to 15 from the bottom left.
    /// The 0th sector is the bottom left 2x2 square of squares, and the 15th sector
    /// is the top right 2x2 square of squares.
    #[inline]
    pub fn get_sector(&self) -> Sector {
        Sector(self.get_sector_number())
    }

    /// Convert this location to a bit representation on a bitboard.
    #[inline]
    pub(crate) fn to_bit(&self) -> u64 {
        1 << (self.get_rank().get_index() * 8 + self.get_file().get_index())
    }

    /// Is this tile a knight move away from the other tile?
    #[inline]
    pub fn is_knight_move_away(&self, other: Tile) -> bool {
        let my_rank = self.get_rank();
        let my_file = self.get_file();
        let other_rank = other.get_rank();
        let other_file = other.get_file();

        my_rank.is_within(other_rank, 1) && my_file.is_within(other_file, 2)
            || my_rank.is_within(other_rank, 2) && my_file.is_within(other_file, 1)
    }

    /// Is diagonal to the other tile?
    #[inline]
    pub fn is_diagonal_to(&self, other: Tile) -> bool {
        let my_rank = self.get_rank().get_index() as i32;
        let my_file = self.get_file().get_index() as i32;
        let other_rank = other.get_rank().get_index() as i32;
        let other_file = other.get_file().get_index() as i32;
        (my_rank - other_rank).abs() == (my_file - other_file).abs()
    }

    /// Is this tile a king move away from the other tile?
    #[inline]
    pub fn is_king_move_away(&self, other: Tile) -> bool {
        let my_rank = self.get_rank();
        let my_file = self.get_file();
        let other_rank = other.get_rank();
        let other_file = other.get_file();

        my_rank.is_within(other_rank, 1) && my_file.is_within(other_file, 1)
    }

    /// Is this tile a rook move away from the other tile?
    #[inline]
    pub fn is_rook_move_away(&self, other: Tile) -> bool {
        self.get_rank() == other.get_rank() || self.get_file() == other.get_file()
    }

    /// Is this tile a bishop move away from the other tile?
    #[inline]
    pub fn is_bishop_move_away(&self, other: Tile) -> bool {
        self.is_diagonal_to(other)
    }

    /// Is this tile a queen move away from the other tile?
    /// A queen move is either a rook move or a bishop move.
    /// A rook move is a move along a rank or a file.
    /// A bishop move is a move along a diagonal.
    #[inline]
    pub fn is_queen_move_away(&self, other: Tile) -> bool {
        self.is_rook_move_away(other) || self.is_bishop_move_away(other)
    }

    /// Is this tile a pawn move away from the other tile?
    #[inline]
    pub fn is_pawn_move_away(&self, other: Tile, color: Color, is_attack: bool, en_passant_square: Option<Tile>) -> bool {
        let my_rank = self.get_rank();
        let my_file = self.get_file();
        let other_rank = other.get_rank();
        let other_file = other.get_file();

        if is_attack {
            // Is one rank ahead, and within one file away.
            return my_rank.advance(color, 1) == other_rank && my_file.is_within(other_file, 1) && my_file != other_file;
        } else if let Some(en_passant) = en_passant_square {
            // Is one rank ahead, and within one file away.
            if en_passant == other && my_rank.advance(color, 1) == other_rank && my_file.is_within(other_file, 1) && my_file != other_file {
                return true;
            }
        }

        // Check if on start rank and is two ranks away.
        if color == Color::White && my_rank == Rank::PAWN_STARTER_WHITE && my_rank.advance(color, 2) == other_rank && my_file == other_file {
            return true;
        }

        // Check if on start rank and is two ranks away.
        if color == Color::Black && my_rank == Rank::PAWN_STARTER_BLACK && my_rank.advance(color, 2) == other_rank && my_file == other_file {
            return true;
        }

        // Is one rank ahead, and within one file away.
        my_rank.advance(color, 1) == other_rank && my_file == other_file
    }

    /// Is this tile a castling rook?
    /// A castling rook is the rook that the king castles with.
    #[inline]
    pub fn is_rook_square(&self, color: Color) -> bool {
        ((color == Color::White && self.get_rank() == Rank::BACK_RANK_WHITE)
            || (color == Color::Black && self.get_rank() == Rank::BACK_RANK_BLACK))
            && (self.get_file() == File::KINGSIDE_ROOK || self.get_file() == File::QUEENSIDE_ROOK)
    }

    /// Is this tile a castling destination?
    /// A castling destination is the tile that the king moves to when castling.
    #[inline]
    pub fn is_castling_destination_for_king(&self, color: Color) -> bool {
        match color {
            Color::White => self == &Self::WHITE_KINGSIDE_CASTLE_DESTINATION
                || self == &Self::WHITE_QUEENSIDE_CASTLE_DESTINATION,
            Color::Black => self == &Self::BLACK_KINGSIDE_CASTLE_DESTINATION
                || self == &Self::BLACK_QUEENSIDE_CASTLE_DESTINATION,
        }
    }

    /// The attacking bits for this tile.
    #[inline]
    pub fn attacking_bits(&self, piece_type: PieceType, color: Color) -> u64 {
        match piece_type {
            PieceType::Pawn => self.pawn_attacking_bits(color),
            PieceType::Knight => self.knight_attacking_bits(),
            PieceType::Bishop => self.bishop_attacking_bits(),
            PieceType::Rook => self.rook_attacking_bits(),
            PieceType::Queen => self.queen_attacking_bits(),
            PieceType::King => self.king_attacking_bits(),
        }
    }

    /// The attacking bits for a pawn on this tile.
    pub fn pawn_attacking_bits(&self, color: Color) -> u64 {
        let mut bits = 0;
        let rank = self.get_rank().advance(color, 1).get_index();
        let file = self.get_file().get_index();
        if self.get_file() > File::LEFTMOST {
            bits |= 1 << (rank * 8 + file - 1);
        }
        if self.get_file() < File::RIGHTMOST {
            bits |= 1 << (rank * 8 + file + 1);
        }
        bits
    }

    /// The attacking bits for a knight on this tile.
    pub fn knight_attacking_bits(&self) -> u64 {
        let mut bits = 0;
        let rank = self.get_rank();
        let file = self.get_file();

        if rank > Rank::BOTTOM + 1 && file > File::LEFTMOST {
            bits |= Tile::new(rank - 2, file - 1).to_bit();
        }
        if rank > Rank::BOTTOM + 1 && file < File::RIGHTMOST {
            bits |= Tile::new(rank - 2, file + 1).to_bit();
        }
        if rank > Rank::BOTTOM && file > File::LEFTMOST + 1 {
            bits |= Tile::new(rank - 1, file - 2).to_bit();
        }
        if rank > Rank::BOTTOM && file < File::RIGHTMOST - 1 {
            bits |= Tile::new(rank - 1, file + 2).to_bit();
        }
        if rank < Rank::TOP && file > File::LEFTMOST + 1 {
            bits |= Tile::new(rank + 1, file - 2).to_bit();
        }
        if rank < Rank::TOP && file < File::RIGHTMOST - 1 {
            bits |= Tile::new(rank + 1, file + 2).to_bit();
        }
        if rank < Rank::TOP - 1 && file > File::LEFTMOST {
            bits |= Tile::new(rank + 2, file - 1).to_bit();
        }
        if rank < Rank::TOP - 1 && file < File::RIGHTMOST {
            bits |= Tile::new(rank + 2, file + 1).to_bit();
        }

        bits
    }

    /// All the tiles along the diagonals from this tile.
    fn diagonal_tiles(self) -> impl Iterator<Item = Tile> {
        Tile::all().filter(move |tile| self.is_diagonal_to(*tile))
    }

    /// All the tiles on the same rank as this tile.
    fn rank_tiles(&self) -> impl Iterator<Item = Tile> {
        let file = self.get_file().get_index();

        (Rank::BOTTOM.get_index()..=Rank::TOP.get_index())
            .map(move |r| Tile::new(Rank::from_index(r), File::from_index(file)))
    }

    /// All the tiles on the same file as this tile.
    fn file_tiles(&self) -> impl Iterator<Item = Tile> {
        let rank = self.get_rank().get_index();

        (File::LEFTMOST.get_index()..=File::RIGHTMOST.get_index())
            .map(move |f| Tile::new(Rank::from_index(rank), File::from_index(f)))
    }

    /// The possible tiles a piece might try to move to from this tile.
    fn get_moves(&self, piece: Piece) -> Box<dyn Iterator<Item = Tile>> {
        match piece.get_type() {
            PieceType::Pawn => Box::new(self.pawn_moves(piece.get_color())),
            PieceType::Knight => Box::new(self.knight_moves()),
            PieceType::Bishop => Box::new(self.bishop_moves()),
            PieceType::Rook => Box::new(self.rook_moves()),
            PieceType::Queen => Box::new(self.queen_moves()),
            PieceType::King => Box::new(self.king_moves()),
        }
    }

    /// The possible tiles a pawn might try to move to from this tile.
    fn pawn_moves(&self, color: Color) -> impl Iterator<Item = Tile> {
        // Return the two tiles diagonally in front of the pawn,
        // the tile directly in front of the pawn, and maybe
        // the tile two ranks in front of the pawn.
        let direction = match color {
            Color::White => Rank::WHITE_DIRECTION,
            Color::Black => Rank::BLACK_DIRECTION,
        };

        let rank = self.get_rank();
        // If this is the pawn's starting rank, return the tile two ranks in front of the pawn.
        
        let mut result = vec![
            self.move_by(direction, 1),
            self.move_by(direction, -1),
            self.move_by(direction, 0),
        ];

        if rank == Rank::PAWN_STARTER_WHITE || rank == Rank::PAWN_STARTER_BLACK {
            result.push(self.move_by(direction * 2, 0));
        }

        result.into_iter().filter_map(|x| x)
    }

    /// The possible tiles a knight might try to move to from this tile.
    fn knight_moves(&self) -> impl Iterator<Item = Tile> {
        // Return the 8 tiles that are a knight's move away.
        [
            self.move_by(2, 1),
            self.move_by(2, -1),
            self.move_by(-2, 1),
            self.move_by(-2, -1),
            self.move_by(1, 2),
            self.move_by(1, -2),
            self.move_by(-1, 2),
            self.move_by(-1, -2),
        ].into_iter().filter_map(|x| x)
    }

    /// The possible tiles a bishop might try to move to from this tile.
    fn bishop_moves(&self) -> impl Iterator<Item = Tile> {
        // Return the tiles along the diagonals.
        self.diagonal_tiles()
    }

    /// The possible tiles a rook might try to move to from this tile.
    fn rook_moves(&self) -> impl Iterator<Item = Tile> {
        // Return the tiles along the ranks and files.
        self.rank_tiles().chain(self.file_tiles())
    }

    /// The possible tiles a queen might try to move to from this tile.
    fn queen_moves(&self) -> impl Iterator<Item = Tile> {
        // Return the tiles along the ranks, files, and diagonals.
        self.rank_tiles().chain(self.file_tiles()).chain(self.diagonal_tiles())
    }

    /// The possible tiles a king might try to move to from this tile.
    fn king_moves(&self) -> impl Iterator<Item = Tile> {
        // Return the 8 tiles that are a king's move away.
        [
            self.move_by(1, 1),
            self.move_by(1, -1),
            self.move_by(-1, 1),
            self.move_by(-1, -1),
            self.move_by(1, 0),
            self.move_by(-1, 0),
            self.move_by(0, 1),
            self.move_by(0, -1),
        ].into_iter().filter_map(|x| x)
    }

    /// The attacking bits for a bishop on this tile.
    #[inline]
    pub fn bishop_attacking_bits(&self) -> u64 {
        let mut bits = 0;
        for tile in self.diagonal_tiles() {
            bits |= tile.to_bit();
        }
        bits
    }

    /// The attacking bits for a rook on this tile.
    #[inline]
    pub fn rook_attacking_bits(&self) -> u64 {
        let mut bits = 0;
        for tile in self.rank_tiles() {
            bits |= tile.to_bit();
        }
        for tile in self.file_tiles() {
            bits |= tile.to_bit();
        }
        bits
    }

    /// The attacking bits for a queen on this tile.
    #[inline]
    pub fn queen_attacking_bits(&self) -> u64 {
        self.rook_attacking_bits() | self.bishop_attacking_bits()
    }

    /// The attacking bits for a king on this tile.
    #[inline]
    pub fn king_attacking_bits(&self) -> u64 {
        let mut bits = 0;
        let my_rank = self.get_rank().get_index() as i8;
        let my_file = self.get_file().get_index() as i8;
        for rank in my_rank as i8 - 1..=my_rank as i8 + 1 {
            for file in my_file as i8 - 1..=my_file as i8 + 1 {
                if rank == my_rank && file == my_file {
                    continue;
                }
                if rank < 0 || rank > 7 || file < 0 || file > 7 {
                    continue;
                }
                bits |= 1 << (rank * 8 + file);
            }
        }
        bits
    }


    /// The castling bits for this tile. This returns the vulnerable bits of the king while castling.
    #[inline]
    pub fn castling_bits(&self, to: Tile) -> u64 {
        let side = self.get_castling_side();
        let color = self.get_player_side();

        
        // Confirm the move is a castling move.
        let proper_to_square = Self::castling_destination_for_king(color, side);
        if to != proper_to_square {
            return 0;
        }

        // Return the bits the king moves through.
        match (color, side) {
            (Color::White, CastlingSide::King) => Tile::WHITE_KING_START.move_by(0, 1).unwrap().to_bit() | Tile::WHITE_KING_START.move_by(0, 2).unwrap().to_bit(),
            (Color::White, CastlingSide::Queen) => Tile::WHITE_KING_START.move_by(0, -1).unwrap().to_bit() | Tile::WHITE_KING_START.move_by(0, -2).unwrap().to_bit() | Tile::WHITE_KING_START.move_by(0, -3).unwrap().to_bit(),
            (Color::Black, CastlingSide::King) => Tile::BLACK_KING_START.move_by(0, 1).unwrap().to_bit() | Tile::BLACK_KING_START.move_by(0, 2).unwrap().to_bit(),
            (Color::Black, CastlingSide::Queen) => Tile::BLACK_KING_START.move_by(0, -1).unwrap().to_bit() | Tile::BLACK_KING_START.move_by(0, -2).unwrap().to_bit() | Tile::BLACK_KING_START.move_by(0, -3).unwrap().to_bit(),
        }
    }
}

impl FromStr for Tile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(());
        }

        let mut chars = s.chars();
        let file = chars.next().ok_or(())?;
        let rank = chars.next().ok_or(())?;

        Ok(Self::new(Rank::from_char(rank), File::from_char(file)))
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}{}", self.get_file(), self.get_rank())
    }
}

impl Debug for Tile {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}{}", self.get_file(), self.get_rank())
    }
}