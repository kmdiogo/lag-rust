//
// Created by Kenny on 2/13/2019.
//
#include "TokenReturner.h"
using namespace std;

pair<Tokens, string> getNextToken(ifstream &file, bool aggregrate) {
    char cur, lookahead;
    while (file >> cur) {
        lookahead = file.peek();
        if (!aggregrate) {
            if (cur == '[' && lookahead == '^') {
                file.get();
                return make_pair(SetStartNegate, "[^");
            }
            else if (cur == '[')
                return make_pair(SetStart, string() + cur);
            else if (cur == '-' && lookahead == ']') {
                file.get();
                return make_pair(DashSetEnd, "-]");
            }
            else if (cur == ']') {
                return make_pair(SetEnd, "]");
            }
            else if (cur == '(') {
                return make_pair(OpenParen, "(");
            }
            else if (cur == ')') {
                return make_pair(CloseParen, ")");
            }
            else if (cur == '/') {
                return make_pair(Slash, "/");
            }
            else if (cur == '*') {
                return make_pair(Star, "*");
            }
            else if (cur == '+') {
                return make_pair(Plus, "+");
            }
            else if (cur == '?') {
                return make_pair(Question, "?");
            }
            else if (cur == '-') {
                return make_pair(Dash, string() + cur);
            }
            else if (cur == '|') {
                return make_pair(Pipe, "|");
            }
            else if (cur == '\\') {
                file.get();
                if (lookahead == 'n')
                    return make_pair(Character, "\n");
                else if (lookahead == 't')
                    return make_pair(Character, "\t");
                else if (lookahead == 'f')
                    return make_pair(Character, "\f");
                else if (lookahead == 'v')
                    return make_pair(Character, "\v");
                else if (lookahead == 'r')
                    return make_pair(Character, "\r");
                return make_pair(Character, string() + lookahead);
            }
            else {
                return make_pair(Character, string() + cur);
            }
        }
        else {
            if (cur == '/' && lookahead == '/') {
                while (true) {
                    file.get(cur);
                    if (cur == '\n')
                        break;
                }
            }
            if ( (isalnum(cur) || cur == '_'))
                return getCTII(file, cur);
        }
    }
    return make_pair(EOI, "$");
}


pair<Tokens, string> getCTII(ifstream &file, char cur) {
    char c;
    string lexeme = "";
    lexeme += cur;

    while (isalnum(file.peek()) || file.peek() == '_') {
        file >> c;
        lexeme += c;
    }

    /*string temp;
    file >> temp;
    string lexeme = cur + temp;*/
    if (lexeme == "class")
        return make_pair(Class, lexeme);
    else if (lexeme == "token")
        return make_pair(Token, lexeme);
    else if (lexeme == "ignore")
        return make_pair(Ignore, lexeme);
    else {
        if (!isalpha(lexeme[0]) && lexeme[0] != '_') {
            cout << "Invalid identifier: " << lexeme << endl;
            exit(0);
        }
        for (int i=1; i < lexeme.length(); i++) {
            if (!isalnum(lexeme[i]) && lexeme[i] != '_') {
                cout << "Invalid identifier: " << lexeme << endl;
                exit(0);
            }
        }
        return make_pair(Id, lexeme);
    }
}
