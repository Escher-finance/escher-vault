/* A Bison parser, made by GNU Bison 3.8.2.  */

/* Bison interface for Yacc-like parsers in C

   Copyright (C) 1984, 1989-1990, 2000-2015, 2018-2021 Free Software Foundation,
   Inc.

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.

   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* DO NOT RELY ON FEATURES THAT ARE NOT DOCUMENTED in the manual,
   especially those whose name start with YY_ or yy_.  They are
   private implementation details that can be changed or removed.  */

#ifndef YY_YY_PARSER_TAB_HH_INCLUDED
# define YY_YY_PARSER_TAB_HH_INCLUDED
/* Debug traces.  */
#ifndef YYDEBUG
# define YYDEBUG 0
#endif
#if YYDEBUG
extern int yydebug;
#endif
/* "%code requires" blocks.  */
#line 13 "../parser.y"


#ifndef BISON_HEADER
#define BISON_HEADER

#include <variant>

#include "nix/util/finally.hh"
#include "nix/util/util.hh"
#include "nix/util/users.hh"

#include "nix/expr/nixexpr.hh"
#include "nix/expr/eval.hh"
#include "nix/expr/eval-settings.hh"
#include "nix/expr/parser-state.hh"

// Bison seems to have difficulty growing the parser stack when using C++ with
// a custom location type. This undocumented macro tells Bison that our
// location type is "trivially copyable" in C++-ese, so it is safe to use the
// same memcpy macro it uses to grow the stack that it uses with its own
// default location type. Without this, we get "error: memory exhausted" when
// parsing some large Nix files. Our other options are to increase the initial
// stack size (200 by default) to be as large as we ever want to support (so
// that growing the stack is unnecessary), or redefine the stack-relocation
// macro ourselves (which is also undocumented).
#define YYLTYPE_IS_TRIVIAL 1

#define YY_DECL int yylex \
    (YYSTYPE * yylval_param, YYLTYPE * yylloc_param, yyscan_t yyscanner, nix::ParserState * state)

// For efficiency, we only track offsets; not line,column coordinates
# define YYLLOC_DEFAULT(Current, Rhs, N)                                \
    do                                                                  \
      if (N)                                                            \
        {                                                               \
          (Current).beginOffset = YYRHSLOC (Rhs, 1).beginOffset;        \
          (Current).endOffset  = YYRHSLOC (Rhs, N).endOffset;           \
        }                                                               \
      else                                                              \
        {                                                               \
          (Current).beginOffset = (Current).endOffset =                 \
            YYRHSLOC (Rhs, 0).endOffset;                                \
        }                                                               \
    while (0)

namespace nix {

typedef std::unordered_map<PosIdx, DocComment> DocCommentMap;

Expr * parseExprFromBuf(
    char * text,
    size_t length,
    Pos::Origin origin,
    const SourcePath & basePath,
    SymbolTable & symbols,
    const EvalSettings & settings,
    PosTable & positions,
    DocCommentMap & docComments,
    const ref<SourceAccessor> rootFS,
    const Expr::AstSymbols & astSymbols);

}

#endif


#line 116 "parser-tab.hh"

/* Token kinds.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
  enum yytokentype
  {
    YYEMPTY = -2,
    YYEOF = 0,                     /* "end of file"  */
    YYerror = 256,                 /* error  */
    YYUNDEF = 257,                 /* "invalid token"  */
    ID = 258,                      /* ID  */
    STR = 259,                     /* STR  */
    IND_STR = 260,                 /* IND_STR  */
    INT_LIT = 261,                 /* INT_LIT  */
    FLOAT_LIT = 262,               /* FLOAT_LIT  */
    PATH = 263,                    /* PATH  */
    HPATH = 264,                   /* HPATH  */
    SPATH = 265,                   /* SPATH  */
    PATH_END = 266,                /* PATH_END  */
    URI = 267,                     /* URI  */
    IF = 268,                      /* IF  */
    THEN = 269,                    /* THEN  */
    ELSE = 270,                    /* ELSE  */
    ASSERT = 271,                  /* ASSERT  */
    WITH = 272,                    /* WITH  */
    LET = 273,                     /* LET  */
    IN_KW = 274,                   /* IN_KW  */
    REC = 275,                     /* REC  */
    INHERIT = 276,                 /* INHERIT  */
    EQ = 277,                      /* EQ  */
    NEQ = 278,                     /* NEQ  */
    AND = 279,                     /* AND  */
    OR = 280,                      /* OR  */
    IMPL = 281,                    /* IMPL  */
    OR_KW = 282,                   /* OR_KW  */
    PIPE_FROM = 283,               /* PIPE_FROM  */
    PIPE_INTO = 284,               /* PIPE_INTO  */
    DOLLAR_CURLY = 285,            /* DOLLAR_CURLY  */
    IND_STRING_OPEN = 286,         /* IND_STRING_OPEN  */
    IND_STRING_CLOSE = 287,        /* IND_STRING_CLOSE  */
    ELLIPSIS = 288,                /* ELLIPSIS  */
    LEQ = 289,                     /* LEQ  */
    GEQ = 290,                     /* GEQ  */
    UPDATE = 291,                  /* UPDATE  */
    NOT = 292,                     /* NOT  */
    CONCAT = 293,                  /* CONCAT  */
    NEGATE = 294                   /* NEGATE  */
  };
  typedef enum yytokentype yytoken_kind_t;
#endif

/* Value type.  */
#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
union YYSTYPE
{
#line 122 "../parser.y"

  // !!! We're probably leaking stuff here.
  nix::Expr * e;
  nix::ExprList * list;
  nix::ExprAttrs * attrs;
  nix::Formals * formals;
  nix::Formal * formal;
  nix::NixInt n;
  nix::NixFloat nf;
  nix::StringToken id; // !!! -> Symbol
  nix::StringToken path;
  nix::StringToken uri;
  nix::StringToken str;
  std::vector<nix::AttrName> * attrNames;
  std::vector<std::pair<nix::AttrName, nix::PosIdx>> * inheritAttrs;
  std::vector<std::pair<nix::PosIdx, nix::Expr *>> * string_parts;
  std::vector<std::pair<nix::PosIdx, std::variant<nix::Expr *, nix::StringToken>>> * ind_string_parts;

#line 191 "parser-tab.hh"

};
typedef union YYSTYPE YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define YYSTYPE_IS_DECLARED 1
#endif

/* Location type.  */
typedef  ::nix::ParserLocation  YYLTYPE;




int yyparse (void * scanner, nix::ParserState * state);


#endif /* !YY_YY_PARSER_TAB_HH_INCLUDED  */
