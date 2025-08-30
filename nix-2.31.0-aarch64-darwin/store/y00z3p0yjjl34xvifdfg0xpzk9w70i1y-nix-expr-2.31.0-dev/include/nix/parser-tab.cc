/* A Bison parser, made by GNU Bison 3.8.2.  */

/* Bison implementation for Yacc-like parsers in C

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

/* C LALR(1) parser skeleton written by Richard Stallman, by
   simplifying the original so-called "semantic" parser.  */

/* DO NOT RELY ON FEATURES THAT ARE NOT DOCUMENTED in the manual,
   especially those whose name start with YY_ or yy_.  They are
   private implementation details that can be changed or removed.  */

/* All symbols defined below should begin with yy or YY, to avoid
   infringing on user name space.  This should be done even for local
   variables, as they might otherwise be expanded by user macros.
   There are some unavoidable exceptions within include files to
   define necessary library symbols; they are noted "INFRINGES ON
   USER NAME SPACE" below.  */

/* Identify Bison output, and Bison version.  */
#define YYBISON 30802

/* Bison version string.  */
#define YYBISON_VERSION "3.8.2"

/* Skeleton name.  */
#define YYSKELETON_NAME "yacc.c"

/* Pure parsers.  */
#define YYPURE 1

/* Push parsers.  */
#define YYPUSH 0

/* Pull parsers.  */
#define YYPULL 1




/* First part of user prologue.  */
#line 80 "../parser.y"


#include "parser-tab.hh"
#include "lexer-tab.hh"

YY_DECL;

using namespace nix;

#define CUR_POS state->at(yyloc)


void yyerror(YYLTYPE * loc, yyscan_t scanner, ParserState * state, const char * error)
{
    if (std::string_view(error).starts_with("syntax error, unexpected end of file")) {
        loc->beginOffset = loc->endOffset;
    }
    throw ParseError({
        .msg = HintFmt(error),
        .pos = state->positions[state->at(*loc)]
    });
}

#define SET_DOC_POS(lambda, pos) setDocPosition(state->lexerState, lambda, state->at(pos))
static void setDocPosition(const LexerState & lexerState, ExprLambda * lambda, PosIdx start) {
    auto it = lexerState.positionToDocComment.find(start);
    if (it != lexerState.positionToDocComment.end()) {
        lambda->setDocComment(it->second);
    }
}

static Expr * makeCall(PosIdx pos, Expr * fn, Expr * arg) {
    if (auto e2 = dynamic_cast<ExprCall *>(fn)) {
        e2->args.push_back(arg);
        return fn;
    }
    return new ExprCall(pos, fn, {arg});
}



#line 113 "parser-tab.cc"

# ifndef YY_CAST
#  ifdef __cplusplus
#   define YY_CAST(Type, Val) static_cast<Type> (Val)
#   define YY_REINTERPRET_CAST(Type, Val) reinterpret_cast<Type> (Val)
#  else
#   define YY_CAST(Type, Val) ((Type) (Val))
#   define YY_REINTERPRET_CAST(Type, Val) ((Type) (Val))
#  endif
# endif
# ifndef YY_NULLPTR
#  if defined __cplusplus
#   if 201103L <= __cplusplus
#    define YY_NULLPTR nullptr
#   else
#    define YY_NULLPTR 0
#   endif
#  else
#   define YY_NULLPTR ((void*)0)
#  endif
# endif

#include "parser-tab.hh"
/* Symbol kind.  */
enum yysymbol_kind_t
{
  YYSYMBOL_YYEMPTY = -2,
  YYSYMBOL_YYEOF = 0,                      /* "end of file"  */
  YYSYMBOL_YYerror = 1,                    /* error  */
  YYSYMBOL_YYUNDEF = 2,                    /* "invalid token"  */
  YYSYMBOL_ID = 3,                         /* ID  */
  YYSYMBOL_STR = 4,                        /* STR  */
  YYSYMBOL_IND_STR = 5,                    /* IND_STR  */
  YYSYMBOL_INT_LIT = 6,                    /* INT_LIT  */
  YYSYMBOL_FLOAT_LIT = 7,                  /* FLOAT_LIT  */
  YYSYMBOL_PATH = 8,                       /* PATH  */
  YYSYMBOL_HPATH = 9,                      /* HPATH  */
  YYSYMBOL_SPATH = 10,                     /* SPATH  */
  YYSYMBOL_PATH_END = 11,                  /* PATH_END  */
  YYSYMBOL_URI = 12,                       /* URI  */
  YYSYMBOL_IF = 13,                        /* IF  */
  YYSYMBOL_THEN = 14,                      /* THEN  */
  YYSYMBOL_ELSE = 15,                      /* ELSE  */
  YYSYMBOL_ASSERT = 16,                    /* ASSERT  */
  YYSYMBOL_WITH = 17,                      /* WITH  */
  YYSYMBOL_LET = 18,                       /* LET  */
  YYSYMBOL_IN_KW = 19,                     /* IN_KW  */
  YYSYMBOL_REC = 20,                       /* REC  */
  YYSYMBOL_INHERIT = 21,                   /* INHERIT  */
  YYSYMBOL_EQ = 22,                        /* EQ  */
  YYSYMBOL_NEQ = 23,                       /* NEQ  */
  YYSYMBOL_AND = 24,                       /* AND  */
  YYSYMBOL_OR = 25,                        /* OR  */
  YYSYMBOL_IMPL = 26,                      /* IMPL  */
  YYSYMBOL_OR_KW = 27,                     /* OR_KW  */
  YYSYMBOL_PIPE_FROM = 28,                 /* PIPE_FROM  */
  YYSYMBOL_PIPE_INTO = 29,                 /* PIPE_INTO  */
  YYSYMBOL_DOLLAR_CURLY = 30,              /* DOLLAR_CURLY  */
  YYSYMBOL_IND_STRING_OPEN = 31,           /* IND_STRING_OPEN  */
  YYSYMBOL_IND_STRING_CLOSE = 32,          /* IND_STRING_CLOSE  */
  YYSYMBOL_ELLIPSIS = 33,                  /* ELLIPSIS  */
  YYSYMBOL_34_ = 34,                       /* '<'  */
  YYSYMBOL_35_ = 35,                       /* '>'  */
  YYSYMBOL_LEQ = 36,                       /* LEQ  */
  YYSYMBOL_GEQ = 37,                       /* GEQ  */
  YYSYMBOL_UPDATE = 38,                    /* UPDATE  */
  YYSYMBOL_NOT = 39,                       /* NOT  */
  YYSYMBOL_40_ = 40,                       /* '+'  */
  YYSYMBOL_41_ = 41,                       /* '-'  */
  YYSYMBOL_42_ = 42,                       /* '*'  */
  YYSYMBOL_43_ = 43,                       /* '/'  */
  YYSYMBOL_CONCAT = 44,                    /* CONCAT  */
  YYSYMBOL_45_ = 45,                       /* '?'  */
  YYSYMBOL_NEGATE = 46,                    /* NEGATE  */
  YYSYMBOL_47_ = 47,                       /* ':'  */
  YYSYMBOL_48_ = 48,                       /* '@'  */
  YYSYMBOL_49_ = 49,                       /* ';'  */
  YYSYMBOL_50_ = 50,                       /* '!'  */
  YYSYMBOL_51_ = 51,                       /* '.'  */
  YYSYMBOL_52_ = 52,                       /* '"'  */
  YYSYMBOL_53_ = 53,                       /* '('  */
  YYSYMBOL_54_ = 54,                       /* ')'  */
  YYSYMBOL_55_ = 55,                       /* '{'  */
  YYSYMBOL_56_ = 56,                       /* '}'  */
  YYSYMBOL_57_ = 57,                       /* '['  */
  YYSYMBOL_58_ = 58,                       /* ']'  */
  YYSYMBOL_59_ = 59,                       /* '='  */
  YYSYMBOL_60_ = 60,                       /* ','  */
  YYSYMBOL_YYACCEPT = 61,                  /* $accept  */
  YYSYMBOL_start = 62,                     /* start  */
  YYSYMBOL_expr = 63,                      /* expr  */
  YYSYMBOL_expr_function = 64,             /* expr_function  */
  YYSYMBOL_expr_if = 65,                   /* expr_if  */
  YYSYMBOL_expr_pipe_from = 66,            /* expr_pipe_from  */
  YYSYMBOL_expr_pipe_into = 67,            /* expr_pipe_into  */
  YYSYMBOL_expr_op = 68,                   /* expr_op  */
  YYSYMBOL_expr_app = 69,                  /* expr_app  */
  YYSYMBOL_expr_select = 70,               /* expr_select  */
  YYSYMBOL_expr_simple = 71,               /* expr_simple  */
  YYSYMBOL_string_parts = 72,              /* string_parts  */
  YYSYMBOL_string_parts_interpolated = 73, /* string_parts_interpolated  */
  YYSYMBOL_path_start = 74,                /* path_start  */
  YYSYMBOL_ind_string_parts = 75,          /* ind_string_parts  */
  YYSYMBOL_binds = 76,                     /* binds  */
  YYSYMBOL_binds1 = 77,                    /* binds1  */
  YYSYMBOL_attrs = 78,                     /* attrs  */
  YYSYMBOL_attrpath = 79,                  /* attrpath  */
  YYSYMBOL_attr = 80,                      /* attr  */
  YYSYMBOL_string_attr = 81,               /* string_attr  */
  YYSYMBOL_expr_list = 82,                 /* expr_list  */
  YYSYMBOL_formal_set = 83,                /* formal_set  */
  YYSYMBOL_formals = 84,                   /* formals  */
  YYSYMBOL_formal = 85                     /* formal  */
};
typedef enum yysymbol_kind_t yysymbol_kind_t;




#ifdef short
# undef short
#endif

/* On compilers that do not define __PTRDIFF_MAX__ etc., make sure
   <limits.h> and (if available) <stdint.h> are included
   so that the code can choose integer types of a good width.  */

#ifndef __PTRDIFF_MAX__
# include <limits.h> /* INFRINGES ON USER NAME SPACE */
# if defined __STDC_VERSION__ && 199901 <= __STDC_VERSION__
#  include <stdint.h> /* INFRINGES ON USER NAME SPACE */
#  define YY_STDINT_H
# endif
#endif

/* Narrow types that promote to a signed type and that can represent a
   signed or unsigned integer of at least N bits.  In tables they can
   save space and decrease cache pressure.  Promoting to a signed type
   helps avoid bugs in integer arithmetic.  */

#ifdef __INT_LEAST8_MAX__
typedef __INT_LEAST8_TYPE__ yytype_int8;
#elif defined YY_STDINT_H
typedef int_least8_t yytype_int8;
#else
typedef signed char yytype_int8;
#endif

#ifdef __INT_LEAST16_MAX__
typedef __INT_LEAST16_TYPE__ yytype_int16;
#elif defined YY_STDINT_H
typedef int_least16_t yytype_int16;
#else
typedef short yytype_int16;
#endif

/* Work around bug in HP-UX 11.23, which defines these macros
   incorrectly for preprocessor constants.  This workaround can likely
   be removed in 2023, as HPE has promised support for HP-UX 11.23
   (aka HP-UX 11i v2) only through the end of 2022; see Table 2 of
   <https://h20195.www2.hpe.com/V2/getpdf.aspx/4AA4-7673ENW.pdf>.  */
#ifdef __hpux
# undef UINT_LEAST8_MAX
# undef UINT_LEAST16_MAX
# define UINT_LEAST8_MAX 255
# define UINT_LEAST16_MAX 65535
#endif

#if defined __UINT_LEAST8_MAX__ && __UINT_LEAST8_MAX__ <= __INT_MAX__
typedef __UINT_LEAST8_TYPE__ yytype_uint8;
#elif (!defined __UINT_LEAST8_MAX__ && defined YY_STDINT_H \
       && UINT_LEAST8_MAX <= INT_MAX)
typedef uint_least8_t yytype_uint8;
#elif !defined __UINT_LEAST8_MAX__ && UCHAR_MAX <= INT_MAX
typedef unsigned char yytype_uint8;
#else
typedef short yytype_uint8;
#endif

#if defined __UINT_LEAST16_MAX__ && __UINT_LEAST16_MAX__ <= __INT_MAX__
typedef __UINT_LEAST16_TYPE__ yytype_uint16;
#elif (!defined __UINT_LEAST16_MAX__ && defined YY_STDINT_H \
       && UINT_LEAST16_MAX <= INT_MAX)
typedef uint_least16_t yytype_uint16;
#elif !defined __UINT_LEAST16_MAX__ && USHRT_MAX <= INT_MAX
typedef unsigned short yytype_uint16;
#else
typedef int yytype_uint16;
#endif

#ifndef YYPTRDIFF_T
# if defined __PTRDIFF_TYPE__ && defined __PTRDIFF_MAX__
#  define YYPTRDIFF_T __PTRDIFF_TYPE__
#  define YYPTRDIFF_MAXIMUM __PTRDIFF_MAX__
# elif defined PTRDIFF_MAX
#  ifndef ptrdiff_t
#   include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  endif
#  define YYPTRDIFF_T ptrdiff_t
#  define YYPTRDIFF_MAXIMUM PTRDIFF_MAX
# else
#  define YYPTRDIFF_T long
#  define YYPTRDIFF_MAXIMUM LONG_MAX
# endif
#endif

#ifndef YYSIZE_T
# ifdef __SIZE_TYPE__
#  define YYSIZE_T __SIZE_TYPE__
# elif defined size_t
#  define YYSIZE_T size_t
# elif defined __STDC_VERSION__ && 199901 <= __STDC_VERSION__
#  include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  define YYSIZE_T size_t
# else
#  define YYSIZE_T unsigned
# endif
#endif

#define YYSIZE_MAXIMUM                                  \
  YY_CAST (YYPTRDIFF_T,                                 \
           (YYPTRDIFF_MAXIMUM < YY_CAST (YYSIZE_T, -1)  \
            ? YYPTRDIFF_MAXIMUM                         \
            : YY_CAST (YYSIZE_T, -1)))

#define YYSIZEOF(X) YY_CAST (YYPTRDIFF_T, sizeof (X))


/* Stored state numbers (used for stacks). */
typedef yytype_uint8 yy_state_t;

/* State numbers in computations.  */
typedef int yy_state_fast_t;

#ifndef YY_
# if defined YYENABLE_NLS && YYENABLE_NLS
#  if ENABLE_NLS
#   include <libintl.h> /* INFRINGES ON USER NAME SPACE */
#   define YY_(Msgid) dgettext ("bison-runtime", Msgid)
#  endif
# endif
# ifndef YY_
#  define YY_(Msgid) Msgid
# endif
#endif


#ifndef YY_ATTRIBUTE_PURE
# if defined __GNUC__ && 2 < __GNUC__ + (96 <= __GNUC_MINOR__)
#  define YY_ATTRIBUTE_PURE __attribute__ ((__pure__))
# else
#  define YY_ATTRIBUTE_PURE
# endif
#endif

#ifndef YY_ATTRIBUTE_UNUSED
# if defined __GNUC__ && 2 < __GNUC__ + (7 <= __GNUC_MINOR__)
#  define YY_ATTRIBUTE_UNUSED __attribute__ ((__unused__))
# else
#  define YY_ATTRIBUTE_UNUSED
# endif
#endif

/* Suppress unused-variable warnings by "using" E.  */
#if ! defined lint || defined __GNUC__
# define YY_USE(E) ((void) (E))
#else
# define YY_USE(E) /* empty */
#endif

/* Suppress an incorrect diagnostic about yylval being uninitialized.  */
#if defined __GNUC__ && ! defined __ICC && 406 <= __GNUC__ * 100 + __GNUC_MINOR__
# if __GNUC__ * 100 + __GNUC_MINOR__ < 407
#  define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN                           \
    _Pragma ("GCC diagnostic push")                                     \
    _Pragma ("GCC diagnostic ignored \"-Wuninitialized\"")
# else
#  define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN                           \
    _Pragma ("GCC diagnostic push")                                     \
    _Pragma ("GCC diagnostic ignored \"-Wuninitialized\"")              \
    _Pragma ("GCC diagnostic ignored \"-Wmaybe-uninitialized\"")
# endif
# define YY_IGNORE_MAYBE_UNINITIALIZED_END      \
    _Pragma ("GCC diagnostic pop")
#else
# define YY_INITIAL_VALUE(Value) Value
#endif
#ifndef YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_END
#endif
#ifndef YY_INITIAL_VALUE
# define YY_INITIAL_VALUE(Value) /* Nothing. */
#endif

#if defined __cplusplus && defined __GNUC__ && ! defined __ICC && 6 <= __GNUC__
# define YY_IGNORE_USELESS_CAST_BEGIN                          \
    _Pragma ("GCC diagnostic push")                            \
    _Pragma ("GCC diagnostic ignored \"-Wuseless-cast\"")
# define YY_IGNORE_USELESS_CAST_END            \
    _Pragma ("GCC diagnostic pop")
#endif
#ifndef YY_IGNORE_USELESS_CAST_BEGIN
# define YY_IGNORE_USELESS_CAST_BEGIN
# define YY_IGNORE_USELESS_CAST_END
#endif


#define YY_ASSERT(E) ((void) (0 && (E)))

#if 1

/* The parser invokes alloca or malloc; define the necessary symbols.  */

# ifdef YYSTACK_USE_ALLOCA
#  if YYSTACK_USE_ALLOCA
#   ifdef __GNUC__
#    define YYSTACK_ALLOC __builtin_alloca
#   elif defined __BUILTIN_VA_ARG_INCR
#    include <alloca.h> /* INFRINGES ON USER NAME SPACE */
#   elif defined _AIX
#    define YYSTACK_ALLOC __alloca
#   elif defined _MSC_VER
#    include <malloc.h> /* INFRINGES ON USER NAME SPACE */
#    define alloca _alloca
#   else
#    define YYSTACK_ALLOC alloca
#    if ! defined _ALLOCA_H && ! defined EXIT_SUCCESS
#     include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
      /* Use EXIT_SUCCESS as a witness for stdlib.h.  */
#     ifndef EXIT_SUCCESS
#      define EXIT_SUCCESS 0
#     endif
#    endif
#   endif
#  endif
# endif

# ifdef YYSTACK_ALLOC
   /* Pacify GCC's 'empty if-body' warning.  */
#  define YYSTACK_FREE(Ptr) do { /* empty */; } while (0)
#  ifndef YYSTACK_ALLOC_MAXIMUM
    /* The OS might guarantee only one guard page at the bottom of the stack,
       and a page size can be as small as 4096 bytes.  So we cannot safely
       invoke alloca (N) if N exceeds 4096.  Use a slightly smaller number
       to allow for a few compiler-allocated temporary stack slots.  */
#   define YYSTACK_ALLOC_MAXIMUM 4032 /* reasonable circa 2006 */
#  endif
# else
#  define YYSTACK_ALLOC YYMALLOC
#  define YYSTACK_FREE YYFREE
#  ifndef YYSTACK_ALLOC_MAXIMUM
#   define YYSTACK_ALLOC_MAXIMUM YYSIZE_MAXIMUM
#  endif
#  if (defined __cplusplus && ! defined EXIT_SUCCESS \
       && ! ((defined YYMALLOC || defined malloc) \
             && (defined YYFREE || defined free)))
#   include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#   ifndef EXIT_SUCCESS
#    define EXIT_SUCCESS 0
#   endif
#  endif
#  ifndef YYMALLOC
#   define YYMALLOC malloc
#   if ! defined malloc && ! defined EXIT_SUCCESS
void *malloc (YYSIZE_T); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
#  ifndef YYFREE
#   define YYFREE free
#   if ! defined free && ! defined EXIT_SUCCESS
void free (void *); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
# endif
#endif /* 1 */

#if (! defined yyoverflow \
     && (! defined __cplusplus \
         || (defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL \
             && defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yy_state_t yyss_alloc;
  YYSTYPE yyvs_alloc;
  YYLTYPE yyls_alloc;
};

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (YYSIZEOF (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (YYSIZEOF (yy_state_t) + YYSIZEOF (YYSTYPE) \
             + YYSIZEOF (YYLTYPE)) \
      + 2 * YYSTACK_GAP_MAXIMUM)

# define YYCOPY_NEEDED 1

/* Relocate STACK from its old location to the new one.  The
   local variables YYSIZE and YYSTACKSIZE give the old and new number of
   elements in the stack, and YYPTR gives the new location of the
   stack.  Advance YYPTR to a properly aligned location for the next
   stack.  */
# define YYSTACK_RELOCATE(Stack_alloc, Stack)                           \
    do                                                                  \
      {                                                                 \
        YYPTRDIFF_T yynewbytes;                                         \
        YYCOPY (&yyptr->Stack_alloc, Stack, yysize);                    \
        Stack = &yyptr->Stack_alloc;                                    \
        yynewbytes = yystacksize * YYSIZEOF (*Stack) + YYSTACK_GAP_MAXIMUM; \
        yyptr += yynewbytes / YYSIZEOF (*yyptr);                        \
      }                                                                 \
    while (0)

#endif

#if defined YYCOPY_NEEDED && YYCOPY_NEEDED
/* Copy COUNT objects from SRC to DST.  The source and destination do
   not overlap.  */
# ifndef YYCOPY
#  if defined __GNUC__ && 1 < __GNUC__
#   define YYCOPY(Dst, Src, Count) \
      __builtin_memcpy (Dst, Src, YY_CAST (YYSIZE_T, (Count)) * sizeof (*(Src)))
#  else
#   define YYCOPY(Dst, Src, Count)              \
      do                                        \
        {                                       \
          YYPTRDIFF_T yyi;                      \
          for (yyi = 0; yyi < (Count); yyi++)   \
            (Dst)[yyi] = (Src)[yyi];            \
        }                                       \
      while (0)
#  endif
# endif
#endif /* !YYCOPY_NEEDED */

/* YYFINAL -- State number of the termination state.  */
#define YYFINAL  67
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   407

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  61
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  25
/* YYNRULES -- Number of rules.  */
#define YYNRULES  99
/* YYNSTATES -- Number of states.  */
#define YYNSTATES  197

/* YYMAXUTOK -- Last valid token kind.  */
#define YYMAXUTOK   294


/* YYTRANSLATE(TOKEN-NUM) -- Symbol number corresponding to TOKEN-NUM
   as returned by yylex, with out-of-bounds checking.  */
#define YYTRANSLATE(YYX)                                \
  (0 <= (YYX) && (YYX) <= YYMAXUTOK                     \
   ? YY_CAST (yysymbol_kind_t, yytranslate[YYX])        \
   : YYSYMBOL_YYUNDEF)

/* YYTRANSLATE[TOKEN-NUM] -- Symbol number corresponding to TOKEN-NUM
   as returned by yylex.  */
static const yytype_int8 yytranslate[] =
{
       0,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    50,    52,     2,     2,     2,     2,     2,
      53,    54,    42,    40,    60,    41,    51,    43,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,    47,    49,
      34,    59,    35,    45,    48,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,    57,     2,    58,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    55,     2,    56,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     1,     2,     3,     4,
       5,     6,     7,     8,     9,    10,    11,    12,    13,    14,
      15,    16,    17,    18,    19,    20,    21,    22,    23,    24,
      25,    26,    27,    28,    29,    30,    31,    32,    33,    36,
      37,    38,    39,    44,    46
};

#if YYDEBUG
/* YYRLINE[YYN] -- Source line where rule number YYN was defined.  */
static const yytype_int16 yyrline[] =
{
       0,   182,   182,   189,   192,   197,   202,   209,   216,   218,
     220,   228,   232,   233,   234,   235,   239,   240,   244,   245,
     249,   250,   251,   252,   253,   254,   255,   256,   257,   258,
     259,   260,   261,   262,   264,   265,   266,   267,   268,   272,
     277,   281,   283,   292,   294,   298,   305,   306,   307,   308,
     312,   313,   317,   324,   333,   336,   338,   340,   342,   344,
     348,   349,   350,   354,   356,   357,   358,   366,   389,   402,
     403,   404,   408,   409,   413,   418,   429,   447,   455,   456,
     468,   472,   473,   482,   483,   495,   496,   500,   501,   505,
     506,   510,   511,   512,   513,   514,   518,   520,   525,   526
};
#endif

/** Accessing symbol of state STATE.  */
#define YY_ACCESSING_SYMBOL(State) YY_CAST (yysymbol_kind_t, yystos[State])

#if 1
/* The user-facing name of the symbol whose (internal) number is
   YYSYMBOL.  No bounds checking.  */
static const char *yysymbol_name (yysymbol_kind_t yysymbol) YY_ATTRIBUTE_UNUSED;

/* YYTNAME[SYMBOL-NUM] -- String name of the symbol SYMBOL-NUM.
   First, the terminals, then, starting at YYNTOKENS, nonterminals.  */
static const char *const yytname[] =
{
  "\"end of file\"", "error", "\"invalid token\"", "ID", "STR", "IND_STR",
  "INT_LIT", "FLOAT_LIT", "PATH", "HPATH", "SPATH", "PATH_END", "URI",
  "IF", "THEN", "ELSE", "ASSERT", "WITH", "LET", "IN_KW", "REC", "INHERIT",
  "EQ", "NEQ", "AND", "OR", "IMPL", "OR_KW", "PIPE_FROM", "PIPE_INTO",
  "DOLLAR_CURLY", "IND_STRING_OPEN", "IND_STRING_CLOSE", "ELLIPSIS", "'<'",
  "'>'", "LEQ", "GEQ", "UPDATE", "NOT", "'+'", "'-'", "'*'", "'/'",
  "CONCAT", "'?'", "NEGATE", "':'", "'@'", "';'", "'!'", "'.'", "'\"'",
  "'('", "')'", "'{'", "'}'", "'['", "']'", "'='", "','", "$accept",
  "start", "expr", "expr_function", "expr_if", "expr_pipe_from",
  "expr_pipe_into", "expr_op", "expr_app", "expr_select", "expr_simple",
  "string_parts", "string_parts_interpolated", "path_start",
  "ind_string_parts", "binds", "binds1", "attrs", "attrpath", "attr",
  "string_attr", "expr_list", "formal_set", "formals", "formal", YY_NULLPTR
};

static const char *
yysymbol_name (yysymbol_kind_t yysymbol)
{
  return yytname[yysymbol];
}
#endif

#define YYPACT_NINF (-103)

#define yypact_value_is_default(Yyn) \
  ((Yyn) == YYPACT_NINF)

#define YYTABLE_NINF (-99)

#define yytable_value_is_error(Yyn) \
  ((Yyn) == YYTABLE_NINF)

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
static const yytype_int16 yypact[] =
{
     156,   -22,  -103,  -103,  -103,  -103,  -103,  -103,   156,   156,
     156,    50,   -39,  -103,   174,   174,    65,   156,    10,  -103,
      20,  -103,  -103,  -103,  -103,     1,   230,   192,  -103,    36,
      85,    72,   156,    -7,    53,    21,    37,  -103,  -103,   156,
      65,    94,   135,    94,   -41,  -103,  -103,    94,     9,  -103,
      46,    55,  -103,   265,    74,   156,    62,    88,    93,   -33,
     104,   103,   140,    61,   -49,  -103,    26,  -103,   174,   174,
     174,   174,   174,   174,   174,   174,   174,   174,   174,   174,
     174,   174,   174,   174,   174,   174,    94,  -103,  -103,    94,
      74,  -103,   221,   156,   106,  -103,    12,   120,   156,   156,
     156,   114,   123,     0,   156,   125,    57,    94,   156,     3,
    -103,   156,  -103,  -103,   156,   129,  -103,  -103,   156,  -103,
     156,  -103,  -103,  -103,    19,  -103,  -103,   278,   326,   326,
     350,   302,   278,  -103,   254,   278,   362,   362,   362,   362,
     289,   176,   176,   113,   113,   113,   128,    71,  -103,  -103,
     141,   144,  -103,   156,   178,  -103,  -103,  -103,  -103,  -103,
    -103,   156,    96,   156,  -103,  -103,   142,  -103,   147,   151,
    -103,   160,  -103,   166,  -103,  -103,   192,   156,  -103,   156,
     179,  -103,  -103,  -103,   181,  -103,  -103,  -103,  -103,  -103,
    -103,  -103,  -103,  -103,  -103,   187,  -103
};

/* YYDEFACT[STATE-NUM] -- Default reduction number in state STATE-NUM.
   Performed when YYTABLE does not specify something else to do.  Zero
   means the default is an error.  */
static const yytype_int8 yydefact[] =
{
       0,    45,    46,    47,    67,    68,    52,    53,     0,     0,
       0,    73,     0,    71,     0,     0,    62,     0,    73,    90,
       0,     2,     3,    11,    13,    14,    15,    38,    40,    44,
       0,     0,     0,     0,     0,     0,     0,    85,    86,     0,
      62,    73,     0,    72,     0,    83,    84,    73,     0,    45,
       0,    73,    21,    20,    60,     0,     0,    61,     0,    85,
       0,    58,     0,    72,     0,    97,     0,     1,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,    39,    43,     0,
       0,    50,     0,     0,     0,     4,     0,     0,     0,     0,
       0,     0,     0,     0,     0,    80,     0,     0,     0,     0,
      69,     0,    49,    58,     0,     0,    48,    63,     0,    54,
       0,    92,    57,    94,     0,    59,    89,    18,    22,    23,
      28,    29,    30,    16,    17,    19,    24,    26,    25,    27,
      31,    33,    34,    35,    36,    37,    32,    41,    51,     5,
       0,    98,    95,     0,     0,     8,     9,    88,    87,    55,
      10,     0,     0,     0,    81,    82,     0,    56,     0,     0,
      65,     0,    99,     0,    93,    96,     0,     0,     7,     0,
       0,    75,    78,    79,     0,    77,    70,    66,    64,    91,
      42,     6,    12,    80,    74,     0,    76
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -103,  -103,    -8,   -28,  -103,   122,  -103,    59,  -103,   -24,
    -103,   188,   204,  -103,  -103,     8,    -1,    42,   -35,  -102,
    -101,  -103,   205,  -103,   116
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_uint8 yydefgoto[] =
{
       0,    20,    21,    22,    23,    24,    25,    26,    27,    28,
      29,    56,    57,    30,    48,    62,    43,   162,    44,    45,
      46,    66,    31,    64,    65
};

/* YYTABLE[YYPACT[STATE-NUM]] -- What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule whose
   number is the opposite.  If YYTABLE_NINF, syntax error.  */
static const yytype_int16 yytable[] =
{
      34,    35,    36,    87,    95,   164,   165,   123,   106,    58,
     107,   124,   120,    59,   110,   151,    47,    63,   108,    42,
      67,   105,   151,   -98,   105,    32,    33,   -98,   106,    49,
      68,   101,     2,     3,     4,     5,     6,    38,     7,   111,
      39,   112,   126,    60,    50,    60,    12,   115,    96,   103,
      63,   146,   173,    37,   147,   109,   159,    13,    37,   167,
     182,   183,    40,    88,    37,   149,    61,    98,   152,    54,
      99,   155,   156,    52,    53,   174,   160,    38,    16,    17,
      39,    51,    38,    19,   125,    39,   100,    89,    38,    90,
     154,    39,   117,   182,   183,    55,    91,    37,   176,    37,
     166,    41,    40,   168,   114,    41,   169,    40,   107,   150,
     171,   113,   172,    40,   116,    55,   163,   122,   118,    93,
      94,    38,   107,    38,    39,   178,    39,   127,   128,   129,
     130,   131,   132,   134,   135,   136,   137,   138,   139,   140,
     141,   142,   143,   144,   145,   181,    40,   119,    40,   191,
     -95,   -95,   190,   180,   104,   184,   105,    85,    86,     1,
     121,   105,     2,     3,     4,     5,     6,   153,     7,     8,
     157,   192,     9,    10,    11,   158,    12,    49,   161,   107,
       2,     3,     4,     5,     6,   170,     7,    13,   177,   120,
      37,   185,    50,   179,    12,    49,   133,    14,     2,     3,
       4,     5,     6,   186,     7,    13,    15,   187,    16,    17,
      50,    18,    12,    19,    38,    14,   188,    39,    83,    84,
      85,    86,   189,    13,    15,   117,    16,    17,   102,    51,
     194,    19,   148,   193,    92,   195,   196,     0,    97,    40,
     175,     0,     0,     0,    16,    17,     0,    51,     0,    19,
       0,   118,    69,    70,    71,    72,    73,     0,    74,    75,
       0,     0,     0,     0,    76,    77,    78,    79,    80,     0,
      81,    82,    83,    84,    85,    86,    69,    70,    71,    72,
      73,     0,    74,     0,     0,     0,     0,     0,    76,    77,
      78,    79,    80,     0,    81,    82,    83,    84,    85,    86,
      69,    70,    71,    72,    73,    81,    82,    83,    84,    85,
      86,     0,    76,    77,    78,    79,    80,     0,    81,    82,
      83,    84,    85,    86,    69,    70,    71,    80,     0,    81,
      82,    83,    84,    85,    86,     0,    76,    77,    78,    79,
      80,     0,    81,    82,    83,    84,    85,    86,   -99,   -99,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
      76,    77,    78,    79,    80,     0,    81,    82,    83,    84,
      85,    86,    69,    70,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,    76,    77,    78,    79,    80,     0,
      81,    82,    83,    84,    85,    86,   -99,   -99,   -99,   -99,
      80,     0,    81,    82,    83,    84,    85,    86
};

static const yytype_int16 yycheck[] =
{
       8,     9,    10,    27,    32,   107,   107,    56,    43,    17,
      51,    60,    45,     3,     5,     3,    55,    18,    59,    11,
       0,    21,     3,    56,    21,    47,    48,    60,    63,     3,
      29,    39,     6,     7,     8,     9,    10,    27,    12,    30,
      30,    32,    66,    33,    18,    33,    20,    55,    55,    41,
      51,    86,    33,     3,    89,    47,    56,    31,     3,    56,
     162,   162,    52,    27,     3,    93,    56,    14,    56,     4,
      49,    99,   100,    14,    15,    56,   104,    27,    52,    53,
      30,    55,    27,    57,    58,    30,    49,    51,    27,     4,
      98,    30,     4,   195,   195,    30,    11,     3,    27,     3,
     108,    55,    52,   111,    30,    55,   114,    52,    51,     3,
     118,    56,   120,    52,    52,    30,    59,    56,    30,    47,
      48,    27,    51,    27,    30,   153,    30,    68,    69,    70,
      71,    72,    73,    74,    75,    76,    77,    78,    79,    80,
      81,    82,    83,    84,    85,    49,    52,    54,    52,   177,
      47,    48,   176,   161,    19,   163,    21,    44,    45,     3,
      56,    21,     6,     7,     8,     9,    10,    47,    12,    13,
      56,   179,    16,    17,    18,    52,    20,     3,    53,    51,
       6,     7,     8,     9,    10,    56,    12,    31,    47,    45,
       3,    49,    18,    15,    20,     3,    74,    41,     6,     7,
       8,     9,    10,    56,    12,    31,    50,    56,    52,    53,
      18,    55,    20,    57,    27,    41,    56,    30,    42,    43,
      44,    45,    56,    31,    50,     4,    52,    53,    40,    55,
      49,    57,    11,    54,    30,   193,    49,    -1,    33,    52,
     124,    -1,    -1,    -1,    52,    53,    -1,    55,    -1,    57,
      -1,    30,    22,    23,    24,    25,    26,    -1,    28,    29,
      -1,    -1,    -1,    -1,    34,    35,    36,    37,    38,    -1,
      40,    41,    42,    43,    44,    45,    22,    23,    24,    25,
      26,    -1,    28,    -1,    -1,    -1,    -1,    -1,    34,    35,
      36,    37,    38,    -1,    40,    41,    42,    43,    44,    45,
      22,    23,    24,    25,    26,    40,    41,    42,    43,    44,
      45,    -1,    34,    35,    36,    37,    38,    -1,    40,    41,
      42,    43,    44,    45,    22,    23,    24,    38,    -1,    40,
      41,    42,    43,    44,    45,    -1,    34,    35,    36,    37,
      38,    -1,    40,    41,    42,    43,    44,    45,    22,    23,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      34,    35,    36,    37,    38,    -1,    40,    41,    42,    43,
      44,    45,    22,    23,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    34,    35,    36,    37,    38,    -1,
      40,    41,    42,    43,    44,    45,    34,    35,    36,    37,
      38,    -1,    40,    41,    42,    43,    44,    45
};

/* YYSTOS[STATE-NUM] -- The symbol kind of the accessing symbol of
   state STATE-NUM.  */
static const yytype_int8 yystos[] =
{
       0,     3,     6,     7,     8,     9,    10,    12,    13,    16,
      17,    18,    20,    31,    41,    50,    52,    53,    55,    57,
      62,    63,    64,    65,    66,    67,    68,    69,    70,    71,
      74,    83,    47,    48,    63,    63,    63,     3,    27,    30,
      52,    55,    76,    77,    79,    80,    81,    55,    75,     3,
      18,    55,    68,    68,     4,    30,    72,    73,    63,     3,
      33,    56,    76,    77,    84,    85,    82,     0,    29,    22,
      23,    24,    25,    26,    28,    29,    34,    35,    36,    37,
      38,    40,    41,    42,    43,    44,    45,    70,    27,    51,
       4,    11,    73,    47,    48,    64,    55,    83,    14,    49,
      49,    63,    72,    76,    19,    21,    79,    51,    59,    76,
       5,    30,    32,    56,    30,    63,    52,     4,    30,    54,
      45,    56,    56,    56,    60,    58,    70,    68,    68,    68,
      68,    68,    68,    66,    68,    68,    68,    68,    68,    68,
      68,    68,    68,    68,    68,    68,    79,    79,    11,    64,
       3,     3,    56,    47,    63,    64,    64,    56,    52,    56,
      64,    53,    78,    59,    80,    81,    63,    56,    63,    63,
      56,    63,    63,    33,    56,    85,    27,    47,    64,    15,
      63,    49,    80,    81,    63,    49,    56,    56,    56,    56,
      70,    64,    63,    54,    49,    78,    49
};

/* YYR1[RULE-NUM] -- Symbol kind of the left-hand side of rule RULE-NUM.  */
static const yytype_int8 yyr1[] =
{
       0,    61,    62,    63,    64,    64,    64,    64,    64,    64,
      64,    64,    65,    65,    65,    65,    66,    66,    67,    67,
      68,    68,    68,    68,    68,    68,    68,    68,    68,    68,
      68,    68,    68,    68,    68,    68,    68,    68,    68,    69,
      69,    70,    70,    70,    70,    71,    71,    71,    71,    71,
      71,    71,    71,    71,    71,    71,    71,    71,    71,    71,
      72,    72,    72,    73,    73,    73,    73,    74,    74,    75,
      75,    75,    76,    76,    77,    77,    77,    77,    78,    78,
      78,    79,    79,    79,    79,    80,    80,    81,    81,    82,
      82,    83,    83,    83,    83,    83,    84,    84,    85,    85
};

/* YYR2[RULE-NUM] -- Number of symbols on the right-hand side of rule RULE-NUM.  */
static const yytype_int8 yyr2[] =
{
       0,     2,     1,     1,     3,     3,     5,     5,     4,     4,
       4,     1,     6,     1,     1,     1,     3,     3,     3,     3,
       2,     2,     3,     3,     3,     3,     3,     3,     3,     3,
       3,     3,     3,     3,     3,     3,     3,     3,     1,     2,
       1,     3,     5,     2,     1,     1,     1,     1,     3,     3,
       2,     3,     1,     1,     3,     4,     4,     3,     2,     3,
       1,     1,     0,     2,     4,     3,     4,     1,     1,     2,
       4,     0,     1,     0,     5,     4,     7,     4,     2,     2,
       0,     3,     3,     1,     1,     1,     1,     3,     3,     2,
       0,     5,     3,     4,     3,     2,     3,     1,     1,     3
};


enum { YYENOMEM = -2 };

#define yyerrok         (yyerrstatus = 0)
#define yyclearin       (yychar = YYEMPTY)

#define YYACCEPT        goto yyacceptlab
#define YYABORT         goto yyabortlab
#define YYERROR         goto yyerrorlab
#define YYNOMEM         goto yyexhaustedlab


#define YYRECOVERING()  (!!yyerrstatus)

#define YYBACKUP(Token, Value)                                    \
  do                                                              \
    if (yychar == YYEMPTY)                                        \
      {                                                           \
        yychar = (Token);                                         \
        yylval = (Value);                                         \
        YYPOPSTACK (yylen);                                       \
        yystate = *yyssp;                                         \
        goto yybackup;                                            \
      }                                                           \
    else                                                          \
      {                                                           \
        yyerror (&yylloc, scanner, state, YY_("syntax error: cannot back up")); \
        YYERROR;                                                  \
      }                                                           \
  while (0)

/* Backward compatibility with an undocumented macro.
   Use YYerror or YYUNDEF. */
#define YYERRCODE YYUNDEF

/* YYLLOC_DEFAULT -- Set CURRENT to span from RHS[1] to RHS[N].
   If N is 0, then set CURRENT to the empty location which ends
   the previous symbol: RHS[0] (always defined).  */

#ifndef YYLLOC_DEFAULT
# define YYLLOC_DEFAULT(Current, Rhs, N)                                \
    do                                                                  \
      if (N)                                                            \
        {                                                               \
          (Current).first_line   = YYRHSLOC (Rhs, 1).first_line;        \
          (Current).first_column = YYRHSLOC (Rhs, 1).first_column;      \
          (Current).last_line    = YYRHSLOC (Rhs, N).last_line;         \
          (Current).last_column  = YYRHSLOC (Rhs, N).last_column;       \
        }                                                               \
      else                                                              \
        {                                                               \
          (Current).first_line   = (Current).last_line   =              \
            YYRHSLOC (Rhs, 0).last_line;                                \
          (Current).first_column = (Current).last_column =              \
            YYRHSLOC (Rhs, 0).last_column;                              \
        }                                                               \
    while (0)
#endif

#define YYRHSLOC(Rhs, K) ((Rhs)[K])


/* Enable debugging if requested.  */
#if YYDEBUG

# ifndef YYFPRINTF
#  include <stdio.h> /* INFRINGES ON USER NAME SPACE */
#  define YYFPRINTF fprintf
# endif

# define YYDPRINTF(Args)                        \
do {                                            \
  if (yydebug)                                  \
    YYFPRINTF Args;                             \
} while (0)


/* YYLOCATION_PRINT -- Print the location on the stream.
   This macro was not mandated originally: define only if we know
   we won't break user code: when these are the locations we know.  */

# ifndef YYLOCATION_PRINT

#  if defined YY_LOCATION_PRINT

   /* Temporary convenience wrapper in case some people defined the
      undocumented and private YY_LOCATION_PRINT macros.  */
#   define YYLOCATION_PRINT(File, Loc)  YY_LOCATION_PRINT(File, *(Loc))

#  elif defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL

/* Print *YYLOCP on YYO.  Private, do not rely on its existence. */

YY_ATTRIBUTE_UNUSED
static int
yy_location_print_ (FILE *yyo, YYLTYPE const * const yylocp)
{
  int res = 0;
  int end_col = 0 != yylocp->last_column ? yylocp->last_column - 1 : 0;
  if (0 <= yylocp->first_line)
    {
      res += YYFPRINTF (yyo, "%d", yylocp->first_line);
      if (0 <= yylocp->first_column)
        res += YYFPRINTF (yyo, ".%d", yylocp->first_column);
    }
  if (0 <= yylocp->last_line)
    {
      if (yylocp->first_line < yylocp->last_line)
        {
          res += YYFPRINTF (yyo, "-%d", yylocp->last_line);
          if (0 <= end_col)
            res += YYFPRINTF (yyo, ".%d", end_col);
        }
      else if (0 <= end_col && yylocp->first_column < end_col)
        res += YYFPRINTF (yyo, "-%d", end_col);
    }
  return res;
}

#   define YYLOCATION_PRINT  yy_location_print_

    /* Temporary convenience wrapper in case some people defined the
       undocumented and private YY_LOCATION_PRINT macros.  */
#   define YY_LOCATION_PRINT(File, Loc)  YYLOCATION_PRINT(File, &(Loc))

#  else

#   define YYLOCATION_PRINT(File, Loc) ((void) 0)
    /* Temporary convenience wrapper in case some people defined the
       undocumented and private YY_LOCATION_PRINT macros.  */
#   define YY_LOCATION_PRINT  YYLOCATION_PRINT

#  endif
# endif /* !defined YYLOCATION_PRINT */


# define YY_SYMBOL_PRINT(Title, Kind, Value, Location)                    \
do {                                                                      \
  if (yydebug)                                                            \
    {                                                                     \
      YYFPRINTF (stderr, "%s ", Title);                                   \
      yy_symbol_print (stderr,                                            \
                  Kind, Value, Location, scanner, state); \
      YYFPRINTF (stderr, "\n");                                           \
    }                                                                     \
} while (0)


/*-----------------------------------.
| Print this symbol's value on YYO.  |
`-----------------------------------*/

static void
yy_symbol_value_print (FILE *yyo,
                       yysymbol_kind_t yykind, YYSTYPE const * const yyvaluep, YYLTYPE const * const yylocationp, void * scanner, nix::ParserState * state)
{
  FILE *yyoutput = yyo;
  YY_USE (yyoutput);
  YY_USE (yylocationp);
  YY_USE (scanner);
  YY_USE (state);
  if (!yyvaluep)
    return;
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  YY_USE (yykind);
  YY_IGNORE_MAYBE_UNINITIALIZED_END
}


/*---------------------------.
| Print this symbol on YYO.  |
`---------------------------*/

static void
yy_symbol_print (FILE *yyo,
                 yysymbol_kind_t yykind, YYSTYPE const * const yyvaluep, YYLTYPE const * const yylocationp, void * scanner, nix::ParserState * state)
{
  YYFPRINTF (yyo, "%s %s (",
             yykind < YYNTOKENS ? "token" : "nterm", yysymbol_name (yykind));

  YYLOCATION_PRINT (yyo, yylocationp);
  YYFPRINTF (yyo, ": ");
  yy_symbol_value_print (yyo, yykind, yyvaluep, yylocationp, scanner, state);
  YYFPRINTF (yyo, ")");
}

/*------------------------------------------------------------------.
| yy_stack_print -- Print the state stack from its BOTTOM up to its |
| TOP (included).                                                   |
`------------------------------------------------------------------*/

static void
yy_stack_print (yy_state_t *yybottom, yy_state_t *yytop)
{
  YYFPRINTF (stderr, "Stack now");
  for (; yybottom <= yytop; yybottom++)
    {
      int yybot = *yybottom;
      YYFPRINTF (stderr, " %d", yybot);
    }
  YYFPRINTF (stderr, "\n");
}

# define YY_STACK_PRINT(Bottom, Top)                            \
do {                                                            \
  if (yydebug)                                                  \
    yy_stack_print ((Bottom), (Top));                           \
} while (0)


/*------------------------------------------------.
| Report that the YYRULE is going to be reduced.  |
`------------------------------------------------*/

static void
yy_reduce_print (yy_state_t *yyssp, YYSTYPE *yyvsp, YYLTYPE *yylsp,
                 int yyrule, void * scanner, nix::ParserState * state)
{
  int yylno = yyrline[yyrule];
  int yynrhs = yyr2[yyrule];
  int yyi;
  YYFPRINTF (stderr, "Reducing stack by rule %d (line %d):\n",
             yyrule - 1, yylno);
  /* The symbols being reduced.  */
  for (yyi = 0; yyi < yynrhs; yyi++)
    {
      YYFPRINTF (stderr, "   $%d = ", yyi + 1);
      yy_symbol_print (stderr,
                       YY_ACCESSING_SYMBOL (+yyssp[yyi + 1 - yynrhs]),
                       &yyvsp[(yyi + 1) - (yynrhs)],
                       &(yylsp[(yyi + 1) - (yynrhs)]), scanner, state);
      YYFPRINTF (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)          \
do {                                    \
  if (yydebug)                          \
    yy_reduce_print (yyssp, yyvsp, yylsp, Rule, scanner, state); \
} while (0)

/* Nonzero means print parse trace.  It is left uninitialized so that
   multiple parsers can coexist.  */
int yydebug;
#else /* !YYDEBUG */
# define YYDPRINTF(Args) ((void) 0)
# define YY_SYMBOL_PRINT(Title, Kind, Value, Location)
# define YY_STACK_PRINT(Bottom, Top)
# define YY_REDUCE_PRINT(Rule)
#endif /* !YYDEBUG */


/* YYINITDEPTH -- initial size of the parser's stacks.  */
#ifndef YYINITDEPTH
# define YYINITDEPTH 200
#endif

/* YYMAXDEPTH -- maximum size the stacks can grow to (effective only
   if the built-in stack extension method is used).

   Do not make this value too large; the results are undefined if
   YYSTACK_ALLOC_MAXIMUM < YYSTACK_BYTES (YYMAXDEPTH)
   evaluated with infinite-precision integer arithmetic.  */

#ifndef YYMAXDEPTH
# define YYMAXDEPTH 10000
#endif


/* Context of a parse error.  */
typedef struct
{
  yy_state_t *yyssp;
  yysymbol_kind_t yytoken;
  YYLTYPE *yylloc;
} yypcontext_t;

/* Put in YYARG at most YYARGN of the expected tokens given the
   current YYCTX, and return the number of tokens stored in YYARG.  If
   YYARG is null, return the number of expected tokens (guaranteed to
   be less than YYNTOKENS).  Return YYENOMEM on memory exhaustion.
   Return 0 if there are more than YYARGN expected tokens, yet fill
   YYARG up to YYARGN. */
static int
yypcontext_expected_tokens (const yypcontext_t *yyctx,
                            yysymbol_kind_t yyarg[], int yyargn)
{
  /* Actual size of YYARG. */
  int yycount = 0;
  int yyn = yypact[+*yyctx->yyssp];
  if (!yypact_value_is_default (yyn))
    {
      /* Start YYX at -YYN if negative to avoid negative indexes in
         YYCHECK.  In other words, skip the first -YYN actions for
         this state because they are default actions.  */
      int yyxbegin = yyn < 0 ? -yyn : 0;
      /* Stay within bounds of both yycheck and yytname.  */
      int yychecklim = YYLAST - yyn + 1;
      int yyxend = yychecklim < YYNTOKENS ? yychecklim : YYNTOKENS;
      int yyx;
      for (yyx = yyxbegin; yyx < yyxend; ++yyx)
        if (yycheck[yyx + yyn] == yyx && yyx != YYSYMBOL_YYerror
            && !yytable_value_is_error (yytable[yyx + yyn]))
          {
            if (!yyarg)
              ++yycount;
            else if (yycount == yyargn)
              return 0;
            else
              yyarg[yycount++] = YY_CAST (yysymbol_kind_t, yyx);
          }
    }
  if (yyarg && yycount == 0 && 0 < yyargn)
    yyarg[0] = YYSYMBOL_YYEMPTY;
  return yycount;
}




#ifndef yystrlen
# if defined __GLIBC__ && defined _STRING_H
#  define yystrlen(S) (YY_CAST (YYPTRDIFF_T, strlen (S)))
# else
/* Return the length of YYSTR.  */
static YYPTRDIFF_T
yystrlen (const char *yystr)
{
  YYPTRDIFF_T yylen;
  for (yylen = 0; yystr[yylen]; yylen++)
    continue;
  return yylen;
}
# endif
#endif

#ifndef yystpcpy
# if defined __GLIBC__ && defined _STRING_H && defined _GNU_SOURCE
#  define yystpcpy stpcpy
# else
/* Copy YYSRC to YYDEST, returning the address of the terminating '\0' in
   YYDEST.  */
static char *
yystpcpy (char *yydest, const char *yysrc)
{
  char *yyd = yydest;
  const char *yys = yysrc;

  while ((*yyd++ = *yys++) != '\0')
    continue;

  return yyd - 1;
}
# endif
#endif

#ifndef yytnamerr
/* Copy to YYRES the contents of YYSTR after stripping away unnecessary
   quotes and backslashes, so that it's suitable for yyerror.  The
   heuristic is that double-quoting is unnecessary unless the string
   contains an apostrophe, a comma, or backslash (other than
   backslash-backslash).  YYSTR is taken from yytname.  If YYRES is
   null, do not copy; instead, return the length of what the result
   would have been.  */
static YYPTRDIFF_T
yytnamerr (char *yyres, const char *yystr)
{
  if (*yystr == '"')
    {
      YYPTRDIFF_T yyn = 0;
      char const *yyp = yystr;
      for (;;)
        switch (*++yyp)
          {
          case '\'':
          case ',':
            goto do_not_strip_quotes;

          case '\\':
            if (*++yyp != '\\')
              goto do_not_strip_quotes;
            else
              goto append;

          append:
          default:
            if (yyres)
              yyres[yyn] = *yyp;
            yyn++;
            break;

          case '"':
            if (yyres)
              yyres[yyn] = '\0';
            return yyn;
          }
    do_not_strip_quotes: ;
    }

  if (yyres)
    return yystpcpy (yyres, yystr) - yyres;
  else
    return yystrlen (yystr);
}
#endif


static int
yy_syntax_error_arguments (const yypcontext_t *yyctx,
                           yysymbol_kind_t yyarg[], int yyargn)
{
  /* Actual size of YYARG. */
  int yycount = 0;
  /* There are many possibilities here to consider:
     - If this state is a consistent state with a default action, then
       the only way this function was invoked is if the default action
       is an error action.  In that case, don't check for expected
       tokens because there are none.
     - The only way there can be no lookahead present (in yychar) is if
       this state is a consistent state with a default action.  Thus,
       detecting the absence of a lookahead is sufficient to determine
       that there is no unexpected or expected token to report.  In that
       case, just report a simple "syntax error".
     - Don't assume there isn't a lookahead just because this state is a
       consistent state with a default action.  There might have been a
       previous inconsistent state, consistent state with a non-default
       action, or user semantic action that manipulated yychar.
     - Of course, the expected token list depends on states to have
       correct lookahead information, and it depends on the parser not
       to perform extra reductions after fetching a lookahead from the
       scanner and before detecting a syntax error.  Thus, state merging
       (from LALR or IELR) and default reductions corrupt the expected
       token list.  However, the list is correct for canonical LR with
       one exception: it will still contain any token that will not be
       accepted due to an error action in a later state.
  */
  if (yyctx->yytoken != YYSYMBOL_YYEMPTY)
    {
      int yyn;
      if (yyarg)
        yyarg[yycount] = yyctx->yytoken;
      ++yycount;
      yyn = yypcontext_expected_tokens (yyctx,
                                        yyarg ? yyarg + 1 : yyarg, yyargn - 1);
      if (yyn == YYENOMEM)
        return YYENOMEM;
      else
        yycount += yyn;
    }
  return yycount;
}

/* Copy into *YYMSG, which is of size *YYMSG_ALLOC, an error message
   about the unexpected token YYTOKEN for the state stack whose top is
   YYSSP.

   Return 0 if *YYMSG was successfully written.  Return -1 if *YYMSG is
   not large enough to hold the message.  In that case, also set
   *YYMSG_ALLOC to the required number of bytes.  Return YYENOMEM if the
   required number of bytes is too large to store.  */
static int
yysyntax_error (YYPTRDIFF_T *yymsg_alloc, char **yymsg,
                const yypcontext_t *yyctx)
{
  enum { YYARGS_MAX = 5 };
  /* Internationalized format string. */
  const char *yyformat = YY_NULLPTR;
  /* Arguments of yyformat: reported tokens (one for the "unexpected",
     one per "expected"). */
  yysymbol_kind_t yyarg[YYARGS_MAX];
  /* Cumulated lengths of YYARG.  */
  YYPTRDIFF_T yysize = 0;

  /* Actual size of YYARG. */
  int yycount = yy_syntax_error_arguments (yyctx, yyarg, YYARGS_MAX);
  if (yycount == YYENOMEM)
    return YYENOMEM;

  switch (yycount)
    {
#define YYCASE_(N, S)                       \
      case N:                               \
        yyformat = S;                       \
        break
    default: /* Avoid compiler warnings. */
      YYCASE_(0, YY_("syntax error"));
      YYCASE_(1, YY_("syntax error, unexpected %s"));
      YYCASE_(2, YY_("syntax error, unexpected %s, expecting %s"));
      YYCASE_(3, YY_("syntax error, unexpected %s, expecting %s or %s"));
      YYCASE_(4, YY_("syntax error, unexpected %s, expecting %s or %s or %s"));
      YYCASE_(5, YY_("syntax error, unexpected %s, expecting %s or %s or %s or %s"));
#undef YYCASE_
    }

  /* Compute error message size.  Don't count the "%s"s, but reserve
     room for the terminator.  */
  yysize = yystrlen (yyformat) - 2 * yycount + 1;
  {
    int yyi;
    for (yyi = 0; yyi < yycount; ++yyi)
      {
        YYPTRDIFF_T yysize1
          = yysize + yytnamerr (YY_NULLPTR, yytname[yyarg[yyi]]);
        if (yysize <= yysize1 && yysize1 <= YYSTACK_ALLOC_MAXIMUM)
          yysize = yysize1;
        else
          return YYENOMEM;
      }
  }

  if (*yymsg_alloc < yysize)
    {
      *yymsg_alloc = 2 * yysize;
      if (! (yysize <= *yymsg_alloc
             && *yymsg_alloc <= YYSTACK_ALLOC_MAXIMUM))
        *yymsg_alloc = YYSTACK_ALLOC_MAXIMUM;
      return -1;
    }

  /* Avoid sprintf, as that infringes on the user's name space.
     Don't have undefined behavior even if the translation
     produced a string with the wrong number of "%s"s.  */
  {
    char *yyp = *yymsg;
    int yyi = 0;
    while ((*yyp = *yyformat) != '\0')
      if (*yyp == '%' && yyformat[1] == 's' && yyi < yycount)
        {
          yyp += yytnamerr (yyp, yytname[yyarg[yyi++]]);
          yyformat += 2;
        }
      else
        {
          ++yyp;
          ++yyformat;
        }
  }
  return 0;
}


/*-----------------------------------------------.
| Release the memory associated to this symbol.  |
`-----------------------------------------------*/

static void
yydestruct (const char *yymsg,
            yysymbol_kind_t yykind, YYSTYPE *yyvaluep, YYLTYPE *yylocationp, void * scanner, nix::ParserState * state)
{
  YY_USE (yyvaluep);
  YY_USE (yylocationp);
  YY_USE (scanner);
  YY_USE (state);
  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yykind, yyvaluep, yylocationp);

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  YY_USE (yykind);
  YY_IGNORE_MAYBE_UNINITIALIZED_END
}






/*----------.
| yyparse.  |
`----------*/

int
yyparse (void * scanner, nix::ParserState * state)
{
/* Lookahead token kind.  */
int yychar;


/* The semantic value of the lookahead symbol.  */
/* Default value used for initialization, for pacifying older GCCs
   or non-GCC compilers.  */
YY_INITIAL_VALUE (static YYSTYPE yyval_default;)
YYSTYPE yylval YY_INITIAL_VALUE (= yyval_default);

/* Location data for the lookahead symbol.  */
static YYLTYPE yyloc_default
# if defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL
  = { 1, 1, 1, 1 }
# endif
;
YYLTYPE yylloc = yyloc_default;

    /* Number of syntax errors so far.  */
    int yynerrs = 0;

    yy_state_fast_t yystate = 0;
    /* Number of tokens to shift before error messages enabled.  */
    int yyerrstatus = 0;

    /* Refer to the stacks through separate pointers, to allow yyoverflow
       to reallocate them elsewhere.  */

    /* Their size.  */
    YYPTRDIFF_T yystacksize = YYINITDEPTH;

    /* The state stack: array, bottom, top.  */
    yy_state_t yyssa[YYINITDEPTH];
    yy_state_t *yyss = yyssa;
    yy_state_t *yyssp = yyss;

    /* The semantic value stack: array, bottom, top.  */
    YYSTYPE yyvsa[YYINITDEPTH];
    YYSTYPE *yyvs = yyvsa;
    YYSTYPE *yyvsp = yyvs;

    /* The location stack: array, bottom, top.  */
    YYLTYPE yylsa[YYINITDEPTH];
    YYLTYPE *yyls = yylsa;
    YYLTYPE *yylsp = yyls;

  int yyn;
  /* The return value of yyparse.  */
  int yyresult;
  /* Lookahead symbol kind.  */
  yysymbol_kind_t yytoken = YYSYMBOL_YYEMPTY;
  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;
  YYLTYPE yyloc;

  /* The locations where the error started and ended.  */
  YYLTYPE yyerror_range[3];

  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYPTRDIFF_T yymsg_alloc = sizeof yymsgbuf;

#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N), yylsp -= (N))

  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yychar = YYEMPTY; /* Cause a token to be read.  */

  yylsp[0] = yylloc;
  goto yysetstate;


/*------------------------------------------------------------.
| yynewstate -- push a new state, which is found in yystate.  |
`------------------------------------------------------------*/
yynewstate:
  /* In all cases, when you get here, the value and location stacks
     have just been pushed.  So pushing a state here evens the stacks.  */
  yyssp++;


/*--------------------------------------------------------------------.
| yysetstate -- set current state (the top of the stack) to yystate.  |
`--------------------------------------------------------------------*/
yysetstate:
  YYDPRINTF ((stderr, "Entering state %d\n", yystate));
  YY_ASSERT (0 <= yystate && yystate < YYNSTATES);
  YY_IGNORE_USELESS_CAST_BEGIN
  *yyssp = YY_CAST (yy_state_t, yystate);
  YY_IGNORE_USELESS_CAST_END
  YY_STACK_PRINT (yyss, yyssp);

  if (yyss + yystacksize - 1 <= yyssp)
#if !defined yyoverflow && !defined YYSTACK_RELOCATE
    YYNOMEM;
#else
    {
      /* Get the current used size of the three stacks, in elements.  */
      YYPTRDIFF_T yysize = yyssp - yyss + 1;

# if defined yyoverflow
      {
        /* Give user a chance to reallocate the stack.  Use copies of
           these so that the &'s don't force the real ones into
           memory.  */
        yy_state_t *yyss1 = yyss;
        YYSTYPE *yyvs1 = yyvs;
        YYLTYPE *yyls1 = yyls;

        /* Each stack pointer address is followed by the size of the
           data in use in that stack, in bytes.  This used to be a
           conditional around just the two extra args, but that might
           be undefined if yyoverflow is a macro.  */
        yyoverflow (YY_("memory exhausted"),
                    &yyss1, yysize * YYSIZEOF (*yyssp),
                    &yyvs1, yysize * YYSIZEOF (*yyvsp),
                    &yyls1, yysize * YYSIZEOF (*yylsp),
                    &yystacksize);
        yyss = yyss1;
        yyvs = yyvs1;
        yyls = yyls1;
      }
# else /* defined YYSTACK_RELOCATE */
      /* Extend the stack our own way.  */
      if (YYMAXDEPTH <= yystacksize)
        YYNOMEM;
      yystacksize *= 2;
      if (YYMAXDEPTH < yystacksize)
        yystacksize = YYMAXDEPTH;

      {
        yy_state_t *yyss1 = yyss;
        union yyalloc *yyptr =
          YY_CAST (union yyalloc *,
                   YYSTACK_ALLOC (YY_CAST (YYSIZE_T, YYSTACK_BYTES (yystacksize))));
        if (! yyptr)
          YYNOMEM;
        YYSTACK_RELOCATE (yyss_alloc, yyss);
        YYSTACK_RELOCATE (yyvs_alloc, yyvs);
        YYSTACK_RELOCATE (yyls_alloc, yyls);
#  undef YYSTACK_RELOCATE
        if (yyss1 != yyssa)
          YYSTACK_FREE (yyss1);
      }
# endif

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;
      yylsp = yyls + yysize - 1;

      YY_IGNORE_USELESS_CAST_BEGIN
      YYDPRINTF ((stderr, "Stack size increased to %ld\n",
                  YY_CAST (long, yystacksize)));
      YY_IGNORE_USELESS_CAST_END

      if (yyss + yystacksize - 1 <= yyssp)
        YYABORT;
    }
#endif /* !defined yyoverflow && !defined YYSTACK_RELOCATE */


  if (yystate == YYFINAL)
    YYACCEPT;

  goto yybackup;


/*-----------.
| yybackup.  |
`-----------*/
yybackup:
  /* Do appropriate processing given the current state.  Read a
     lookahead token if we need one and don't already have one.  */

  /* First try to decide what to do without reference to lookahead token.  */
  yyn = yypact[yystate];
  if (yypact_value_is_default (yyn))
    goto yydefault;

  /* Not known => get a lookahead token if don't already have one.  */

  /* YYCHAR is either empty, or end-of-input, or a valid lookahead.  */
  if (yychar == YYEMPTY)
    {
      YYDPRINTF ((stderr, "Reading a token\n"));
      yychar = yylex (&yylval, &yylloc, scanner, state);
    }

  if (yychar <= YYEOF)
    {
      yychar = YYEOF;
      yytoken = YYSYMBOL_YYEOF;
      YYDPRINTF ((stderr, "Now at end of input.\n"));
    }
  else if (yychar == YYerror)
    {
      /* The scanner already issued an error message, process directly
         to error recovery.  But do not keep the error token as
         lookahead, it is too special and may lead us to an endless
         loop in error recovery. */
      yychar = YYUNDEF;
      yytoken = YYSYMBOL_YYerror;
      yyerror_range[1] = yylloc;
      goto yyerrlab1;
    }
  else
    {
      yytoken = YYTRANSLATE (yychar);
      YY_SYMBOL_PRINT ("Next token is", yytoken, &yylval, &yylloc);
    }

  /* If the proper action on seeing token YYTOKEN is to reduce or to
     detect an error, take that action.  */
  yyn += yytoken;
  if (yyn < 0 || YYLAST < yyn || yycheck[yyn] != yytoken)
    goto yydefault;
  yyn = yytable[yyn];
  if (yyn <= 0)
    {
      if (yytable_value_is_error (yyn))
        goto yyerrlab;
      yyn = -yyn;
      goto yyreduce;
    }

  /* Count tokens shifted since error; after three, turn off error
     status.  */
  if (yyerrstatus)
    yyerrstatus--;

  /* Shift the lookahead token.  */
  YY_SYMBOL_PRINT ("Shifting", yytoken, &yylval, &yylloc);
  yystate = yyn;
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END
  *++yylsp = yylloc;

  /* Discard the shifted token.  */
  yychar = YYEMPTY;
  goto yynewstate;


/*-----------------------------------------------------------.
| yydefault -- do the default action for the current state.  |
`-----------------------------------------------------------*/
yydefault:
  yyn = yydefact[yystate];
  if (yyn == 0)
    goto yyerrlab;
  goto yyreduce;


/*-----------------------------.
| yyreduce -- do a reduction.  |
`-----------------------------*/
yyreduce:
  /* yyn is the number of a rule to reduce with.  */
  yylen = yyr2[yyn];

  /* If YYLEN is nonzero, implement the default value of the action:
     '$$ = $1'.

     Otherwise, the following line sets YYVAL to garbage.
     This behavior is undocumented and Bison
     users should not rely upon it.  Assigning to YYVAL
     unconditionally makes the parser a bit smaller, and it avoids a
     GCC warning that YYVAL may be used uninitialized.  */
  yyval = yyvsp[1-yylen];

  /* Default location. */
  YYLLOC_DEFAULT (yyloc, (yylsp - yylen), yylen);
  yyerror_range[1] = yyloc;
  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
  case 2: /* start: expr  */
#line 182 "../parser.y"
            {
  state->result = (yyvsp[0].e);

  // This parser does not use yynerrs; suppress the warning.
  (void) yynerrs;
}
#line 1758 "parser-tab.cc"
    break;

  case 4: /* expr_function: ID ':' expr_function  */
#line 193 "../parser.y"
    { auto me = new ExprLambda(CUR_POS, state->symbols.create((yyvsp[-2].id)), 0, (yyvsp[0].e));
      (yyval.e) = me;
      SET_DOC_POS(me, (yylsp[-2]));
    }
#line 1767 "parser-tab.cc"
    break;

  case 5: /* expr_function: formal_set ':' expr_function  */
#line 198 "../parser.y"
    { auto me = new ExprLambda(CUR_POS, state->validateFormals((yyvsp[-2].formals)), (yyvsp[0].e));
      (yyval.e) = me;
      SET_DOC_POS(me, (yylsp[-2]));
    }
#line 1776 "parser-tab.cc"
    break;

  case 6: /* expr_function: formal_set '@' ID ':' expr_function  */
#line 203 "../parser.y"
    {
      auto arg = state->symbols.create((yyvsp[-2].id));
      auto me = new ExprLambda(CUR_POS, arg, state->validateFormals((yyvsp[-4].formals), CUR_POS, arg), (yyvsp[0].e));
      (yyval.e) = me;
      SET_DOC_POS(me, (yylsp[-4]));
    }
#line 1787 "parser-tab.cc"
    break;

  case 7: /* expr_function: ID '@' formal_set ':' expr_function  */
#line 210 "../parser.y"
    {
      auto arg = state->symbols.create((yyvsp[-4].id));
      auto me = new ExprLambda(CUR_POS, arg, state->validateFormals((yyvsp[-2].formals), CUR_POS, arg), (yyvsp[0].e));
      (yyval.e) = me;
      SET_DOC_POS(me, (yylsp[-4]));
    }
#line 1798 "parser-tab.cc"
    break;

  case 8: /* expr_function: ASSERT expr ';' expr_function  */
#line 217 "../parser.y"
    { (yyval.e) = new ExprAssert(CUR_POS, (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1804 "parser-tab.cc"
    break;

  case 9: /* expr_function: WITH expr ';' expr_function  */
#line 219 "../parser.y"
    { (yyval.e) = new ExprWith(CUR_POS, (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1810 "parser-tab.cc"
    break;

  case 10: /* expr_function: LET binds IN_KW expr_function  */
#line 221 "../parser.y"
    { if (!(yyvsp[-2].attrs)->dynamicAttrs.empty())
        throw ParseError({
            .msg = HintFmt("dynamic attributes not allowed in let"),
            .pos = state->positions[CUR_POS]
        });
      (yyval.e) = new ExprLet((yyvsp[-2].attrs), (yyvsp[0].e));
    }
#line 1822 "parser-tab.cc"
    break;

  case 12: /* expr_if: IF expr THEN expr ELSE expr  */
#line 232 "../parser.y"
                                { (yyval.e) = new ExprIf(CUR_POS, (yyvsp[-4].e), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1828 "parser-tab.cc"
    break;

  case 16: /* expr_pipe_from: expr_op PIPE_FROM expr_pipe_from  */
#line 239 "../parser.y"
                                     { (yyval.e) = makeCall(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1834 "parser-tab.cc"
    break;

  case 17: /* expr_pipe_from: expr_op PIPE_FROM expr_op  */
#line 240 "../parser.y"
                                     { (yyval.e) = makeCall(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1840 "parser-tab.cc"
    break;

  case 18: /* expr_pipe_into: expr_pipe_into PIPE_INTO expr_op  */
#line 244 "../parser.y"
                                     { (yyval.e) = makeCall(state->at((yylsp[-1])), (yyvsp[0].e), (yyvsp[-2].e)); }
#line 1846 "parser-tab.cc"
    break;

  case 19: /* expr_pipe_into: expr_op PIPE_INTO expr_op  */
#line 245 "../parser.y"
                                     { (yyval.e) = makeCall(state->at((yylsp[-1])), (yyvsp[0].e), (yyvsp[-2].e)); }
#line 1852 "parser-tab.cc"
    break;

  case 20: /* expr_op: '!' expr_op  */
#line 249 "../parser.y"
                          { (yyval.e) = new ExprOpNot((yyvsp[0].e)); }
#line 1858 "parser-tab.cc"
    break;

  case 21: /* expr_op: '-' expr_op  */
#line 250 "../parser.y"
                             { (yyval.e) = new ExprCall(CUR_POS, new ExprVar(state->s.sub), {new ExprInt(0), (yyvsp[0].e)}); }
#line 1864 "parser-tab.cc"
    break;

  case 22: /* expr_op: expr_op EQ expr_op  */
#line 251 "../parser.y"
                       { (yyval.e) = new ExprOpEq((yyvsp[-2].e), (yyvsp[0].e)); }
#line 1870 "parser-tab.cc"
    break;

  case 23: /* expr_op: expr_op NEQ expr_op  */
#line 252 "../parser.y"
                        { (yyval.e) = new ExprOpNEq((yyvsp[-2].e), (yyvsp[0].e)); }
#line 1876 "parser-tab.cc"
    break;

  case 24: /* expr_op: expr_op '<' expr_op  */
#line 253 "../parser.y"
                        { (yyval.e) = new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.lessThan), {(yyvsp[-2].e), (yyvsp[0].e)}); }
#line 1882 "parser-tab.cc"
    break;

  case 25: /* expr_op: expr_op LEQ expr_op  */
#line 254 "../parser.y"
                        { (yyval.e) = new ExprOpNot(new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.lessThan), {(yyvsp[0].e), (yyvsp[-2].e)})); }
#line 1888 "parser-tab.cc"
    break;

  case 26: /* expr_op: expr_op '>' expr_op  */
#line 255 "../parser.y"
                        { (yyval.e) = new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.lessThan), {(yyvsp[0].e), (yyvsp[-2].e)}); }
#line 1894 "parser-tab.cc"
    break;

  case 27: /* expr_op: expr_op GEQ expr_op  */
#line 256 "../parser.y"
                        { (yyval.e) = new ExprOpNot(new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.lessThan), {(yyvsp[-2].e), (yyvsp[0].e)})); }
#line 1900 "parser-tab.cc"
    break;

  case 28: /* expr_op: expr_op AND expr_op  */
#line 257 "../parser.y"
                        { (yyval.e) = new ExprOpAnd(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1906 "parser-tab.cc"
    break;

  case 29: /* expr_op: expr_op OR expr_op  */
#line 258 "../parser.y"
                       { (yyval.e) = new ExprOpOr(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1912 "parser-tab.cc"
    break;

  case 30: /* expr_op: expr_op IMPL expr_op  */
#line 259 "../parser.y"
                         { (yyval.e) = new ExprOpImpl(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1918 "parser-tab.cc"
    break;

  case 31: /* expr_op: expr_op UPDATE expr_op  */
#line 260 "../parser.y"
                           { (yyval.e) = new ExprOpUpdate(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1924 "parser-tab.cc"
    break;

  case 32: /* expr_op: expr_op '?' attrpath  */
#line 261 "../parser.y"
                         { (yyval.e) = new ExprOpHasAttr((yyvsp[-2].e), std::move(*(yyvsp[0].attrNames))); delete (yyvsp[0].attrNames); }
#line 1930 "parser-tab.cc"
    break;

  case 33: /* expr_op: expr_op '+' expr_op  */
#line 263 "../parser.y"
    { (yyval.e) = new ExprConcatStrings(state->at((yylsp[-1])), false, new std::vector<std::pair<PosIdx, Expr *> >({{state->at((yylsp[-2])), (yyvsp[-2].e)}, {state->at((yylsp[0])), (yyvsp[0].e)}})); }
#line 1936 "parser-tab.cc"
    break;

  case 34: /* expr_op: expr_op '-' expr_op  */
#line 264 "../parser.y"
                        { (yyval.e) = new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.sub), {(yyvsp[-2].e), (yyvsp[0].e)}); }
#line 1942 "parser-tab.cc"
    break;

  case 35: /* expr_op: expr_op '*' expr_op  */
#line 265 "../parser.y"
                        { (yyval.e) = new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.mul), {(yyvsp[-2].e), (yyvsp[0].e)}); }
#line 1948 "parser-tab.cc"
    break;

  case 36: /* expr_op: expr_op '/' expr_op  */
#line 266 "../parser.y"
                        { (yyval.e) = new ExprCall(state->at((yylsp[-1])), new ExprVar(state->s.div), {(yyvsp[-2].e), (yyvsp[0].e)}); }
#line 1954 "parser-tab.cc"
    break;

  case 37: /* expr_op: expr_op CONCAT expr_op  */
#line 267 "../parser.y"
                           { (yyval.e) = new ExprOpConcatLists(state->at((yylsp[-1])), (yyvsp[-2].e), (yyvsp[0].e)); }
#line 1960 "parser-tab.cc"
    break;

  case 39: /* expr_app: expr_app expr_select  */
#line 272 "../parser.y"
                         { (yyval.e) = makeCall(CUR_POS, (yyvsp[-1].e), (yyvsp[0].e)); (yyvsp[0].e)->warnIfCursedOr(state->symbols, state->positions); }
#line 1966 "parser-tab.cc"
    break;

  case 40: /* expr_app: expr_select  */
#line 277 "../parser.y"
                { (yyval.e) = (yyvsp[0].e); (yyval.e)->resetCursedOr(); }
#line 1972 "parser-tab.cc"
    break;

  case 41: /* expr_select: expr_simple '.' attrpath  */
#line 282 "../parser.y"
    { (yyval.e) = new ExprSelect(CUR_POS, (yyvsp[-2].e), std::move(*(yyvsp[0].attrNames)), nullptr); delete (yyvsp[0].attrNames); }
#line 1978 "parser-tab.cc"
    break;

  case 42: /* expr_select: expr_simple '.' attrpath OR_KW expr_select  */
#line 284 "../parser.y"
    { (yyval.e) = new ExprSelect(CUR_POS, (yyvsp[-4].e), std::move(*(yyvsp[-2].attrNames)), (yyvsp[0].e)); delete (yyvsp[-2].attrNames); (yyvsp[0].e)->warnIfCursedOr(state->symbols, state->positions); }
#line 1984 "parser-tab.cc"
    break;

  case 43: /* expr_select: expr_simple OR_KW  */
#line 293 "../parser.y"
    { (yyval.e) = new ExprCall(CUR_POS, (yyvsp[-1].e), {new ExprVar(CUR_POS, state->s.or_)}, state->positions.add(state->origin, (yyloc).endOffset)); }
#line 1990 "parser-tab.cc"
    break;

  case 45: /* expr_simple: ID  */
#line 298 "../parser.y"
       {
      std::string_view s = "__curPos";
      if ((yyvsp[0].id).l == s.size() && strncmp((yyvsp[0].id).p, s.data(), s.size()) == 0)
          (yyval.e) = new ExprPos(CUR_POS);
      else
          (yyval.e) = new ExprVar(CUR_POS, state->symbols.create((yyvsp[0].id)));
  }
#line 2002 "parser-tab.cc"
    break;

  case 46: /* expr_simple: INT_LIT  */
#line 305 "../parser.y"
            { (yyval.e) = new ExprInt((yyvsp[0].n)); }
#line 2008 "parser-tab.cc"
    break;

  case 47: /* expr_simple: FLOAT_LIT  */
#line 306 "../parser.y"
              { (yyval.e) = new ExprFloat((yyvsp[0].nf)); }
#line 2014 "parser-tab.cc"
    break;

  case 48: /* expr_simple: '"' string_parts '"'  */
#line 307 "../parser.y"
                         { (yyval.e) = (yyvsp[-1].e); }
#line 2020 "parser-tab.cc"
    break;

  case 49: /* expr_simple: IND_STRING_OPEN ind_string_parts IND_STRING_CLOSE  */
#line 308 "../parser.y"
                                                      {
      (yyval.e) = state->stripIndentation(CUR_POS, std::move(*(yyvsp[-1].ind_string_parts)));
      delete (yyvsp[-1].ind_string_parts);
  }
#line 2029 "parser-tab.cc"
    break;

  case 51: /* expr_simple: path_start string_parts_interpolated PATH_END  */
#line 313 "../parser.y"
                                                  {
      (yyvsp[-1].string_parts)->insert((yyvsp[-1].string_parts)->begin(), {state->at((yylsp[-2])), (yyvsp[-2].e)});
      (yyval.e) = new ExprConcatStrings(CUR_POS, false, (yyvsp[-1].string_parts));
  }
#line 2038 "parser-tab.cc"
    break;

  case 52: /* expr_simple: SPATH  */
#line 317 "../parser.y"
          {
      std::string path((yyvsp[0].path).p + 1, (yyvsp[0].path).l - 2);
      (yyval.e) = new ExprCall(CUR_POS,
          new ExprVar(state->s.findFile),
          {new ExprVar(state->s.nixPath),
           new ExprString(std::move(path))});
  }
#line 2050 "parser-tab.cc"
    break;

  case 53: /* expr_simple: URI  */
#line 324 "../parser.y"
        {
      static bool noURLLiterals = experimentalFeatureSettings.isEnabled(Xp::NoUrlLiterals);
      if (noURLLiterals)
          throw ParseError({
              .msg = HintFmt("URL literals are disabled"),
              .pos = state->positions[CUR_POS]
          });
      (yyval.e) = new ExprString(std::string((yyvsp[0].uri)));
  }
#line 2064 "parser-tab.cc"
    break;

  case 54: /* expr_simple: '(' expr ')'  */
#line 333 "../parser.y"
                 { (yyval.e) = (yyvsp[-1].e); }
#line 2070 "parser-tab.cc"
    break;

  case 55: /* expr_simple: LET '{' binds '}'  */
#line 337 "../parser.y"
    { (yyvsp[-1].attrs)->recursive = true; (yyvsp[-1].attrs)->pos = CUR_POS; (yyval.e) = new ExprSelect(noPos, (yyvsp[-1].attrs), state->s.body); }
#line 2076 "parser-tab.cc"
    break;

  case 56: /* expr_simple: REC '{' binds '}'  */
#line 339 "../parser.y"
    { (yyvsp[-1].attrs)->recursive = true; (yyvsp[-1].attrs)->pos = CUR_POS; (yyval.e) = (yyvsp[-1].attrs); }
#line 2082 "parser-tab.cc"
    break;

  case 57: /* expr_simple: '{' binds1 '}'  */
#line 341 "../parser.y"
    { (yyvsp[-1].attrs)->pos = CUR_POS; (yyval.e) = (yyvsp[-1].attrs); }
#line 2088 "parser-tab.cc"
    break;

  case 58: /* expr_simple: '{' '}'  */
#line 343 "../parser.y"
    { (yyval.e) = new ExprAttrs(CUR_POS); }
#line 2094 "parser-tab.cc"
    break;

  case 59: /* expr_simple: '[' expr_list ']'  */
#line 344 "../parser.y"
                      { (yyval.e) = (yyvsp[-1].list); }
#line 2100 "parser-tab.cc"
    break;

  case 60: /* string_parts: STR  */
#line 348 "../parser.y"
        { (yyval.e) = new ExprString(std::string((yyvsp[0].str))); }
#line 2106 "parser-tab.cc"
    break;

  case 61: /* string_parts: string_parts_interpolated  */
#line 349 "../parser.y"
                              { (yyval.e) = new ExprConcatStrings(CUR_POS, true, (yyvsp[0].string_parts)); }
#line 2112 "parser-tab.cc"
    break;

  case 62: /* string_parts: %empty  */
#line 350 "../parser.y"
    { (yyval.e) = new ExprString(""); }
#line 2118 "parser-tab.cc"
    break;

  case 63: /* string_parts_interpolated: string_parts_interpolated STR  */
#line 355 "../parser.y"
  { (yyval.string_parts) = (yyvsp[-1].string_parts); (yyvsp[-1].string_parts)->emplace_back(state->at((yylsp[0])), new ExprString(std::string((yyvsp[0].str)))); }
#line 2124 "parser-tab.cc"
    break;

  case 64: /* string_parts_interpolated: string_parts_interpolated DOLLAR_CURLY expr '}'  */
#line 356 "../parser.y"
                                                    { (yyval.string_parts) = (yyvsp[-3].string_parts); (yyvsp[-3].string_parts)->emplace_back(state->at((yylsp[-2])), (yyvsp[-1].e)); }
#line 2130 "parser-tab.cc"
    break;

  case 65: /* string_parts_interpolated: DOLLAR_CURLY expr '}'  */
#line 357 "../parser.y"
                          { (yyval.string_parts) = new std::vector<std::pair<PosIdx, Expr *>>; (yyval.string_parts)->emplace_back(state->at((yylsp[-2])), (yyvsp[-1].e)); }
#line 2136 "parser-tab.cc"
    break;

  case 66: /* string_parts_interpolated: STR DOLLAR_CURLY expr '}'  */
#line 358 "../parser.y"
                              {
      (yyval.string_parts) = new std::vector<std::pair<PosIdx, Expr *>>;
      (yyval.string_parts)->emplace_back(state->at((yylsp[-3])), new ExprString(std::string((yyvsp[-3].str))));
      (yyval.string_parts)->emplace_back(state->at((yylsp[-2])), (yyvsp[-1].e));
    }
#line 2146 "parser-tab.cc"
    break;

  case 67: /* path_start: PATH  */
#line 366 "../parser.y"
         {
    std::string_view literal({(yyvsp[0].path).p, (yyvsp[0].path).l});

    /* check for short path literals */
    if (state->settings.warnShortPathLiterals && literal.front() != '/' && literal.front() != '.') {
        logWarning({
            .msg = HintFmt("relative path literal '%s' should be prefixed with '.' for clarity: './%s'. (" ANSI_BOLD "warn-short-path-literals" ANSI_NORMAL " = true)", literal, literal),
            .pos = state->positions[CUR_POS]
        });
    }

    Path path(absPath(literal, state->basePath.path.abs()));
    /* add back in the trailing '/' to the first segment */
    if (literal.size() > 1 && literal.back() == '/')
      path += '/';
    (yyval.e) =
        /* Absolute paths are always interpreted relative to the
           root filesystem accessor, rather than the accessor of the
           current Nix expression. */
        literal.front() == '/'
        ? new ExprPath(state->rootFS, std::move(path))
        : new ExprPath(state->basePath.accessor, std::move(path));
  }
#line 2174 "parser-tab.cc"
    break;

  case 68: /* path_start: HPATH  */
#line 389 "../parser.y"
          {
    if (state->settings.pureEval) {
        throw Error(
            "the path '%s' can not be resolved in pure mode",
            std::string_view((yyvsp[0].path).p, (yyvsp[0].path).l)
        );
    }
    Path path(getHome() + std::string((yyvsp[0].path).p + 1, (yyvsp[0].path).l - 1));
    (yyval.e) = new ExprPath(ref<SourceAccessor>(state->rootFS), std::move(path));
  }
#line 2189 "parser-tab.cc"
    break;

  case 69: /* ind_string_parts: ind_string_parts IND_STR  */
#line 402 "../parser.y"
                             { (yyval.ind_string_parts) = (yyvsp[-1].ind_string_parts); (yyvsp[-1].ind_string_parts)->emplace_back(state->at((yylsp[0])), (yyvsp[0].str)); }
#line 2195 "parser-tab.cc"
    break;

  case 70: /* ind_string_parts: ind_string_parts DOLLAR_CURLY expr '}'  */
#line 403 "../parser.y"
                                           { (yyval.ind_string_parts) = (yyvsp[-3].ind_string_parts); (yyvsp[-3].ind_string_parts)->emplace_back(state->at((yylsp[-2])), (yyvsp[-1].e)); }
#line 2201 "parser-tab.cc"
    break;

  case 71: /* ind_string_parts: %empty  */
#line 404 "../parser.y"
    { (yyval.ind_string_parts) = new std::vector<std::pair<PosIdx, std::variant<Expr *, StringToken>>>; }
#line 2207 "parser-tab.cc"
    break;

  case 73: /* binds: %empty  */
#line 409 "../parser.y"
    { (yyval.attrs) = new ExprAttrs; }
#line 2213 "parser-tab.cc"
    break;

  case 74: /* binds1: binds1 attrpath '=' expr ';'  */
#line 414 "../parser.y"
    { (yyval.attrs) = (yyvsp[-4].attrs);
      state->addAttr((yyval.attrs), std::move(*(yyvsp[-3].attrNames)), (yylsp[-3]), (yyvsp[-1].e), (yylsp[-1]));
      delete (yyvsp[-3].attrNames);
    }
#line 2222 "parser-tab.cc"
    break;

  case 75: /* binds1: binds INHERIT attrs ';'  */
#line 419 "../parser.y"
    { (yyval.attrs) = (yyvsp[-3].attrs);
      for (auto & [i, iPos] : *(yyvsp[-1].inheritAttrs)) {
          if ((yyvsp[-3].attrs)->attrs.find(i.symbol) != (yyvsp[-3].attrs)->attrs.end())
              state->dupAttr(i.symbol, iPos, (yyvsp[-3].attrs)->attrs[i.symbol].pos);
          (yyvsp[-3].attrs)->attrs.emplace(
              i.symbol,
              ExprAttrs::AttrDef(new ExprVar(iPos, i.symbol), iPos, ExprAttrs::AttrDef::Kind::Inherited));
      }
      delete (yyvsp[-1].inheritAttrs);
    }
#line 2237 "parser-tab.cc"
    break;

  case 76: /* binds1: binds INHERIT '(' expr ')' attrs ';'  */
#line 430 "../parser.y"
    { (yyval.attrs) = (yyvsp[-6].attrs);
      if (!(yyvsp[-6].attrs)->inheritFromExprs)
          (yyvsp[-6].attrs)->inheritFromExprs = std::make_unique<std::vector<Expr *>>();
      (yyvsp[-6].attrs)->inheritFromExprs->push_back((yyvsp[-3].e));
      auto from = new nix::ExprInheritFrom(state->at((yylsp[-3])), (yyvsp[-6].attrs)->inheritFromExprs->size() - 1);
      for (auto & [i, iPos] : *(yyvsp[-1].inheritAttrs)) {
          if ((yyvsp[-6].attrs)->attrs.find(i.symbol) != (yyvsp[-6].attrs)->attrs.end())
              state->dupAttr(i.symbol, iPos, (yyvsp[-6].attrs)->attrs[i.symbol].pos);
          (yyvsp[-6].attrs)->attrs.emplace(
              i.symbol,
              ExprAttrs::AttrDef(
                  new ExprSelect(iPos, from, i.symbol),
                  iPos,
                  ExprAttrs::AttrDef::Kind::InheritedFrom));
      }
      delete (yyvsp[-1].inheritAttrs);
    }
#line 2259 "parser-tab.cc"
    break;

  case 77: /* binds1: attrpath '=' expr ';'  */
#line 448 "../parser.y"
    { (yyval.attrs) = new ExprAttrs;
      state->addAttr((yyval.attrs), std::move(*(yyvsp[-3].attrNames)), (yylsp[-3]), (yyvsp[-1].e), (yylsp[-1]));
      delete (yyvsp[-3].attrNames);
    }
#line 2268 "parser-tab.cc"
    break;

  case 78: /* attrs: attrs attr  */
#line 455 "../parser.y"
               { (yyval.inheritAttrs) = (yyvsp[-1].inheritAttrs); (yyvsp[-1].inheritAttrs)->emplace_back(AttrName(state->symbols.create((yyvsp[0].id))), state->at((yylsp[0]))); }
#line 2274 "parser-tab.cc"
    break;

  case 79: /* attrs: attrs string_attr  */
#line 457 "../parser.y"
    { (yyval.inheritAttrs) = (yyvsp[-1].inheritAttrs);
      ExprString * str = dynamic_cast<ExprString *>((yyvsp[0].e));
      if (str) {
          (yyval.inheritAttrs)->emplace_back(AttrName(state->symbols.create(str->s)), state->at((yylsp[0])));
          delete str;
      } else
          throw ParseError({
              .msg = HintFmt("dynamic attributes not allowed in inherit"),
              .pos = state->positions[state->at((yylsp[0]))]
          });
    }
#line 2290 "parser-tab.cc"
    break;

  case 80: /* attrs: %empty  */
#line 468 "../parser.y"
    { (yyval.inheritAttrs) = new std::vector<std::pair<AttrName, PosIdx>>; }
#line 2296 "parser-tab.cc"
    break;

  case 81: /* attrpath: attrpath '.' attr  */
#line 472 "../parser.y"
                      { (yyval.attrNames) = (yyvsp[-2].attrNames); (yyvsp[-2].attrNames)->push_back(AttrName(state->symbols.create((yyvsp[0].id)))); }
#line 2302 "parser-tab.cc"
    break;

  case 82: /* attrpath: attrpath '.' string_attr  */
#line 474 "../parser.y"
    { (yyval.attrNames) = (yyvsp[-2].attrNames);
      ExprString * str = dynamic_cast<ExprString *>((yyvsp[0].e));
      if (str) {
          (yyval.attrNames)->push_back(AttrName(state->symbols.create(str->s)));
          delete str;
      } else
          (yyval.attrNames)->push_back(AttrName((yyvsp[0].e)));
    }
#line 2315 "parser-tab.cc"
    break;

  case 83: /* attrpath: attr  */
#line 482 "../parser.y"
         { (yyval.attrNames) = new std::vector<AttrName>; (yyval.attrNames)->push_back(AttrName(state->symbols.create((yyvsp[0].id)))); }
#line 2321 "parser-tab.cc"
    break;

  case 84: /* attrpath: string_attr  */
#line 484 "../parser.y"
    { (yyval.attrNames) = new std::vector<AttrName>;
      ExprString *str = dynamic_cast<ExprString *>((yyvsp[0].e));
      if (str) {
          (yyval.attrNames)->push_back(AttrName(state->symbols.create(str->s)));
          delete str;
      } else
          (yyval.attrNames)->push_back(AttrName((yyvsp[0].e)));
    }
#line 2334 "parser-tab.cc"
    break;

  case 86: /* attr: OR_KW  */
#line 496 "../parser.y"
          { (yyval.id) = {"or", 2}; }
#line 2340 "parser-tab.cc"
    break;

  case 87: /* string_attr: '"' string_parts '"'  */
#line 500 "../parser.y"
                         { (yyval.e) = (yyvsp[-1].e); }
#line 2346 "parser-tab.cc"
    break;

  case 88: /* string_attr: DOLLAR_CURLY expr '}'  */
#line 501 "../parser.y"
                          { (yyval.e) = (yyvsp[-1].e); }
#line 2352 "parser-tab.cc"
    break;

  case 89: /* expr_list: expr_list expr_select  */
#line 505 "../parser.y"
                          { (yyval.list) = (yyvsp[-1].list); (yyvsp[-1].list)->elems.push_back((yyvsp[0].e)); /* !!! dangerous */; (yyvsp[0].e)->warnIfCursedOr(state->symbols, state->positions); }
#line 2358 "parser-tab.cc"
    break;

  case 90: /* expr_list: %empty  */
#line 506 "../parser.y"
    { (yyval.list) = new ExprList; }
#line 2364 "parser-tab.cc"
    break;

  case 91: /* formal_set: '{' formals ',' ELLIPSIS '}'  */
#line 510 "../parser.y"
                                 { (yyval.formals) = (yyvsp[-3].formals);    (yyval.formals)->ellipsis = true; }
#line 2370 "parser-tab.cc"
    break;

  case 92: /* formal_set: '{' ELLIPSIS '}'  */
#line 511 "../parser.y"
                                 { (yyval.formals) = new Formals; (yyval.formals)->ellipsis = true; }
#line 2376 "parser-tab.cc"
    break;

  case 93: /* formal_set: '{' formals ',' '}'  */
#line 512 "../parser.y"
                                 { (yyval.formals) = (yyvsp[-2].formals);    (yyval.formals)->ellipsis = false; }
#line 2382 "parser-tab.cc"
    break;

  case 94: /* formal_set: '{' formals '}'  */
#line 513 "../parser.y"
                                 { (yyval.formals) = (yyvsp[-1].formals);    (yyval.formals)->ellipsis = false; }
#line 2388 "parser-tab.cc"
    break;

  case 95: /* formal_set: '{' '}'  */
#line 514 "../parser.y"
                                 { (yyval.formals) = new Formals; (yyval.formals)->ellipsis = false; }
#line 2394 "parser-tab.cc"
    break;

  case 96: /* formals: formals ',' formal  */
#line 519 "../parser.y"
    { (yyval.formals) = (yyvsp[-2].formals); (yyval.formals)->formals.emplace_back(*(yyvsp[0].formal)); delete (yyvsp[0].formal); }
#line 2400 "parser-tab.cc"
    break;

  case 97: /* formals: formal  */
#line 521 "../parser.y"
    { (yyval.formals) = new Formals; (yyval.formals)->formals.emplace_back(*(yyvsp[0].formal)); delete (yyvsp[0].formal); }
#line 2406 "parser-tab.cc"
    break;

  case 98: /* formal: ID  */
#line 525 "../parser.y"
       { (yyval.formal) = new Formal{CUR_POS, state->symbols.create((yyvsp[0].id)), 0}; }
#line 2412 "parser-tab.cc"
    break;

  case 99: /* formal: ID '?' expr  */
#line 526 "../parser.y"
                { (yyval.formal) = new Formal{CUR_POS, state->symbols.create((yyvsp[-2].id)), (yyvsp[0].e)}; }
#line 2418 "parser-tab.cc"
    break;


#line 2422 "parser-tab.cc"

      default: break;
    }
  /* User semantic actions sometimes alter yychar, and that requires
     that yytoken be updated with the new translation.  We take the
     approach of translating immediately before every use of yytoken.
     One alternative is translating here after every semantic action,
     but that translation would be missed if the semantic action invokes
     YYABORT, YYACCEPT, or YYERROR immediately after altering yychar or
     if it invokes YYBACKUP.  In the case of YYABORT or YYACCEPT, an
     incorrect destructor might then be invoked immediately.  In the
     case of YYERROR or YYBACKUP, subsequent parser actions might lead
     to an incorrect destructor call or verbose syntax error message
     before the lookahead is translated.  */
  YY_SYMBOL_PRINT ("-> $$ =", YY_CAST (yysymbol_kind_t, yyr1[yyn]), &yyval, &yyloc);

  YYPOPSTACK (yylen);
  yylen = 0;

  *++yyvsp = yyval;
  *++yylsp = yyloc;

  /* Now 'shift' the result of the reduction.  Determine what state
     that goes to, based on the state we popped back to and the rule
     number reduced by.  */
  {
    const int yylhs = yyr1[yyn] - YYNTOKENS;
    const int yyi = yypgoto[yylhs] + *yyssp;
    yystate = (0 <= yyi && yyi <= YYLAST && yycheck[yyi] == *yyssp
               ? yytable[yyi]
               : yydefgoto[yylhs]);
  }

  goto yynewstate;


/*--------------------------------------.
| yyerrlab -- here on detecting error.  |
`--------------------------------------*/
yyerrlab:
  /* Make sure we have latest lookahead translation.  See comments at
     user semantic actions for why this is necessary.  */
  yytoken = yychar == YYEMPTY ? YYSYMBOL_YYEMPTY : YYTRANSLATE (yychar);
  /* If not already recovering from an error, report this error.  */
  if (!yyerrstatus)
    {
      ++yynerrs;
      {
        yypcontext_t yyctx
          = {yyssp, yytoken, &yylloc};
        char const *yymsgp = YY_("syntax error");
        int yysyntax_error_status;
        yysyntax_error_status = yysyntax_error (&yymsg_alloc, &yymsg, &yyctx);
        if (yysyntax_error_status == 0)
          yymsgp = yymsg;
        else if (yysyntax_error_status == -1)
          {
            if (yymsg != yymsgbuf)
              YYSTACK_FREE (yymsg);
            yymsg = YY_CAST (char *,
                             YYSTACK_ALLOC (YY_CAST (YYSIZE_T, yymsg_alloc)));
            if (yymsg)
              {
                yysyntax_error_status
                  = yysyntax_error (&yymsg_alloc, &yymsg, &yyctx);
                yymsgp = yymsg;
              }
            else
              {
                yymsg = yymsgbuf;
                yymsg_alloc = sizeof yymsgbuf;
                yysyntax_error_status = YYENOMEM;
              }
          }
        yyerror (&yylloc, scanner, state, yymsgp);
        if (yysyntax_error_status == YYENOMEM)
          YYNOMEM;
      }
    }

  yyerror_range[1] = yylloc;
  if (yyerrstatus == 3)
    {
      /* If just tried and failed to reuse lookahead token after an
         error, discard it.  */

      if (yychar <= YYEOF)
        {
          /* Return failure if at end of input.  */
          if (yychar == YYEOF)
            YYABORT;
        }
      else
        {
          yydestruct ("Error: discarding",
                      yytoken, &yylval, &yylloc, scanner, state);
          yychar = YYEMPTY;
        }
    }

  /* Else will try to reuse lookahead token after shifting the error
     token.  */
  goto yyerrlab1;


/*---------------------------------------------------.
| yyerrorlab -- error raised explicitly by YYERROR.  |
`---------------------------------------------------*/
yyerrorlab:
  /* Pacify compilers when the user code never invokes YYERROR and the
     label yyerrorlab therefore never appears in user code.  */
  if (0)
    YYERROR;
  ++yynerrs;

  /* Do not reclaim the symbols of the rule whose action triggered
     this YYERROR.  */
  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);
  yystate = *yyssp;
  goto yyerrlab1;


/*-------------------------------------------------------------.
| yyerrlab1 -- common code for both syntax error and YYERROR.  |
`-------------------------------------------------------------*/
yyerrlab1:
  yyerrstatus = 3;      /* Each real token shifted decrements this.  */

  /* Pop stack until we find a state that shifts the error token.  */
  for (;;)
    {
      yyn = yypact[yystate];
      if (!yypact_value_is_default (yyn))
        {
          yyn += YYSYMBOL_YYerror;
          if (0 <= yyn && yyn <= YYLAST && yycheck[yyn] == YYSYMBOL_YYerror)
            {
              yyn = yytable[yyn];
              if (0 < yyn)
                break;
            }
        }

      /* Pop the current state because it cannot handle the error token.  */
      if (yyssp == yyss)
        YYABORT;

      yyerror_range[1] = *yylsp;
      yydestruct ("Error: popping",
                  YY_ACCESSING_SYMBOL (yystate), yyvsp, yylsp, scanner, state);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END

  yyerror_range[2] = yylloc;
  ++yylsp;
  YYLLOC_DEFAULT (*yylsp, yyerror_range, 2);

  /* Shift the error token.  */
  YY_SYMBOL_PRINT ("Shifting", YY_ACCESSING_SYMBOL (yyn), yyvsp, yylsp);

  yystate = yyn;
  goto yynewstate;


/*-------------------------------------.
| yyacceptlab -- YYACCEPT comes here.  |
`-------------------------------------*/
yyacceptlab:
  yyresult = 0;
  goto yyreturnlab;


/*-----------------------------------.
| yyabortlab -- YYABORT comes here.  |
`-----------------------------------*/
yyabortlab:
  yyresult = 1;
  goto yyreturnlab;


/*-----------------------------------------------------------.
| yyexhaustedlab -- YYNOMEM (memory exhaustion) comes here.  |
`-----------------------------------------------------------*/
yyexhaustedlab:
  yyerror (&yylloc, scanner, state, YY_("memory exhausted"));
  yyresult = 2;
  goto yyreturnlab;


/*----------------------------------------------------------.
| yyreturnlab -- parsing is finished, clean up and return.  |
`----------------------------------------------------------*/
yyreturnlab:
  if (yychar != YYEMPTY)
    {
      /* Make sure we have latest lookahead translation.  See comments at
         user semantic actions for why this is necessary.  */
      yytoken = YYTRANSLATE (yychar);
      yydestruct ("Cleanup: discarding lookahead",
                  yytoken, &yylval, &yylloc, scanner, state);
    }
  /* Do not reclaim the symbols of the rule whose action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
                  YY_ACCESSING_SYMBOL (+*yyssp), yyvsp, yylsp, scanner, state);
      YYPOPSTACK (1);
    }
#ifndef yyoverflow
  if (yyss != yyssa)
    YYSTACK_FREE (yyss);
#endif
  if (yymsg != yymsgbuf)
    YYSTACK_FREE (yymsg);
  return yyresult;
}

#line 529 "../parser.y"


#include "nix/expr/eval.hh"


namespace nix {

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
    const Expr::AstSymbols & astSymbols)
{
    yyscan_t scanner;
    LexerState lexerState {
        .positionToDocComment = docComments,
        .positions = positions,
        .origin = positions.addOrigin(origin, length),
    };
    ParserState state {
        .lexerState = lexerState,
        .symbols = symbols,
        .positions = positions,
        .basePath = basePath,
        .origin = lexerState.origin,
        .rootFS = rootFS,
        .s = astSymbols,
        .settings = settings,
    };

    yylex_init_extra(&lexerState, &scanner);
    Finally _destroy([&] { yylex_destroy(scanner); });

    yy_scan_buffer(text, length, scanner);
    yyparse(scanner, &state);

    return state.result;
}


}
