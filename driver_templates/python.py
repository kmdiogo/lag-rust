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
        self.accepting: dict = dfa["accepting"]
        self.entry: str = dfa["entry"]
        self.states: dict = dfa["states"]
        self.prev_input: str = ""
        self.input_pos = -1

    def _next_state(self, current_state: str, input_char: str) -> str | None:
        """
        Move to next state based on transition table
        """
        transition_table: dict = self.states[current_state]
        if input_char in transition_table:
            return transition_table[input_char]

        return None

    def _read_input(self, input_stream: Generator[str, Any, Any]) -> str | None:
        # If at the most recent input (haven't rewinded) get next char from input stream
        if self.input_pos == len(self.prev_input) - 1:
            try:
                char = next(input_stream)
            except StopIteration:
                return None
            self.prev_input += char
            return_char = char
        # Lexer has rewinded, get input from cached input array
        else:
            return_char = self.prev_input[self.input_pos + 1]

        self.input_pos += 1
        return return_char


    def get_token(self, input: Generator[str, Any, Any]) -> TokenEntry:
        state: str = self.entry
        last_accepting: tuple[Token, str, int] | None = None
        return_token: Token | None = None
        lexeme = ""
        def reset_lexer():
            nonlocal state, last_accepting, return_token, lexeme
            state = self.entry
            last_accepting = None
            return_token = None
            lexeme = ""

        while True:
            char = self._read_input(input)
            if char is None:
                return_token = Token.EOI
                break

            if len(char) > 1:
                raise ValueError(f"Next string '{char}' in the input generator is not a single length character.")
            lexeme += char

            next_state = self._next_state(state, char)
            if next_state is None:
                break

            state = next_state
            if state in self.accepting:
                token_str = self.accepting[state][0]
                # Ignore token found, reset and move to next input
                if token_str == '!':
                    reset_lexer()
                    continue
                last_accepting = (STATE_TOKEN_MAPPING[token_str], lexeme, self.input_pos)

        if return_token is not None and last_accepting is None:
            return TokenEntry(return_token, lexeme)
        if last_accepting is not None:
            last_token, last_token_lexeme, last_input_pos = last_accepting
            self.input_pos = last_input_pos
            return TokenEntry(last_token, last_token_lexeme)

        raise LexerError(f"No token found by lexer. Ensure all characters in the language have been covered by the lexer generator rules.")

# @formatter:on