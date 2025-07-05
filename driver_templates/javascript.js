// @formatter:off
class LexerError extends Error {
    constructor(message) {
        super(message);
        this.name = "LexerError";
    }
}

const token = {
    EOI: Symbol("EOI"),
'__TOKEN_ENTRIES__'
};

class TokenEntry {
    constructor(tokenType, lexeme) {
        this.token = tokenType;
        this.lexeme = lexeme;
    }
}

const stateTokenMapping = '__STATE_TOKEN_MAPPING__'

export class Lexer {
    constructor(dfa) {
        this.accepting = dfa.accepting;
        this.entry = dfa.entry;
        this.states = dfa.states;
        this.prevInput = "";
        this.inputPos = -1;
    }

    nextState(currentState, inputChar) {
        const transitionTable = this.states[currentState];
        return transitionTable[inputChar] ?? null;
    }

    readInput(inputIter) {
        if (this.inputPos === this.prevInput.length - 1) {
            const next = inputIter.next();
            if (next.done) return null;
            const char = next.value;
            this.prevInput += char;
            this.inputPos += 1;
            return char;
        } else {
            this.inputPos += 1;
            return this.prevInput[this.inputPos];
        }
    }

    getToken(inputIter) {
        let state = this.entry;
        let lastAccepting = null;
        let returnToken = null;
        let lexeme = "";

        const resetLexer = () => {
            state = this.entry;
            lastAccepting = null;
            returnToken = null;
            lexeme = "";
        };

        while (true) {
            const char = this.readInput(inputIter);
            if (char === null) {
                returnToken = token.EOI;
                break;
            }

            if (char.length > 1) {
                throw new Error(`Next string '${char}' is not a single character.`);
            }

            lexeme += char;
            const nextState = this.nextState(state, char);
            if (nextState === null) break;

            state = nextState;
            if (this.accepting.hasOwnProperty(state)) {
                const tokenStr = this.accepting[state][0];
                if (tokenStr === "!") {
                    resetLexer();
                    continue;
                }
                lastAccepting = [stateTokenMapping[tokenStr], lexeme, this.inputPos];
            }
        }

        if (returnToken !== null && lastAccepting === null) {
            return new TokenEntry(returnToken, lexeme);
        }

        if (lastAccepting !== null) {
            const [lastToken, lastTokenLexeme, lastInputPos] = lastAccepting;
            this.inputPos = lastInputPos;
            return new TokenEntry(lastToken, lastTokenLexeme);
        }

        throw new LexerError(
            "No token found. Ensure all characters are covered by the DFA rules."
        );
    }
}
// @formatter:on
