use super::utils::*;

use super::BitBoard;

const SECOND_RANK: BitBoard = 0x000000000000FF00;
const SEVENTH_RANK: BitBoard = 0x00FF000000000000;
pub const PAWN_MOVES: [[BitBoard; 64]; 2] = {
    let mut res = [[0; 64]; 2];
    let mut i: BitBoard = 0;
    while i < 64 {
        let square = (1 as BitBoard) << i;
        let mut moveable_squares: BitBoard = 0;
        moveable_squares |= north_one(square);
        if square & SECOND_RANK != 0 {
            moveable_squares |= north_one(north_one(square));
        }
        res[0][i as usize] = moveable_squares;
        i += 1;
    }
    let mut i: BitBoard = 0;
    while i < 64 {
        let square = (1 as BitBoard) << i;
        let mut moveable_squares: BitBoard = 0;
        moveable_squares |= south_one(square);
        if square & SEVENTH_RANK != 0 {
            moveable_squares |= south_one(south_one(square));
        }
        res[1][i as usize] = moveable_squares;
        i += 1;
    }
    res
};

pub const PAWN_ATTACKS: [[BitBoard; 64]; 2] = {
    let mut res = [[0; 64]; 2];
    let mut i: BitBoard = 0;
    while i < 64 {
        let square = (1 as BitBoard) << i;
        res[0][i as usize] = north_east_one(square) | north_west_one(square);
        i += 1;
    }
    let mut i: BitBoard = 0;
    while i < 64 {
        let square = (1 as BitBoard) << i;
        res[1][i as usize] = south_east_one(square) | south_west_one(square);
        i += 1;
    }
    res
};

/*Directions are clockwise starting from north
0: north
1: northeast
2: east
3: southeast
4: south
5: southwest
6: west
7: northwest*/
pub const RAYS: [[BitBoard; 64]; 8] = {
    let mut res = [[0; 64]; 8];
    let mut i = 0;
    //Bad code but computed once :pensive: (no for loop in const)
    while i < 64 {
        let x: BitBoard = 1 << i;
        res[0][i] = north_one(x);
        res[1][i] = north_east_one(x);
        res[2][i] = east_one(x);
        res[3][i] = south_east_one(x);
        res[4][i] = south_one(x);
        res[5][i] = south_west_one(x);
        res[6][i] = west_one(x);
        res[7][i] = north_west_one(x);
        let mut j = 0;
        while j < 6 {
            res[0][i] |= north_one(res[0][i]);
            res[1][i] |= north_east_one(res[1][i]);
            res[2][i] |= east_one(res[2][i]);
            res[3][i] |= south_east_one(res[3][i]);
            res[4][i] |= south_one(res[4][i]);
            res[5][i] |= south_west_one(res[5][i]);
            res[6][i] |= west_one(res[6][i]);
            res[7][i] |= north_west_one(res[7][i]);
            j += 1;
        }
        i += 1;
    }
    res
};

pub const KING_MOVES: [BitBoard; 64] = {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let x: BitBoard = 1 << i;
        res[i] = east_one(x)
            | west_one(x)
            | north_one(x)
            | north_east_one(x)
            | north_west_one(x)
            | south_one(x)
            | south_east_one(x)
            | south_west_one(x);
        i += 1;
    }
    res
};

pub const KNIGHT_MOVES: [BitBoard; 64] = {
    let mut res = [0; 64];
    let mut i = 0;
    //May be slow, idk tbh, but its precomupted lmao
    while i < 64 {
        let x: BitBoard = 1 << i;
        let mut y: BitBoard = south_east_one(south_one(x)) | south_west_one(south_one(x));
        y |= north_east_one(north_one(x)) | north_west_one(north_one(x));
        y |= south_west_one(west_one(x)) | north_west_one(west_one(x));
        y |= south_east_one(east_one(x)) | north_east_one(east_one(x));
        res[i] = y;
        i += 1;
    }
    res
};

pub const PATH_BETWEEN: [[BitBoard; 64]; 64] = {
    let mut res: [[BitBoard; 64]; 64] = [[0; 64]; 64];
    let mut from: BitBoard = 0;
    while from < 64 {
        let mut to: BitBoard = 0;
        while to < 64 {
            if from == to {
                to += 1;
                continue;
            }
            res[from as usize][to as usize] = path_between(from, to);
            to += 1;
        }
        from += 1;
    }
    res
};

const fn path_between(from: BitBoard, to: BitBoard) -> BitBoard {
    let to_mask = (1 as BitBoard) << to;
    let mut k = 0;
    while k < 8 {
        if to_mask & RAYS[k][from as usize] != 0 {
            break;
        }
        k += 1;
    }
    if k == 8 {
        0
    } else {
        RAYS[k][from as usize] & !RAYS[k][to as usize]
    }
}

//Source: https://rhysre.net/fast-chess-move-generation-with-magic-bitboards.html
#[inline]
pub const fn bishop_moves(square: usize, empty: BitBoard) -> BitBoard {
    let blockers = !empty;
    let mut attacks: BitBoard = 0;

    // North West
    attacks |= RAYS[7][square];
    if RAYS[7][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[7][square] & blockers) as usize;
        attacks &= !RAYS[7][blocker_index];
    }

    // North East
    attacks |= RAYS[1][square];
    if RAYS[1][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[1][square] & blockers) as usize;
        attacks &= !RAYS[1][blocker_index];
    }

    // South East
    attacks |= RAYS[3][square];
    if RAYS[3][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[3][square] & blockers) as usize;
        attacks &= !RAYS[3][63 - blocker_index];
    }

    // South West
    attacks |= RAYS[5][square];
    if RAYS[5][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[5][square] & blockers) as usize;
        attacks &= !RAYS[5][63 - blocker_index];
    }
    attacks
}

//Source: https://rhysre.net/fast-chess-move-generation-with-magic-bitboards.html
#[inline]
pub const fn rook_moves(square: usize, empty: BitBoard) -> BitBoard {
    let blockers = !empty;
    let mut attacks: BitBoard = 0;

    // North
    attacks |= RAYS[0][square];
    if RAYS[0][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[0][square] & blockers) as usize;
        attacks &= !RAYS[0][blocker_index];
    }

    // East
    attacks |= RAYS[2][square];
    if RAYS[2][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[2][square] & blockers) as usize;
        attacks &= !RAYS[2][blocker_index];
    }

    // West
    attacks |= RAYS[6][square];
    if RAYS[6][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[6][square] & blockers) as usize;
        attacks &= !RAYS[6][63 - blocker_index];
    }

    // South
    attacks |= RAYS[4][square];
    if RAYS[4][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[4][square] & blockers) as usize;
        attacks &= !RAYS[4][63 - blocker_index];
    }

    attacks
}

#[inline]
pub fn rook_xray(square: usize, empty: BitBoard) -> BitBoard {
    let mut blockers = !empty;
    let mut attacks: BitBoard = 0;

    // North
    attacks |= RAYS[0][square];
    if RAYS[0][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[0][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << blocker_index);
        if RAYS[0][square] & blockers != 0 {
            let blocker_index = BitBoard::trailing_zeros(RAYS[0][square] & blockers) as usize;
            attacks &= !RAYS[0][blocker_index];
        }
    }

    //East
    attacks |= RAYS[2][square];
    if RAYS[2][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[2][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << blocker_index);
        if RAYS[2][square] & blockers != 0 {
            let blocker_index = BitBoard::trailing_zeros(RAYS[2][square] & blockers) as usize;
            attacks &= !RAYS[2][blocker_index];
        }
    }

    //South
    attacks |= RAYS[4][square];
    if RAYS[4][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[4][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << 63 - blocker_index);
        if RAYS[4][square] & blockers != 0 {
            let blocker_index = BitBoard::leading_zeros(RAYS[4][square] & blockers) as usize;
            attacks &= !RAYS[4][63 - blocker_index];
        }
    }

    //West
    attacks |= RAYS[6][square];
    if RAYS[6][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[6][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << 63 - blocker_index);
        if RAYS[6][square] & blockers != 0 {
            let blocker_index = BitBoard::leading_zeros(RAYS[6][square] & blockers) as usize;
            attacks &= !RAYS[6][63 - blocker_index];
        }
    }

    attacks
}

#[inline]
pub fn bishop_xray(square: usize, empty: BitBoard) -> BitBoard {
    let mut blockers = !empty;
    let mut attacks: BitBoard = 0;

    // North
    attacks |= RAYS[7][square];
    if RAYS[7][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[7][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << blocker_index);
        if RAYS[7][square] & blockers != 0 {
            let blocker_index = BitBoard::trailing_zeros(RAYS[7][square] & blockers) as usize;
            attacks &= !RAYS[7][blocker_index];
        }
    }

    //East
    attacks |= RAYS[1][square];
    if RAYS[1][square] & blockers != 0 {
        let blocker_index = BitBoard::trailing_zeros(RAYS[1][square] & blockers) as usize;

        blockers &= !((1 as BitBoard) << blocker_index);
        if RAYS[1][square] & blockers != 0 {
            let blocker_index = BitBoard::trailing_zeros(RAYS[1][square] & blockers) as usize;
            attacks &= !RAYS[1][blocker_index];
        }
    }

    //South
    attacks |= RAYS[3][square];
    if RAYS[3][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[3][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << 63 - blocker_index);
        if RAYS[3][square] & blockers != 0 {
            let blocker_index = BitBoard::leading_zeros(RAYS[3][square] & blockers) as usize;
            attacks &= !RAYS[3][63 - blocker_index];
        }
    }

    //West
    attacks |= RAYS[5][square];
    if RAYS[5][square] & blockers != 0 {
        let blocker_index = BitBoard::leading_zeros(RAYS[5][square] & blockers) as usize;
        blockers &= !((1 as BitBoard) << 63 - blocker_index);
        if RAYS[5][square] & blockers != 0 {
            let blocker_index = BitBoard::leading_zeros(RAYS[5][square] & blockers) as usize;
            attacks &= !RAYS[5][63 - blocker_index];
        }
    }

    attacks
}

#[inline]
pub fn queen_xray(square: usize, empty: BitBoard) -> BitBoard {
    rook_xray(square, empty) | bishop_xray(square, empty)
}

#[inline]
pub fn queen_moves(square: usize, empty: BitBoard) -> BitBoard {
    rook_moves(square, empty) | bishop_moves(square, empty)
}
