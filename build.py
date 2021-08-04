import argparse
import os

THREADS = 3

GAMES = 30
TIME = 10.0
INC = 0.0

OPENINGS = (
    "rnbqkb1r/pppppp1p/5np1/8/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkb1r/pppp1ppp/4pn2/8/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkbnr/pp2pppp/2p5/3p4/2PP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkb1r/ppp1pppp/5n2/8/2pP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4",
    "rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3",
    "r1bqkbnr/pp1ppppp/2n5/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
    "rnbqkbnr/ppp2ppp/4p3/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkbnr/pp1p1ppp/4p3/2p5/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
    "rnbqkb1r/ppp1pppp/3p1n2/8/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 1 3",
    "rnbqkbnr/pp2pppp/2p5/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
    "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
    "r1bqkbnr/pppp1ppp/2n5/8/3pP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4",
    "rnbqkb1r/ppp2ppp/3p4/8/4n3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 5",
    "rnbqkbnr/ppp2ppp/3p4/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3",
    "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4",
    "rnbqkbnr/pp2pppp/8/2pp4/8/5NP1/PPPPPP1P/RNBQKB1R w KQkq - 0 3",
    "rnbqkb1r/ppp1pppp/5n2/3p4/3P4/5N2/PPP1PPPP/RNBQKB1R w KQkq - 1 3",
    "rnbqkbnr/ppp2ppp/4p3/3p4/2P5/5N2/PP1PPPPP/RNBQKB1R w KQkq - 0 3",
    "rnbqkbnr/ppp1pppp/8/8/2Pp4/5N2/PP1PPPPP/RNBQKB1R w KQkq - 0 3",
    "rnbqkb1r/pppppp1p/5np1/8/2P5/5N2/PP1PPPPP/RNBQKB1R w KQkq - 0 3",
    "rnbqkb1r/pppppp1p/5np1/8/2P5/2N5/PP1PPPPP/R1BQKBNR w KQkq - 0 3",
    "rnbqkb1r/pppp1ppp/5n2/4p3/2P5/2N5/PP1PPPPP/R1BQKBNR w KQkq - 2 3",
    "rnbq1rk1/ppppppbp/5np1/8/8/5NP1/PPPPPPBP/RNBQ1RK1 w - - 4 5",
    "rnbqkb1r/pp2pppp/2p2n2/3p4/8/5NP1/PPPPPPBP/RNBQK2R w KQkq - 0 4",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
)


def run_once(id_: int, q):
    import chess.engine
    import time
    import random

    stat = [0, 0, 0]
    for g in range(GAMES):
        e1 = chess.engine.SimpleEngine.popen_uci("./target/release/rustfish")
        e2 = chess.engine.SimpleEngine.popen_uci("./rustfish")

        reverse = g % 2 == 1

        if reverse:
            e1, e2 = e2, e1

        times = [TIME, TIME]
        engines = [e1, e2]
        color = 0

        board = chess.Board(fen=random.choice(OPENINGS))
        draw = 0.0
        while not board.is_game_over():
            now = time.time()
            result = engines[color].play(board, chess.engine.Limit(
                white_clock=times[0], black_clock=times[1],
                white_inc=INC, black_inc=INC,
            ), info=chess.engine.INFO_SCORE)
            # print(f"{['White', 'Black'][color]} played {result.move} score {result.info['score']}")
            # print(f"Times: {times}")
            board.push(result.move)
            times[color] += INC - (time.time() - now)
            if times[color] <= 0.01:
                times[color] = 0.01
            color ^= 1

            if 'score' in result.info:
                if len(board.move_stack) > 80:
                    if -5 < result.info['score'].white().score(mate_score=100000) < 5:
                        draw += 0.5
                    else:
                        draw = 0.0

                if draw >= 10.0:
                    break

        e1.close()
        e2.close()

        print(f"GAME {GAMES * id_ + g + 1} Finished")

        if draw >= 10.0:
            print("Adjunct Draw")
            print("1/2-1/2")
        else:
            print(board.result())

        print(board.fen())
        print()

        if draw >= 10.0 or board.result() == "1/2-1/2":
            stat[1] += 1
        elif board.result() == "1-0":
            if reverse:
                stat[2] += 1
            else:
                stat[0] += 1
        elif board.result() == "0-1":
            if reverse:
                stat[0] += 1
            else:
                stat[2] += 1

    q.put(stat)

    return stat


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Build Rustfish')
    parser.add_argument('--test', action='store_const', dest="test", default=False, const=True)

    if "RUSTFLAGS" not in os.environ:
        os.environ["RUSTFLAGS"] = ""

    NATIVE = "-C target-cpu=native"

    os.environ["RUSTFLAGS"] = ' '.join((NATIVE,))

    print("NEW RUSTFLAGS: " + repr(os.environ["RUSTFLAGS"]))

    command = "cargo build --release"

    print("building with " + repr(command))

    os.system(command)

    os.environ["RUSTFLAGS"] = ""

    print("RESTORED RUSTFLAGS TO " + repr(os.environ["RUSTFLAGS"]))

    args = parser.parse_args()
    if args.test:
        from multiprocessing import Process, Queue

        queues = []

        for i in range(THREADS):
            q = Queue()

            p = Process(target=run_once, args=(i, q))
            p.start()

            queues.append(q)

        stats = [0, 0, 0]

        for q in queues:
            res = q.get(block=True)
            stats[0] += res[0]
            stats[1] += res[1]
            stats[2] += res[2]

        print(f"NEW VERSION: win {stats[0]} draw {stats[1]} lose {stats[2]}")
