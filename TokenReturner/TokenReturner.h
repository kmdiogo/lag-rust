//
// Created by Kenny on 2/13/2019.
//

#ifndef LAG_TOKENRECOGNIZER_H
#define LAG_TOKENRECOGNIZER_H

#include <iostream>
#include <fstream>
#include <utility>
#include <cstdlib>
using namespace std;

enum Tokens {Class, Token, Id, Ignore,
        SetStart, SetStartNegate, SetEnd, DashSetEnd,
        OpenParen, CloseParen, Slash, Pipe,
        Character, Dash, Star, Plus, Question, EOI};

pair<Tokens, string> getNextToken(ifstream &file, bool aggregrate);

pair<Tokens, string> getCTII(ifstream &file, char cur);


#endif //LAG_TOKENRECOGNIZER_H
