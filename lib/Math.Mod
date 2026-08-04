MODULE Math;

IMPORT OberonRuntime;

CONST
  pi* = 3.14159265358979;
  e* = 2.71828182845905;

PROCEDURE sqrt*(x: REAL): REAL;
RETURN OberonRuntime.MathSqrt(x)
END sqrt;

PROCEDURE power*(base, exp: REAL): REAL;
RETURN OberonRuntime.MathPower(base, exp)
END power;

PROCEDURE exp*(x: REAL): REAL;
RETURN OberonRuntime.MathExp(x)
END exp;

PROCEDURE ln*(x: REAL): REAL;
RETURN OberonRuntime.MathLn(x)
END ln;

PROCEDURE log*(x, base: REAL): REAL;
RETURN OberonRuntime.MathLog(x, base)
END log;

PROCEDURE round*(x: REAL): REAL;
RETURN OberonRuntime.MathRound(x)
END round;

PROCEDURE sin*(x: REAL): REAL;
RETURN OberonRuntime.MathSin(x)
END sin;

PROCEDURE cos*(x: REAL): REAL;
RETURN OberonRuntime.MathCos(x)
END cos;

PROCEDURE tan*(x: REAL): REAL;
RETURN OberonRuntime.MathTan(x)
END tan;

PROCEDURE arcsin*(x: REAL): REAL;
RETURN OberonRuntime.MathArcsin(x)
END arcsin;

PROCEDURE arccos*(x: REAL): REAL;
RETURN OberonRuntime.MathArccos(x)
END arccos;

PROCEDURE arctan*(x: REAL): REAL;
RETURN OberonRuntime.MathArctan(x)
END arctan;

PROCEDURE arctan2*(y, x: REAL): REAL;
RETURN OberonRuntime.MathArctan2(y, x)
END arctan2;

PROCEDURE sinh*(x: REAL): REAL;
RETURN OberonRuntime.MathSinh(x)
END sinh;

PROCEDURE cosh*(x: REAL): REAL;
RETURN OberonRuntime.MathCosh(x)
END cosh;

PROCEDURE tanh*(x: REAL): REAL;
RETURN OberonRuntime.MathTanh(x)
END tanh;

PROCEDURE arcsinh*(x: REAL): REAL;
RETURN OberonRuntime.MathArcsinh(x)
END arcsinh;

PROCEDURE arccosh*(x: REAL): REAL;
RETURN OberonRuntime.MathArccosh(x)
END arccosh;

PROCEDURE arctanh*(x: REAL): REAL;
RETURN OberonRuntime.MathArctanh(x)
END arctanh;

END Math.
