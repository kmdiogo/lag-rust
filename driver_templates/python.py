# @formatter:off
from enum import auto, Enum
from typing import Generator, Any


class LexerError(Exception):
    def __init__(self, message: str):
        self.message = message


class Token(Enum):
    EOI = auto()
'__TOKEN_ENTRIES__'


class TokenEntry:
    def __init__(self, token: Token, lexeme: str):
        self.token = token
        self.lexeme = lexeme


STATE_TOKEN_MAPPING: dict[str, Token] = '__STATE_TOKEN_MAPPING__'


class Lexer:
    def __init__(self, dfa):
        self.accepting = dfa["accepting"]
        self.entry: str = dfa["entry"]
        self.states = dfa["states"]

    def get_token(self, input: Generator[str, Any, Any]) -> TokenEntry:
        state: str = self.entry
        last_accepting: tuple[Token, str] | None = None
        return_token: Token | None = None
        lexeme = ""
        def reset_lexer():
            nonlocal state, last_accepting, return_token, lexeme
            state = self.entry
            last_accepting = None
            return_token = None
            lexeme = ""

        while True:
            try:
                char = next(input)
            except StopIteration:
                return_token = Token.EOI
                break

            if len(char) > 1:
                raise ValueError(f"Next string '{char}' in the input generator is not a single length character.")
            lexeme += char

            if char not in self.states[state]:
                break

            state = self.states[state][char]
            if state in self.accepting:
                token_str = self.accepting[state][0]
                if token_str == '!':
                    reset_lexer()
                    continue
                last_accepting = (STATE_TOKEN_MAPPING[token_str], lexeme)

        if return_token is not None and last_accepting is None:
            return TokenEntry(return_token, lexeme)
        if last_accepting is not None:
            return TokenEntry(last_accepting[0], last_accepting[1])

        raise LexerError(f"No token found by lexer. Ensure all characters in the language have been covered by the lexer generator rules.")

# @formatter:on