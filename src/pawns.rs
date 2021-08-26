// SPDX-License-Identifier: GPL-3.0-or-later

use std;

use bitboard::*;
use position::Position;
use types::*;

macro_rules! v {
    ($x:expr) => {
        Value($x)
    };
}
macro_rules! s {
    ($x:expr, $y:expr) => {
        Score(($y << 16) + $x)
    };
}

const V0: Value = Value::ZERO;

// Isolated pawn penalty
const ISOLATED: Score = s!(13, 18);

// Backward pawn penalty
const BACKWARD: Score = s!(24, 12);

// Connected pawn bonus by opposed, phalanx, #support and rank
static mut CONNECTED: [[[[Score; 8]; 3]; 2]; 2] = [[[[Score::ZERO; 8]; 3]; 2]; 2];

// Doubled pawn penalty
const DOUBLED: Score = s!(18, 38);

// Strength of pawn shelter for our king by [distance from edge][rank].
// RANK_1 = 0 is used for files where we have no pawn, or pawn is behind our king.
const SHELTER_STRENGTH: [[Value; 8]; 4] = [
    [v!(-9), v!(64), v!(77), v!(44), v!(4), v!(-1), v!(-11), V0],
    [
        v!(-15),
        v!(83),
        v!(51),
        v!(-10),
        v!(1),
        v!(-10),
        v!(-28),
        V0,
    ],
    [
        v!(-18),
        v!(84),
        v!(27),
        v!(-12),
        v!(21),
        v!(-7),
        v!(-36),
        V0,
    ],
    [v!(12), v!(79), v!(25), v!(19), v!(9), v!(-6), v!(-33), V0],
];

// Danger of enemy pawns moving toward our king by
// [type][distance from edge][rank]. For the unopposed and unblocked cases,
// RANK_1 = 0 is used when opponent has no pawn on the given file or their
// pawn is behind our king.
const STORM_DANGER: [[[Value; 8]; 4]; 3] = [
    // Unopposed
    [
        [v!(4), v!(73), v!(132), v!(46), v!(31), V0, V0, V0],
        [v!(1), v!(64), v!(143), v!(26), v!(13), V0, V0, V0],
        [v!(1), v!(47), v!(110), v!(44), v!(24), V0, V0, V0],
        [v!(0), v!(72), v!(127), v!(50), v!(31), V0, V0, V0],
    ],
    // BlockedByPawn
    [
        [v!(0), v!(0), v!(19), v!(23), v!(1), V0, V0, V0],
        [v!(0), v!(0), v!(88), v!(27), v!(2), V0, V0, V0],
        [v!(0), v!(0), v!(101), v!(16), v!(1), V0, V0, V0],
        [v!(0), v!(0), v!(111), v!(22), v!(15), V0, V0, V0],
    ],
    // Unblocked
    [
        [v!(22), v!(45), v!(104), v!(62), v!(6), V0, V0, V0],
        [v!(31), v!(30), v!(99), v!(39), v!(19), V0, V0, V0],
        [v!(23), v!(29), v!(96), v!(41), v!(15), V0, V0, V0],
        [v!(21), v!(23), v!(116), v!(41), v!(15), V0, V0, V0],
    ],
];

// pawns::Entry contains various information about a pawn structure. A lookup
// in the pawn hash table (performed by calling the probing function) returns
// a pointer to an Entry object.

pub struct Entry {
    key: Key,
    score: Score,
    passed_pawns: [Bitboard; 2],
    pawn_attacks: [Bitboard; 2],
    pawn_attacks_span: [Bitboard; 2],
    king_squares: [Square; 2],
    king_safety: [Score; 2],
    weak_unopposed: [i32; 2],
    castling_rights: [CastlingRight; 2],
    semiopen_files: [i32; 2],
    pawns_on_squares: [[i32; 2]; 2],
    asymmetry: i32,
    open_files: i32,
}

impl Entry {
    pub fn new() -> Entry {
        Entry {
            key: Key(0),
            score: Score::ZERO,
            passed_pawns: [Bitboard(0); 2],
            pawn_attacks: [Bitboard(0); 2],
            pawn_attacks_span: [Bitboard(0); 2],
            king_squares: [Square(0); 2],
            king_safety: [Score::ZERO; 2],
            weak_unopposed: [0; 2],
            castling_rights: [CastlingRight(0); 2],
            semiopen_files: [0; 2],
            pawns_on_squares: [[0; 2]; 2], // [Color][light/dark squares]
            asymmetry: 0,
            open_files: 0,
        }
    }

    pub fn pawns_score(&self) -> Score {
        self.score
    }

    pub fn pawn_attacks(&self, c: Color) -> Bitboard {
        self.pawn_attacks[c.0 as usize]
    }

    pub fn passed_pawns(&self, c: Color) -> Bitboard {
        self.passed_pawns[c.0 as usize]
    }

    pub fn pawn_attacks_span(&self, c: Color) -> Bitboard {
        self.pawn_attacks_span[c.0 as usize]
    }

    pub fn weak_unopposed(&self, c: Color) -> i32 {
        self.weak_unopposed[c.0 as usize]
    }

    pub fn pawn_asymmetry(&self) -> i32 {
        self.asymmetry
    }

    pub fn open_files(&self) -> i32 {
        self.open_files
    }

    pub fn semiopen_file(&self, c: Color, f: File) -> i32 {
        self.semiopen_files[c.0 as usize] & (1 << f)
    }

    pub fn pawns_on_same_color_squares(&self, c: Color, s: Square) -> i32 {
        self.pawns_on_squares[c.0 as usize][((DARK_SQUARES & s) != 0) as usize]
    }

    pub fn king_safety<Us: ColorTrait>(&mut self, pos: &Position, ksq: Square) -> Score {
        let us = Us::COLOR;
        if self.king_squares[us.0 as usize] != ksq
            || self.castling_rights[us.0 as usize] != pos.castling_rights(us)
        {
            self.king_safety[us.0 as usize] = self.do_king_safety::<Us>(pos, ksq);
        }
        self.king_safety[us.0 as usize]
    }

    /// evaluate_shelter() calculates the shelter bonus and the storm
    /// penalty for a king, looking at the king file and the two closest files.

    fn evaluate_shelter<Us: ColorTrait>(&self, pos: &Position, ksq: Square) -> Value {
        let us = Us::COLOR;
        let them = if us == WHITE { BLACK } else { WHITE };
        let down = if us == WHITE { SOUTH } else { NORTH };
        let block_ranks = if us == WHITE {
            RANK1_BB | RANK2_BB
        } else {
            RANK8_BB | RANK7_BB
        };

        const UNOPPOSED: usize = 0;
        const BLOCKED_BY_PAWN: usize = 1;
        const UNBLOCKED: usize = 2;

        let b = pos.pieces_p(PAWN) & (forward_ranks_bb(us, ksq) | ksq.rank_bb());
        let our_pawns = b & pos.pieces_c(us);
        let their_pawns = b & pos.pieces_c(them);
        let mut safety = if (our_pawns & ksq.file_bb()).0 != 0 {
            Value(5)
        } else {
            Value(-5)
        };

        if (their_pawns.shift(down) & (FILEA_BB | FILEH_BB) & block_ranks & ksq.bb()).0 != 0 {
            safety += 374;
        }

        let center = std::cmp::max(FILE_B, std::cmp::min(FILE_G, ksq.file()));
        for f in (center - 1)..(center + 2) {
            let b = our_pawns & file_bb(f);
            let rk_us = if b != 0 {
                backmost_sq(us, b).relative_rank(us)
            } else {
                RANK_1
            };

            let b = their_pawns & file_bb(f);
            let rk_them = if b != 0 {
                frontmost_sq(them, b).relative_rank(us)
            } else {
                RANK_1
            };

            let d = std::cmp::min(f, FILE_H - f);
            safety += SHELTER_STRENGTH[d as usize][rk_us as usize]
                - STORM_DANGER[if rk_us == RANK_1 {
                    UNOPPOSED
                } else if rk_us == rk_them - 1 {
                    BLOCKED_BY_PAWN
                } else {
                    UNBLOCKED
                }][d as usize][rk_them as usize];
        }

        safety
    }

    // do_king_safety() calculates a bonus for king safety. It is called only
    // when king square changes, which is in about 20% of total king_safety()
    // calls.

    fn do_king_safety<Us: ColorTrait>(&mut self, pos: &Position, ksq: Square) -> Score {
        let us = Us::COLOR;
        self.king_squares[us.0 as usize] = ksq;
        self.castling_rights[us.0 as usize] = pos.castling_rights(us);
        let mut min_king_pawn_distance = 0i32;

        let pawns = pos.pieces_cp(us, PAWN);
        if pawns != 0 {
            while distance_ring_bb(ksq, min_king_pawn_distance) & pawns == 0 {
                min_king_pawn_distance += 1;
            }
            min_king_pawn_distance += 1;
        }

        let mut bonus = self.evaluate_shelter::<Us>(pos, ksq);

        // If we can castle use the bonus after the castling if it is bigger
        if pos.has_castling_right(us | CastlingSide::KING) {
            bonus = std::cmp::max(
                bonus,
                self.evaluate_shelter::<Us>(pos, Square::G1.relative(us)),
            );
        }

        if pos.has_castling_right(us | CastlingSide::QUEEN) {
            bonus = std::cmp::max(
                bonus,
                self.evaluate_shelter::<Us>(pos, Square::C1.relative(us)),
            );
        }

        Score::make(bonus.0, -16 * min_king_pawn_distance)
    }
}

// pawns::init() initializes some tables needed by evaluation.

pub fn init() {
    const SEED: [i32; 8] = [0, 13, 24, 18, 65, 100, 175, 330];

    for opposed in 0..2 {
        for phalanx in 0..2 {
            for support in 0..3 {
                for r in 1..7i32 {
                    let v = 17 * (support as i32)
                        + ((SEED[r as usize]
                            + (if phalanx != 0 {
                                (SEED[(r + 1) as usize] - SEED[r as usize]) / 2
                            } else {
                                0
                            }))
                            >> opposed);
                    unsafe {
                        CONNECTED[opposed as usize][phalanx as usize][support as usize]
                            [r as usize] = Score::make(v, v * (r - 2) / 4);
                    }
                }
            }
        }
    }
}

// pawns::probe() looks up the current position's pawn configuration in the
// pawn hash table. If it is not found, it is computed and stored in the table.

pub fn probe(pos: &Position) -> &mut Entry {
    let key = pos.pawn_key();
    let e = pos.pawns_table[(key.0 & 16383) as usize].get();
    let e: &'static mut Entry = unsafe { &mut *e };

    if e.key == key {
        return e;
    }

    e.key = key;
    e.score = evaluate::<White>(pos, e) - evaluate::<Black>(pos, e);
    e.open_files = (e.semiopen_files[WHITE.0 as usize] & e.semiopen_files[BLACK.0 as usize])
        .count_ones() as i32;
    e.asymmetry = (e.passed_pawns[WHITE.0 as usize].0
        | e.passed_pawns[BLACK.0 as usize].0
        | (e.semiopen_files[WHITE.0 as usize] ^ e.semiopen_files[BLACK.0 as usize]) as u64)
        .count_ones() as i32;

    e
}

fn evaluate<Us: ColorTrait>(pos: &Position, e: &mut Entry) -> Score {
    let us = Us::COLOR;
    let them = if us == WHITE { BLACK } else { WHITE };
    let up = if us == WHITE { NORTH } else { SOUTH };
    let right = if us == WHITE { NORTH_EAST } else { SOUTH_WEST };
    let left = if us == WHITE { NORTH_WEST } else { SOUTH_EAST };

    let mut score = Score::ZERO;

    let our_pawns = pos.pieces_cp(us, PAWN);
    let their_pawns = pos.pieces_cp(them, PAWN);

    e.passed_pawns[us.0 as usize] = Bitboard(0);
    e.pawn_attacks_span[us.0 as usize] = Bitboard(0);
    e.weak_unopposed[us.0 as usize] = 0;
    e.semiopen_files[us.0 as usize] = 0xff;
    e.king_squares[us.0 as usize] = Square::NONE;
    e.pawn_attacks[us.0 as usize] = our_pawns.shift(right) | our_pawns.shift(left);
    e.pawns_on_squares[us.0 as usize][BLACK.0 as usize] = popcount(our_pawns & DARK_SQUARES) as i32;
    e.pawns_on_squares[us.0 as usize][WHITE.0 as usize] =
        popcount(our_pawns & !DARK_SQUARES) as i32;

    // Loop through all pawns of the current color and score each pawn
    for s in pos.square_list(us, PAWN) {
        debug_assert!(pos.piece_on(s) == Piece::make(us, PAWN));

        let f = s.file();

        e.semiopen_files[us.0 as usize] &= !(1 << f);
        e.pawn_attacks_span[us.0 as usize] |= pawn_attack_span(us, s);

        // Flag the pawn
        let opposed = their_pawns & forward_file_bb(us, s);
        let stoppers = their_pawns & passed_pawn_mask(us, s);
        let lever = their_pawns & pawn_attacks(us, s);
        let lever_push = their_pawns & pawn_attacks(us, s + up);
        let doubled = our_pawns & (s - up);
        let neighbours = our_pawns & adjacent_files_bb(f);
        let phalanx = neighbours & s.rank_bb();
        let supported = neighbours & (s - up).rank_bb();

        let backward;

        // A pawn is backward if it is behind all pawns of the same color on
        // the adjacent files and cannot be safely advanced.
        backward = (our_pawns & pawn_attack_span(them, s + up)).0 == 0
            && (stoppers & (lever_push | (s + up))).0 == 0;

        // Passed pawns will be properly scored in evaluation because we need
        // full attack info to evaluate them. Include also not passed pawns
        // which could become passed after one or two pawn pushes.
        if stoppers ^ lever ^ lever_push == 0
            && our_pawns & forward_file_bb(us, s) == 0
            && popcount(supported) >= popcount(lever)  // - 1
            && popcount(phalanx) >= popcount(lever_push)
        {
            e.passed_pawns[us.0 as usize] |= s;
        } else if stoppers ^ (s + up) == 0 && s.relative_rank(us) >= RANK_5 {
            for sq in supported.shift(up) & !their_pawns {
                if !more_than_one(their_pawns & pawn_attacks(us, sq)) {
                    e.passed_pawns[us.0 as usize] |= s;
                }
            }
        }

        // Score this pawn
        if supported | phalanx != 0 {
            score += unsafe {
                CONNECTED[(opposed != 0) as usize][(phalanx != 0) as usize]
                    [popcount(supported) as usize][s.relative_rank(us) as usize]
            };
        } else if neighbours == 0 {
            score -= ISOLATED;
            e.weak_unopposed[us.0 as usize] += (opposed == 0) as i32;
        } else if backward {
            score -= BACKWARD;
            e.weak_unopposed[us.0 as usize] += (opposed == 0) as i32;
        }

        if doubled != 0 && supported == 0 {
            score -= DOUBLED;
        }
    }

    score
}
