use rustc_hash::FxHashMap;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

use shakmat_core::{Move, Square, PieceType::*, Board};

pub struct OpeningBook {
    book: FxHashMap<u64, Vec<WeightedMove>>
}

// Aux struct to hold a pair (Move, weight)
struct WeightedMove {
    mv: Move,
    weight: u16,
}

impl OpeningBook {
    // TO-DO: Allow Shakmat to read these as files in the future
    // instead of hardcoding them into the engine
    
    // The polyglot file format is a binary chunk of data, where each
    // entry is 16 bytes long. The format of every entry is:
    // - Bytes 0-7: Zobrist key
    // - Bytes 8-9: Move
    // - Bytes 10-11: Weight
    // - Bytes 12-15: "Learn"
    pub fn load() -> Self {
        let bytes = include_bytes!("openings.bin");
        let mut book: FxHashMap<u64, Vec<WeightedMove>> = FxHashMap::default();

        for pos_data in bytes.chunks_exact(16) {
            // Load the info from the entry (we can ignore the "learn" data)
            let zobrist = u64::from_be_bytes(pos_data[..8].try_into().unwrap());
            let move_data = u16::from_be_bytes(pos_data[8..10].try_into().unwrap());
            let weight = u16::from_be_bytes(pos_data[10..12].try_into().unwrap());

            let mv = u16_to_move(move_data);
            book.entry(zobrist).or_default().push(WeightedMove{ mv, weight });
        }

        Self { book }
    }

    pub fn get_move(&self, board: &Board) -> Option<Move> {
        self.book.get(&board.zobrist_key()).map(|ls| {
            // We have a hit from the book, grab a random entry
            // according to its weight
            let dist = WeightedIndex::new(ls.iter().map(|entry| entry.weight)).unwrap();
            let mut rng = thread_rng();
            ls[dist.sample(&mut rng)].mv
        })
    }
}

fn u16_to_move(bits: u16) -> Move {
/*  
    Polyglot encodes moves in 16 bits, as follows:
    bits      meaning
    =========================
    0,1,2     to file
    3,4,5     to row
    6,7,8     from file
    9,10,11   from row
    12,13,14  promotion piece (0-4) =-(None, N, B, R, Q)
    
    Also, castling is represented as:
        white short      e1h1
        white long       e1a1
        black short      e8h8
        black long       e8a8
*/
    let to_file = bits & 0x7;
    let to_row = (bits & 0x3F) >> 3;
    let from_file = (bits & 0x1FF) >> 6;
    let from_row = (bits & 0xFFF) >> 9;
    let promote_to_id = (bits & 0x7FFF) >> 12;

    let from_square = Square::from_file_rank(from_file as u8, from_row as u8).unwrap().square();
    let to_square = Square::from_file_rank(to_file as u8, to_row as u8).unwrap().square();

    if (from_square == 3 && to_square == 0) || (from_square == 59 && to_square == 56) {
        Move::ShortCastle
    } else if (from_square == 3 && to_square == 7) || (from_square == 59 && to_square == 63) {
        Move::LongCastle
    } else if promote_to_id != 0 {
        let promote_to = match promote_to_id {
            1 => Knight,
            2 => Bishop,
            3 => Rook,
            4 => Queen,
            _ => unreachable!(),
        };

        Move::PawnPromotion{from: from_square, to: to_square, promote_to}
    } else {
        Move::Normal{from: from_square, to: to_square}
    }

}