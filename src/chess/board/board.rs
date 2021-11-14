use std::fmt::Display;
use std::result::Result;

use crate::chess::{CastlingRights, Color, Move, Position, Piece};
use crate::chess::fen::{read_fen, DEFAULT_FEN};

pub type PieceArray = [Option<Piece>; 16];
type BoardSquares = [[Option<PieceArrayPos>; 8]; 8];


#[derive(Debug, Clone, Copy)]
pub struct Board {
    castling_rights: CastlingRights,
    en_passant_target: Option<Position>,
    turn: Color,
    half_turns_til_50move_draw: u16,
    full_turns: u16,
    squares: BoardSquares,
    white_pieces: PieceArray,
    black_pieces: PieceArray,
}

#[derive(Debug, Clone, Copy)]
struct PieceArrayPos {
    pub color: Color,
    pub index: usize,
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let fen_info = read_fen(fen)?;
        let mut board = Board {
            castling_rights: fen_info.castling_rights,
            turn: fen_info.turn,
            en_passant_target: fen_info.en_passant_square,
            half_turns_til_50move_draw: 100 - fen_info.halfmoves_since_capture,
            full_turns: fen_info.fullmoves_since_start,
            squares: [[None; 8]; 8],
            white_pieces: fen_info.white_pieces,
            black_pieces: fen_info.black_pieces
        };

        // Initialize the square reference info
        for (i, piece_opt) in board.white_pieces.iter().enumerate() {
            if let Some(piece) = piece_opt {
                let pos = piece.position();
                board.squares[pos.rank_u()][pos.file_u()] = Some(PieceArrayPos {
                    color: Color::White,
                    index: i
                });
            }
        }

        for (i, piece_opt) in board.black_pieces.iter().enumerate() {
            if let Some(piece) = piece_opt {
                let pos = piece.position();
                board.squares[pos.rank_u()][pos.file_u()] = Some(PieceArrayPos {
                    color: Color::Black,
                    index: i
                });
            }
        }

        Ok(board)
    }

    pub fn make_move(&self, movement: Move, check_legality: bool) -> Result<Board, String> {
        // todo
        if check_legality {
            // This move was received from the user, check that it is indeed legal
            // We do this by making sure it exists in the list of allowed moves
            // Even though we generate all the moves just to check, this is only
            // done for user-provided moves. The moves made by the engine when
            // it is analyzing a position bypass this check
            if !self.get_current_turn_moves().contains(&movement) {
                return Err("Illegal move".to_owned())
            }
        }

        Ok(*self) // TODO
    }

    pub fn get_current_turn_moves(&self) -> Vec<Move> {
        self.get_pieces(self.turn)
            .iter()
            .filter_map(|&p| p)
            .flat_map(|piece| piece.get_legal_moves(self).into_iter())
            .collect()
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        self.get_king_position(color).is_attacked_by(self, !color)
    }

    pub fn get_pos(&self, pos: &Position) -> Option<&Piece> {
        self.squares[pos.rank_u()][pos.file_u()].as_ref()
            .map(|arr_info| self.get_pieces(arr_info.color)[arr_info.index].as_ref().unwrap())

    }

    pub fn get_en_passant_target(&self) -> &Option<Position> {
        &self.en_passant_target
    }

    pub fn castling_info(&self) -> &CastlingRights {
        &self.castling_rights
    }

    pub fn get_pieces(&self, color: Color) -> &PieceArray {
        match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces
        }
    }

    pub fn get_king_position(&self, color: Color) -> Position {
        // The king is guaranteed to exist and to be in the
        // first position of the piece array, hence, we can unwrap it safely
        *self.get_pieces(color)[0].unwrap().position()
    }
}

impl Default for Board {
    fn default() -> Self {
        // The default FEN is hard-coded and correct,
        // so we can unwrap it safely
        Board::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl Display for Board {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{:?} to play, turn #{}\n", self.turn, self.full_turns)?;
        writeln!(f, "  ┌───┬───┬───┬───┬───┬───┬───┬───┐")?;

        for rank in (0..8).rev() {
            let pieces_line = (0..8)
                .map(|file| Position::new_0based(file, rank))
                .map(|sqre| match self.get_pos(&sqre) {
                    None => "   ".to_string(),
                    Some(piece) => format!(" {} ", piece.as_char().to_string())
                })
                .collect::<Vec<String>>()
                .join("│");

            writeln!(f, "{} │{}│", rank + 1, pieces_line)?;

            if rank != 0 {
                writeln!(f, "  ├───┼───┼───┼───┼───┼───┼───┼───┤")?;
            }
        }

        writeln!(f, "  └───┴───┴───┴───┴───┴───┴───┴───┘")?;
        writeln!(f, "    a   b   c   d   e   f   g   h ")?;
        Ok(())
    }

}