# Plays several matches between two versions of Shakmat using different openings,
# and reports on the final results, both in term of scores and average move speed.
import requests as rq
from datetime import datetime

OLD_VER = {"port": 8000, "name": "v1"}
NEW_VER = {"port": 8001, "name": "v2"}

class ShakmatVer:
    def __init__(self, port, name):
        self.name = name
        self.port = port
        self.current_game = None
        self.move_speeds = {}
        self.scores = {k: {s: 0 for s in ("win", "lose", "draw")} for k in ("black", "white")}

    def create_game(self):
        resp = rq.post(f"http://127.0.0.1:{self.port}/games").json()
        self.current_game = resp["key"]

    def make_move(self, move):
        url = f"http://127.0.0.1:{self.port}/games/{self.current_game}/move"
        resp = rq.post(url, json={"move": move})
        assert resp.status_code == 200
        return resp.json()["turn_info"]

    def get_best_move(self):
        url = f"http://127.0.0.1:{self.port}/games/{self.current_game}/move_suggestion"
        return rq.get(url).json()["move"]

    def update_score(self, color, result):
        assert color in ("black", "white")
        assert result in ("win", "lose", "draw")
        self.scores[color][result] += 1

    def update_moving_time(self, ply, time):
        self.move_speeds[ply] = self.move_speeds.get(ply, []) + [time]


class Match:
    def __init__(self, white, black, opening):
        self.white = white
        self.black = black
        self.opening_line = opening
        self.ply = 0

        self.white.create_game()
        self.black.create_game()

    def play(self):
        while True:
            (moving_player, other_player) = (self.white, self.black) if self.ply % 2 == 0 \
                                            else (self.black, self.white)
            
            move = None
            if self.ply < len(self.opening_line):
                # Inside the opening line, grab the best move from there
                move = self.opening_line[self.ply]
            else:
                # Not inside the opening line, ask the engine for the best move
                # and log the time it takes to reply
                t1 = datetime.now()
                move = moving_player.get_best_move()
                t2 = datetime.now()
                elapsed = (t2 - t1).total_seconds()

                moving_player.update_moving_time(self.ply, elapsed)
            
            # Make the move on both sides
            moving_player.make_move(move)
            turn_info = other_player.make_move(move)

            if not turn_info["moves"]:
                # No moves available, check whether this is checkmate or a draw
                if turn_info["in_check"]:
                    # Checkmate
                    if moving_player is self.white:
                        # White checkmated black
                        self.white.update_score("white", "win")
                        self.black.update_score("black", "lose")
                        return "1-0"
                    else:
                        # Black checkmated white
                        self.white.update_score("white", "lose")
                        self.black.update_score("black", "win")
                        return "0-1"
                else:
                    # Draw
                    self.white.update_score("white", "draw")
                    self.black.update_score("black", "draw")
                    return "1/2-1/2"

            self.ply += 1

old_engine = ShakmatVer(OLD_VER["port"], OLD_VER["name"])
new_engine = ShakmatVer(NEW_VER["port"], NEW_VER["name"])

with open("openings.txt", "r") as f:
    for i, line in enumerate(f, start=1):
        opening_line = line.strip().split(" ")

        print(f"Opening {i}, game 1... ", end="", flush=True)
        print(Match(old_engine, new_engine, opening_line).play(), flush=True)

        print(f"Opening {i}, game 2... ", end="", flush=True)
        print(Match(new_engine, old_engine, opening_line).play(), flush=True)


print(old_engine.scores)
print(new_engine.scores)