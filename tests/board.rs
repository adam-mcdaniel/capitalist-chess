/*
 * This is a test of the board.
 * It places pieces on the board and confirms the pieces can only
 * move to legal locations.
 */

use eco_chess::*;
use std::str::FromStr;

static mut ALREADY_INIT: bool = false;

fn init() {
    unsafe {
        if ALREADY_INIT {
            return;
        }
        ALREADY_INIT = true;
    }
    let _ = env_logger::builder().is_test(true).try_init();
}

/// This tests if the pawn can move forward one tile.
#[test]
fn pawn_move_forward_one() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("e7")?);

    // Confirm the white pawn can move forward
    board.apply(Move::from_str("e3")?)?;

    // Confirm the black pawn can move forward
    board.apply(Move::from_str("e6")?)?;

    Ok(())
}

/// This tests if the pawn can move forward two tiles.
#[test]
fn pawn_move_forward_two() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("d7")?);
    board.sanity_check()?;

    // Confirm the white pawn can move forward
    board.apply(Move::from_str("e4")?)?;
    board.sanity_check()?;

    // Confirm the black pawn can move forward
    board.apply(Move::from_str("d5")?)?;
    board.sanity_check()?;

    Ok(())
}

/// Test for en passant capture.
#[test]
fn pawn_en_passant_capture() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("d7")?);
    board.spawn_black_pawn(Tile::from_str("f7")?);
    board.apply(Move::from_str("e4")?)?;
    println!("{}", board);
    board.sanity_check()?;
    println!("Passed sanity check");
    board.apply(Move::from_str("d6")?)?;
    println!("{}", board);
    board.sanity_check()?;
    println!("Passed sanity check");
    board.apply(Move::from_str("e5")?)?;
    println!("{}", board);
    board.sanity_check()?;
    println!("Passed sanity check");
    board.apply(Move::from_str("f5")?)?;
    println!("{}", board);
    board.sanity_check()?;
    println!("Passed sanity check");
    board.apply(Move::from_str("f6").unwrap()).expect("en-passant capture failed");
    println!("{}", board);
    board.sanity_check()?;
    println!("Passed sanity check");

    Ok(())
}

/// Test en passant expiration.
#[test]
fn pawn_en_passant_expiration() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("a2")?);
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("d7")?);
    board.spawn_black_pawn(Tile::from_str("f7")?);
    board.sanity_check()?;
    board.apply(Move::from_str("e4")?)?;
    board.sanity_check()?;
    board.apply(Move::from_str("d6")?)?;
    board.sanity_check()?;
    board.apply(Move::from_str("e5")?)?;
    board.sanity_check()?;
    board.apply(Move::from_str("f5")?)?;
    board.sanity_check()?;
    board.apply(Move::from_str("a3")?)?;
    board.sanity_check()?;
    board.apply(Move::from_str("d5")?)?;
    board.sanity_check()?;
    assert!(board.apply(Move::from_str("f6")?).is_err());
    board.sanity_check()?;

    Ok(())
}

/// Test for pawn promotion.
#[test]
fn pawn_promotion() -> Result<(), ()> {
    // Test promote to queen
    init();
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("e7")?);
    board.spawn_black_pawn(Tile::from_str("d2")?);
    board.spawn_black_pawn(Tile::from_str("f2")?);
    board.apply(Move::PieceTo {
        piece: PieceType::Pawn,
        to: Tile::from_str("e8")?,
        promotion: Some(PieceType::Queen),
    })?;
    board.sanity_check()?;
    assert_eq!(board.get_piece(Tile::from_str("e8")?), Some(Piece::new(PieceType::Queen, Color::White)));
    
    // Test promote to knight
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("e7")?);
    board.spawn_black_pawn(Tile::from_str("d2")?);
    board.spawn_black_pawn(Tile::from_str("f2")?);
    board.apply(Move::PieceTo {
        piece: PieceType::Pawn,
        to: Tile::from_str("e8")?,
        promotion: Some(PieceType::Knight),
    })?;
    board.sanity_check()?;
    assert_eq!(board.get_piece(Tile::from_str("e8")?), Some(Piece::new(PieceType::Knight, Color::White)));
    
    Ok(())
}

/// Test pawn attacks.
#[test]
fn pawn_attacks() -> Result<(), ()> {
    init();

    let mut board = Board::empty();

    // Test white pawn attacks
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("d3")?);

    board.apply(Move::from_str("e2d3")?)?;
    board.sanity_check()?;
    
    let mut board = Board::empty();

    // Test white pawn attacks
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("f3")?);

    board.apply(Move::from_str("e2f3")?)?;
    board.sanity_check()?;
    
    let mut board = Board::empty();

    // Test white pawn attacks
    board.spawn_white_pawn(Tile::from_str("e2")?);
    board.spawn_black_pawn(Tile::from_str("e3")?);

    // Cant attack directly forward
    assert!(board.apply(Move::from_str("e2e3")?).is_err());
    board.sanity_check()?;
    
    Ok(())
}

/// Test if the board can detect checks and checkmates.
#[test]
pub fn checkmate_detection() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_king(Tile::from_str("e1")?);
    board.spawn_black_rook(Tile::from_str("e8")?);
    println!("{}", board);
    assert!(board.is_in_check(Color::White));
    assert!(!board.is_in_check(Color::Black));
    assert!(!board.is_in_checkmate(Color::White));

    // Test if the board can detect checkmate
    let mut board = Board::empty();
    board.spawn_white_king(Tile::from_str("a1")?);
    board.spawn_black_rook(Tile::from_str("h1")?);
    board.spawn_black_rook(Tile::from_str("g2")?);
    assert!(board.is_in_check(Color::White));
    assert!(board.is_in_checkmate(Color::White));
    assert!(!board.is_in_check(Color::Black));

    // Remove the piece that is blocking the king's path
    board.remove_piece(Tile::from_str("g2")?);
    assert!(board.is_in_check(Color::White));
    assert!(!board.is_in_checkmate(Color::White));
    
    board.remove_piece(Tile::from_str("h1")?);
    assert!(!board.is_in_check(Color::White));
    assert!(!board.is_in_checkmate(Color::White));

    Ok(())
}


/// Test if the board can detect stalemates.
#[test]
fn stalemate_detection() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_king(Tile::from_str("a2")?);
    board.spawn_black_rook(Tile::from_str("h1")?);
    board.spawn_black_rook(Tile::from_str("b8")?);
    board.spawn_black_rook(Tile::from_str("g3")?);
    assert!(!board.is_in_check(Color::White));
    assert!(!board.is_in_checkmate(Color::White));
    assert!(board.is_stalemate());

    board.remove_piece(Tile::from_str("g3")?);

    assert!(!board.is_in_check(Color::White));
    assert!(!board.is_in_checkmate(Color::White));
    assert!(!board.is_stalemate());

    Ok(())
}

/// Test legal move generation.
#[test]
fn legal_move_generation() -> Result<(), ()> {
    init();
    let mut board = Board::empty();
    board.spawn_white_king(Tile::from_str("a2")?);
    board.spawn_black_rook(Tile::from_str("h1")?);
    board.spawn_black_rook(Tile::from_str("b8")?);
    board.spawn_black_rook(Tile::from_str("g3")?);
    assert!(!board.is_in_checkmate(Color::White));
    assert!(!board.is_in_check(Color::Black));
    assert!(board.is_stalemate());

    let moves = Move::legal_moves(&board);
    assert_eq!(moves.len(), 0);

    board.remove_piece(Tile::from_str("b8")?);    
    board.remove_piece(Tile::from_str("c8")?);    

    let moves = Move::legal_moves(&board);
    assert_eq!(moves.len(), 1);

    // Check if can generate en passant capture
    let mut board = Board::empty();
    board.spawn_white_pawn(Tile::from_str("a2")?);
    board.spawn_black_pawn(Tile::from_str("b7")?);
    board.spawn_black_pawn(Tile::from_str("c7")?);
    board.apply(Move::from_str("a4")?)?;
    board.apply(Move::from_str("c5")?)?;
    board.apply(Move::from_str("a5")?)?;
    board.apply(Move::from_str("b5")?)?;
    assert_eq!(Move::legal_moves(&board), vec![Move::from_str("a5b6")?, Move::from_str("a5a6")?]);

    Ok(())
}

/// Test rook movement.
#[test]
fn rook_movement() -> Result<(), ()> {
    init();

    let mut board = Board::empty();
    board.spawn_white_rook(Tile::from_str("e3")?);

    // Test rook movement
    board.set_turn(Color::White);
    board.apply(Move::from_str("e3h3")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("h3h8")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("h8a8")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    assert!(board.apply(Move::from_str("a8h1")?).is_err());
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("a8c8")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    assert!(board.apply(Move::from_str("c8b7")?).is_err());

    // Put an enemy piece in the way
    board.spawn_black_rook(Tile::from_str("c4")?);
    board.sanity_check()?;
    board.set_turn(Color::White);
    assert!(board.apply(Move::from_str("c8c1")?).is_err());
    assert!(board.apply(Move::from_str("c8c2")?).is_err());
    assert!(board.apply(Move::from_str("c8c3")?).is_err());
    board.apply(Move::from_str("c8c4")?)?;

    Ok(())
}

/// Test bishop movement.
#[test]
fn bishop_movement() -> Result<(), ()> {
    init();

    let mut board = Board::empty();
    board.spawn_white_bishop(Tile::from_str("e3")?);

    // Test bishop movement
    board.set_turn(Color::White);
    board.apply(Move::from_str("e3h6")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("h6f4")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("f4d2")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    board.apply(Move::from_str("d2b4")?)?;
    board.sanity_check()?;
    board.set_turn(Color::White);
    assert!(board.apply(Move::from_str("b4c6")?).is_err());

    // Put an enemy piece in the way
    board.spawn_black_bishop(Tile::from_str("d6")?);
    board.sanity_check()?;
    board.set_turn(Color::White);
    assert!(board.apply(Move::from_str("b4f8")?).is_err());
    assert!(board.apply(Move::from_str("b4e7")?).is_err());
    board.apply(Move::from_str("b4d6")?)?;
    
    Ok(())
}