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
#define YYPURE 0

/* Push parsers.  */
#define YYPUSH 0

/* Pull parsers.  */
#define YYPULL 1




/* First part of user prologue.  */
#line 18 "Oberon.y"

#include "Config.h"
#include "Error.h"
#include "Files.h"
#include "Generate.h"
#include "lex.yy.h"
#include "Maps.h"
#include "Oberon.h"
#include "ModulePaths.h"
#include "Paths.h"
#include "Range.h"
#include "Table.h"
#include "Types.h"
#include "Trees.h"
#include "Util.h"
#include "../lib/obnc/OBNC.h"
#include <assert.h>
#include <ctype.h>
#include <float.h>
#include <limits.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/*assignment contexts*/
#define ASSIGNMENT_CONTEXT 0
#define PARAM_SUBST_CONTEXT 1
#define PROC_RESULT_CONTEXT 2

static int initialized = 0;

static const char *inputFilename;
static int parseMode;
static char *inputModuleName;

static Trees_Node unresolvedPointerTypes;
static Trees_Node currentTypeIdentdef;
static Trees_Node recordDeclarationStack;
static Trees_Node caseExpressionStack;
static Trees_Node caseLabelsStack;
static Trees_Node procedureDeclarationStack;

void yyerror(const char msg[]);
static void CheckUnusedIdentifiers(void);

/*constant predicate functions*/

static int IsBoolean(Trees_Node node);
static int IsChar(Trees_Node node);
static int IsInteger(Trees_Node node);
static int IsReal(Trees_Node node);
static int IsString(Trees_Node node);
static int IsSet(Trees_Node node);

/*functions for type declaration productions*/

static Trees_Node ResolvedType(Trees_Node type, int isTypeDecl);
static void ResolvePointerTypes(Trees_Node baseType);
static const char *TypeString(Trees_Node type);

/*functions for expression productions*/

static Trees_Node Designator(const char ident[], Trees_Node selectorList);
static int IsDesignator(Trees_Node exp);
static Trees_Node BaseIdent(Trees_Node designator);
static Trees_Node FirstSelector(Trees_Node designator);
static const char *DesignatorString(Trees_Node designator);
static void CheckIsValueExpression(Trees_Node exp);
static void SetSelectorTypes(Trees_Node identType, Trees_Node designator, int *parameterListFound);
static void RemoveActualParameters(Trees_Node *designator, Trees_Node *actualParameters);
static int IsConstExpression(Trees_Node exp);
static Trees_Node ExpressionConstValue(int relation, Trees_Node expA, Trees_Node expB);
static Trees_Node SimpleExpressionConstValue(int addOperator, Trees_Node expA, Trees_Node expB);
static Trees_Node TermConstValue(int mulOperator, Trees_Node expA, Trees_Node expB);
static const char *OperatorString(int operator);

/*functions for statement productions*/

static int Writable(Trees_Node designator);
static void ValidateAssignment(Trees_Node expression, Trees_Node targetType, int context, int paramPos);
static void HandleProcedureCall(Trees_Node designator, Trees_Node actualParameters, int isFunctionCall, Trees_Node *ast);
static void CheckCaseLabelUniqueness(Trees_Node label);

/*functions for module productions*/

static void ExportSymbolTable(const char symfilePath[]);

#line 160 "y.tab.c"

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

/* Use api.header.include to #include this header
   instead of duplicating it here.  */
#ifndef YY_YY_Y_TAB_H_INCLUDED
# define YY_YY_Y_TAB_H_INCLUDED
/* Debug traces.  */
#ifndef YYDEBUG
# define YYDEBUG 1
#endif
#if YYDEBUG
extern int yydebug;
#endif

/* Token kinds.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
  enum yytokentype
  {
    YYEMPTY = -2,
    YYEOF = 0,                     /* "end of file"  */
    YYerror = 256,                 /* error  */
    YYUNDEF = 257,                 /* "invalid token"  */
    TOKEN_START = 258,             /* TOKEN_START  */
    ARRAY = 259,                   /* ARRAY  */
    BEGIN_ = 260,                  /* BEGIN_  */
    BY = 261,                      /* BY  */
    CASE = 262,                    /* CASE  */
    CONST = 263,                   /* CONST  */
    DIV = 264,                     /* DIV  */
    DO = 265,                      /* DO  */
    ELSE = 266,                    /* ELSE  */
    ELSIF = 267,                   /* ELSIF  */
    END = 268,                     /* END  */
    FALSE = 269,                   /* FALSE  */
    FOR = 270,                     /* FOR  */
    IF = 271,                      /* IF  */
    IMPORT = 272,                  /* IMPORT  */
    IN = 273,                      /* IN  */
    IS = 274,                      /* IS  */
    MOD = 275,                     /* MOD  */
    MODULE = 276,                  /* MODULE  */
    NIL = 277,                     /* NIL  */
    OF = 278,                      /* OF  */
    OR = 279,                      /* OR  */
    POINTER = 280,                 /* POINTER  */
    PROCEDURE = 281,               /* PROCEDURE  */
    RECORD = 282,                  /* RECORD  */
    REPEAT = 283,                  /* REPEAT  */
    RETURN = 284,                  /* RETURN  */
    THEN = 285,                    /* THEN  */
    TO = 286,                      /* TO  */
    TRUE = 287,                    /* TRUE  */
    TYPE = 288,                    /* TYPE  */
    UNTIL = 289,                   /* UNTIL  */
    VAR = 290,                     /* VAR  */
    WHILE = 291,                   /* WHILE  */
    BECOMES = 292,                 /* BECOMES  */
    DOTDOT = 293,                  /* DOTDOT  */
    GE = 294,                      /* GE  */
    LE = 295,                      /* LE  */
    IDENT = 296,                   /* IDENT  */
    INTEGER = 297,                 /* INTEGER  */
    REAL = 298,                    /* REAL  */
    STRING = 299,                  /* STRING  */
    TOKEN_END = 300                /* TOKEN_END  */
  };
  typedef enum yytokentype yytoken_kind_t;
#endif
/* Token kinds.  */
#define YYEMPTY -2
#define YYEOF 0
#define YYerror 256
#define YYUNDEF 257
#define TOKEN_START 258
#define ARRAY 259
#define BEGIN_ 260
#define BY 261
#define CASE 262
#define CONST 263
#define DIV 264
#define DO 265
#define ELSE 266
#define ELSIF 267
#define END 268
#define FALSE 269
#define FOR 270
#define IF 271
#define IMPORT 272
#define IN 273
#define IS 274
#define MOD 275
#define MODULE 276
#define NIL 277
#define OF 278
#define OR 279
#define POINTER 280
#define PROCEDURE 281
#define RECORD 282
#define REPEAT 283
#define RETURN 284
#define THEN 285
#define TO 286
#define TRUE 287
#define TYPE 288
#define UNTIL 289
#define VAR 290
#define WHILE 291
#define BECOMES 292
#define DOTDOT 293
#define GE 294
#define LE 295
#define IDENT 296
#define INTEGER 297
#define REAL 298
#define STRING 299
#define TOKEN_END 300

/* Value type.  */
#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
union YYSTYPE
{
#line 107 "Oberon.y"

	const char *ident;
	OBNC_INTEGER integer;
	OBNC_REAL real;
	const char *string;
	Trees_Node node;

#line 311 "y.tab.c"

};
typedef union YYSTYPE YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define YYSTYPE_IS_DECLARED 1
#endif


extern YYSTYPE yylval;


int yyparse (void);


#endif /* !YY_YY_Y_TAB_H_INCLUDED  */
/* Symbol kind.  */
enum yysymbol_kind_t
{
  YYSYMBOL_YYEMPTY = -2,
  YYSYMBOL_YYEOF = 0,                      /* "end of file"  */
  YYSYMBOL_YYerror = 1,                    /* error  */
  YYSYMBOL_YYUNDEF = 2,                    /* "invalid token"  */
  YYSYMBOL_TOKEN_START = 3,                /* TOKEN_START  */
  YYSYMBOL_ARRAY = 4,                      /* ARRAY  */
  YYSYMBOL_BEGIN_ = 5,                     /* BEGIN_  */
  YYSYMBOL_BY = 6,                         /* BY  */
  YYSYMBOL_CASE = 7,                       /* CASE  */
  YYSYMBOL_CONST = 8,                      /* CONST  */
  YYSYMBOL_DIV = 9,                        /* DIV  */
  YYSYMBOL_DO = 10,                        /* DO  */
  YYSYMBOL_ELSE = 11,                      /* ELSE  */
  YYSYMBOL_ELSIF = 12,                     /* ELSIF  */
  YYSYMBOL_END = 13,                       /* END  */
  YYSYMBOL_FALSE = 14,                     /* FALSE  */
  YYSYMBOL_FOR = 15,                       /* FOR  */
  YYSYMBOL_IF = 16,                        /* IF  */
  YYSYMBOL_IMPORT = 17,                    /* IMPORT  */
  YYSYMBOL_IN = 18,                        /* IN  */
  YYSYMBOL_IS = 19,                        /* IS  */
  YYSYMBOL_MOD = 20,                       /* MOD  */
  YYSYMBOL_MODULE = 21,                    /* MODULE  */
  YYSYMBOL_NIL = 22,                       /* NIL  */
  YYSYMBOL_OF = 23,                        /* OF  */
  YYSYMBOL_OR = 24,                        /* OR  */
  YYSYMBOL_POINTER = 25,                   /* POINTER  */
  YYSYMBOL_PROCEDURE = 26,                 /* PROCEDURE  */
  YYSYMBOL_RECORD = 27,                    /* RECORD  */
  YYSYMBOL_REPEAT = 28,                    /* REPEAT  */
  YYSYMBOL_RETURN = 29,                    /* RETURN  */
  YYSYMBOL_THEN = 30,                      /* THEN  */
  YYSYMBOL_TO = 31,                        /* TO  */
  YYSYMBOL_TRUE = 32,                      /* TRUE  */
  YYSYMBOL_TYPE = 33,                      /* TYPE  */
  YYSYMBOL_UNTIL = 34,                     /* UNTIL  */
  YYSYMBOL_VAR = 35,                       /* VAR  */
  YYSYMBOL_WHILE = 36,                     /* WHILE  */
  YYSYMBOL_BECOMES = 37,                   /* BECOMES  */
  YYSYMBOL_DOTDOT = 38,                    /* DOTDOT  */
  YYSYMBOL_GE = 39,                        /* GE  */
  YYSYMBOL_LE = 40,                        /* LE  */
  YYSYMBOL_IDENT = 41,                     /* IDENT  */
  YYSYMBOL_INTEGER = 42,                   /* INTEGER  */
  YYSYMBOL_REAL = 43,                      /* REAL  */
  YYSYMBOL_STRING = 44,                    /* STRING  */
  YYSYMBOL_TOKEN_END = 45,                 /* TOKEN_END  */
  YYSYMBOL_46_ = 46,                       /* '.'  */
  YYSYMBOL_47_ = 47,                       /* '*'  */
  YYSYMBOL_48_ = 48,                       /* '='  */
  YYSYMBOL_49_ = 49,                       /* ','  */
  YYSYMBOL_50_ = 50,                       /* '('  */
  YYSYMBOL_51_ = 51,                       /* ')'  */
  YYSYMBOL_52_ = 52,                       /* ';'  */
  YYSYMBOL_53_ = 53,                       /* ':'  */
  YYSYMBOL_54_ = 54,                       /* '#'  */
  YYSYMBOL_55_ = 55,                       /* '<'  */
  YYSYMBOL_56_ = 56,                       /* '>'  */
  YYSYMBOL_57_ = 57,                       /* '+'  */
  YYSYMBOL_58_ = 58,                       /* '-'  */
  YYSYMBOL_59_ = 59,                       /* '/'  */
  YYSYMBOL_60_ = 60,                       /* '&'  */
  YYSYMBOL_61_ = 61,                       /* '~'  */
  YYSYMBOL_62_ = 62,                       /* '['  */
  YYSYMBOL_63_ = 63,                       /* ']'  */
  YYSYMBOL_64_ = 64,                       /* '^'  */
  YYSYMBOL_65_ = 65,                       /* '{'  */
  YYSYMBOL_66_ = 66,                       /* '}'  */
  YYSYMBOL_67_ = 67,                       /* '|'  */
  YYSYMBOL_YYACCEPT = 68,                  /* $accept  */
  YYSYMBOL_qualident = 69,                 /* qualident  */
  YYSYMBOL_identdef = 70,                  /* identdef  */
  YYSYMBOL_ExportMarkOpt = 71,             /* ExportMarkOpt  */
  YYSYMBOL_number = 72,                    /* number  */
  YYSYMBOL_ConstDeclaration = 73,          /* ConstDeclaration  */
  YYSYMBOL_ConstExpression = 74,           /* ConstExpression  */
  YYSYMBOL_TypeDeclaration = 75,           /* TypeDeclaration  */
  YYSYMBOL_TypeIdentDef = 76,              /* TypeIdentDef  */
  YYSYMBOL_type = 77,                      /* type  */
  YYSYMBOL_ArrayType = 78,                 /* ArrayType  */
  YYSYMBOL_ArrayLengthOf = 79,             /* ArrayLengthOf  */
  YYSYMBOL_LengthRep = 80,                 /* LengthRep  */
  YYSYMBOL_length = 81,                    /* length  */
  YYSYMBOL_RecordType = 82,                /* RecordType  */
  YYSYMBOL_RecordHeading = 83,             /* RecordHeading  */
  YYSYMBOL_BaseTypeOpt = 84,               /* BaseTypeOpt  */
  YYSYMBOL_BaseType = 85,                  /* BaseType  */
  YYSYMBOL_FieldListSequenceOpt = 86,      /* FieldListSequenceOpt  */
  YYSYMBOL_FieldListSequence = 87,         /* FieldListSequence  */
  YYSYMBOL_FieldList = 88,                 /* FieldList  */
  YYSYMBOL_IdentList = 89,                 /* IdentList  */
  YYSYMBOL_PointerType = 90,               /* PointerType  */
  YYSYMBOL_PointerTo = 91,                 /* PointerTo  */
  YYSYMBOL_ProcedureType = 92,             /* ProcedureType  */
  YYSYMBOL_ProcedureTypeSansParam = 93,    /* ProcedureTypeSansParam  */
  YYSYMBOL_FormalParametersOpt = 94,       /* FormalParametersOpt  */
  YYSYMBOL_VariableDeclaration = 95,       /* VariableDeclaration  */
  YYSYMBOL_expression = 96,                /* expression  */
  YYSYMBOL_relation = 97,                  /* relation  */
  YYSYMBOL_SimpleExpression = 98,          /* SimpleExpression  */
  YYSYMBOL_SignOpt = 99,                   /* SignOpt  */
  YYSYMBOL_AddOperator = 100,              /* AddOperator  */
  YYSYMBOL_term = 101,                     /* term  */
  YYSYMBOL_MulOperator = 102,              /* MulOperator  */
  YYSYMBOL_factor = 103,                   /* factor  */
  YYSYMBOL_designator = 104,               /* designator  */
  YYSYMBOL_SelectorOptRep = 105,           /* SelectorOptRep  */
  YYSYMBOL_selector = 106,                 /* selector  */
  YYSYMBOL_set = 107,                      /* set  */
  YYSYMBOL_ElementRep = 108,               /* ElementRep  */
  YYSYMBOL_element = 109,                  /* element  */
  YYSYMBOL_ExpList = 110,                  /* ExpList  */
  YYSYMBOL_statement = 111,                /* statement  */
  YYSYMBOL_assignment = 112,               /* assignment  */
  YYSYMBOL_ProcedureCall = 113,            /* ProcedureCall  */
  YYSYMBOL_StatementSequence = 114,        /* StatementSequence  */
  YYSYMBOL_StatementSequenceReversed = 115, /* StatementSequenceReversed  */
  YYSYMBOL_IfStatement = 116,              /* IfStatement  */
  YYSYMBOL_guard = 117,                    /* guard  */
  YYSYMBOL_ElseIfThenOptRep = 118,         /* ElseIfThenOptRep  */
  YYSYMBOL_ElseOpt = 119,                  /* ElseOpt  */
  YYSYMBOL_CaseStatement = 120,            /* CaseStatement  */
  YYSYMBOL_CaseExpression = 121,           /* CaseExpression  */
  YYSYMBOL_CaseRep = 122,                  /* CaseRep  */
  YYSYMBOL_case = 123,                     /* case  */
  YYSYMBOL_CaseLabelList = 124,            /* CaseLabelList  */
  YYSYMBOL_LabelRange = 125,               /* LabelRange  */
  YYSYMBOL_label = 126,                    /* label  */
  YYSYMBOL_WhileStatement = 127,           /* WhileStatement  */
  YYSYMBOL_ElseIfDoOptRep = 128,           /* ElseIfDoOptRep  */
  YYSYMBOL_RepeatStatement = 129,          /* RepeatStatement  */
  YYSYMBOL_ForStatement = 130,             /* ForStatement  */
  YYSYMBOL_ForInit = 131,                  /* ForInit  */
  YYSYMBOL_ForLimit = 132,                 /* ForLimit  */
  YYSYMBOL_ByOpt = 133,                    /* ByOpt  */
  YYSYMBOL_ProcedureDeclaration = 134,     /* ProcedureDeclaration  */
  YYSYMBOL_ProcedureHeading = 135,         /* ProcedureHeading  */
  YYSYMBOL_ProcedureHeadingSansParam = 136, /* ProcedureHeadingSansParam  */
  YYSYMBOL_StatementSequenceOpt = 137,     /* StatementSequenceOpt  */
  YYSYMBOL_ReturnClauseOpt = 138,          /* ReturnClauseOpt  */
  YYSYMBOL_DeclarationSequence = 139,      /* DeclarationSequence  */
  YYSYMBOL_ConstSectionOpt = 140,          /* ConstSectionOpt  */
  YYSYMBOL_ConstDeclarationOptRep = 141,   /* ConstDeclarationOptRep  */
  YYSYMBOL_TypeSectionOpt = 142,           /* TypeSectionOpt  */
  YYSYMBOL_TypeKeyword = 143,              /* TypeKeyword  */
  YYSYMBOL_TypeDeclarationOptRep = 144,    /* TypeDeclarationOptRep  */
  YYSYMBOL_VariableSectionOpt = 145,       /* VariableSectionOpt  */
  YYSYMBOL_VariableDeclarationOptRep = 146, /* VariableDeclarationOptRep  */
  YYSYMBOL_ProcedureDeclarationOptRep = 147, /* ProcedureDeclarationOptRep  */
  YYSYMBOL_FormalParameters = 148,         /* FormalParameters  */
  YYSYMBOL_FPSectionsOpt = 149,            /* FPSectionsOpt  */
  YYSYMBOL_FPSectionRep = 150,             /* FPSectionRep  */
  YYSYMBOL_ResultTypeOpt = 151,            /* ResultTypeOpt  */
  YYSYMBOL_FPSection = 152,                /* FPSection  */
  YYSYMBOL_ParameterKindOpt = 153,         /* ParameterKindOpt  */
  YYSYMBOL_IdentRep = 154,                 /* IdentRep  */
  YYSYMBOL_FormalType = 155,               /* FormalType  */
  YYSYMBOL_OpenArrayOptRep = 156,          /* OpenArrayOptRep  */
  YYSYMBOL_module = 157,                   /* module  */
  YYSYMBOL_ModuleHeading = 158,            /* ModuleHeading  */
  YYSYMBOL_ImportListOpt = 159,            /* ImportListOpt  */
  YYSYMBOL_ImportList = 160,               /* ImportList  */
  YYSYMBOL_ImportRep = 161,                /* ImportRep  */
  YYSYMBOL_import = 162,                   /* import  */
  YYSYMBOL_BecomesIdentOpt = 163,          /* BecomesIdentOpt  */
  YYSYMBOL_ModuleStatements = 164          /* ModuleStatements  */
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
typedef yytype_int16 yy_state_t;

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
         || (defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yy_state_t yyss_alloc;
  YYSTYPE yyvs_alloc;
};

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (YYSIZEOF (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (YYSIZEOF (yy_state_t) + YYSIZEOF (YYSTYPE)) \
      + YYSTACK_GAP_MAXIMUM)

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
#define YYFINAL  5
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   236

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  68
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  97
/* YYNRULES -- Number of rules.  */
#define YYNRULES  181
/* YYNSTATES -- Number of states.  */
#define YYNSTATES  284

/* YYMAXUTOK -- Last valid token kind.  */
#define YYMAXUTOK   300


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
       2,     2,     2,     2,     2,    54,     2,     2,    60,     2,
      50,    51,    47,    57,    49,    58,    46,    59,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,    53,    52,
      55,    48,    56,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,    62,     2,    63,    64,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    65,    67,    66,    61,     2,     2,     2,
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
      25,    26,    27,    28,    29,    30,    31,    32,    33,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45
};

#if YYDEBUG
/* YYRLINE[YYN] -- Source line where rule number YYN was defined.  */
static const yytype_int16 yyrline[] =
{
       0,   221,   221,   225,   232,   250,   255,   264,   268,   278,
     294,   309,   332,   342,   343,   344,   345,   346,   350,   370,
     380,   384,   391,   411,   419,   430,   435,   441,   482,   488,
     494,   498,   526,   575,   579,   589,   624,   633,   640,   647,
     649,   658,   696,   697,   744,   748,   752,   756,   760,   764,
     768,   772,   779,   801,   830,   834,   839,   845,   849,   853,
     860,   861,   891,   895,   899,   903,   907,   914,   915,   919,
     924,   928,   932,   938,   958,   963,   984,  1004,  1022,  1028,
    1036,  1054,  1059,  1064,  1071,  1075,  1082,  1083,  1096,  1116,
    1140,  1145,  1159,  1160,  1161,  1162,  1163,  1164,  1165,  1167,
    1173,  1216,  1237,  1245,  1253,  1264,  1287,  1300,  1305,  1311,
    1316,  1322,  1343,  1381,  1389,  1400,  1406,  1412,  1416,  1432,
    1441,  1488,  1497,  1511,  1596,  1619,  1624,  1630,  1645,  1663,
    1689,  1700,  1718,  1727,  1777,  1798,  1813,  1818,  1824,  1829,
    1835,  1839,  1840,  1844,  1845,  1849,  1861,  1867,  1874,  1875,
    1879,  1880,  1884,  1885,  1889,  1890,  1894,  1901,  1907,  1913,
    1918,  1951,  1971,  1977,  1996,  2001,  2007,  2011,  2032,  2048,
    2053,  2062,  2089,  2103,  2110,  2118,  2154,  2162,  2196,  2268,
    2273,  2279
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
  "\"end of file\"", "error", "\"invalid token\"", "TOKEN_START", "ARRAY",
  "BEGIN_", "BY", "CASE", "CONST", "DIV", "DO", "ELSE", "ELSIF", "END",
  "FALSE", "FOR", "IF", "IMPORT", "IN", "IS", "MOD", "MODULE", "NIL", "OF",
  "OR", "POINTER", "PROCEDURE", "RECORD", "REPEAT", "RETURN", "THEN", "TO",
  "TRUE", "TYPE", "UNTIL", "VAR", "WHILE", "BECOMES", "DOTDOT", "GE", "LE",
  "IDENT", "INTEGER", "REAL", "STRING", "TOKEN_END", "'.'", "'*'", "'='",
  "','", "'('", "')'", "';'", "':'", "'#'", "'<'", "'>'", "'+'", "'-'",
  "'/'", "'&'", "'~'", "'['", "']'", "'^'", "'{'", "'}'", "'|'", "$accept",
  "qualident", "identdef", "ExportMarkOpt", "number", "ConstDeclaration",
  "ConstExpression", "TypeDeclaration", "TypeIdentDef", "type",
  "ArrayType", "ArrayLengthOf", "LengthRep", "length", "RecordType",
  "RecordHeading", "BaseTypeOpt", "BaseType", "FieldListSequenceOpt",
  "FieldListSequence", "FieldList", "IdentList", "PointerType",
  "PointerTo", "ProcedureType", "ProcedureTypeSansParam",
  "FormalParametersOpt", "VariableDeclaration", "expression", "relation",
  "SimpleExpression", "SignOpt", "AddOperator", "term", "MulOperator",
  "factor", "designator", "SelectorOptRep", "selector", "set",
  "ElementRep", "element", "ExpList", "statement", "assignment",
  "ProcedureCall", "StatementSequence", "StatementSequenceReversed",
  "IfStatement", "guard", "ElseIfThenOptRep", "ElseOpt", "CaseStatement",
  "CaseExpression", "CaseRep", "case", "CaseLabelList", "LabelRange",
  "label", "WhileStatement", "ElseIfDoOptRep", "RepeatStatement",
  "ForStatement", "ForInit", "ForLimit", "ByOpt", "ProcedureDeclaration",
  "ProcedureHeading", "ProcedureHeadingSansParam", "StatementSequenceOpt",
  "ReturnClauseOpt", "DeclarationSequence", "ConstSectionOpt",
  "ConstDeclarationOptRep", "TypeSectionOpt", "TypeKeyword",
  "TypeDeclarationOptRep", "VariableSectionOpt",
  "VariableDeclarationOptRep", "ProcedureDeclarationOptRep",
  "FormalParameters", "FPSectionsOpt", "FPSectionRep", "ResultTypeOpt",
  "FPSection", "ParameterKindOpt", "IdentRep", "FormalType",
  "OpenArrayOptRep", "module", "ModuleHeading", "ImportListOpt",
  "ImportList", "ImportRep", "import", "BecomesIdentOpt",
  "ModuleStatements", YY_NULLPTR
};

static const char *
yysymbol_name (yysymbol_kind_t yysymbol)
{
  return yytname[yysymbol];
}
#endif

#define YYPACT_NINF (-132)

#define yypact_value_is_default(Yyn) \
  ((Yyn) == YYPACT_NINF)

#define YYTABLE_NINF (-166)

#define yytable_value_is_error(Yyn) \
  0

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
static const yytype_int16 yypact[] =
{
      -5,   -21,    49,    -8,  -132,  -132,    37,    19,    54,  -132,
      36,    46,  -132,  -132,    75,    61,    58,  -132,    19,  -132,
      68,   100,  -132,    98,  -132,    90,  -132,  -132,  -132,    70,
      79,    80,    29,    92,    29,   100,    29,  -132,    94,  -132,
    -132,  -132,  -132,    83,  -132,  -132,  -132,  -132,  -132,    93,
    -132,  -132,    68,  -132,  -132,    29,  -132,  -132,  -132,  -132,
      64,     9,   114,   101,   108,  -132,   110,   109,   132,   -32,
      29,   100,    99,    68,   118,   103,    95,    20,  -132,  -132,
    -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,
    -132,    29,     9,  -132,  -132,  -132,  -132,  -132,  -132,    29,
       9,   -24,  -132,     8,  -132,  -132,  -132,    82,    29,    29,
     100,    29,   100,   105,     7,    29,  -132,  -132,  -132,  -132,
    -132,  -132,   -27,    96,    68,    97,   104,   111,  -132,  -132,
      29,   119,  -132,   113,   106,  -132,  -132,  -132,    20,  -132,
      68,  -132,    20,  -132,   111,    33,     8,   107,  -132,  -132,
     115,   -37,  -132,  -132,  -132,  -132,  -132,  -132,     9,  -132,
    -132,  -132,    -4,  -132,    23,  -132,   116,  -132,  -132,   149,
    -132,  -132,  -132,  -132,  -132,  -132,   -12,   -36,    68,    20,
    -132,  -132,  -132,    54,   -20,  -132,  -132,  -132,   -13,  -132,
    -132,   124,  -132,   125,  -132,   146,   126,  -132,    43,  -132,
    -132,  -132,    29,    29,  -132,  -132,  -132,    82,    82,   100,
      82,    29,   150,   102,   117,    29,  -132,  -132,  -132,  -132,
      75,  -132,   120,   127,  -132,   128,  -132,    29,  -132,   129,
    -132,  -132,    68,    20,  -132,  -132,  -132,  -132,  -132,  -132,
    -132,   100,   100,    29,   154,    29,  -132,  -132,   139,   123,
     135,  -132,    44,  -132,  -132,  -132,  -132,   164,  -132,   152,
    -132,   173,    29,   171,   124,  -132,  -132,   145,  -132,  -132,
     100,   100,  -132,   147,  -132,  -132,  -132,    15,  -132,  -132,
    -132,   166,  -132,  -132
};

/* YYDEFACT[STATE-NUM] -- Default reduction number in state STATE-NUM.
   Performed when YYTABLE does not specify something else to do.  Zero
   means the default is an error.  */
static const yytype_uint8 yydefact[] =
{
       0,     0,     0,     0,   172,     1,   174,     0,   142,   173,
     180,     0,   176,   144,   137,   146,     0,   178,     0,   175,
     141,    99,   181,     0,   147,   151,   149,   179,   177,     6,
       0,     0,    56,     0,    56,    99,    56,    78,   101,   103,
      92,    93,   136,   102,    94,    95,    96,    97,    98,     0,
     153,   155,   145,     5,     4,    56,   143,    54,    55,   112,
      42,     0,     0,     0,     0,   106,     0,     0,     0,    76,
      56,    99,     0,   150,   140,     0,     0,     0,     9,    10,
      50,    51,    59,    49,    47,    44,    45,    46,    48,    57,
      58,    56,     0,    71,    69,    70,     7,     8,    68,    56,
       0,    56,    67,    52,    60,    73,    72,   116,    56,    56,
      99,    56,    99,     0,    56,    56,    81,    77,   100,   104,
     171,    33,     0,     0,     0,     0,     0,    40,    12,   148,
      56,     0,    38,    26,     2,    13,    11,    14,     0,    15,
      29,    16,     0,    17,    40,    43,    53,     0,    75,    84,
      88,     0,    86,    64,    65,    62,    63,    66,     0,   121,
     122,   123,     0,   113,     0,   117,   119,   129,   130,   132,
     108,   127,   126,    79,    83,    90,     0,     0,     0,     0,
     152,   135,   154,   142,   158,   134,    39,    22,     0,    20,
      36,     0,    24,     0,    18,     0,    28,    30,     0,    35,
      37,    74,    56,    56,    85,    61,   111,   116,     0,    99,
       0,    56,     0,   110,     0,    56,    82,    80,    34,    41,
     137,   164,     0,   157,   159,     0,    19,    56,    27,     0,
       3,    23,     0,     0,    89,    87,   114,   118,   115,   120,
     131,    99,    99,    56,     0,    56,   124,    91,   139,   162,
     165,   166,     0,    21,    25,    31,    32,     0,   109,     0,
     105,     0,    56,     0,     0,   156,   160,     0,   170,   128,
      99,    99,   138,     0,   161,   167,   163,     0,   107,   125,
     133,     0,   168,   169
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -132,  -102,   -14,  -132,  -132,  -132,   -54,  -132,  -132,  -131,
    -132,  -132,  -132,   -40,  -132,  -132,  -132,  -132,  -132,  -132,
     -42,   121,  -132,  -132,  -132,  -132,    47,  -132,   -30,  -132,
     112,  -132,  -132,   122,  -132,   -92,   -57,  -132,  -132,  -132,
    -132,   -11,    78,   130,  -132,  -132,   -35,  -132,  -132,   -33,
    -132,  -132,  -132,  -132,  -132,    -9,  -132,    -6,   -15,  -132,
    -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,   -16,
    -132,    13,  -132,  -132,  -132,  -132,  -132,  -132,  -132,  -132,
    -132,  -132,  -132,  -132,   -51,  -132,  -132,  -132,  -132,  -132,
    -132,  -132,  -132,  -132,   182,  -132,  -132
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
       0,   135,   121,    54,   102,    31,   187,    76,    77,   136,
     137,   138,   188,   189,   139,   140,   192,   229,   195,   196,
     197,   198,   141,   142,   143,   144,   185,   123,    65,    91,
      60,    61,    92,   103,   158,   104,    38,    69,   117,   106,
     151,   152,   176,    39,    40,    41,    42,    43,    44,    66,
     213,   244,    45,    62,   162,   163,   164,   165,   166,    46,
     214,    47,    48,    64,   169,   212,   125,   126,   127,    22,
     263,    14,    15,    20,    25,    26,    52,    51,    73,    74,
     186,   222,   223,   265,   224,   225,   252,   276,   277,     2,
       3,     8,     9,    11,    12,    17,    23
};

/* YYTABLE[YYPACT[STATE-NUM]] -- What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule whose
   number is the opposite.  If YYTABLE_NINF, syntax error.  */
static const yytype_int16 yytable[] =
{
      67,    78,    59,    68,   105,   161,    30,   194,   148,   206,
     226,   199,   203,   215,   113,   221,     1,   153,   114,   281,
       4,  -165,   178,    93,   130,    79,   179,   217,   154,   204,
     115,    94,   116,    57,    58,   105,   227,   215,    75,   216,
     118,    95,   149,   105,     6,   131,   132,   133,   219,     5,
      37,    96,    97,    98,     7,   155,   134,    82,   174,    99,
      10,   134,    13,   207,    57,    58,   205,   156,   157,   147,
     100,   150,   208,    16,   101,   170,   209,   172,   167,   168,
      21,   171,    80,    81,   175,   175,    57,    58,    82,   228,
      89,    90,   178,   267,    24,    18,   233,   268,    19,    27,
      79,   105,   256,    83,    84,   161,   161,    32,   161,    29,
     181,    49,    85,   242,   243,    33,    34,    53,    86,    87,
      88,    89,    90,   134,   159,    50,   160,    55,    35,   245,
     246,    70,    56,    63,    72,    71,    36,   107,   108,   109,
     110,    37,   112,   111,   124,   120,   173,   129,   180,   182,
     190,   128,   193,   202,   210,   211,   183,   240,   201,   231,
     241,   184,   274,   191,   218,   134,   230,   260,   262,   251,
     221,   249,   234,   150,   238,   282,   264,   269,   232,   250,
     254,    79,   270,   271,   273,   247,   275,   253,   280,   283,
     255,   200,   235,   177,   122,   239,   220,    79,   236,   266,
      28,   119,   237,   145,   248,     0,   257,   258,     0,     0,
     259,     0,   261,     0,   146,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,   272,     0,     0,   278,   279
};

static const yytype_int16 yycheck[] =
{
      35,    55,    32,    36,    61,   107,    20,   138,   100,    13,
      23,   142,    49,    49,    46,    35,    21,     9,    50,     4,
      41,    41,    49,    14,     4,    55,    53,    63,    20,    66,
      62,    22,    64,    57,    58,    92,    49,    49,    52,    51,
      70,    32,    66,   100,    52,    25,    26,    27,   179,     0,
      41,    42,    43,    44,    17,    47,    41,    24,    51,    50,
      41,    41,     8,    67,    57,    58,   158,    59,    60,    99,
      61,   101,    49,    37,    65,   110,    53,   112,   108,   109,
       5,   111,    18,    19,   114,   115,    57,    58,    24,   191,
      57,    58,    49,    49,    33,    49,    53,    53,    52,    41,
     130,   158,   233,    39,    40,   207,   208,     7,   210,    41,
     124,    13,    48,    11,    12,    15,    16,    47,    54,    55,
      56,    57,    58,    41,    42,    35,    44,    48,    28,    12,
      13,    37,    52,    41,    41,    52,    36,    23,    37,    31,
      30,    41,    10,    34,    26,    46,    41,    52,    52,    52,
      31,    48,    46,    38,    38,     6,    52,   211,    51,    13,
      10,    50,   264,    50,   178,    41,    41,    13,    29,    41,
      35,    51,   202,   203,   209,   277,    53,    13,    52,    52,
      51,   211,    30,    10,    13,   215,    41,   227,    41,    23,
     232,   144,   203,   115,    73,   210,   183,   227,   207,   250,
      18,    71,   208,    91,   220,    -1,   241,   242,    -1,    -1,
     243,    -1,   245,    -1,    92,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,   262,    -1,    -1,   270,   271
};

/* YYSTOS[STATE-NUM] -- The symbol kind of the accessing symbol of
   state STATE-NUM.  */
static const yytype_uint8 yystos[] =
{
       0,    21,   157,   158,    41,     0,    52,    17,   159,   160,
      41,   161,   162,     8,   139,   140,    37,   163,    49,    52,
     141,     5,   137,   164,    33,   142,   143,    41,   162,    41,
      70,    73,     7,    15,    16,    28,    36,    41,   104,   111,
     112,   113,   114,   115,   116,   120,   127,   129,   130,    13,
      35,   145,   144,    47,    71,    48,    52,    57,    58,    96,
      98,    99,   121,    41,   131,    96,   117,   114,   117,   105,
      37,    52,    41,   146,   147,    70,    75,    76,    74,    96,
      18,    19,    24,    39,    40,    48,    54,    55,    56,    57,
      58,    97,   100,    14,    22,    32,    42,    43,    44,    50,
      61,    65,    72,   101,   103,   104,   107,    23,    37,    31,
      30,    34,    10,    46,    50,    62,    64,   106,    96,   111,
      46,    70,    89,    95,    26,   134,   135,   136,    48,    52,
       4,    25,    26,    27,    41,    69,    77,    78,    79,    82,
      83,    90,    91,    92,    93,    98,   101,    96,   103,    66,
      96,   108,   109,     9,    20,    47,    59,    60,   102,    42,
      44,    69,   122,   123,   124,   125,   126,    96,    96,   132,
     114,    96,   114,    41,    51,    96,   110,   110,    49,    53,
      52,    70,    52,    52,    50,    94,   148,    74,    80,    81,
      31,    50,    84,    46,    77,    86,    87,    88,    89,    77,
      94,    51,    38,    49,    66,   103,    13,    67,    49,    53,
      38,     6,   133,   118,   128,    49,    51,    63,    70,    77,
     139,    35,   149,   150,   152,   153,    23,    49,    69,    85,
      41,    13,    52,    53,    96,   109,   123,   125,   114,   126,
      74,    10,    11,    12,   119,    12,    13,    96,   137,    51,
      52,    41,   154,    81,    51,    88,    77,   114,   114,   117,
      13,   117,    29,   138,    53,   151,   152,    49,    53,    13,
      30,    10,    96,    13,    69,    41,   155,   156,   114,   114,
      41,     4,    69,    23
};

/* YYR1[RULE-NUM] -- Symbol kind of the left-hand side of rule RULE-NUM.  */
static const yytype_uint8 yyr1[] =
{
       0,    68,    69,    69,    70,    71,    71,    72,    72,    73,
      74,    75,    76,    77,    77,    77,    77,    77,    78,    79,
      80,    80,    81,    82,    83,    84,    84,    85,    86,    86,
      87,    87,    88,    89,    89,    90,    91,    92,    93,    94,
      94,    95,    96,    96,    97,    97,    97,    97,    97,    97,
      97,    97,    98,    98,    99,    99,    99,   100,   100,   100,
     101,   101,   102,   102,   102,   102,   102,   103,   103,   103,
     103,   103,   103,   103,   103,   103,   104,   105,   105,   106,
     106,   106,   106,   106,   107,   107,   108,   108,   109,   109,
     110,   110,   111,   111,   111,   111,   111,   111,   111,   111,
     112,   113,   114,   115,   115,   116,   117,   118,   118,   119,
     119,   120,   121,   122,   122,   123,   123,   124,   124,   125,
     125,   126,   126,   126,   127,   128,   128,   129,   130,   131,
     132,   133,   133,   134,   135,   136,   137,   137,   138,   138,
     139,   140,   140,   141,   141,   142,   142,   143,   144,   144,
     145,   145,   146,   146,   147,   147,   148,   149,   149,   150,
     150,   151,   151,   152,   153,   153,   154,   154,   155,   156,
     156,   157,   158,   159,   159,   160,   161,   161,   162,   163,
     163,   164
};

/* YYR2[RULE-NUM] -- Number of symbols on the right-hand side of rule RULE-NUM.  */
static const yytype_int8 yyr2[] =
{
       0,     2,     1,     3,     2,     1,     0,     1,     1,     3,
       1,     2,     2,     1,     1,     1,     1,     1,     2,     3,
       1,     3,     1,     3,     2,     3,     0,     1,     1,     0,
       1,     3,     3,     1,     3,     2,     2,     2,     1,     1,
       0,     3,     1,     3,     1,     1,     1,     1,     1,     1,
       1,     1,     2,     3,     1,     1,     0,     1,     1,     1,
       1,     3,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     3,     2,     2,     2,     0,     2,
       3,     1,     3,     2,     2,     3,     1,     3,     1,     3,
       1,     3,     1,     1,     1,     1,     1,     1,     1,     0,
       3,     1,     1,     1,     3,     7,     1,     5,     0,     2,
       0,     5,     1,     1,     3,     3,     0,     1,     3,     1,
       3,     1,     1,     1,     6,     5,     0,     4,     8,     3,
       1,     2,     0,     7,     2,     2,     2,     0,     2,     0,
       4,     2,     0,     3,     0,     2,     0,     1,     3,     0,
       2,     0,     3,     0,     3,     0,     4,     1,     0,     1,
       3,     2,     0,     4,     1,     0,     1,     3,     2,     3,
       0,     8,     2,     1,     0,     3,     1,     3,     2,     2,
       0,     1
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
        yyerror (YY_("syntax error: cannot back up")); \
        YYERROR;                                                  \
      }                                                           \
  while (0)

/* Backward compatibility with an undocumented macro.
   Use YYerror or YYUNDEF. */
#define YYERRCODE YYUNDEF


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




# define YY_SYMBOL_PRINT(Title, Kind, Value, Location)                    \
do {                                                                      \
  if (yydebug)                                                            \
    {                                                                     \
      YYFPRINTF (stderr, "%s ", Title);                                   \
      yy_symbol_print (stderr,                                            \
                  Kind, Value); \
      YYFPRINTF (stderr, "\n");                                           \
    }                                                                     \
} while (0)


/*-----------------------------------.
| Print this symbol's value on YYO.  |
`-----------------------------------*/

static void
yy_symbol_value_print (FILE *yyo,
                       yysymbol_kind_t yykind, YYSTYPE const * const yyvaluep)
{
  FILE *yyoutput = yyo;
  YY_USE (yyoutput);
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
                 yysymbol_kind_t yykind, YYSTYPE const * const yyvaluep)
{
  YYFPRINTF (yyo, "%s %s (",
             yykind < YYNTOKENS ? "token" : "nterm", yysymbol_name (yykind));

  yy_symbol_value_print (yyo, yykind, yyvaluep);
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
yy_reduce_print (yy_state_t *yyssp, YYSTYPE *yyvsp,
                 int yyrule)
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
                       &yyvsp[(yyi + 1) - (yynrhs)]);
      YYFPRINTF (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)          \
do {                                    \
  if (yydebug)                          \
    yy_reduce_print (yyssp, yyvsp, Rule); \
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
            yysymbol_kind_t yykind, YYSTYPE *yyvaluep)
{
  YY_USE (yyvaluep);
  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yykind, yyvaluep, yylocationp);

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  YY_USE (yykind);
  YY_IGNORE_MAYBE_UNINITIALIZED_END
}


/* Lookahead token kind.  */
int yychar;

/* The semantic value of the lookahead symbol.  */
YYSTYPE yylval;
/* Number of syntax errors so far.  */
int yynerrs;




/*----------.
| yyparse.  |
`----------*/

int
yyparse (void)
{
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

  int yyn;
  /* The return value of yyparse.  */
  int yyresult;
  /* Lookahead symbol kind.  */
  yysymbol_kind_t yytoken = YYSYMBOL_YYEMPTY;
  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;

  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYPTRDIFF_T yymsg_alloc = sizeof yymsgbuf;

#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N))

  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yychar = YYEMPTY; /* Cause a token to be read.  */

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

        /* Each stack pointer address is followed by the size of the
           data in use in that stack, in bytes.  This used to be a
           conditional around just the two extra args, but that might
           be undefined if yyoverflow is a macro.  */
        yyoverflow (YY_("memory exhausted"),
                    &yyss1, yysize * YYSIZEOF (*yyssp),
                    &yyvs1, yysize * YYSIZEOF (*yyvsp),
                    &yystacksize);
        yyss = yyss1;
        yyvs = yyvs1;
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
#  undef YYSTACK_RELOCATE
        if (yyss1 != yyssa)
          YYSTACK_FREE (yyss1);
      }
# endif

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;

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
      yychar = yylex ();
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


  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
  case 2: /* qualident: IDENT  */
#line 222 "Oberon.y"
        {
		(yyval.node) = Trees_NewIdent((yyvsp[0].ident));
	}
#line 1950 "y.tab.c"
    break;

  case 3: /* qualident: IDENT '.' IDENT  */
#line 226 "Oberon.y"
        {
		(yyval.node) = Trees_NewIdent(Util_String("%s.%s", (yyvsp[-2].ident), (yyvsp[0].ident)));
	}
#line 1958 "y.tab.c"
    break;

  case 4: /* identdef: IDENT ExportMarkOpt  */
#line 233 "Oberon.y"
        {
		if (! Table_LocallyDeclared((yyvsp[-1].ident)) || (recordDeclarationStack != NULL)) {
			(yyval.node) = Trees_NewIdent((yyvsp[-1].ident));
			if ((yyvsp[0].integer)) {
				Trees_SetExported((yyval.node));
			}
			if (Table_ScopeLocal()) {
				Trees_SetLocal((yyval.node));
			}
		} else {
			Oberon_PrintError("error: redeclaration of identifier: %s", (yyvsp[-1].ident));
			YYABORT;
		}
	}
#line 1977 "y.tab.c"
    break;

  case 5: /* ExportMarkOpt: '*'  */
#line 251 "Oberon.y"
        {
		(yyval.integer) = 1;
	}
#line 1985 "y.tab.c"
    break;

  case 6: /* ExportMarkOpt: %empty  */
#line 255 "Oberon.y"
        {
		(yyval.integer) = 0;
	}
#line 1993 "y.tab.c"
    break;

  case 7: /* number: INTEGER  */
#line 265 "Oberon.y"
        {
		(yyval.node) = Trees_NewInteger((yyvsp[0].integer));
	}
#line 2001 "y.tab.c"
    break;

  case 8: /* number: REAL  */
#line 269 "Oberon.y"
        {
		(yyval.node) = Trees_NewReal((yyvsp[0].real));
	}
#line 2009 "y.tab.c"
    break;

  case 9: /* ConstDeclaration: identdef '=' ConstExpression  */
#line 279 "Oberon.y"
        {
		if (! (Trees_Exported((yyvsp[-2].node)) && Trees_Local((yyvsp[-2].node)))) {
			Trees_SetKind(TREES_CONSTANT_KIND, (yyvsp[-2].node));
			Trees_SetType(Trees_Type((yyvsp[0].node)), (yyvsp[-2].node));
			Trees_SetValue((yyvsp[0].node), (yyvsp[-2].node));
			Table_Put((yyvsp[-2].node));
			Generate_ConstDeclaration((yyvsp[-2].node));
		} else {
			Oberon_PrintError("error: cannot export local constant: %s", Trees_Name((yyvsp[-2].node)));
			YYABORT;
		}
	}
#line 2026 "y.tab.c"
    break;

  case 10: /* ConstExpression: expression  */
#line 295 "Oberon.y"
        {
		if (IsConstExpression((yyvsp[0].node))) {
			(yyval.node) = (yyvsp[0].node);
		} else {
			Oberon_PrintError("error: constant expression expected");
			YYABORT;
		}
	}
#line 2039 "y.tab.c"
    break;

  case 11: /* TypeDeclaration: TypeIdentDef type  */
#line 310 "Oberon.y"
        {
		Trees_Node sourceType;

		sourceType = ResolvedType((yyvsp[0].node), 1);
		if (sourceType != NULL) {
			if (! (Trees_Exported((yyvsp[-1].node)) && Trees_Local((yyvsp[-1].node)))) {
				Trees_SetType(sourceType, (yyvsp[-1].node));
				ResolvePointerTypes((yyvsp[-1].node));
				currentTypeIdentdef = NULL;
				Generate_TypeDeclaration((yyvsp[-1].node));
			} else {
				Oberon_PrintError("error: cannot export local type: %s", Trees_Name((yyvsp[-1].node)));
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			YYABORT;
		}
	}
#line 2063 "y.tab.c"
    break;

  case 12: /* TypeIdentDef: identdef '='  */
#line 333 "Oberon.y"
        {
		Trees_SetKind(TREES_TYPE_KIND, (yyvsp[-1].node));
		currentTypeIdentdef = (yyvsp[-1].node);
		Table_Put((yyvsp[-1].node));
		(yyval.node) = (yyvsp[-1].node);
	}
#line 2074 "y.tab.c"
    break;

  case 18: /* ArrayType: ArrayLengthOf type  */
#line 351 "Oberon.y"
        {
		Trees_Node reversedLengths, length;

		(yyval.node) = ResolvedType((yyvsp[0].node), 0);
		if ((yyval.node) != NULL) {
			reversedLengths = (yyvsp[-1].node);
			do {
				length = Trees_Left(reversedLengths);
				(yyval.node) = Types_NewArray(length, (yyval.node));
				reversedLengths = Trees_Right(reversedLengths);
			} while (reversedLengths != NULL);
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			exit(EXIT_FAILURE);
		}
	}
#line 2095 "y.tab.c"
    break;

  case 19: /* ArrayLengthOf: ARRAY LengthRep OF  */
#line 371 "Oberon.y"
        {
		if ((currentTypeIdentdef != NULL) && (Trees_Type(currentTypeIdentdef) == NULL)) {
			Trees_SetType(Trees_NewLeaf(ARRAY), currentTypeIdentdef); /*incomplete type*/
		}
		(yyval.node) = (yyvsp[-1].node);
	}
#line 2106 "y.tab.c"
    break;

  case 20: /* LengthRep: length  */
#line 381 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), NULL);
	}
#line 2114 "y.tab.c"
    break;

  case 21: /* LengthRep: LengthRep ',' length  */
#line 385 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), (yyvsp[-2].node));
	}
#line 2122 "y.tab.c"
    break;

  case 22: /* length: ConstExpression  */
#line 392 "Oberon.y"
        {
		if (Types_IsInteger(Trees_Type((yyvsp[0].node)))) {
			if (IsInteger((yyvsp[0].node))) {
				if (Trees_Integer((yyvsp[0].node)) <= 0) {
					Oberon_PrintError("error: positive length expected: %" OBNC_INT_MOD "d", Trees_Integer((yyvsp[0].node)));
					YYABORT;
				}
			} else {
				Oberon_PrintError("error: fully evaluated constant expression expected as increment");
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: integer length expected");
			YYABORT;
		}
	}
#line 2143 "y.tab.c"
    break;

  case 23: /* RecordType: RecordHeading FieldListSequenceOpt END  */
#line 412 "Oberon.y"
        {
		recordDeclarationStack = Trees_Right(recordDeclarationStack);
		(yyval.node) = Types_NewRecord(Types_RecordBaseType((yyvsp[-2].node)), (yyvsp[-1].node));
	}
#line 2152 "y.tab.c"
    break;

  case 24: /* RecordHeading: RECORD BaseTypeOpt  */
#line 420 "Oberon.y"
        {
		(yyval.node) = Types_NewRecord((yyvsp[0].node), NULL);
		if ((currentTypeIdentdef != NULL) && (Trees_Type(currentTypeIdentdef) == NULL)) {
			Trees_SetType((yyval.node), currentTypeIdentdef);
		}
		recordDeclarationStack = Trees_NewNode(TREES_NOSYM, (yyval.node), recordDeclarationStack);
	}
#line 2164 "y.tab.c"
    break;

  case 25: /* BaseTypeOpt: '(' BaseType ')'  */
#line 431 "Oberon.y"
        {
		(yyval.node) = (yyvsp[-1].node);
	}
#line 2172 "y.tab.c"
    break;

  case 26: /* BaseTypeOpt: %empty  */
#line 435 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 2180 "y.tab.c"
    break;

  case 27: /* BaseType: qualident  */
#line 442 "Oberon.y"
        {
		const char *name;
		Trees_Node symbol;

		(yyval.node) = NULL;
		name = Trees_Name((yyvsp[0].node));
		symbol = Table_At(name);
		if (symbol != NULL) {
			if (Trees_Kind(symbol) == TREES_TYPE_KIND) {
				if (symbol != currentTypeIdentdef) {
					switch (Trees_Symbol(Types_Structure(symbol))) {
						case POINTER:
							if (Types_Same(Types_PointerBaseType(symbol), currentTypeIdentdef)) {
								Oberon_PrintError("error: self-referring base type: %s", name);
								YYABORT;
							}
							/*fall through*/
						case RECORD:
							(yyval.node) = symbol;
							break;
						default:
							Oberon_PrintError("error: record or pointer base type expected: %s", name);
							YYABORT;
					}
				} else {
					Oberon_PrintError("error: self-referring base type: %s", name);
					YYABORT;
				}
			} else {
				Oberon_PrintError("error: type name expected: %s", name);
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", name);
			YYABORT;
		}
	}
#line 2222 "y.tab.c"
    break;

  case 28: /* FieldListSequenceOpt: FieldListSequence  */
#line 483 "Oberon.y"
        {
		Trees_ReverseList(&(yyvsp[0].node)); /*correct order*/
		(yyval.node) = (yyvsp[0].node);
	}
#line 2231 "y.tab.c"
    break;

  case 29: /* FieldListSequenceOpt: %empty  */
#line 488 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 2239 "y.tab.c"
    break;

  case 30: /* FieldListSequence: FieldList  */
#line 495 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_FIELD_LIST_SEQUENCE, (yyvsp[0].node), NULL);
	}
#line 2247 "y.tab.c"
    break;

  case 31: /* FieldListSequence: FieldListSequence ';' FieldList  */
#line 499 "Oberon.y"
        {
		Trees_Node currSeq, currList, currSeqList;
		const char *seqIdentName, *listIdentName;

		currList = (yyvsp[0].node);
		while (currList != NULL) {
			listIdentName = Trees_Name(Trees_Left(currList));
			currSeq = (yyvsp[-2].node);
			while (currSeq != NULL) {
				currSeqList = Trees_Left(currSeq);
				while (currSeqList != NULL) {
					seqIdentName = Trees_Name(Trees_Left(currSeqList));
					if (strcmp(listIdentName, seqIdentName) == 0) {
						Oberon_PrintError("error: redeclaration of field: %s", listIdentName);
						YYABORT;
					}
					currSeqList = Trees_Right(currSeqList);
				}
				currSeq = Trees_Right(currSeq);
			}
			currList = Trees_Right(currList);
		}
		(yyval.node) = Trees_NewNode(TREES_FIELD_LIST_SEQUENCE, (yyvsp[0].node), (yyvsp[-2].node));
	}
#line 2276 "y.tab.c"
    break;

  case 32: /* FieldList: IdentList ':' type  */
#line 527 "Oberon.y"
        {
		Trees_Node type, tail, ident, p, directBaseType, baseTypeField, baseType;

		(yyval.node) = NULL;
		type = ResolvedType((yyvsp[0].node), 0);
		if (type != NULL) {
			if (! ((type == currentTypeIdentdef) && ! Types_IsPointer(type))) {
				Trees_ReverseList(&(yyvsp[-2].node)); /*correct order*/
				tail = (yyvsp[-2].node);
				do {
					ident = Trees_Left(tail);
					p = (yyvsp[-2].node);
					while ((p != tail) && (strcmp(Trees_Name(ident), Trees_Name(Trees_Left(p))) != 0)) {
						p = Trees_Right(p);
					}
					if (p == tail) {
						assert(recordDeclarationStack != NULL);
						directBaseType = Types_RecordBaseType(Trees_Left(recordDeclarationStack));
						if (directBaseType != NULL) {
							Types_GetFieldIdent(Trees_Name(ident), directBaseType, 0, &baseTypeField, &baseType);
						}
						if ((directBaseType == NULL) || (baseTypeField == NULL)) {
							Trees_SetKind(TREES_FIELD_KIND, ident);
							Trees_SetType(type, ident);
						} else {
							Oberon_PrintError("error: redeclaration of field: %s defined in %s", Trees_Name(ident), Trees_Name(baseType));
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: redeclaration of field: %s", Trees_Name(ident));
						YYABORT;
					}
					tail = Trees_Right(tail);
				} while (tail != NULL);

				(yyval.node) = (yyvsp[-2].node);
			} else {
				Oberon_PrintError("error: recursive field type must be a pointer: %s", Trees_Name((yyvsp[0].node)));
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: undeclared type: %s", Trees_Name((yyvsp[0].node)));
			YYABORT;
		}
	}
#line 2326 "y.tab.c"
    break;

  case 33: /* IdentList: identdef  */
#line 576 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_IDENT_LIST, (yyvsp[0].node), NULL);
	}
#line 2334 "y.tab.c"
    break;

  case 34: /* IdentList: IdentList ',' identdef  */
#line 580 "Oberon.y"
        {
		Trees_Node reversedIdents;

		reversedIdents = Trees_NewNode(TREES_IDENT_LIST, (yyvsp[0].node), (yyvsp[-2].node));
		(yyval.node) = reversedIdents;
	}
#line 2345 "y.tab.c"
    break;

  case 35: /* PointerType: PointerTo type  */
#line 590 "Oberon.y"
        {
		const char *baseTypeName;
		Trees_Node declaredBaseType;

		(yyval.node) = NULL;
		if (Trees_Symbol((yyvsp[0].node)) == IDENT) {
			baseTypeName = Trees_Name((yyvsp[0].node));
			declaredBaseType = Table_At(baseTypeName);
			if (declaredBaseType != NULL) {
				if (Types_IsRecord(declaredBaseType)) {
					(yyval.node) = Types_NewPointer(declaredBaseType);
				} else {
					Oberon_PrintError("error: record expected as pointer base type: %s", baseTypeName);
					YYABORT;
				}
			} else if (currentTypeIdentdef != NULL) {
				Trees_SetKind(TREES_TYPE_KIND, (yyvsp[0].node));
				Trees_SetType(Types_NewRecord(NULL, NULL), (yyvsp[0].node));
				(yyval.node) = Types_NewPointer((yyvsp[0].node));
				unresolvedPointerTypes = Trees_NewNode(TREES_NOSYM, (yyval.node), unresolvedPointerTypes);
			} else {
				Oberon_PrintError("error: undeclared type: %s", baseTypeName);
				YYABORT;
			}
		} else if(Trees_Symbol((yyvsp[0].node)) == RECORD) {
			(yyval.node) = Types_NewPointer((yyvsp[0].node));
		} else {
			Oberon_PrintError("error: record expected as pointer base type");
			YYABORT;
		}
	}
#line 2381 "y.tab.c"
    break;

  case 36: /* PointerTo: POINTER TO  */
#line 625 "Oberon.y"
        {
		if ((currentTypeIdentdef != NULL) && (Trees_Type(currentTypeIdentdef) == NULL)) {
			Trees_SetType(Types_NewPointer(NULL), currentTypeIdentdef); /*incomplete type*/
		}
	}
#line 2391 "y.tab.c"
    break;

  case 37: /* ProcedureType: ProcedureTypeSansParam FormalParametersOpt  */
#line 634 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
	}
#line 2399 "y.tab.c"
    break;

  case 38: /* ProcedureTypeSansParam: PROCEDURE  */
#line 641 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 2407 "y.tab.c"
    break;

  case 40: /* FormalParametersOpt: %empty  */
#line 649 "Oberon.y"
        {
		(yyval.node) = Trees_NewLeaf(PROCEDURE);
	}
#line 2415 "y.tab.c"
    break;

  case 41: /* VariableDeclaration: IdentList ':' type  */
#line 659 "Oberon.y"
        {
		Trees_Node type, identList, ident;

		type = ResolvedType((yyvsp[0].node), 0);
		if (type != NULL) {
			Trees_ReverseList(&(yyvsp[-2].node)); /*correct order*/
			identList = (yyvsp[-2].node);
			do {
				ident = Trees_Left(identList);
				if (! (Trees_Exported(ident) && Trees_Local(ident))) {
					if (! Table_LocallyDeclared(Trees_Name(ident))) {
						Trees_SetKind(TREES_VARIABLE_KIND, ident);
						Trees_SetType(type, ident);
						Table_Put(ident);
					} else {
						Oberon_PrintError("error: redeclaration of identifier with the same name: %s", Trees_Name(ident));
						YYABORT;
					}
				} else {
					Oberon_PrintError("error: cannot export local variable: %s", Trees_Name(ident));
					YYABORT;
				}
				identList = Trees_Right(identList);
			} while (identList != NULL);

			Generate_VariableDeclaration((yyvsp[-2].node));
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			exit(EXIT_FAILURE);
		}
	}
#line 2451 "y.tab.c"
    break;

  case 43: /* expression: SimpleExpression relation SimpleExpression  */
#line 698 "Oberon.y"
        {
		Trees_Node expA, expB, typeA, typeB;
		int op = (int) (yyvsp[-1].integer);

		expA = (yyvsp[-2].node);
		expB = (yyvsp[0].node);
		typeA = Trees_Type((yyvsp[-2].node));
		typeB = Trees_Type((yyvsp[0].node));

		CheckIsValueExpression((yyvsp[-2].node));
		if (op == IS) {
			if (IsDesignator((yyvsp[-2].node))) {
				if (! (Types_IsRecord(typeA) && (Trees_Kind(BaseIdent((yyvsp[-2].node))) != TREES_VAR_PARAM_KIND))) {
					expB = BaseIdent((yyvsp[0].node));
					typeB = BaseIdent((yyvsp[0].node));
				} else {
					Oberon_PrintError("error: variable parameter expected as first operand of IS");
					YYABORT;
				}
			} else {
				Oberon_PrintError("error: identifier expected as first operand of IS");
				YYABORT;
			}
		} else {
			CheckIsValueExpression((yyvsp[0].node));
		}
		if (Types_ExpressionCompatible(op, typeA, typeB)) {
			(yyval.node) = ExpressionConstValue(op, expA, expB);
			if ((yyval.node) == NULL) {
				if (IsString(expA) && Types_IsChar(typeB)) {
					expA = Trees_NewChar(Trees_String(expA)[0]);
				} else if (Types_IsChar(typeA) && IsString(expB)) {
					expB = Trees_NewChar(Trees_String(expB)[0]);
				}
				(yyval.node) = Trees_NewNode(op, expA, expB);
				Trees_SetType(Trees_NewLeaf(TREES_BOOLEAN_TYPE), (yyval.node));
			}
		} else {
			Oberon_PrintError("error: incompatible types in relation \"%s\": %s, %s",
				OperatorString(op), TypeString(typeA), TypeString(typeB));
			YYABORT;
		}
	}
#line 2499 "y.tab.c"
    break;

  case 44: /* relation: '='  */
#line 745 "Oberon.y"
        {
		(yyval.integer) = '=';
	}
#line 2507 "y.tab.c"
    break;

  case 45: /* relation: '#'  */
#line 749 "Oberon.y"
        {
		(yyval.integer) = '#';
	}
#line 2515 "y.tab.c"
    break;

  case 46: /* relation: '<'  */
#line 753 "Oberon.y"
        {
		(yyval.integer) = '<';
	}
#line 2523 "y.tab.c"
    break;

  case 47: /* relation: LE  */
#line 757 "Oberon.y"
        {
		(yyval.integer) = LE;
	}
#line 2531 "y.tab.c"
    break;

  case 48: /* relation: '>'  */
#line 761 "Oberon.y"
        {
		(yyval.integer) = '>';
	}
#line 2539 "y.tab.c"
    break;

  case 49: /* relation: GE  */
#line 765 "Oberon.y"
        {
		(yyval.integer) = GE;
	}
#line 2547 "y.tab.c"
    break;

  case 50: /* relation: IN  */
#line 769 "Oberon.y"
        {
		(yyval.integer) = IN;
	}
#line 2555 "y.tab.c"
    break;

  case 51: /* relation: IS  */
#line 773 "Oberon.y"
        {
		(yyval.integer) = IS;
	}
#line 2563 "y.tab.c"
    break;

  case 52: /* SimpleExpression: SignOpt term  */
#line 780 "Oberon.y"
        {
		int op = (int) (yyvsp[-1].integer);
		(yyval.node) = (yyvsp[0].node);
		if (op >= 0) {
			CheckIsValueExpression((yyvsp[0].node));
			if (Types_ExpressionCompatible(op, Trees_Type((yyvsp[0].node)), NULL)) {
				(yyval.node) = SimpleExpressionConstValue(op, (yyvsp[0].node), NULL);
				if ((yyval.node) == NULL) {
					(yyval.node) = Trees_NewNode(op, (yyvsp[0].node), NULL);
					if (Types_IsByte(Trees_Type((yyvsp[0].node)))) {
						Trees_SetType(Trees_NewLeaf(TREES_INTEGER_TYPE), (yyval.node));
					} else {
						Trees_SetType(Trees_Type((yyvsp[0].node)), (yyval.node));
					}
				}
			} else {
				Oberon_PrintError("error: incompatible type in unary operation \"%s\": %s", OperatorString(op), TypeString(Trees_Type((yyvsp[0].node))));
				YYABORT;
			}
		}
	}
#line 2589 "y.tab.c"
    break;

  case 53: /* SimpleExpression: SimpleExpression AddOperator term  */
#line 802 "Oberon.y"
        {
		int op = (int) (yyvsp[-1].integer);

		(yyval.node) = NULL;

		CheckIsValueExpression((yyvsp[-2].node));
		CheckIsValueExpression((yyvsp[0].node));

		if (Types_ExpressionCompatible(op, Trees_Type((yyvsp[-2].node)), Trees_Type((yyvsp[0].node)))) {
			(yyval.node) = SimpleExpressionConstValue(op, (yyvsp[-2].node), (yyvsp[0].node));
			if ((yyval.node) == NULL) {
				(yyval.node) = Trees_NewNode(op, (yyvsp[-2].node), (yyvsp[0].node));
				if (Types_IsByte(Trees_Type((yyvsp[-2].node))) || Types_IsByte(Trees_Type((yyvsp[0].node)))) {
					Trees_SetType(Trees_NewLeaf(TREES_INTEGER_TYPE), (yyval.node));
				} else {
					Trees_SetType(Trees_Type((yyvsp[-2].node)), (yyval.node));
				}
			}
		} else {
			Oberon_PrintError("error: incompatible types in operation \"%s\": %s, %s",
				OperatorString(op), TypeString(Trees_Type((yyvsp[-2].node))), TypeString(Trees_Type((yyvsp[0].node))));
			YYABORT;
		}
		assert((yyval.node) != NULL);
	}
#line 2619 "y.tab.c"
    break;

  case 54: /* SignOpt: '+'  */
#line 831 "Oberon.y"
        {
		(yyval.integer) = '+';
	}
#line 2627 "y.tab.c"
    break;

  case 55: /* SignOpt: '-'  */
#line 835 "Oberon.y"
        {
		(yyval.integer) = '-';
	}
#line 2635 "y.tab.c"
    break;

  case 56: /* SignOpt: %empty  */
#line 839 "Oberon.y"
        {
		(yyval.integer) = -1;
	}
#line 2643 "y.tab.c"
    break;

  case 57: /* AddOperator: '+'  */
#line 846 "Oberon.y"
        {
		(yyval.integer) = '+';
	}
#line 2651 "y.tab.c"
    break;

  case 58: /* AddOperator: '-'  */
#line 850 "Oberon.y"
        {
		(yyval.integer) = '-';
	}
#line 2659 "y.tab.c"
    break;

  case 59: /* AddOperator: OR  */
#line 854 "Oberon.y"
        {
		(yyval.integer) = OR;
	}
#line 2667 "y.tab.c"
    break;

  case 61: /* term: term MulOperator factor  */
#line 862 "Oberon.y"
        {
		int op = (int) (yyvsp[-1].integer);

		(yyval.node) = NULL;

		CheckIsValueExpression((yyvsp[-2].node));
		CheckIsValueExpression((yyvsp[0].node));

		if (Types_ExpressionCompatible(op, Trees_Type((yyvsp[-2].node)), Trees_Type((yyvsp[0].node)))) {
			(yyval.node) = TermConstValue(op, (yyvsp[-2].node), (yyvsp[0].node));
			if ((yyval.node) == NULL) {
				(yyval.node) = Trees_NewNode(op, (yyvsp[-2].node), (yyvsp[0].node));
				if (Types_IsByte(Trees_Type((yyvsp[-2].node))) || Types_IsByte(Trees_Type((yyvsp[0].node)))) {
					Trees_SetType(Trees_NewLeaf(TREES_INTEGER_TYPE), (yyval.node));
				} else {
					Trees_SetType(Trees_Type((yyvsp[-2].node)), (yyval.node));
				}
			}
		} else {
			Oberon_PrintError("error: incompatible types in operation \"%s\": %s, %s",
				OperatorString(op), TypeString(Trees_Type((yyvsp[-2].node))), TypeString(Trees_Type((yyvsp[0].node))));
			YYABORT;
		}

		assert((yyval.node) != NULL);
	}
#line 2698 "y.tab.c"
    break;

  case 62: /* MulOperator: '*'  */
#line 892 "Oberon.y"
        {
		(yyval.integer) = '*';
	}
#line 2706 "y.tab.c"
    break;

  case 63: /* MulOperator: '/'  */
#line 896 "Oberon.y"
        {
		(yyval.integer) = '/';
	}
#line 2714 "y.tab.c"
    break;

  case 64: /* MulOperator: DIV  */
#line 900 "Oberon.y"
        {
		(yyval.integer) = DIV;
	}
#line 2722 "y.tab.c"
    break;

  case 65: /* MulOperator: MOD  */
#line 904 "Oberon.y"
        {
		(yyval.integer) = MOD;
	}
#line 2730 "y.tab.c"
    break;

  case 66: /* MulOperator: '&'  */
#line 908 "Oberon.y"
        {
		(yyval.integer) = '&';
	}
#line 2738 "y.tab.c"
    break;

  case 68: /* factor: STRING  */
#line 916 "Oberon.y"
        {
		(yyval.node) = Trees_NewString((yyvsp[0].string));
	}
#line 2746 "y.tab.c"
    break;

  case 69: /* factor: NIL  */
#line 920 "Oberon.y"
        {
		(yyval.node) = Trees_NewLeaf(NIL);
		Trees_SetType(Trees_NewLeaf(TREES_NIL_TYPE), (yyval.node));
	}
#line 2755 "y.tab.c"
    break;

  case 70: /* factor: TRUE  */
#line 925 "Oberon.y"
        {
		(yyval.node) = Trees_NewBoolean(1);
	}
#line 2763 "y.tab.c"
    break;

  case 71: /* factor: FALSE  */
#line 929 "Oberon.y"
        {
		(yyval.node) = Trees_NewBoolean(0);
	}
#line 2771 "y.tab.c"
    break;

  case 72: /* factor: set  */
#line 933 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
		Trees_SetType(Trees_NewLeaf(TREES_SET_TYPE), (yyval.node));
	}
#line 2780 "y.tab.c"
    break;

  case 73: /* factor: designator  */
#line 939 "Oberon.y"
        {
		const int isFunctionCall = 1;
		Trees_Node designator, actualParameters, ident;

		(yyval.node) = NULL;
		if (Trees_Symbol((yyvsp[0].node)) == TREES_PROCEDURE_CALL) {
			designator = Trees_Left((yyvsp[0].node));
			actualParameters = Trees_Right((yyvsp[0].node));
			HandleProcedureCall(designator, actualParameters, isFunctionCall, &(yyval.node));
		} else {
			ident = Trees_Left((yyvsp[0].node));
			if (Trees_Kind(ident) == TREES_CONSTANT_KIND) {
				(yyval.node) = Trees_Value(ident);
			} else {
				(yyval.node) = (yyvsp[0].node);
			}
		}
		assert((yyval.node) != NULL);
	}
#line 2804 "y.tab.c"
    break;

  case 74: /* factor: '(' expression ')'  */
#line 959 "Oberon.y"
        {
		CheckIsValueExpression((yyvsp[-1].node));
		(yyval.node) = (yyvsp[-1].node);
	}
#line 2813 "y.tab.c"
    break;

  case 75: /* factor: '~' factor  */
#line 964 "Oberon.y"
        {
		(yyval.node) = NULL;
		CheckIsValueExpression((yyvsp[0].node));
		if (Types_ExpressionCompatible('~', Trees_Type((yyvsp[0].node)), NULL)) {
			if (IsBoolean((yyvsp[0].node))) {
				(yyval.node) = Trees_NewBoolean(! Trees_Boolean((yyvsp[0].node)));
			} else {
				(yyval.node) = Trees_NewNode('~', (yyvsp[0].node), NULL);
				Trees_SetType(Trees_NewLeaf(TREES_BOOLEAN_TYPE), (yyval.node));
			}
		} else {
			Oberon_PrintError("error: incompatible type in operation \"~\": %s", TypeString(Trees_Type((yyvsp[0].node))));
			YYABORT;
		}
		assert((yyval.node) != NULL);
	}
#line 2834 "y.tab.c"
    break;

  case 76: /* designator: IDENT SelectorOptRep  */
#line 985 "Oberon.y"
        {
		Trees_Node designator, identType, actualParameters;
		int parameterListFound; /*possibly empty*/

		Trees_ReverseList(&(yyvsp[0].node)); /*correct order*/
		designator = Designator((yyvsp[-1].ident), (yyvsp[0].node));

		identType = Trees_Type(BaseIdent(designator));
		SetSelectorTypes(identType, designator, &parameterListFound);
		if (parameterListFound) {
			RemoveActualParameters(&designator, &actualParameters);
			(yyval.node) = Trees_NewNode(TREES_PROCEDURE_CALL, designator, actualParameters);
		} else {
			(yyval.node) = designator;
		}
	}
#line 2855 "y.tab.c"
    break;

  case 77: /* SelectorOptRep: SelectorOptRep selector  */
#line 1005 "Oberon.y"
        {
		Trees_Node curr;

		if ((Trees_Symbol((yyvsp[0].node)) == '[') && (Trees_Right((yyvsp[0].node)) != NULL)) { /*multi-dimensional element selector*/
			Trees_ReverseList(&(yyvsp[0].node));
			/*append $1 to $2*/
			curr = (yyvsp[0].node);
			do {
				curr = Trees_Right(curr);
			} while (Trees_Right(curr) != NULL);
			Trees_SetRight((yyvsp[-1].node), curr);
			(yyval.node) = (yyvsp[0].node);
		} else {
			(yyval.node) = Trees_NewNode(Trees_Symbol((yyvsp[0].node)), Trees_Left((yyvsp[0].node)), (yyvsp[-1].node));
		}
	}
#line 2876 "y.tab.c"
    break;

  case 78: /* SelectorOptRep: %empty  */
#line 1022 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 2884 "y.tab.c"
    break;

  case 79: /* selector: '.' IDENT  */
#line 1029 "Oberon.y"
        {
		Trees_Node field;

		field = Trees_NewIdent((yyvsp[0].ident));
		Trees_SetKind(TREES_FIELD_KIND, field);
		(yyval.node) = Trees_NewNode('.', field, NULL);
	}
#line 2896 "y.tab.c"
    break;

  case 80: /* selector: '[' ExpList ']'  */
#line 1037 "Oberon.y"
        {
		Trees_Node curr, exp;

		/*create one selector node per index*/
		(yyval.node) = NULL;
		curr = (yyvsp[-1].node); /*NOTE: ExpList is reversed*/
		do {
			exp = Trees_Left(curr);
			if (Types_IsInteger(Trees_Type(exp))) {
				(yyval.node) = Trees_NewNode('[', Trees_Left(curr), (yyval.node));
			} else {
				Oberon_PrintError("error: integer array index expected");
				YYABORT;
			}
			curr = Trees_Right(curr);
		} while (curr != NULL);
	}
#line 2918 "y.tab.c"
    break;

  case 81: /* selector: '^'  */
#line 1055 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode('^', NULL, NULL);
	}
#line 2926 "y.tab.c"
    break;

  case 82: /* selector: '(' ExpList ')'  */
#line 1060 "Oberon.y"
        {
		Trees_ReverseList(&(yyvsp[-1].node)); /*correct order*/
		(yyval.node) = Trees_NewNode('(', (yyvsp[-1].node), NULL);
	}
#line 2935 "y.tab.c"
    break;

  case 83: /* selector: '(' ')'  */
#line 1065 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode('(', NULL, NULL);
	}
#line 2943 "y.tab.c"
    break;

  case 84: /* set: '{' '}'  */
#line 1072 "Oberon.y"
        {
		(yyval.node) = Trees_NewSet(0x0u);
	}
#line 2951 "y.tab.c"
    break;

  case 85: /* set: '{' ElementRep '}'  */
#line 1076 "Oberon.y"
        {
		(yyval.node) = (yyvsp[-1].node);
	}
#line 2959 "y.tab.c"
    break;

  case 87: /* ElementRep: ElementRep ',' element  */
#line 1084 "Oberon.y"
        {
		if ((Trees_Symbol((yyvsp[-2].node)) == TREES_SET_CONSTANT)
				&& (Trees_Symbol((yyvsp[0].node)) == TREES_SET_CONSTANT)) {
			(yyval.node) = Trees_NewSet(Trees_Set((yyvsp[-2].node)) | Trees_Set((yyvsp[0].node)));
		} else {
			(yyval.node) = Trees_NewNode('+', (yyvsp[-2].node), (yyvsp[0].node));
			Trees_SetType(Trees_NewLeaf(TREES_SET_TYPE), (yyval.node));
		}
	}
#line 2973 "y.tab.c"
    break;

  case 88: /* element: expression  */
#line 1097 "Oberon.y"
        {
		OBNC_INTEGER i;
		Trees_Node type;

		CheckIsValueExpression((yyvsp[0].node));
		(yyval.node) = NULL;
		type = Trees_Type((yyvsp[0].node));
		if (IsInteger((yyvsp[0].node))) {
			i = Trees_Integer((yyvsp[0].node));
			Range_CheckSetElement(i);
			(yyval.node) = Trees_NewSet((unsigned OBNC_INTEGER) 1 << i);
		} else if (Types_IsInteger(type)) {
			(yyval.node) = Trees_NewNode(TREES_SINGLE_ELEMENT_SET, (yyvsp[0].node), NULL);
			Trees_SetType(Trees_NewLeaf(TREES_SET_TYPE), (yyval.node));
		} else {
			Oberon_PrintError("error: element must have integer type");
			YYABORT;
		}
	}
#line 2997 "y.tab.c"
    break;

  case 89: /* element: expression DOTDOT expression  */
#line 1117 "Oberon.y"
        {
		CheckIsValueExpression((yyvsp[-2].node));
		CheckIsValueExpression((yyvsp[0].node));
		(yyval.node) = NULL;
		if (IsInteger((yyvsp[-2].node))) {
			Range_CheckSetElement(Trees_Integer((yyvsp[-2].node)));
		}
		if (IsInteger((yyvsp[0].node))) {
			Range_CheckSetElement(Trees_Integer((yyvsp[0].node)));
		}
		if (IsInteger((yyvsp[-2].node)) && IsInteger((yyvsp[0].node))) {
			(yyval.node) = Trees_NewSet(OBNC_RANGE(Trees_Integer((yyvsp[-2].node)), Trees_Integer((yyvsp[0].node))));
		} else if (Types_IsInteger(Trees_Type((yyvsp[-2].node))) && Types_IsInteger(Trees_Type((yyvsp[0].node)))) {
			(yyval.node) = Trees_NewNode(TREES_RANGE_SET, (yyvsp[-2].node), (yyvsp[0].node));
			Trees_SetType(Trees_NewLeaf(TREES_SET_TYPE), (yyval.node));
		} else {
			Oberon_PrintError("error: element must have integer type");
			YYABORT;
		}
	}
#line 3022 "y.tab.c"
    break;

  case 90: /* ExpList: expression  */
#line 1141 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_EXP_LIST, (yyvsp[0].node), NULL);
		Trees_SetType(Trees_Type((yyvsp[0].node)), (yyval.node));
	}
#line 3031 "y.tab.c"
    break;

  case 91: /* ExpList: ExpList ',' expression  */
#line 1146 "Oberon.y"
        {
		Trees_Node reversedList;

		reversedList = Trees_NewNode(TREES_EXP_LIST, (yyvsp[0].node), (yyvsp[-2].node));
		(yyval.node) = reversedList;
		Trees_SetType(Trees_Type((yyvsp[0].node)), (yyval.node));
	}
#line 3043 "y.tab.c"
    break;

  case 99: /* statement: %empty  */
#line 1167 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3051 "y.tab.c"
    break;

  case 100: /* assignment: designator BECOMES expression  */
#line 1174 "Oberon.y"
        {
		Trees_Node designator, ident, designatorType, exp;

		CheckIsValueExpression((yyvsp[0].node));
		switch (Trees_Symbol((yyvsp[-2].node))) {
			case TREES_DESIGNATOR:
				designator = (yyvsp[-2].node);
				exp = (yyvsp[0].node);
				ident = BaseIdent((yyvsp[-2].node));
				designatorType = Trees_Type((yyvsp[-2].node));
				switch (Trees_Kind(ident)) {
					case TREES_VARIABLE_KIND:
					case TREES_VALUE_PARAM_KIND:
					case TREES_VAR_PARAM_KIND:
						if (Writable((yyvsp[-2].node))) {
							ValidateAssignment(exp, designatorType, ASSIGNMENT_CONTEXT, 0);
							if (Types_IsChar(designatorType) && IsString(exp)) {
								exp = Trees_NewChar(Trees_String(exp)[0]);
							}
						} else {
							Oberon_PrintError("error: assignment to read-only variable");
							YYABORT;
						}
						break;
					default:
						Oberon_PrintError("error: assignment to non-variable");
						YYABORT;
				}
				(yyval.node) = Trees_NewNode(BECOMES, designator, exp);
				break;
			case TREES_PROCEDURE_CALL:
				Oberon_PrintError("error: unexpected procedure call in assignment target");
				YYABORT;
				break;
			default:
				assert(0);
		}
	}
#line 3094 "y.tab.c"
    break;

  case 101: /* ProcedureCall: designator  */
#line 1217 "Oberon.y"
        {
		const int isFunctionCall = 0;
		Trees_Node designator, actualParameters;

		if (Trees_Symbol((yyvsp[0].node)) == TREES_PROCEDURE_CALL) {
			designator = Trees_Left((yyvsp[0].node));
			actualParameters = Trees_Right((yyvsp[0].node));
		} else if (Types_IsProcedure(Trees_Type((yyvsp[0].node)))) { /*procedure variable without parameters*/
			designator = (yyvsp[0].node);
			actualParameters = NULL;
		} else {
			Oberon_PrintError("error: procedure expected");
			YYABORT;
		}
		HandleProcedureCall(designator, actualParameters, isFunctionCall, &(yyval.node));
		assert((yyval.node) != NULL);
	}
#line 3116 "y.tab.c"
    break;

  case 102: /* StatementSequence: StatementSequenceReversed  */
#line 1238 "Oberon.y"
        {
		Trees_ReverseList(&(yyvsp[0].node)); /*correct order*/
		(yyval.node) = (yyvsp[0].node);
	}
#line 3125 "y.tab.c"
    break;

  case 103: /* StatementSequenceReversed: statement  */
#line 1246 "Oberon.y"
        {
		if ((yyvsp[0].node) == NULL) {
			(yyval.node) = NULL;
		} else {
			(yyval.node) = Trees_NewNode(TREES_STATEMENT_SEQUENCE, (yyvsp[0].node), NULL);
		}
	}
#line 3137 "y.tab.c"
    break;

  case 104: /* StatementSequenceReversed: StatementSequenceReversed ';' statement  */
#line 1254 "Oberon.y"
        {
		if ((yyvsp[0].node) != NULL) {
			(yyval.node) = Trees_NewNode(TREES_STATEMENT_SEQUENCE, (yyvsp[0].node), (yyvsp[-2].node));
		} else {
			(yyval.node) = (yyvsp[-2].node);
		}
	}
#line 3149 "y.tab.c"
    break;

  case 105: /* IfStatement: IF guard THEN StatementSequence ElseIfThenOptRep ElseOpt END  */
#line 1265 "Oberon.y"
        {
		Trees_Node currElsif, currExp, currThen, currStmt;

		if ((yyvsp[-2].node) == NULL) {
			(yyval.node) = Trees_NewNode(IF, (yyvsp[-5].node), Trees_NewNode(THEN, (yyvsp[-3].node), (yyvsp[-1].node)));
		} else {
			/*correct order of elsif nodes*/
			(yyval.node) = (yyvsp[-1].node);
			currElsif = (yyvsp[-2].node);
			do {
				currExp = Trees_Left(currElsif);
				currThen = Trees_Right(currElsif);
				currStmt = Trees_Left(currThen);
				(yyval.node) = Trees_NewNode(ELSIF, currExp, Trees_NewNode(THEN, currStmt, (yyval.node)));
				currElsif = Trees_Right(currThen);
			} while (currElsif != NULL);
			(yyval.node) = Trees_NewNode(IF, (yyvsp[-5].node), Trees_NewNode(THEN, (yyvsp[-3].node), (yyval.node)));
		}
	}
#line 3173 "y.tab.c"
    break;

  case 106: /* guard: expression  */
#line 1288 "Oberon.y"
        {
		CheckIsValueExpression((yyvsp[0].node));
		if (Types_IsBoolean(Trees_Type((yyvsp[0].node)))) {
			(yyval.node) = (yyvsp[0].node);
		} else {
			Oberon_PrintError("error: boolean expression expected");
			YYABORT;
		}
	}
#line 3187 "y.tab.c"
    break;

  case 107: /* ElseIfThenOptRep: ElseIfThenOptRep ELSIF guard THEN StatementSequence  */
#line 1301 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(ELSIF, (yyvsp[-2].node), Trees_NewNode(THEN, (yyvsp[0].node), (yyvsp[-4].node)));
	}
#line 3195 "y.tab.c"
    break;

  case 108: /* ElseIfThenOptRep: %empty  */
#line 1305 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3203 "y.tab.c"
    break;

  case 109: /* ElseOpt: ELSE StatementSequence  */
#line 1312 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(ELSE, (yyvsp[0].node), NULL);
	}
#line 3211 "y.tab.c"
    break;

  case 110: /* ElseOpt: %empty  */
#line 1316 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3219 "y.tab.c"
    break;

  case 111: /* CaseStatement: CASE CaseExpression OF CaseRep END  */
#line 1323 "Oberon.y"
        {
		Trees_Node expType, caseVariable;

		if ((yyvsp[-1].node) != NULL) {
			Trees_ReverseList(&(yyvsp[-1].node)); /*correct order*/
		}
		assert(caseLabelsStack != NULL);
		caseLabelsStack = Trees_Right(caseLabelsStack);
		caseExpressionStack = Trees_Right(caseExpressionStack);
		expType = Trees_Type((yyvsp[-3].node));
		if (Types_IsRecord(expType) || Types_IsPointer(expType)) {
			/*reset original type*/
			caseVariable = Trees_Left((yyvsp[-3].node));
			Trees_SetType(Trees_Type((yyvsp[-3].node)), caseVariable);
		}
		(yyval.node) = Trees_NewNode(CASE, (yyvsp[-3].node), (yyvsp[-1].node));
	}
#line 3241 "y.tab.c"
    break;

  case 112: /* CaseExpression: expression  */
#line 1344 "Oberon.y"
        {
		Trees_Node typeStruct, caseVariable;

		CheckIsValueExpression((yyvsp[0].node));
		typeStruct = Types_Structure(Trees_Type((yyvsp[0].node)));
		switch (Trees_Symbol(typeStruct)) {
			case RECORD:
				/*fall through*/
			case POINTER:
				if (IsDesignator((yyvsp[0].node)) && (FirstSelector((yyvsp[0].node)) == NULL)) {
					caseVariable = BaseIdent((yyvsp[0].node));
					if (! Types_IsRecord(typeStruct) || (Trees_Kind(caseVariable) == TREES_VAR_PARAM_KIND)) {
						(yyval.node) = (yyvsp[0].node);
					} else {
						Oberon_PrintError("error: record CASE expression must be a variable parameter");
						YYABORT;
					}
				} else {
					Oberon_PrintError("error: non-integral CASE expression must be a variable");
					YYABORT;
				}
				/*fall through*/
			case TREES_INTEGER_TYPE:
				/*fall through*/
			case TREES_CHAR_TYPE:
				caseExpressionStack = Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), caseExpressionStack);
				caseLabelsStack = Trees_NewNode(TREES_NOSYM, NULL, caseLabelsStack);
				(yyval.node) = (yyvsp[0].node);
				break;
			default:
				Oberon_PrintError("error: invalid type of CASE expression");
				YYABORT;
		}
	}
#line 3280 "y.tab.c"
    break;

  case 113: /* CaseRep: case  */
#line 1382 "Oberon.y"
        {
		if ((yyvsp[0].node) != NULL) {
			(yyval.node) = Trees_NewNode(TREES_CASE_REP, (yyvsp[0].node), NULL);
		} else {
			(yyval.node) = NULL;
		}
	}
#line 3292 "y.tab.c"
    break;

  case 114: /* CaseRep: CaseRep '|' case  */
#line 1390 "Oberon.y"
        {
		if ((yyvsp[0].node) != NULL) {
			(yyval.node) = Trees_NewNode(TREES_CASE_REP, (yyvsp[0].node), (yyvsp[-2].node));
		} else {
			(yyval.node) = (yyvsp[-2].node);
		}
	}
#line 3304 "y.tab.c"
    break;

  case 115: /* case: CaseLabelList ':' StatementSequence  */
#line 1401 "Oberon.y"
        {
		Trees_ReverseList(&(yyvsp[-2].node)); /*correct order*/
		(yyval.node) = Trees_NewNode(TREES_CASE, (yyvsp[-2].node), (yyvsp[0].node));
	}
#line 3313 "y.tab.c"
    break;

  case 116: /* case: %empty  */
#line 1406 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3321 "y.tab.c"
    break;

  case 117: /* CaseLabelList: LabelRange  */
#line 1413 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_CASE_LABEL_LIST, (yyvsp[0].node), NULL);
	}
#line 3329 "y.tab.c"
    break;

  case 118: /* CaseLabelList: CaseLabelList ',' LabelRange  */
#line 1417 "Oberon.y"
        {
		switch (Trees_Symbol((yyvsp[0].node))) {
			case INTEGER:
			case TREES_CHAR_CONSTANT:
			case DOTDOT:
				(yyval.node) = Trees_NewNode(TREES_CASE_LABEL_LIST, (yyvsp[0].node), (yyvsp[-2].node));
				break;
			default:
				Oberon_PrintError("error: unexpected list of type name case labels");
				YYABORT;
		}
	}
#line 3346 "y.tab.c"
    break;

  case 119: /* LabelRange: label  */
#line 1433 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
		CheckCaseLabelUniqueness((yyvsp[0].node));
		assert(caseLabelsStack != NULL);
		caseLabelsStack = Trees_NewNode(TREES_NOSYM,
			Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), Trees_Left(caseLabelsStack)),
			Trees_Right(caseLabelsStack));
	}
#line 3359 "y.tab.c"
    break;

  case 120: /* LabelRange: label DOTDOT label  */
#line 1442 "Oberon.y"
        {
		const int rangeLenMax = 255;
		int leftSym, rightSym;
		OBNC_INTEGER rangeMin, rangeMax;

		leftSym = Trees_Symbol((yyvsp[-2].node));
		rightSym = Trees_Symbol((yyvsp[0].node));
		if (leftSym == rightSym) {
			switch (leftSym) {
				case INTEGER:
					rangeMin = Trees_Integer((yyvsp[-2].node));
					rangeMax = Trees_Integer((yyvsp[0].node));
					if (rangeMin <= rangeMax) {
						if (rangeMax - rangeMin > rangeLenMax) {
							Oberon_PrintError("warning: maximum range length of %d exceeded", rangeLenMax);
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: left integer must be less than right integer in case range");
						YYABORT;
					}
					break;
				case TREES_CHAR_CONSTANT:
					if (Trees_Char((yyvsp[-2].node)) >= Trees_Char((yyvsp[0].node))) {
						Oberon_PrintError("error: left string must be less than right string in case range");
						YYABORT;
					}
					break;
				default:
					Oberon_PrintError("error: case label ranges must contain integers or single-character strings");
					YYABORT;
			}
		} else {
			Oberon_PrintError("error: case labels in a range must have the same type");
			YYABORT;
		}
		(yyval.node) = Trees_NewNode(DOTDOT, (yyvsp[-2].node), (yyvsp[0].node));
		CheckCaseLabelUniqueness((yyval.node));
		assert(caseLabelsStack != NULL);
		caseLabelsStack = Trees_NewNode(TREES_NOSYM,
			Trees_NewNode(TREES_NOSYM, (yyval.node), Trees_Left(caseLabelsStack)),
			Trees_Right(caseLabelsStack));
	}
#line 3407 "y.tab.c"
    break;

  case 121: /* label: INTEGER  */
#line 1489 "Oberon.y"
        {
		if (Types_IsInteger(Trees_Type(Trees_Left(caseExpressionStack)))) {
			(yyval.node) = Trees_NewInteger((yyvsp[0].integer));
		} else {
			Oberon_PrintError("error: unexpected integer case label");
			YYABORT;
		}
	}
#line 3420 "y.tab.c"
    break;

  case 122: /* label: STRING  */
#line 1498 "Oberon.y"
        {
		if (Types_IsChar(Trees_Type(Trees_Left(caseExpressionStack)))) {
			if (strlen((yyvsp[0].string)) <= 1) {
				(yyval.node) = Trees_NewChar((yyvsp[0].string)[0]);
			} else {
				Oberon_PrintError("error: single-character string expected: \"%s\"", (yyvsp[0].string));
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: unexpected string case label: \"%s\"", (yyvsp[0].string));
			YYABORT;
		}
	}
#line 3438 "y.tab.c"
    break;

  case 123: /* label: qualident  */
#line 1512 "Oberon.y"
        {
		Trees_Node caseExp, constValue, caseVariable;

		(yyval.node) = Table_At(Trees_Name((yyvsp[0].node)));
		if ((yyval.node) != NULL) {
			caseExp = Trees_Left(caseExpressionStack);
			switch (Trees_Symbol(Types_Structure(Trees_Type(caseExp)))) {
				case TREES_INTEGER_TYPE:
					if (Trees_Kind((yyval.node)) == TREES_CONSTANT_KIND) {
						constValue = Trees_Value((yyval.node));
						if (Trees_Symbol(constValue) == INTEGER) {
							if (Trees_Integer(constValue) >= 0) {
								(yyval.node) = constValue;
							} else {
								Oberon_PrintError("error: non-negative case label expected: %" OBNC_INT_MOD "d", Trees_Integer(constValue));
								YYABORT;
							}
						} else {
							Oberon_PrintError("error: integer case label expected");
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: constant identifier expected: %s", Trees_Name((yyval.node)));
						YYABORT;
					}
					break;
				case TREES_CHAR_TYPE:
					if (Trees_Kind((yyval.node)) == TREES_CONSTANT_KIND) {
						constValue = Trees_Value((yyval.node));
						if (Trees_Symbol(constValue) == STRING) {
							if (Types_StringLength(Trees_Type(constValue)) <= 1) {
								(yyval.node) = Trees_NewChar(Trees_String(constValue)[0]);
							} else {
								Oberon_PrintError("error: single-character string expected: %s", Trees_String(constValue));
								YYABORT;
							}
						} else {
							Oberon_PrintError("error: character case label expected");
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: constant identifier expected: %s", Trees_Name((yyval.node)));
						YYABORT;
					}
					break;
				case RECORD:
					if (Types_IsType((yyval.node)) && Types_IsRecord((yyval.node))) {
						if (Types_Extends(Trees_Type(caseExp), (yyval.node))) {
							caseVariable = Trees_Left(caseExp);
							Trees_SetType((yyval.node), caseVariable);
						} else {
							Oberon_PrintError("error: case label is not an extension of %s: %s", Trees_Name(Trees_Type(caseExp)), Trees_Name((yyval.node)));
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: record type case label expected");
						YYABORT;
					}
					break;
				case POINTER:
					if (Types_IsType((yyval.node)) && Types_IsPointer((yyval.node))) {
						if (Types_Extends(Trees_Type(caseExp), (yyval.node))) {
							caseVariable = Trees_Left(caseExp);
							Trees_SetType((yyval.node), caseVariable);
						} else {
							Oberon_PrintError("error: case label is not an extension of %s: %s", Trees_Name(Trees_Type(caseExp)), Trees_Name((yyval.node)));
							YYABORT;
						}
					} else {
						Oberon_PrintError("error: pointer type case label expected");
						YYABORT;
					}
					break;
				default:
					assert(0);
			}
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			YYABORT;
		}
	}
#line 3524 "y.tab.c"
    break;

  case 124: /* WhileStatement: WHILE guard DO StatementSequence ElseIfDoOptRep END  */
#line 1597 "Oberon.y"
        {
		Trees_Node currElsif, currExp, currThen, currStmt;

		if ((yyvsp[-1].node) == NULL) {
			(yyval.node) = Trees_NewNode(WHILE, (yyvsp[-4].node), Trees_NewNode(DO, (yyvsp[-2].node), NULL));
		} else {
			/*correct order of elsif nodes*/
			(yyval.node) = NULL;
			currElsif = (yyvsp[-1].node);
			do {
				currExp = Trees_Left(currElsif);
				currThen = Trees_Right(currElsif);
				currStmt = Trees_Left(currThen);
				(yyval.node) = Trees_NewNode(ELSIF, currExp, Trees_NewNode(THEN, currStmt, (yyval.node)));
				currElsif = Trees_Right(currThen);
			} while (currElsif != NULL);
			(yyval.node) = Trees_NewNode(WHILE, (yyvsp[-4].node), Trees_NewNode(THEN, (yyvsp[-2].node), (yyval.node)));
		}
	}
#line 3548 "y.tab.c"
    break;

  case 125: /* ElseIfDoOptRep: ElseIfDoOptRep ELSIF guard DO StatementSequence  */
#line 1620 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(ELSIF, (yyvsp[-2].node), Trees_NewNode(THEN, (yyvsp[0].node), (yyvsp[-4].node)));
	}
#line 3556 "y.tab.c"
    break;

  case 126: /* ElseIfDoOptRep: %empty  */
#line 1624 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3564 "y.tab.c"
    break;

  case 127: /* RepeatStatement: REPEAT StatementSequence UNTIL expression  */
#line 1631 "Oberon.y"
        {
		CheckIsValueExpression((yyvsp[0].node));
		(yyval.node) = NULL;
		if (Types_IsBoolean(Trees_Type((yyvsp[0].node)))) {
			(yyval.node) = Trees_NewNode(REPEAT, (yyvsp[-2].node), (yyvsp[0].node));
		} else {
			Oberon_PrintError("error: boolean expression expected");
			YYABORT;
		}
	}
#line 3579 "y.tab.c"
    break;

  case 128: /* ForStatement: FOR ForInit TO ForLimit ByOpt DO StatementSequence END  */
#line 1646 "Oberon.y"
        {
		Trees_Node byExp;

		if ((yyvsp[-3].node) != NULL) {
			byExp = (yyvsp[-3].node);
		} else {
			byExp = Trees_NewInteger(1);
		}
		(yyval.node) = Trees_NewNode(FOR,
			(yyvsp[-6].node),
			Trees_NewNode(TO,
				(yyvsp[-4].node),
				Trees_NewNode(BY, byExp, (yyvsp[-1].node))));
	}
#line 3598 "y.tab.c"
    break;

  case 129: /* ForInit: IDENT BECOMES expression  */
#line 1664 "Oberon.y"
        {
		Trees_Node ctrlVar, ctrlVarType;

		CheckIsValueExpression((yyvsp[0].node));
		ctrlVar = Table_At((yyvsp[-2].ident));
		if (ctrlVar != NULL) {
			ctrlVarType = Trees_Type(ctrlVar);
			if (Types_IsInteger(ctrlVarType)) {
				if (Types_IsInteger(Trees_Type((yyvsp[0].node)))) {
					(yyval.node) = Trees_NewNode(BECOMES, ctrlVar, (yyvsp[0].node));
				} else {
					Oberon_PrintError("error: integer expression expected as initial value");
					YYABORT;
				}
			} else {
				Oberon_PrintError("error: integer control variable expected: %s", (yyvsp[-2].ident));
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: undeclared control variable: %s", (yyvsp[-2].ident));
			YYABORT;
		}
	}
#line 3626 "y.tab.c"
    break;

  case 130: /* ForLimit: expression  */
#line 1690 "Oberon.y"
        {
		CheckIsValueExpression((yyvsp[0].node));
		if (! Types_IsInteger(Trees_Type((yyvsp[0].node)))) {
			Oberon_PrintError("error: integer expression expected as upper limit");
			YYABORT;
		}
	}
#line 3638 "y.tab.c"
    break;

  case 131: /* ByOpt: BY ConstExpression  */
#line 1701 "Oberon.y"
        {
		if (Types_IsInteger(Trees_Type((yyvsp[0].node)))) {
			if (IsInteger((yyvsp[0].node))) {
				if (Trees_Integer((yyvsp[0].node)) == 0) {
					Oberon_PrintError("warning: steps by zero leads to infinite loop");
				}
				(yyval.node) = (yyvsp[0].node);
			} else {
				Oberon_PrintError("error: fully evaluated constant expression expected as increment");
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: integer increment expected");
			YYABORT;
		}
	}
#line 3659 "y.tab.c"
    break;

  case 132: /* ByOpt: %empty  */
#line 1718 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3667 "y.tab.c"
    break;

  case 133: /* ProcedureDeclaration: ProcedureHeading ';' DeclarationSequence StatementSequenceOpt ReturnClauseOpt END IDENT  */
#line 1728 "Oberon.y"
        {
		Trees_Node procIdent, procType, resultType, procStatements, returnExp;
		const char *procName;

		procIdent = (yyvsp[-6].node);
		procName = Trees_Name(procIdent);
		procType = Trees_Type((yyvsp[-6].node));
		resultType = Types_ResultType(procType);
		procStatements = (yyvsp[-3].node);
		returnExp = (yyvsp[-2].node);

		if (strcmp(procName, (yyvsp[0].ident)) == 0) {
			if (resultType == NULL) {
				if (returnExp != NULL) {
					Oberon_PrintError("error: unexpected return clause in proper procedure");
					YYABORT;
				}
			} else {
				if (returnExp != NULL) {
					CheckIsValueExpression(returnExp);
					ValidateAssignment(returnExp, resultType, PROC_RESULT_CONTEXT, 0);
					if ((Trees_Symbol(returnExp) == STRING) && Types_IsChar(resultType)) {
						returnExp = Trees_NewChar(Trees_String(returnExp)[0]);
					}
				} else {
					Oberon_PrintError("error: return clause expected in function procedure");
					YYABORT;
				}
			}
			if (procStatements != NULL) {
				Generate_ProcedureStatements(procStatements);
			}
			if (returnExp != NULL) {
				Generate_ReturnClause(returnExp);
			}
			if (procedureDeclarationStack != NULL) {
				procedureDeclarationStack = Trees_Right(procedureDeclarationStack);
			}
			Generate_ProcedureEnd(procIdent);
			CheckUnusedIdentifiers();
			Table_CloseScope();
		} else {
			Oberon_PrintError("error: expected procedure name: %s", procName);
			YYABORT;
		}
	}
#line 3718 "y.tab.c"
    break;

  case 134: /* ProcedureHeading: ProcedureHeadingSansParam FormalParametersOpt  */
#line 1778 "Oberon.y"
        {
		Trees_Node paramList, param;

		(yyval.node) = NULL;
		Trees_SetType((yyvsp[0].node), (yyvsp[-1].node));

		paramList = Types_Parameters((yyvsp[0].node));
		while (paramList != NULL) {
			param = Trees_Left(paramList);
			Table_Put(param);
			paramList = Trees_Right(paramList);
		}

		procedureDeclarationStack = Trees_NewNode(TREES_NOSYM, (yyvsp[-1].node), procedureDeclarationStack);
		Generate_ProcedureHeading((yyvsp[-1].node));
		(yyval.node) = (yyvsp[-1].node);
	}
#line 3740 "y.tab.c"
    break;

  case 135: /* ProcedureHeadingSansParam: PROCEDURE identdef  */
#line 1799 "Oberon.y"
        {
		if (! (Trees_Exported((yyvsp[0].node)) && Trees_Local((yyvsp[0].node)))) {
			Trees_SetKind(TREES_PROCEDURE_KIND, (yyvsp[0].node));
			Table_Put((yyvsp[0].node));
			Table_OpenScope();
		} else {
			Oberon_PrintError("error: cannot export local procedure: %s", Trees_Name((yyvsp[0].node)));
			YYABORT;
		}
		(yyval.node) = (yyvsp[0].node);
	}
#line 3756 "y.tab.c"
    break;

  case 136: /* StatementSequenceOpt: BEGIN_ StatementSequence  */
#line 1814 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
	}
#line 3764 "y.tab.c"
    break;

  case 137: /* StatementSequenceOpt: %empty  */
#line 1818 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3772 "y.tab.c"
    break;

  case 138: /* ReturnClauseOpt: RETURN expression  */
#line 1825 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
	}
#line 3780 "y.tab.c"
    break;

  case 139: /* ReturnClauseOpt: %empty  */
#line 1829 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3788 "y.tab.c"
    break;

  case 145: /* TypeSectionOpt: TypeKeyword TypeDeclarationOptRep  */
#line 1850 "Oberon.y"
        {
		Trees_Node unresolvedPointerType, undeclaredBaseType;

		if (unresolvedPointerTypes != NULL) {
			unresolvedPointerType = Trees_Left(unresolvedPointerTypes);
			undeclaredBaseType = Types_PointerBaseType(unresolvedPointerType);
			Oberon_PrintError("error: undeclared pointer base type: %s", Trees_Name(undeclaredBaseType));
			YYABORT;
		}
	}
#line 3803 "y.tab.c"
    break;

  case 146: /* TypeSectionOpt: %empty  */
#line 1861 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3811 "y.tab.c"
    break;

  case 147: /* TypeKeyword: TYPE  */
#line 1868 "Oberon.y"
        {
		unresolvedPointerTypes = NULL;
	}
#line 3819 "y.tab.c"
    break;

  case 156: /* FormalParameters: '(' FPSectionsOpt ')' ResultTypeOpt  */
#line 1895 "Oberon.y"
        {
		(yyval.node) = Types_NewProcedure((yyvsp[-2].node), (yyvsp[0].node));
	}
#line 3827 "y.tab.c"
    break;

  case 157: /* FPSectionsOpt: FPSectionRep  */
#line 1902 "Oberon.y"
        {
		Trees_ReverseList(&(yyvsp[0].node)); /*correct order*/
		(yyval.node) = (yyvsp[0].node);
	}
#line 3836 "y.tab.c"
    break;

  case 158: /* FPSectionsOpt: %empty  */
#line 1907 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3844 "y.tab.c"
    break;

  case 159: /* FPSectionRep: FPSection  */
#line 1914 "Oberon.y"
        {
		(yyval.node) = (yyvsp[0].node);
		Trees_ReverseList(&(yyval.node));
	}
#line 3853 "y.tab.c"
    break;

  case 160: /* FPSectionRep: FPSectionRep ';' FPSection  */
#line 1919 "Oberon.y"
        {
		Trees_Node p, p1;
		const char *paramName, *paramName1;

		/*make sure no parameter is repeated*/
		p = (yyvsp[0].node);
		while (p != NULL) {
			paramName = Trees_Name(Trees_Left(p));
			p1 = (yyvsp[-2].node);
			while (p1 != NULL) {
				paramName1 = Trees_Name(Trees_Left(p1));
				if (strcmp(paramName1, paramName) == 0) {
					Oberon_PrintError("error: repeated parameter: %s", paramName);
					YYABORT;
				}
				p1 = Trees_Right(p1);
			}
			p = Trees_Right(p);
		}

		/*make one list of the two lists*/
		(yyval.node) = (yyvsp[-2].node);
		p = (yyvsp[0].node);
		do {
			(yyval.node) = Trees_NewNode(TREES_IDENT_LIST, Trees_Left(p), (yyval.node));
			p = Trees_Right(p);
		} while (p != NULL);
		/*$$ in reversed order*/
	}
#line 3887 "y.tab.c"
    break;

  case 161: /* ResultTypeOpt: ':' qualident  */
#line 1952 "Oberon.y"
        {
		(yyval.node) = ResolvedType((yyvsp[0].node), 0);
		if ((yyval.node) != NULL) {
			if (Trees_Symbol((yyval.node)) == IDENT) {
				if (Trees_Kind((yyval.node)) != TREES_TYPE_KIND) {
					Oberon_PrintError("error: type name expected as result type: %s", Trees_Name((yyvsp[0].node)));
					YYABORT;
				}
				if (! Types_Scalar((yyval.node))) {
					Oberon_PrintError("error: scalar result type expected: %s", Trees_Name((yyvsp[0].node)));
					YYABORT;
				}
			}
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			YYABORT;
		}
	}
#line 3910 "y.tab.c"
    break;

  case 162: /* ResultTypeOpt: %empty  */
#line 1971 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 3918 "y.tab.c"
    break;

  case 163: /* FPSection: ParameterKindOpt IdentRep ':' FormalType  */
#line 1978 "Oberon.y"
        {
		Trees_Node curr, ident;

		Trees_ReverseList(&(yyvsp[-2].node)); /*correct order*/
		curr = (yyvsp[-2].node);
		do {
			ident = Trees_Left(curr);
			Trees_SetKind((int) (yyvsp[-3].integer), ident);
			Trees_SetType((yyvsp[0].node), ident);
			Trees_SetLocal(ident);
			curr = Trees_Right(curr);
		} while (curr != NULL);

		(yyval.node) = (yyvsp[-2].node);
	}
#line 3938 "y.tab.c"
    break;

  case 164: /* ParameterKindOpt: VAR  */
#line 1997 "Oberon.y"
        {
		(yyval.integer) = TREES_VAR_PARAM_KIND;
	}
#line 3946 "y.tab.c"
    break;

  case 165: /* ParameterKindOpt: %empty  */
#line 2001 "Oberon.y"
        {
		(yyval.integer) = TREES_VALUE_PARAM_KIND;
	}
#line 3954 "y.tab.c"
    break;

  case 166: /* IdentRep: IDENT  */
#line 2008 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(TREES_IDENT_LIST, Trees_NewIdent((yyvsp[0].ident)), NULL);
	}
#line 3962 "y.tab.c"
    break;

  case 167: /* IdentRep: IdentRep ',' IDENT  */
#line 2012 "Oberon.y"
        {
		Trees_Node curr;
		const char *identName;

		/*make sure no name is repeated*/
		curr = (yyvsp[-2].node);
		while (curr != NULL) {
			identName = Trees_Name(Trees_Left(curr));
			if (strcmp(identName, (yyvsp[0].ident)) == 0) {
				Oberon_PrintError("error: repeated identifier: %s", identName);
				YYABORT;
			}
			curr = Trees_Right(curr);
		}

		(yyval.node) = Trees_NewNode(TREES_IDENT_LIST, Trees_NewIdent((yyvsp[0].ident)), (yyvsp[-2].node));
	}
#line 3984 "y.tab.c"
    break;

  case 168: /* FormalType: OpenArrayOptRep qualident  */
#line 2033 "Oberon.y"
        {
		(yyval.node) = ResolvedType((yyvsp[0].node), 0);
		if ((yyval.node) != NULL) {
			while ((yyvsp[-1].node) != NULL) {
				(yyval.node) = Types_NewArray(NULL, (yyval.node));
				(yyvsp[-1].node) = Trees_Right((yyvsp[-1].node));
			}
		} else {
			Oberon_PrintError("error: undeclared identifier: %s", Trees_Name((yyvsp[0].node)));
			exit(EXIT_FAILURE);
		}
	}
#line 4001 "y.tab.c"
    break;

  case 169: /* OpenArrayOptRep: OpenArrayOptRep ARRAY OF  */
#line 2049 "Oberon.y"
        {
		(yyval.node) = Trees_NewNode(ARRAY, NULL, (yyvsp[-2].node));
	}
#line 4009 "y.tab.c"
    break;

  case 170: /* OpenArrayOptRep: %empty  */
#line 2053 "Oberon.y"
        {
		(yyval.node) = NULL;
	}
#line 4017 "y.tab.c"
    break;

  case 171: /* module: ModuleHeading ';' ImportListOpt DeclarationSequence ModuleStatements END IDENT '.'  */
#line 2063 "Oberon.y"
        {
		const char *symfilePath;

		if (strcmp((yyvsp[-1].ident), inputModuleName) == 0) {
			CheckUnusedIdentifiers();
			Generate_ModuleEnd();
			Generate_Close();

			symfilePath = Util_String(".obnc/%s.sym", inputModuleName);
			if (parseMode == OBERON_ENTRY_POINT_MODE) {
				if (Files_Exists(symfilePath)) {
					Files_Remove(symfilePath);
				}
			} else {
				ExportSymbolTable(symfilePath);
			}
			YYACCEPT;
		} else {
			Oberon_PrintError("error: expected identifier %s", inputModuleName);
			YYABORT;
		}
	}
#line 4044 "y.tab.c"
    break;

  case 172: /* ModuleHeading: MODULE IDENT  */
#line 2090 "Oberon.y"
        {
		if (strcmp((yyvsp[0].ident), inputModuleName) == 0) {
			if (parseMode != OBERON_IMPORT_LIST_MODE) {
				Generate_ModuleHeading();
			}
		} else {
			Oberon_PrintError("error: module name does not match filename: %s", (yyvsp[0].ident));
			YYABORT;
		}
	}
#line 4059 "y.tab.c"
    break;

  case 173: /* ImportListOpt: ImportList  */
#line 2104 "Oberon.y"
        {
		if (parseMode == OBERON_IMPORT_LIST_MODE) {
			YYACCEPT;
		}
	}
#line 4069 "y.tab.c"
    break;

  case 174: /* ImportListOpt: %empty  */
#line 2110 "Oberon.y"
        {
		if (parseMode == OBERON_IMPORT_LIST_MODE) {
			YYACCEPT;
		}
	}
#line 4079 "y.tab.c"
    break;

  case 175: /* ImportList: IMPORT ImportRep ';'  */
#line 2119 "Oberon.y"
        {
		const char *impfilePath;
		Trees_Node moduleAndDirPath, module, p;
		FILE *impFile;
		const char *name;

		if ((yyvsp[-1].node) != NULL) {
			Trees_ReverseList(&(yyvsp[-1].node)); /*correct order*/
			if (parseMode == OBERON_IMPORT_LIST_MODE) {
				while ((yyvsp[-1].node) != NULL) {
					name = Trees_Name(Trees_Left((yyvsp[-1].node)));
					puts(name);
					(yyvsp[-1].node) = Trees_Right((yyvsp[-1].node));
				}
			} else {
				if (parseMode == OBERON_NORMAL_MODE) {
					impfilePath = Util_String(".obnc/%s.imp", inputModuleName);
					impFile = Files_Exists(impfilePath)? Files_Old(impfilePath, FILES_WRITE): Files_New(impfilePath);
					p = (yyvsp[-1].node);
					do {
						moduleAndDirPath = Trees_Left(p);
						module = Trees_Left(moduleAndDirPath);
						name = Trees_UnaliasedName(module);
						fprintf(impFile, "%s\n", name);
						p = Trees_Right(p);
					} while (p != NULL);
					Files_Close(&impFile);
				}
				Generate_ImportList((yyvsp[-1].node));
			}
		}
	}
#line 4116 "y.tab.c"
    break;

  case 176: /* ImportRep: import  */
#line 2155 "Oberon.y"
        {
		if ((yyvsp[0].node) != NULL) {
			(yyval.node) = Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), NULL);
		} else {
			(yyval.node) = NULL;
		}
	}
#line 4128 "y.tab.c"
    break;

  case 177: /* ImportRep: ImportRep ',' import  */
#line 2163 "Oberon.y"
        {
		const char *importName;
		Trees_Node p;
		int moduleImported;

		if ((yyvsp[0].node) != NULL) {
			/*check if the module has already been imported (with a different qualifier)*/
			p = (yyvsp[-2].node);
			if (parseMode == OBERON_IMPORT_LIST_MODE) {
				importName = Trees_Name((yyvsp[0].node));
				while ((p != NULL) && (strcmp(Trees_Name(Trees_Left(p)), importName) != 0)) {
					p = Trees_Right(p);
				}
			} else {
				importName = Trees_Name(Trees_Left((yyvsp[0].node)));
				while ((p != NULL) && (strcmp(Trees_Name(Trees_Left(Trees_Left(p))), importName) != 0)) {
					p = Trees_Right(p);
				}
			}
			moduleImported = p != NULL;

			if (! moduleImported) {
				(yyval.node) = Trees_NewNode(TREES_NOSYM, (yyvsp[0].node), (yyvsp[-2].node));
			} else {
				(yyval.node) = (yyvsp[-2].node);
			}
		} else {
			(yyval.node) = (yyvsp[-2].node);
		}
	}
#line 4163 "y.tab.c"
    break;

  case 178: /* import: IDENT BecomesIdentOpt  */
#line 2197 "Oberon.y"
        {
		static Maps_Map importedModules = NULL;
		const char *module, *qualifier, *symbolFileDir, *symbolFileName, *moduleDirPath;
		Trees_Node qualifierSym, moduleIdent;

		if (importedModules == NULL) {
			importedModules = Maps_New();
		}
		if ((yyvsp[0].ident) != NULL) {
			module = (yyvsp[0].ident);
			qualifier = (yyvsp[-1].ident);
		} else {
			module = (yyvsp[-1].ident);
			qualifier = (yyvsp[-1].ident);
		}
		(yyval.node) = NULL;
		if (strcmp(module, inputModuleName) != 0) {
			if (! Maps_HasKey(module, importedModules)) {
				Maps_Put(module, NULL, &importedModules);
			}
			qualifierSym = Table_At(qualifier);
			if (qualifierSym == NULL) {
				qualifierSym = Trees_NewIdent(qualifier);
				if ((yyvsp[0].ident) != NULL) {
					Trees_SetUnaliasedName(module, qualifierSym);
				}
				Trees_SetKind(TREES_QUALIFIER_KIND, qualifierSym);
				Table_Put(qualifierSym);

				if (strcmp(module, "SYSTEM") == 0) {
					if (parseMode != OBERON_IMPORT_LIST_MODE) {
						Table_ImportSystem(qualifier);
					}
				} else if (parseMode == OBERON_IMPORT_LIST_MODE) {
					(yyval.node) = Trees_NewIdent(module);
				} else {
					moduleDirPath = ModulePaths_Directory(module, ".", 0);
					if (moduleDirPath != NULL) {
						/*import identifiers into the symbol table*/
						symbolFileDir = Util_String("%s/.obnc", moduleDirPath);
						if (! Files_Exists(symbolFileDir)) {
							symbolFileDir = Util_String("%s", moduleDirPath);
						}
						symbolFileName = Util_String("%s/%s.sym", symbolFileDir, module);
						if (Files_Exists(symbolFileName)) {
							Table_Import(symbolFileName, module, qualifier);
						} else {
							Oberon_PrintError("error: symbol file not found for module %s: %s", module, symbolFileName);
							YYABORT;
						}

						moduleIdent = Trees_NewIdent(module);
						Trees_SetKind(TREES_QUALIFIER_KIND, moduleIdent);
						(yyval.node) = Trees_NewNode(TREES_NOSYM, moduleIdent, Trees_NewString(moduleDirPath));
					} else {
						Oberon_PrintError("error: imported module not found: %s", module);
						YYABORT;
					}
				}
			} else {
				Oberon_PrintError("error: qualifier already used: %s", qualifier);
				YYABORT;
			}
		} else {
			Oberon_PrintError("error: a module cannot import itself");
			YYABORT;
		}
	}
#line 4236 "y.tab.c"
    break;

  case 179: /* BecomesIdentOpt: BECOMES IDENT  */
#line 2269 "Oberon.y"
        {
		(yyval.ident) = (yyvsp[0].ident);
	}
#line 4244 "y.tab.c"
    break;

  case 180: /* BecomesIdentOpt: %empty  */
#line 2273 "Oberon.y"
        {
		(yyval.ident) = NULL;
	}
#line 4252 "y.tab.c"
    break;

  case 181: /* ModuleStatements: StatementSequenceOpt  */
#line 2280 "Oberon.y"
        {
		Generate_ModuleStatements((yyvsp[0].node));
	}
#line 4260 "y.tab.c"
    break;


#line 4264 "y.tab.c"

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
          = {yyssp, yytoken};
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
        yyerror (yymsgp);
        if (yysyntax_error_status == YYENOMEM)
          YYNOMEM;
      }
    }

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
                      yytoken, &yylval);
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


      yydestruct ("Error: popping",
                  YY_ACCESSING_SYMBOL (yystate), yyvsp);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END


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
  yyerror (YY_("memory exhausted"));
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
                  yytoken, &yylval);
    }
  /* Do not reclaim the symbols of the rule whose action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
                  YY_ACCESSING_SYMBOL (+*yyssp), yyvsp);
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

#line 2285 "Oberon.y"


void Oberon_Init(void)
{
	if (! initialized) {
		initialized = 1;
		Error_Init();
		Files_Init();
		Generate_Init();
		ModulePaths_Init();
		Table_Init();
	}
}


void Oberon_Parse(const char inputFile[], int mode)
{
	const char *impFile;
	FILE *fp;
	int error;

	assert(initialized);
	inputFilename = inputFile;
	parseMode = mode;
	inputModuleName = Paths_SansSuffix(Paths_Basename(inputFile));

	yyin = fopen(inputFile, "r");
	if (yyin != NULL) {
		if (mode != OBERON_IMPORT_LIST_MODE) {
			Generate_Open(inputFile, mode == OBERON_ENTRY_POINT_MODE);

			impFile = Util_String(".obnc/%s.imp", inputModuleName);
			if (parseMode == OBERON_NORMAL_MODE) {
				if (! Files_Exists(impFile)) {
					fp = Files_New(impFile);
					Files_Close(&fp);
				}
			} else {
				assert(parseMode == OBERON_ENTRY_POINT_MODE);
				if (Files_Exists(impFile)) {
					Files_Remove(impFile);
				}
			}
		}
		error = yyparse();
		if (error) {
			exit(EXIT_FAILURE);
		}
	} else {
		Error_Handle(Util_String("error: cannot open file: %s: %s", inputFile, strerror(errno)));
	}
}


void Oberon_PrintError(const char format[], ...)
{
	va_list ap;

	assert(initialized);
	fprintf(stderr, "obnc-compile: %s:%d: ", inputFilename, yylineno);
	va_start(ap, format);
	vfprintf(stderr, format, ap);
	va_end(ap);
	fputc('\n', stderr);
}


void yyerror(const char msg[])
{
	Oberon_PrintError("%s", msg);
}


static void PrintError(int line, const char format[], ...)
	__attribute__ ((format (printf, 2, 3)));

static void PrintError(int line, const char format[], ...)
{
	va_list ap;

	fprintf(stderr, "obnc-compile: %s:%d: ", inputFilename, line);
	va_start(ap, format);
	vfprintf(stderr, format, ap);
	va_end(ap);
	fputc('\n', stderr);
}


static char *IdentKindString(int kind)
{
	char *result;

	switch (kind) {
		case TREES_CONSTANT_KIND:
			result = Util_String("constant");
			break;
		case TREES_TYPE_KIND:
			result = Util_String("type");
			break;
		case TREES_VARIABLE_KIND:
			result = Util_String("variable");
			break;
		case TREES_PROCEDURE_KIND:
			result = Util_String("procedure");
			break;
		case TREES_QUALIFIER_KIND:
			result = Util_String("module");
			break;
		default:
			result = Util_String("identifier");
	}
	return result;
}


static void CheckUnusedIdentifiers(void)
{
	Trees_Node unusedIdents, ident;
	int kind;

	unusedIdents = Table_UnusedIdentifiers();
	while (unusedIdents != NULL) {
		ident = Trees_Left(unusedIdents);
		kind = Trees_Kind(ident);
		if (! Trees_Exported(ident)
				&& (kind != TREES_VALUE_PARAM_KIND)
				&& (kind != TREES_VAR_PARAM_KIND)) {
			PrintError(Trees_LineNumber(ident), "note: unused %s: %s", IdentKindString(Trees_Kind(ident)), Trees_UnaliasedName(ident));
		}
		unusedIdents = Trees_Right(unusedIdents);
	}
}


/*constant predicate functions*/

static int IsBoolean(Trees_Node node)
{
	return (Trees_Symbol(node) == TRUE) || (Trees_Symbol(node) == FALSE);
}


static int IsChar(Trees_Node node)
{
	return Trees_Symbol(node) == TREES_CHAR_CONSTANT;
}


static int IsInteger(Trees_Node node)
{
	return Trees_Symbol(node) == INTEGER;
}


static int IsReal(Trees_Node node)
{
	return Trees_Symbol(node) == REAL;
}


static int IsString(Trees_Node node)
{
	return Trees_Symbol(node) == STRING;
}


static int IsSet(Trees_Node node)
{
	return Trees_Symbol(node) == TREES_SET_CONSTANT;
}


/*functions for type declaration productions*/

static Trees_Node ResolvedType(Trees_Node type, int isTypeDecl)
{
	Trees_Node result, identDef, typeStruct;
	const char *name;

	result = NULL;
	if (Trees_Symbol(type) == IDENT) {
		name = Trees_Name(type);
		identDef = Table_At(name);
		if (identDef != NULL) {
			if (Trees_Kind(identDef) == TREES_TYPE_KIND) {
				typeStruct = Types_Structure(identDef);
				if (typeStruct != NULL) {
					if (Types_Basic(Trees_Type(identDef)) && ! isTypeDecl) {
						result = Trees_Type(identDef);
					} else {
						result = identDef;
					}
				} else {
					Oberon_PrintError("error: unresolved type: %s", name);
					exit(EXIT_FAILURE);
				}
			} else {
				Oberon_PrintError("error: type expected: %s", name);
				exit(EXIT_FAILURE);
			}
		}
	} else {
		result = type;
	}
	return result;
}


static void ResolvePointerTypes(Trees_Node baseType)
{
	const char *baseTypeName;
	Trees_Node prev, curr, currPointerType, currBaseType;

	assert(Trees_Symbol(baseType) == IDENT);
	baseTypeName = Trees_Name(baseType);

	prev = NULL;
	curr = unresolvedPointerTypes;
	while (curr != NULL) {
		currPointerType = Trees_Left(curr);
		currBaseType = Types_PointerBaseType(currPointerType);
		if (strcmp(Trees_Name(currBaseType), baseTypeName) == 0) {
			if (Types_IsRecord(baseType)) {
				Trees_SetUsed(baseType);
				/*update pointer base type*/
				Types_SetPointerBaseType(baseType, currPointerType);
				/*delete current node*/
				if (curr == unresolvedPointerTypes) {
					unresolvedPointerTypes = Trees_Right(curr);
				} else {
					Trees_SetRight(Trees_Right(curr), prev);
				}
			} else {
				Oberon_PrintError("error: record type expected in declaration of pointer base type: %s", baseTypeName);
				exit(EXIT_FAILURE);
			}
		}
		prev = curr;
		curr = Trees_Right(curr);
	}
}


static const char *TypeString(Trees_Node type)
{
	const char *result = Util_String("%s", "");

	assert(Types_IsType(type));

	switch (Trees_Symbol(Types_Structure(type))) {
		case TREES_STRING_TYPE:
			switch (Types_StringLength(type)) {
				case 0:
					result = Util_String("empty string");
					break;
				case 1:
					result = Util_String("single-char string");
					break;
				default:
					result = Util_String("multi-char string");
			}
			break;
		case TREES_BOOLEAN_TYPE:
			result = Util_String("BOOLEAN");
			break;
		case TREES_CHAR_TYPE:
			result = Util_String("CHAR");
			break;
		case TREES_INTEGER_TYPE:
			result = Util_String("INTEGER");
			break;
		case TREES_REAL_TYPE:
			result = Util_String("REAL");
			break;
		case TREES_BYTE_TYPE:
			result = Util_String("BYTE");
			break;
		case TREES_SET_TYPE:
			result = Util_String("SET");
			break;
		case ARRAY:
			if (Types_IsOpenArray(type)) {
				result = Util_String("open array type");
			} else {
				if (Trees_Symbol(type) != IDENT) {
					result = Util_String("anonymous ");
				}
				result = Util_String("array type");
			}
			break;
		case RECORD:
			if (Trees_Symbol(type) != IDENT) {
				result = Util_String("anonymous ");
			}
			result = Util_String("record type");
			break;
		case POINTER:
			if (Trees_Symbol(type) != IDENT) {
				result = Util_String("anonymous ");
			}
			result = Util_String("pointer type");
			break;
		case PROCEDURE:
			if (Trees_Symbol(type) != IDENT) {
				result = Util_String("anonymous ");
			}
			result = Util_String("procedure type");
			break;
		case TREES_NIL_TYPE:
			result = Util_String("NIL");
			break;
		default:
			assert(0);
	}
	if (Trees_Symbol(type) == IDENT) {
		result = Util_String("%s (%s)", Trees_Name(type), result);
	}
	return result;
}


/*functions for expression productions*/

static int IsDesignator(Trees_Node exp)
{
	return Trees_Symbol(exp) == TREES_DESIGNATOR;
}


static int IsVariable(Trees_Node exp)
{
	int result = 0;

	if (IsDesignator(exp)) {
		switch (Trees_Kind(BaseIdent(exp))) {
			case TREES_VARIABLE_KIND:
			case TREES_VALUE_PARAM_KIND:
			case TREES_VAR_PARAM_KIND:
				result = 1;
		}
	}
	return result;
}


static int IsValueExpression(Trees_Node exp)
{
	int result = 1;

	if (IsDesignator(exp)) {
		switch (Trees_Kind(BaseIdent(exp))) {
			case TREES_CONSTANT_KIND:
			case TREES_FIELD_KIND:
			case TREES_VARIABLE_KIND:
			case TREES_PROCEDURE_KIND:
			case TREES_VALUE_PARAM_KIND:
			case TREES_VAR_PARAM_KIND:
				break;
			default:
				result = 0;
		}
	}
	return result;
}


static void CheckIsValueExpression(Trees_Node exp)
{
	if (! IsValueExpression(exp)) {
		Oberon_PrintError("error: value expected: %s", Trees_Name(BaseIdent(exp)));
		exit(EXIT_FAILURE);
	}
}


static Trees_Node Designator(const char identName[], Trees_Node selectorList)
{
	Trees_Node identSym, qualidentSym, designator, qualidentSelectorList;
	const char *qualidentName;

	/*set qualident name, symbol and selector list*/
	qualidentSym = NULL;
	qualidentSelectorList = NULL;
	identSym = Table_At(identName);
	if (((identSym == NULL) || ((Trees_Kind(identSym) == TREES_PROCEDURE_KIND) && ! Trees_Local(identSym)))
			&& (procedureDeclarationStack != NULL)
			&& (strcmp(identName, Trees_Name(Trees_Left(procedureDeclarationStack))) == 0)) {
		/*recursive call in local procedure*/
		qualidentSym = Trees_Left(procedureDeclarationStack);
		qualidentSelectorList = selectorList;
	} else {
		if (identSym != NULL) {
			if (Trees_Kind(identSym) == TREES_QUALIFIER_KIND) {
				if ((selectorList != NULL) && (Trees_Symbol(selectorList) == '.')) {
					qualidentName = Util_String("%s.%s", identName, Trees_Name(Trees_Left(selectorList)));
					qualidentSym = Table_At(qualidentName);
					qualidentSelectorList = Trees_Right(selectorList);
					if (qualidentSym == NULL) {
						Oberon_PrintError("error: undeclared identifier: %s", qualidentName);
						exit(EXIT_FAILURE);
					}
				} else {
					Oberon_PrintError("error: '.' expected after qualifier: %s", identName);
					exit(EXIT_FAILURE);
				}
			} else {
				qualidentSym = identSym;
				qualidentSelectorList = selectorList;
			}

		} else {
			Oberon_PrintError("error: undeclared identifier: %s", identName);
			exit(EXIT_FAILURE);
		}
	}
	assert(qualidentSym != NULL);

	designator = Trees_NewNode(TREES_DESIGNATOR, qualidentSym, qualidentSelectorList);

	return designator;
}


static Trees_Node BaseIdent(Trees_Node designator)
{
	assert(Trees_Symbol(designator) == TREES_DESIGNATOR);

	return Trees_Left(designator);
}


static Trees_Node FirstSelector(Trees_Node designator)
{
	assert(Trees_Symbol(designator) == TREES_DESIGNATOR);

	return Trees_Right(designator);
}


static void SetSelectorTypes(Trees_Node identType, Trees_Node designator, int *parameterListFound)
{
	Trees_Node currType, currTypeStruct, currSelector, prevSelector, indexExp, lengthNode, pointerNode, expList, extendedType, symbol, varField, typeField, fieldBaseType;
	OBNC_INTEGER length, index;
	const char *fieldName;

	currType = identType;
	currSelector = FirstSelector(designator);
	prevSelector = designator;
	*parameterListFound = 0;
	while ((currSelector != NULL) && ! *parameterListFound) {
		currTypeStruct = Types_Structure(currType);
		switch (Trees_Symbol(currSelector)) {
			case '[':
				if ((currTypeStruct != NULL) && (Trees_Symbol(currTypeStruct) == ARRAY)) {
					indexExp = Trees_Left(currSelector);
					lengthNode = Types_ArrayLength(currTypeStruct);
					if ((lengthNode != NULL) && (Trees_Symbol(indexExp) == INTEGER)) {
						length = Trees_Integer(lengthNode);
						index = Trees_Integer(indexExp);
						if ((index < 0) || (index >= length)) {
							Oberon_PrintError("error: invalid array index: %" OBNC_INT_MOD "d not between 0 and %" OBNC_INT_MOD "d", index, (OBNC_INTEGER) (length - 1));
							exit(EXIT_FAILURE);
						}
					}
					Trees_SetType(currType, currSelector);
					currType = Types_ElementType(currTypeStruct);
				} else {
					Oberon_PrintError("error: array variable expected in element selector");
					exit(EXIT_FAILURE);
				}
				break;
			case '.':
				if (currType != NULL) {
					switch (Trees_Symbol(currTypeStruct)) {
						case POINTER:
							pointerNode = Trees_NewNode('^', NULL, currSelector);
							Trees_SetType(currType, pointerNode);
							Trees_SetRight(pointerNode, prevSelector);
							currType = Types_PointerBaseType(currTypeStruct);
							/*fall through*/
						case RECORD:
							Trees_SetType(currType, currSelector);
							varField = Trees_Left(currSelector);
							fieldName = Trees_Name(varField);
							Types_GetFieldIdent(fieldName, currType, Trees_Imported(BaseIdent(designator)), &typeField, &fieldBaseType);
							if (typeField != NULL) {
								if (Trees_Exported(typeField)) {
									Trees_SetExported(varField);
								}
								currType = Trees_Type(typeField);
							} else {
								Oberon_PrintError("error: undeclared field: %s", fieldName);
								exit(EXIT_FAILURE);
							}
							break;
						default:
							Oberon_PrintError("error: record variable expected in field selector");
							exit(EXIT_FAILURE);
					}
				} else {
					Oberon_PrintError("error: record variable expected in field selector");
					exit(EXIT_FAILURE);
				}
				break;
			case '^':
				if ((currType != NULL) && (Trees_Symbol(currTypeStruct) == POINTER)) {
					Trees_SetType(currType, currSelector);
					currType = Types_PointerBaseType(currTypeStruct);
				} else {
					Oberon_PrintError("error: pointer variable expected in pointer dereference");
					exit(EXIT_FAILURE);
				}
				break;
			case '(':
				if (Types_IsProcedure(currTypeStruct)) {
					*parameterListFound = 1;
				} else if (Types_IsRecord(currTypeStruct) || Types_IsPointer(currTypeStruct)) {
					/*type guard*/
					expList = Trees_Left(currSelector);
					if (expList != NULL) {
						if (Trees_Right(expList) == NULL) {
							if ((Trees_Symbol(Trees_Left(expList)) == TREES_DESIGNATOR)
									&& (Trees_Right(Trees_Left(expList)) == NULL)) {
								extendedType = Trees_Left(Trees_Left(expList));
								symbol = Table_At(Trees_Name(extendedType));
								if (symbol != NULL) {
									if (Trees_Kind(symbol) == TREES_TYPE_KIND) {
										if ((Types_IsRecord(currType) && Types_IsRecord(Trees_Type(symbol))
													&& (Trees_Kind(BaseIdent(designator)) == TREES_VAR_PARAM_KIND))
												|| (Types_IsPointer(currType) && Types_IsPointer(Trees_Type(symbol)))) {
											if (Types_Extends(currType, Trees_Type(symbol))) {
												Trees_SetLeft(extendedType, currSelector);
												Trees_SetType(extendedType, currSelector);
												currType = extendedType;
											} else {
												Oberon_PrintError("error: extended type expected: %s", Trees_Name(extendedType));
												exit(EXIT_FAILURE);
											}
										} else {
											if (Types_IsRecord(currType)) {
												if (Trees_Kind(BaseIdent(designator)) != TREES_VAR_PARAM_KIND) {
													Oberon_PrintError("error: variable parameter expected in type guard");
												} else {
													Oberon_PrintError("error: record type expected in type guard: %s", Trees_Name(extendedType));
												}
												exit(EXIT_FAILURE);
											} else {
												Oberon_PrintError("error: pointer type expected in type guard: %s", Trees_Name(extendedType));
												exit(EXIT_FAILURE);
											}
										}
									} else {
										Oberon_PrintError("error: type name expected: %s", Trees_Name(extendedType));
										exit(EXIT_FAILURE);
									}
								} else {
									Oberon_PrintError("error: undeclared identifier: %s", Trees_Name(extendedType));
									exit(EXIT_FAILURE);
								}
							} else {
								Oberon_PrintError("error: identifier expected in type guard");
								exit(EXIT_FAILURE);
							}
						} else {
							Oberon_PrintError("error: unexpected comma in type guard");
							exit(EXIT_FAILURE);
						}
					} else {
						Oberon_PrintError("error: missing identifier in type guard");
						exit(EXIT_FAILURE);
					}
				} else {
					Oberon_PrintError("error: unexpected parenthesis in designator which is not a record, pointer or procedure");
					exit(EXIT_FAILURE);
				}
				break;
			default:
				assert(0);
		}
		prevSelector = currSelector;
		currSelector = Trees_Right(currSelector);
	}

	if (currSelector == NULL) {
		Trees_SetType(currType, designator);
	} else {
		Oberon_PrintError("error: unexpected selector after procedure call");
		exit(EXIT_FAILURE);
	}
}


static void RemoveActualParameters(Trees_Node *designator, Trees_Node *actualParameters)
{
	Trees_Node currSelector;

	currSelector = FirstSelector(*designator);
	assert(currSelector != NULL);
	if (Trees_Right(currSelector) == NULL) {
		*actualParameters = Trees_Left(currSelector);
		Trees_SetRight(NULL, *designator);
	} else {
		while (Trees_Right(Trees_Right(currSelector)) != NULL) {
			currSelector = Trees_Right(currSelector);
		}
		*actualParameters = Trees_Left(Trees_Right(currSelector));
		Trees_SetRight(NULL, currSelector);
	}
}


static int IsConstExpression(Trees_Node exp)
{
	int result = 0;

	assert(exp != NULL);
	switch (Trees_Symbol(exp)) {
		case TRUE:
		case FALSE:
		case STRING:
		case TREES_CHAR_CONSTANT:
		case INTEGER:
		case REAL:
		case TREES_SET_CONSTANT:
		case NIL:
		case TREES_SIZE_PROC: /*type sizes cannot always be evaluated in the Oberon-to-C translation*/
			result = 1;
			break;
		case IDENT:
		case TREES_DESIGNATOR:
		case TREES_ADR_PROC:
		case TREES_BIT_PROC:
			result = 0;
			break;
		default:
			result = ((Trees_Left(exp) == NULL) || IsConstExpression(Trees_Left(exp)))
				&& ((Trees_Right(exp) == NULL) || IsConstExpression(Trees_Right(exp)));
	}
	return result;
}


static Trees_Node ExpressionConstValue(int relation, Trees_Node expA, Trees_Node expB)
{
	Trees_Node result = NULL;

	switch (relation) {
		case '=':
			switch (Trees_Symbol(expA)) {
				case TRUE:
				case FALSE:
					if (IsBoolean(expB)) {
						result = Trees_NewLeaf((Trees_Symbol(expA) == Trees_Symbol(expB))? TRUE: FALSE);
					}
					break;
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) == Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) == Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) == Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) == Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case TREES_SET_CONSTANT:
					if (IsSet(expB)) {
						result = Trees_NewLeaf((Trees_Set(expA) == Trees_Set(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] == Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) == 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case '#':
			switch (Trees_Symbol(expA)) {
				case TRUE:
				case FALSE:
					if (IsBoolean(expB)) {
						result = Trees_NewLeaf((Trees_Symbol(expA) != Trees_Symbol(expB))? TRUE: FALSE);
					}
					break;
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) != Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) != Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) != Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) != Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case TREES_SET_CONSTANT:
					if (IsSet(expB)) {
						result = Trees_NewLeaf((Trees_Set(expA) != Trees_Set(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] != Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) != 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case '<':
			switch (Trees_Symbol(expA)) {
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) < Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) < Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) < Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) < Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] < Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) < 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case LE:
			switch (Trees_Symbol(expA)) {
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) <= Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) <= Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) <= Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) <= Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] <= Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) <= 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case '>':
			switch (Trees_Symbol(expA)) {
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) > Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) > Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) > Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) > Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] > Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) > 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case GE:
			switch (Trees_Symbol(expA)) {
				case TREES_CHAR_CONSTANT:
					if (IsString(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) >= Trees_String(expB)[0])? TRUE: FALSE);
					} else if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_Char(expA) >= Trees_Char(expB))? TRUE: FALSE);
					}
					break;
				case INTEGER:
					if (IsInteger(expB)) {
						result = Trees_NewLeaf((Trees_Integer(expA) >= Trees_Integer(expB))? TRUE: FALSE);
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						result = Trees_NewLeaf((Trees_Real(expA) >= Trees_Real(expB))? TRUE: FALSE);
					}
					break;
				case STRING:
					if (IsChar(expB)) {
						result = Trees_NewLeaf((Trees_String(expA)[0] >= Trees_Char(expB))? TRUE: FALSE);
					} else if (IsString(expB)) {
						result = Trees_NewLeaf((strcmp(Trees_String(expA), Trees_String(expB)) >= 0)? TRUE: FALSE);
					}
					break;
			}
			break;
		case IN:
			if (IsInteger(expA)) {
				Range_CheckSetElement(Trees_Integer(expA));
				if (IsSet(expB)) {
					result = Trees_NewLeaf(OBNC_IN(Trees_Integer(expA), Trees_Set(expB))? TRUE: FALSE);
				}
			}
			break;
	}
	if (result != NULL) {
		Trees_SetType(Trees_NewLeaf(TREES_BOOLEAN_TYPE), result);
	}

	return result;
}


static Trees_Node SimpleExpressionConstValue(int operator, Trees_Node expA, Trees_Node expB)
{
	Trees_Node result = NULL;

	switch (operator) {
		case '+':
			switch (Trees_Symbol(expA)) {
				case INTEGER:
					if (expB == NULL) {
						result = expA;
					} else if (IsInteger(expB)) {
						Range_CheckIntSum(Trees_Integer(expA), Trees_Integer(expB));
						result = Trees_NewInteger(Trees_Integer(expA) + Trees_Integer(expB));
					}
					break;
				case REAL:
					if (expB == NULL) {
						result = expA;
					} else if (IsReal(expB)) {
						Range_CheckRealSum(Trees_Real(expA), Trees_Real(expB));
						result = Trees_NewReal(Trees_Real(expA) + Trees_Real(expB));
					}
					break;
				case TREES_SET_CONSTANT:
					if (expB == NULL) {
						result = expA;
					} else if (IsSet(expB)) {
						result = Trees_NewSet(Trees_Set(expA) | Trees_Set(expB));
					}
					break;
			}
			break;
		case '-':
			switch (Trees_Symbol(expA)) {
				case INTEGER:
					if (expB == NULL) {
						Range_CheckIntDiff(0, Trees_Integer(expA));
						result = Trees_NewInteger(-Trees_Integer(expA));
					} else if (IsInteger(expB)) {
						Range_CheckIntDiff(Trees_Integer(expA), Trees_Integer(expB));
						result = Trees_NewInteger(Trees_Integer(expA) - Trees_Integer(expB));
					}
					break;
				case REAL:
					if (expB == NULL) {
						Range_CheckRealDiff(0.0, Trees_Real(expA));
						result = Trees_NewReal(-Trees_Real(expA));
					} else if (IsReal(expB)) {
						Range_CheckRealDiff(Trees_Real(expA), Trees_Real(expB));
						result = Trees_NewReal(Trees_Real(expA) - Trees_Real(expB));
					}
					break;
				case TREES_SET_CONSTANT:
					if (expB == NULL) {
						result = Trees_NewSet(~Trees_Set(expA));
					} else if (IsSet(expB)) {
						result = Trees_NewSet(Trees_Set(expA) & ~Trees_Set(expB));
					}
					break;
			}
			break;
		case OR:
			if (IsBoolean(expA) && IsBoolean(expB)) {
				result = (Trees_Symbol(expA) == TRUE)? expA: expB;
			}
			break;
	}

	return result;
}


static Trees_Node TermConstValue(int operator, Trees_Node expA, Trees_Node expB)
{
	Trees_Node result = NULL;

	switch (operator) {
		case '*':
			switch (Trees_Symbol(expA)) {
				case INTEGER:
					if (IsInteger(expB)) {
						Range_CheckIntProd(Trees_Integer(expA), Trees_Integer(expB));
						result = Trees_NewInteger(Trees_Integer(expA) * Trees_Integer(expB));
					}
					break;
				case REAL:
					if (IsReal(expB)) {
						Range_CheckRealProd(Trees_Real(expA), Trees_Real(expB));
						result = Trees_NewReal(Trees_Real(expA) * Trees_Real(expB));
					}
					break;
				case TREES_SET_CONSTANT:
					if (IsSet(expB)) {
						result = Trees_NewSet(Trees_Set(expA) & Trees_Set(expB));
					}
					break;
			}
			break;
		case '/':
			switch (Trees_Symbol(expA)) {
				case REAL:
					if (IsReal(expA) && IsReal(expB)) {
						if (Trees_Real(expB) != 0) {
							result = Trees_NewReal(Trees_Real(expA) / Trees_Real(expB));
						} else {
							Oberon_PrintError("warning: division by zero");
						}
					}
					break;
				case TREES_SET_CONSTANT:
					if (IsSet(expB)) {
						result = Trees_NewSet(Trees_Set(expA) ^ Trees_Set(expB));
					}
					break;
			}
			break;
		case DIV:
			if (IsInteger(expA) && IsInteger(expB)) {
				if (Trees_Integer(expB) > 0) {
					result = Trees_NewInteger(OBNC_DIV(Trees_Integer(expA), Trees_Integer(expB)));
				} else {
					Oberon_PrintError("error: positive divisor expected in DIV expression: %" OBNC_INT_MOD "d", Trees_Integer(expB));
					exit(EXIT_FAILURE);
				}
			}
			break;
		case MOD:
			if (IsInteger(expA) && IsInteger(expB)) {
				if (Trees_Integer(expB) > 0) {
					result = Trees_NewInteger(OBNC_MOD(Trees_Integer(expA), Trees_Integer(expB)));
				} else {
					Oberon_PrintError("error: positive divisor expected in MOD expression: %" OBNC_INT_MOD "d", Trees_Integer(expB));
					exit(EXIT_FAILURE);
				}
			}
			break;
		case '&':
			if (IsBoolean(expA) && IsBoolean(expB)) {
				if (Trees_Symbol(expA) == TRUE) {
					result = expB;
				} else {
					result = expA;
				}
			}
			break;
	}

	return result;
}


static const char *DesignatorString(Trees_Node designator)
{
	const char *baseName, *result;

	assert(IsDesignator(designator));

	baseName = Trees_Name(BaseIdent(designator));
	if (FirstSelector(designator) != NULL) {
		result = Util_String("%s...", baseName);
	} else {
		result = Util_String("%s", baseName);
	}
	return result;
}


static const char *OperatorString(int operator)
{
	const char *result = "";

	switch (operator) {
		case '+':
			result = "+";
			break;
		case '-':
			result = "-";
			break;
		case '*':
			result = "*";
			break;
		case '/':
			result = "/";
			break;
		case DIV:
			result = "DIV";
			break;
		case MOD:
			result = "MOD";
			break;
		case OR:
			result = "OR";
			break;
		case '&':
			result = "&";
			break;
		case '~':
			result = "~";
			break;
		case '=':
			result = "=";
			break;
		case '#':
			result = "#";
			break;
		case '<':
			result = "<";
			break;
		case LE:
			result = "<=";
			break;
		case '>':
			result = ">";
			break;
		case GE:
			result = ">=";
			break;
		case IN:
			result = "IN";
			break;
		case IS:
			result = "IS";
			break;
		default:
			assert(0);
	}
	return result;
}


/*functions for statement productions*/

static int Writable(Trees_Node designator)
{
	Trees_Node ident, type;
	int kind, result;

	assert(IsDesignator(designator));

	ident = BaseIdent(designator);
	kind = Trees_Kind(ident);
	type = Trees_Type(ident);
	result = ((kind == TREES_VARIABLE_KIND) && ! Trees_Imported(ident))
		|| (kind == TREES_VAR_PARAM_KIND)
		|| ((kind == TREES_VALUE_PARAM_KIND) && ! Types_IsArray(type) && ! Types_IsRecord(type));
	return result;
}


static const char *AssignmentErrorContext(int context, int paramPos)
{
	const char *result;

	switch (context) {
		case ASSIGNMENT_CONTEXT:
			result = Util_String("assignment");
			break;
		case PARAM_SUBST_CONTEXT:
			assert(paramPos >= 0);
			result = Util_String("substitution of parameter %d", paramPos + 1);
			break;
		case PROC_RESULT_CONTEXT:
			result = Util_String("return clause");
			break;
		default:
			assert(0);
	}
	return result;
}


static void ValidateAssignment(Trees_Node expression, Trees_Node targetType, int context, int paramPos)
{
	const char *errorContext;

	assert(expression != NULL);
	assert(targetType != NULL);
	assert(context >= 0);
	assert(paramPos >= 0);

	errorContext = AssignmentErrorContext(context, paramPos);
	if (Types_AssignmentCompatible(expression, targetType)) {
		if (Types_IsByte(targetType) && IsInteger(expression)) {
			Range_CheckByte(Trees_Integer(expression));
		}
		if (IsDesignator(expression) && (Trees_Kind(BaseIdent(expression)) == TREES_PROCEDURE_KIND) && Trees_Local(BaseIdent(expression))) {
			Oberon_PrintError("error: globally declared procedure expected in %s", errorContext);
			exit(EXIT_FAILURE);
		}
	} else {
		if (IsString(expression) && Types_IsCharacterArray(targetType) && ! Types_IsOpenArray(targetType)) {
			Oberon_PrintError("error: string too long in %s: %" OBNC_INT_MOD "d + 1 > %" OBNC_INT_MOD "d", errorContext, Types_StringLength(Trees_Type(expression)), Trees_Integer(Types_ArrayLength(targetType)));
			exit(EXIT_FAILURE);
		} else if (Types_IsPredeclaredProcedure(Trees_Type(expression))
				&& Types_IsProcedure(targetType)) {
			Oberon_PrintError("error: non-predeclared procedure expected in %s", errorContext);
			exit(EXIT_FAILURE);
		} else {
			Oberon_PrintError("error: incompatible types in %s: %s -> %s",
				errorContext, TypeString(Trees_Type(expression)), TypeString(targetType));
			exit(EXIT_FAILURE);
		}
	}
}


static void ValidateActualParameter(Trees_Node actualParam, Trees_Node formalParam, int paramPos, Trees_Node procDesignator)
{
	Trees_Node formalType, actualType;

	formalType = Trees_Type(formalParam);
	actualType = Trees_Type(actualParam);

	if ((Trees_Kind(formalParam) == TREES_VALUE_PARAM_KIND)
			|| (IsDesignator(actualParam) && Writable(actualParam))) {
		if (Types_IsOpenArray(formalType)) {
			if (! Types_ArrayCompatible(actualType, formalType)) {
				Oberon_PrintError("error: array incompatible types in substitution of parameter %d in %s: %s -> %s", paramPos + 1, DesignatorString(procDesignator), TypeString(actualType), TypeString(formalType));
				exit(EXIT_FAILURE);
			}
		} else if (Trees_Kind(formalParam) == TREES_VALUE_PARAM_KIND) {
			ValidateAssignment(actualParam, formalType, PARAM_SUBST_CONTEXT, paramPos);
		} else {
			assert(Trees_Kind(formalParam) == TREES_VAR_PARAM_KIND);
			if (Types_IsRecord(formalType)) {
				if (Types_IsRecord(actualType)) {
					if (! Types_Extends(formalType, actualType)) {
						Oberon_PrintError("error: incompatible record types in substitution of parameter %d in %s: %s -> %s", paramPos + 1, DesignatorString(procDesignator), TypeString(actualType), TypeString(formalType));
						exit(EXIT_FAILURE);
					}
				} else {
					Oberon_PrintError("error: record expected in substitution of parameter %d in %s: %s -> %s", paramPos + 1, DesignatorString(procDesignator), TypeString(actualType), TypeString(formalType));
					exit(EXIT_FAILURE);
				}
			} else {
				if (! Types_Same(actualType, formalType)) {
					Oberon_PrintError("error: same types expected in substitution of parameter %d in %s: %s -> %s", paramPos + 1, DesignatorString(procDesignator), TypeString(actualType), TypeString(formalType));
					exit(EXIT_FAILURE);
				}
			}
		}
	} else {
		if (IsDesignator(actualParam)) {
			Oberon_PrintError("error: writable variable expected in substitution of parameter %d in %s",
				paramPos + 1, DesignatorString(procDesignator));
		} else {
			Oberon_PrintError("error: variable expected in substitution of parameter %d in %s",
				paramPos + 1, DesignatorString(procDesignator));
		}
		exit(EXIT_FAILURE);
	}
}


static void ValidateProcedureCall(Trees_Node expList, Trees_Node fpList, Trees_Node procDesignator)
{
	Trees_Node exp, formalParam, fpType;
	int pos;

	pos = 0;
	while ((expList != NULL) && (fpList != NULL)) {
		exp = Trees_Left(expList);
		CheckIsValueExpression(exp);
		formalParam = Trees_Left(fpList);
		fpType = Trees_Type(formalParam);
		ValidateActualParameter(exp, formalParam, pos, procDesignator);

		if (Types_IsChar(fpType) && (Trees_Symbol(exp) == STRING)) {
			Trees_SetLeft(Trees_NewChar(Trees_String(exp)[0]), expList);
		}
		expList = Trees_Right(expList);
		fpList = Trees_Right(fpList);
		pos++;
	}
	if ((expList == NULL) && (fpList != NULL)) {
		Oberon_PrintError("error: too few actual parameters in procedure call: %s", DesignatorString(procDesignator));
		exit(EXIT_FAILURE);
	} else if ((expList != NULL) && (fpList == NULL)) {
		Oberon_PrintError("error: too many actual parameters in procedure call: %s", DesignatorString(procDesignator));
		exit(EXIT_FAILURE);
	}
}


static void ValidateProcedureKind(const char procName[], int functionCallExpected, int isFunctionCall)
{
	if (isFunctionCall && ! functionCallExpected) {
		Oberon_PrintError("error: function procedure expected: %s", procName);
		exit(EXIT_FAILURE);
	} else if (! isFunctionCall && functionCallExpected) {
		Oberon_PrintError("error: proper procedure expected: %s", procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateParameterCount(const char procName[], int min, int max, int actual)
{
	assert(min >= 0);
	assert(min <= max);
	assert(actual >= 0);

	if ((actual < min) || (actual > max)) {
		if (min == max) {
			Oberon_PrintError("error: %d parameter(s) expected: %s", min, procName);
		} else {
			Oberon_PrintError("error: %d or %d parameters expected: %s", min, max, procName);
		}
		exit(EXIT_FAILURE);
	}
}


static void ValidateTypeParameter(const char procName[], Trees_Node param, int pos)
{
	if (! (IsDesignator(param) && (Trees_Kind(BaseIdent(param)) == TREES_TYPE_KIND))) {
		Oberon_PrintError("error: type identifier expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateValueParameter(const char procName[], Trees_Node param, int pos)
{
	if (! IsValueExpression(param)) {
		Oberon_PrintError("error: expression expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateVariableParameter(const char procName[], Trees_Node param, int pos)
{
	if (! IsDesignator(param)) {
		Oberon_PrintError("error: variable expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	} else if (! Writable(param)) {
		Oberon_PrintError("error: writable variable expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateScalarParameter(const char procName[], Trees_Node paramType, int pos)
{
	assert(Types_IsType(paramType));

	if (! Types_Scalar(paramType)) {
		Oberon_PrintError("error: scalar type expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateIntegerParameter(const char procName[], Trees_Node param, int pos)
{
	if (! Types_IsInteger(Trees_Type(param))) {
		Oberon_PrintError("error: integer expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static void ValidateRealParameter(const char procName[], Trees_Node param, int pos)
{
	if (! Types_IsReal(Trees_Type(param))) {
		Oberon_PrintError("error: parameter of type REAL expected in substitution of parameter %d: %s", pos + 1, procName);
		exit(EXIT_FAILURE);
	}
}


static OBNC_INTEGER TypeSize(Trees_Node type)
{
	OBNC_INTEGER result = 0;

	switch (Trees_Symbol(Types_Structure(type))) {
		case TREES_BOOLEAN_TYPE:
			result = sizeof (int);
			break;
		case TREES_CHAR_TYPE:
			result = sizeof (char);
			break;
		case TREES_INTEGER_TYPE:
			result = sizeof (OBNC_INTEGER);
			break;
		case TREES_REAL_TYPE:
			result = sizeof (OBNC_REAL);
			break;
		case TREES_BYTE_TYPE:
			result = sizeof (unsigned char);
			break;
		case TREES_SET_TYPE:
			result = sizeof (unsigned OBNC_INTEGER);
			break;
		case ARRAY:
			result = Trees_Integer(Types_ArrayLength(type)) * TypeSize(Types_ElementType(type));
			break;
		case RECORD:
		case POINTER:
		case PROCEDURE:
			/*cannot be calculated in the Oberon-to-C translation*/
			break;
		default:
			assert(0);
	}
	return result;
}


static Trees_Node PredeclaredProcedureAST(const char procName[], Trees_Node expList, int isFunctionCall)
{
	static const struct { const char *name; int symbol; } symbols[] = {
		{"ABS", TREES_ABS_PROC},
		{"ASR", TREES_ASR_PROC},
		{"ASSERT", TREES_ASSERT_PROC},
		{"CHR", TREES_CHR_PROC},
		{"DEC", TREES_DEC_PROC},
		{"EXCL", TREES_EXCL_PROC},
		{"FLOOR", TREES_FLOOR_PROC},
		{"FLT", TREES_FLT_PROC},
		{"INC", TREES_INC_PROC},
		{"INCL", TREES_INCL_PROC},
		{"LEN", TREES_LEN_PROC},
		{"LSL", TREES_LSL_PROC},
		{"NEW", TREES_NEW_PROC},
		{"ODD", TREES_ODD_PROC},
		{"ORD", TREES_ORD_PROC},
		{"PACK", TREES_PACK_PROC},
		{"ROR", TREES_ROR_PROC},
		{"UNPK", TREES_UNPK_PROC},

		{"ADR", TREES_ADR_PROC},
		{"SIZE", TREES_SIZE_PROC},
		{"BIT", TREES_BIT_PROC},
		{"GET", TREES_GET_PROC},
		{"PUT", TREES_PUT_PROC},
		{"COPY", TREES_COPY_PROC},
		{"VAL", TREES_VAL_PROC}};

	int paramCount, pos, symbol;
	Trees_Node curr, resultType, result;
	Trees_Node param[3], paramTypes[3];
	const char *unqualProcName;

	/*set actual parameters*/
	paramCount = 0;
	curr = expList;
	while ((paramCount < LEN(param)) && (curr != NULL)) {
		param[paramCount] = Trees_Left(curr);
		paramTypes[paramCount] = Trees_Type(Trees_Left(curr));
		paramCount++;
		curr = Trees_Right(curr);
	}

	/*find procedure symbol*/
	unqualProcName = strchr(procName, '.');
	if (unqualProcName != NULL) {
		unqualProcName++;
	} else {
		unqualProcName = procName;
	}
	pos = 0;
	while ((pos < LEN(symbols)) && (strcmp(symbols[pos].name, unqualProcName) != 0)) {
		pos++;
	}
	assert(pos < LEN(symbols));
	symbol = symbols[pos].symbol;

	/*validate parameters and build syntax tree*/
	result = NULL;
	resultType = NULL;
	switch (symbol) {
		case TREES_ABS_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			switch (Trees_Symbol(Types_Structure(paramTypes[0]))) {
				case TREES_INTEGER_TYPE:
					if (IsInteger(param[0])) {
						result = Trees_NewInteger(OBNC_ABS_INT(Trees_Integer(param[0])));
					}
					break;
				case TREES_REAL_TYPE:
					if (IsReal(param[0])) {
						result = Trees_NewReal(OBNC_ABS_FLT(Trees_Real(param[0])));
					}
					break;
				case TREES_BYTE_TYPE:
					/*do nothing*/
					break;
				default:
					Oberon_PrintError("error: numeric parameter expected: %s", procName);
					exit(EXIT_FAILURE);
			}
			if (result == NULL) {
				resultType = paramTypes[0];
			}
			break;
		case TREES_ODD_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			if (IsInteger(param[0])) {
				result = Trees_NewBoolean(OBNC_ODD(Trees_Integer(param[0])));
			} else {
				resultType = Trees_NewLeaf(TREES_BOOLEAN_TYPE);
			}
			break;
		case TREES_LEN_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			if (Types_IsArray(paramTypes[0])) {
				if (! Types_IsOpenArray(paramTypes[0])) {
					result = Types_ArrayLength(paramTypes[0]);
				} else {
					resultType = Trees_NewLeaf(TREES_INTEGER_TYPE);
				}
			} else {
				Oberon_PrintError("error: array parameter expected: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_LSL_PROC: /*fall through*/
		case TREES_ASR_PROC: /*fall through*/
		case TREES_ROR_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			ValidateIntegerParameter(procName, param[1], 1);
			if (IsInteger(param[1])) {
				switch (symbol) {
					case TREES_LSL_PROC:
						Range_CheckLSL(Trees_Integer(param[1]));
						break;
					case TREES_ASR_PROC:
						Range_CheckASR(Trees_Integer(param[1]));
						break;
					case TREES_ROR_PROC:
						Range_CheckROR(Trees_Integer(param[1]));
						break;
					default:
						assert(0);
				}
			}
			if (IsInteger(param[0]) && IsInteger(param[1])) {
				switch (symbol) {
					case TREES_LSL_PROC:
						result = Trees_NewInteger(OBNC_LSL(Trees_Integer(param[0]), Trees_Integer(param[1])));
						break;
					case TREES_ASR_PROC:
						result = Trees_NewInteger(OBNC_ASR(Trees_Integer(param[0]), Trees_Integer(param[1])));
						break;
					case TREES_ROR_PROC:
						result = Trees_NewInteger(OBNC_ROR(Trees_Integer(param[0]), Trees_Integer(param[1])));
						break;
					default:
						assert(0);
				}
			}
			if (result == NULL) {
				resultType = paramTypes[0];
			}
			break;
		case TREES_FLOOR_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateRealParameter(procName, param[0], 0);
			if (IsReal(param[0])) {
				OBNC_REAL x = Trees_Real(param[0]);
				Range_CheckFLOOR(x);
				result = Trees_NewInteger(OBNC_FLOOR(x));
			} else {
				resultType = Trees_NewLeaf(TREES_INTEGER_TYPE);
			}
			break;
		case TREES_FLT_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			if (IsInteger(param[0])) {
				result = Trees_NewReal(OBNC_FLT(Trees_Integer(param[0])));
			} else {
				resultType = Trees_NewLeaf(TREES_REAL_TYPE);
			}
			break;
		case TREES_ORD_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			switch (Trees_Symbol(Types_Structure(paramTypes[0]))) {
				case TREES_CHAR_TYPE:
					/*do nothing*/
					break;
				case TREES_STRING_TYPE:
					if (Types_StringLength(paramTypes[0]) <= 1) {
						result = Trees_NewInteger((unsigned char) Trees_String(param[0])[0]);
					} else {
						Oberon_PrintError("error: single-character string parameter expected: %s", procName);
						exit(EXIT_FAILURE);
					}
					break;
				case TREES_BOOLEAN_TYPE:
					if (Trees_Symbol(param[0]) == TRUE) {
						result = Trees_NewInteger(1);
					} else if (Trees_Symbol(param[0]) == FALSE) {
						result = Trees_NewInteger(0);
					}
					break;
				case TREES_SET_TYPE:
					if (IsSet(param[0])) {
						result = Trees_NewInteger((OBNC_INTEGER) Trees_Set(param[0]));
					}
					break;
				default:
					Oberon_PrintError("error: character parameter expected: %s", procName);
					exit(EXIT_FAILURE);
			}
			if (result == NULL) {
				resultType = Trees_NewLeaf(TREES_INTEGER_TYPE);
			}
			break;
		case TREES_CHR_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			if (IsInteger(param[0])) {
				OBNC_INTEGER i = Trees_Integer(param[0]);
				Range_CheckCHR(i);
				result = Trees_NewChar(OBNC_CHR(i));
			} else {
				resultType = Trees_NewLeaf(TREES_CHAR_TYPE);
			}
			break;
		case TREES_INC_PROC: /*fall through*/
		case TREES_DEC_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 1, 2, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			if (paramCount == 2) {
				ValidateIntegerParameter(procName, param[1], 1);
			}
			break;
		case TREES_INCL_PROC: /*fall through*/
		case TREES_EXCL_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 1, 2, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			if (Types_IsSet(paramTypes[0])) {
				if (IsInteger(param[1])) {
					Range_CheckSetElement(Trees_Integer(param[1]));
				} else {
					ValidateIntegerParameter(procName, param[1], 1);
				}
			} else {
				Oberon_PrintError("error: set expected in substitution of parameter 1: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_ASSERT_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			if (! Types_IsBoolean(paramTypes[0])) {
				Oberon_PrintError("error: boolean parameter expected: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_NEW_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			if (! Types_IsPointer(paramTypes[0])) {
				Oberon_PrintError("error: pointer parameter expected: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_PACK_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			ValidateRealParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			ValidateIntegerParameter(procName, param[1], 1);
			break;
		case TREES_UNPK_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			ValidateRealParameter(procName, param[0], 0);
			ValidateVariableParameter(procName, param[1], 1);
			ValidateIntegerParameter(procName, param[1], 1);
			break;
		case TREES_ADR_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 1, 1, paramCount);
			ValidateVariableParameter(procName, param[0], 0);
			resultType = Trees_NewLeaf(TREES_INTEGER_TYPE);
			break;
		case TREES_SIZE_PROC:
			{
				OBNC_INTEGER size;

				ValidateProcedureKind(procName, 1, isFunctionCall);
				ValidateParameterCount(procName, 1, 1, paramCount);
				ValidateTypeParameter(procName, param[0], 0);
				size = TypeSize(Trees_Type(BaseIdent(param[0])));
				if (size > 0) {
					result = Trees_NewInteger(size);
				} else {
					resultType = Trees_NewLeaf(TREES_INTEGER_TYPE);
				}
			}
			break;
		case TREES_BIT_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			ValidateIntegerParameter(procName, param[1], 1);
			if (IsInteger(param[1])) {
				Range_CheckBIT(Trees_Integer(param[1]));
			}
			resultType = Trees_NewLeaf(TREES_BOOLEAN_TYPE);
			break;
		case TREES_GET_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			ValidateVariableParameter(procName, param[1], 1);
			if (! Types_Basic(paramTypes[1])) {
				Oberon_PrintError("error: variable of basic type expected in substitution of parameter 2: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_PUT_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			if (! Types_Basic(paramTypes[1]) && ! Types_IsSingleCharString(paramTypes[1])) {
				Oberon_PrintError("error: expression of basic type expected in substitution of parameter 2: %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		case TREES_COPY_PROC:
			ValidateProcedureKind(procName, 0, isFunctionCall);
			ValidateParameterCount(procName, 3, 3, paramCount);
			ValidateValueParameter(procName, param[0], 0);
			ValidateIntegerParameter(procName, param[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			ValidateIntegerParameter(procName, param[1], 1);
			ValidateValueParameter(procName, param[2], 2);
			ValidateIntegerParameter(procName, param[2], 2);
			if (IsInteger(param[2]) && (Trees_Integer(param[2]) < 0)) {
				Oberon_PrintError("warning: non-negative count expected in %s: %" OBNC_INT_MOD "d", procName, Trees_Integer(param[2]));
			}
			break;
		case TREES_VAL_PROC:
			ValidateProcedureKind(procName, 1, isFunctionCall);
			ValidateParameterCount(procName, 2, 2, paramCount);
			ValidateTypeParameter(procName, param[0], 0);
			ValidateScalarParameter(procName, paramTypes[0], 0);
			ValidateValueParameter(procName, param[1], 1);
			ValidateScalarParameter(procName, paramTypes[1], 1);
			resultType = paramTypes[0];
			if (IsConstExpression(param[1])) {
				result = param[1];
				Trees_SetType(resultType, result);
			} else if (! IsVariable(param[1])) {
				Oberon_PrintError("error: OBNC only supports variable parameters in %s", procName);
				exit(EXIT_FAILURE);
			}
			break;
		default:
			assert(0);
	}

	if (result == NULL) {
		result = Trees_NewNode(symbol, expList, NULL);
		if (isFunctionCall) {
			assert(resultType != NULL);
			Trees_SetType(resultType, result);
		}
	}
	return result;
}


static void HandleProcedureCall(Trees_Node designator, Trees_Node expList, int isFunctionCall, Trees_Node *ast)
{
	Trees_Node ident, designatorTypeStruct, fpList, resultType;

	ident = BaseIdent(designator);
	if (Types_IsPredeclaredProcedure(Trees_Type(ident))) {
		*ast = PredeclaredProcedureAST(Trees_Name(ident), expList, isFunctionCall);
		if (*ast == NULL) {
			Oberon_PrintError("error: procedure expected");
			exit(EXIT_FAILURE);
		}
	} else {
		/*handle non-predeclared procedure*/
		designatorTypeStruct = Types_Structure(Trees_Type(designator));
		if (Types_IsProcedure(designatorTypeStruct)) {
			fpList =Types_Parameters(designatorTypeStruct);
			resultType = Types_ResultType(designatorTypeStruct);
			ValidateProcedureCall(expList, fpList, designator);
			*ast = Trees_NewNode(TREES_PROCEDURE_CALL, designator, expList);
			if (isFunctionCall) {
				if (resultType != NULL) {
					Trees_SetType(resultType, *ast);
				} else {
					Oberon_PrintError("error: function procedure expected: %s", Trees_Name(ident));
					exit(EXIT_FAILURE);
				}
			} else if (resultType != NULL) {
				Oberon_PrintError("error: proper procedure expected: %s", Trees_Name(ident));
				exit(EXIT_FAILURE);
			}
		}
	}
	assert(*ast != NULL);
}


static void CheckIntegerLabelDisjointness(Trees_Node rangeA, Trees_Node rangeB)
{
	OBNC_INTEGER aMin, aMax, bMin, bMax;

	if (Trees_Symbol(rangeA) == DOTDOT) {
		aMin = Trees_Integer(Trees_Left(rangeA));
		aMax = Trees_Integer(Trees_Right(rangeA));
	} else {
		aMin = Trees_Integer(rangeA);
		aMax = Trees_Integer(rangeA);
	}
	if (Trees_Symbol(rangeB) == DOTDOT) {
		bMin = Trees_Integer(Trees_Left(rangeB));
		bMax = Trees_Integer(Trees_Right(rangeB));
	} else {
		bMin = Trees_Integer(rangeB);
		bMax = Trees_Integer(rangeB);
	}

	if ((aMin >= bMin) && (aMin <= bMax)) {
		Oberon_PrintError("error: case label defined twice: %" OBNC_INT_MOD "d", aMin);
		exit(EXIT_FAILURE);
	} else if ((bMin >= aMin) && (bMin <= aMax)) {
		Oberon_PrintError("error: case label defined twice: %" OBNC_INT_MOD "d", bMin);
		exit(EXIT_FAILURE);
	}
}


static void CheckCharLabelDisjointness(Trees_Node rangeA, Trees_Node rangeB)
{
	char aMin, aMax, bMin, bMax;
	int hasRepeatedLabel, repeatedLabel;

	if (Trees_Symbol(rangeA) == DOTDOT) {
		aMin = Trees_Char(Trees_Left(rangeA));
		aMax = Trees_Char(Trees_Right(rangeA));
	} else {
		aMin = Trees_Char(rangeA);
		aMax = Trees_Char(rangeA);
	}
	if (Trees_Symbol(rangeB) == DOTDOT) {
		bMin = Trees_Char(Trees_Left(rangeB));
		bMax = Trees_Char(Trees_Right(rangeB));
	} else {
		bMin = Trees_Char(rangeB);
		bMax = Trees_Char(rangeB);
	}

	if ((aMin >= bMin) && (aMin <= bMax)) {
		hasRepeatedLabel = 1;
		repeatedLabel = aMin;
	} else if ((bMin >= aMin) && (bMin <= aMax)) {
		hasRepeatedLabel = 1;
		repeatedLabel = bMin;
	} else {
		hasRepeatedLabel = 0;
	}
	if (hasRepeatedLabel) {
		if (isprint(repeatedLabel)) {
			Oberon_PrintError("error: case label defined twice: \"%c\"", repeatedLabel);
		} else {
			Oberon_PrintError("error: case label defined twice: 0%XX", repeatedLabel);
		}
		exit(EXIT_FAILURE);
	}
}


static void CheckCaseLabelUniqueness(Trees_Node newLabelRange)
{
	int labelSymbol;
	Trees_Node labelList, definedLabelRange;

	if (Trees_Symbol(newLabelRange) == DOTDOT) {
		labelSymbol = Trees_Symbol(Trees_Left(newLabelRange));
	} else {
		labelSymbol = Trees_Symbol(newLabelRange);
	}

	labelList = Trees_Left(caseLabelsStack);
	while (labelList != NULL) {
		definedLabelRange = Trees_Left(labelList);
		switch (labelSymbol) {
			case INTEGER:
				CheckIntegerLabelDisjointness(definedLabelRange, newLabelRange);
				break;
			case TREES_CHAR_CONSTANT:
				CheckCharLabelDisjointness(definedLabelRange, newLabelRange);
				break;
			case IDENT:
				if (Types_Same(definedLabelRange, newLabelRange)) {
					Oberon_PrintError("error: type case label defined twice: %s", Trees_Name(newLabelRange));
					exit(EXIT_FAILURE);
				}
				break;
			default:
				assert(0);
		}
		labelList = Trees_Right(labelList);
	}
}


/*functions for module productions*/

static void ExportSymbolTable(const char symfilePath[])
{
	const char *tempSymfilePath;

	if (! Files_Exists(".obnc")) {
		Files_CreateDir(".obnc");
	}
	tempSymfilePath = Util_String(".obnc/%s.sym.%d", inputModuleName, getpid());
	Table_Export(tempSymfilePath);
	Files_Move(tempSymfilePath, symfilePath);
}
