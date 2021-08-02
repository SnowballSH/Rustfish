import argparse
import os

THREADS = 4

GAMES = 7
TIME = 60.0
INC = 0.0


def run_once(id_: int, q):
    import chess.engine
    import time

    stat = [0.0, 0.0]
    for g in range(GAMES):
        e1 = chess.engine.SimpleEngine.popen_uci("./target/release/rustfish")
        e2 = chess.engine.SimpleEngine.popen_uci("./rustfish")

        reverse = g % 2 == 1

        if reverse:
            e1, e2 = e2, e1

        times = [TIME, TIME]
        engines = [e1, e2]
        color = 0

        board = chess.Board()
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
            color ^= 1

            if 'score' in result.info:
                if len(board.move_stack) > 80:
                    if -10 < result.info['score'].white().score(mate_score=100000) < 10:
                        draw += 0.5
                    else:
                        draw = 0.0

                if draw >= 3.0:
                    break

        e1.close()
        e2.close()

        print(f"GAME {GAMES * id_ + g + 1} Finished")

        if draw >= 3.0:
            print("Adjunct Draw")
            print("1/2-1/2")
        else:
            print(board.result())

        print(board.fen())
        print()

        if draw >= 3.0 or board.result() == "1/2-1/2":
            stat[0] += 0.5
            stat[1] += 0.5
        elif board.result() == "1-0":
            if reverse:
                stat[0] += 0
                stat[1] += 1
            else:
                stat[0] += 1
                stat[1] += 0
        elif board.result() == "0-1":
            if reverse:
                stat[0] += 1
                stat[1] += 0
            else:
                stat[0] += 0
                stat[1] += 1

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

        stats = [0.0, 0.0]

        for q in queues:
            res = q.get(block=True)
            stats[0] += res[0]
            stats[1] += res[1]

        print(stats)
