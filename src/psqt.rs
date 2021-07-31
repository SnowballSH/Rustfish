// SPDX-License-Identifier: GPL-3.0-or-later

use bitboard::*;
use types::*;

use std;

macro_rules! s {
    ($x:expr, $y:expr) => {
        Score(($y << 16) + $x)
    };
}

const BONUS: [[[Score; 4]; 8]; 6] = [
    [
        // Pawn
        [s!(0, 0), s!(0, 0), s!(0, 0), s!(0, 0)],
        [s!(-11, 7), s!(6, -4), s!(7, 8), s!(3, -2)],
        [s!(-18, -4), s!(-2, -5), s!(19, 5), s!(24, 4)],
        [s!(-17, 3), s!(-9, 3), s!(20, -8), s!(35, -3)],
        [s!(-6, 8), s!(5, 9), s!(3, 7), s!(21, -6)],
        [s!(-6, 8), s!(-8, -5), s!(-6, 2), s!(-2, 4)],
        [s!(-4, 3), s!(20, -9), s!(-8, 1), s!(-4, 18)],
        [s!(0, 0), s!(0, 0), s!(0, 0), s!(0, 0)],
    ],
    [
        // Knight
        [s!(-161, -105), s!(-96, -82), s!(-80, -46), s!(-73, -14)],
        [s!(-83, -69), s!(-43, -54), s!(-21, -17), s!(-10, 9)],
        [s!(-71, -50), s!(-22, -39), s!(0, -7), s!(9, 28)],
        [s!(-25, -41), s!(18, -25), s!(43, 6), s!(47, 38)],
        [s!(-26, -46), s!(16, -25), s!(38, 3), s!(50, 40)],
        [s!(-11, -54), s!(37, -38), s!(56, -7), s!(65, 27)],
        [s!(-63, -65), s!(-19, -50), s!(5, -24), s!(14, 13)],
        [s!(-195, -109), s!(-67, -89), s!(-42, -50), s!(-29, -13)],
    ],
    [
        // Bishop
        [s!(-44, -58), s!(-13, -31), s!(-25, -37), s!(-34, -19)],
        [s!(-20, -34), s!(20, -9), s!(12, -14), s!(1, 4)],
        [s!(-9, -23), s!(27, 0), s!(21, -3), s!(11, 16)],
        [s!(-11, -26), s!(28, -3), s!(21, -5), s!(10, 16)],
        [s!(-11, -26), s!(27, -4), s!(16, -7), s!(9, 14)],
        [s!(-17, -24), s!(16, -2), s!(12, 0), s!(2, 13)],
        [s!(-23, -34), s!(17, -10), s!(6, -12), s!(-2, 6)],
        [s!(-35, -55), s!(-11, -32), s!(-19, -36), s!(-29, -17)],
    ],
    [
        // Rook
        [s!(-25, 0), s!(-16, 0), s!(-16, 0), s!(-9, 0)],
        [s!(-21, 0), s!(-8, 0), s!(-3, 0), s!(0, 0)],
        [s!(-21, 0), s!(-9, 0), s!(-4, 0), s!(2, 0)],
        [s!(-22, 0), s!(-6, 0), s!(-1, 0), s!(2, 0)],
        [s!(-22, 0), s!(-7, 0), s!(0, 0), s!(1, 0)],
        [s!(-21, 0), s!(-7, 0), s!(0, 0), s!(2, 0)],
        [s!(-12, 0), s!(4, 0), s!(8, 0), s!(12, 0)],
        [s!(-23, 0), s!(-15, 0), s!(-11, 0), s!(-5, 0)],
    ],
    [
        // Queen
        [s!(0, -71), s!(-4, -56), s!(-3, -42), s!(-1, -29)],
        [s!(-4, -56), s!(6, -30), s!(9, -21), s!(8, -5)],
        [s!(-2, -39), s!(6, -17), s!(9, -8), s!(9, 5)],
        [s!(-1, -29), s!(8, -5), s!(10, 9), s!(7, 19)],
        [s!(-3, -27), s!(9, -5), s!(8, 10), s!(7, 21)],
        [s!(-2, -40), s!(6, -16), s!(8, -10), s!(10, 3)],
        [s!(-2, -55), s!(7, -30), s!(7, -21), s!(6, -6)],
        [s!(-1, -74), s!(-4, -55), s!(-1, -43), s!(0, -30)],
    ],
    [
        // King
        [s!(267, 0), s!(320, 48), s!(270, 75), s!(195, 84)],
        [s!(264, 43), s!(304, 92), s!(238, 143), s!(180, 132)],
        [s!(200, 83), s!(245, 138), s!(176, 167), s!(110, 165)],
        [s!(177, 106), s!(185, 169), s!(148, 169), s!(110, 179)],
        [s!(149, 108), s!(177, 163), s!(115, 200), s!(66, 203)],
        [s!(118, 95), s!(159, 155), s!(84, 176), s!(41, 174)],
        [s!(87, 50), s!(128, 99), s!(63, 122), s!(20, 139)],
        [s!(63, 9), s!(88, 55), s!(47, 80), s!(0, 90)],
    ],
];

static mut PSQ: [[Score; 64]; 16] = [[Score(0); 64]; 16];

pub fn psq(pc: Piece, s: Square) -> Score {
    unsafe { PSQ[pc.0 as usize][s.0 as usize] }
}

pub fn init() {
    unsafe {
        for i in 1..7 {
            let pc = Piece(i);
            let score = Score::make(piece_value(MG, pc).0, piece_value(EG, pc).0);

            for s in ALL_SQUARES {
                let f = std::cmp::min(s.file(), FILE_H - s.file());
                PSQ[pc.0 as usize][s.0 as usize] =
                    score + BONUS[(pc.0 - 1) as usize][s.rank() as usize][f as usize];
                PSQ[(!pc).0 as usize][(!s).0 as usize] = -PSQ[pc.0 as usize][s.0 as usize];
            }
        }
    }
}
