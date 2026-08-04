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

#line 165 "y.tab.h"

};
typedef union YYSTYPE YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define YYSTYPE_IS_DECLARED 1
#endif


extern YYSTYPE yylval;


int yyparse (void);


#endif /* !YY_YY_Y_TAB_H_INCLUDED  */
