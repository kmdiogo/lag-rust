# @formatter:off
from enum import auto, Enum
from typing import Generator, Any


class LexerError(Exception):
    def __init__(self, message: str):
        self.message = message


class Token(Enum):
    EOI = auto()
    ATOD = auto()
    JUSTB = auto()



class TokenEntry:
    def __init__(self, token: Token, lexeme: str):
        self.token = token
        self.lexeme = lexeme


STATE_TOKEN_MAPPING: dict[str, Token] = {
    "ATOD": Token.ATOD,
    "JUSTB": Token.JUSTB,

}


class Lexer:
    def __init__(self, dfa):
        self.accepting: dict = dfa["accepting"]
        self.entry: str = dfa["entry"]
        self.states: dict = dfa["states"]
        self.class_sets: dict = dfa["class_sets"]

    def _next_state(self, current_state: str, input_char: str) -> str | None:
        transition_table: dict = self.states[current_state]
        if input_char in transition_table:
            return transition_table[input_char]

        # Check if input character is in any of possible class sets
        for possible_input_symbol, transition_state in transition_table.items():
            if not possible_input_symbol.startswith("[") or not possible_input_symbol.endswith("]"):
                continue
            for class_id, class_set_entry in self.class_sets.items():
                if possible_input_symbol != class_id:
                    continue
                if class_set_entry["exclude"] and input_char not in class_set_entry["chars"]:
                    return transition_table[possible_input_symbol]
                if input_char in class_set_entry["chars"]:
                    return transition_table[possible_input_symbol]

        return None

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

            next_state = self._next_state(state, char)
            if next_state is None:
                break

            state = next_state
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