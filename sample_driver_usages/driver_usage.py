from driver import Lexer

if __name__ == "__main__":
    from pathlib import Path
    import json


    def input_generator():
        for c in "abaaaacdb a":
            yield c


    dfa = json.loads(Path("../states.json").read_text())
    lexer = Lexer(dfa)
    gen = input_generator()
    for i in range(10):
        token = lexer.get_token(gen)
        print(token.token, token.lexeme)
