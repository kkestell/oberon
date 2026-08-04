MODULE LibraryMath;

IMPORT Math, Out;

CONST eps = 0.001;

TYPE Unary = PROCEDURE(x: REAL): REAL;

VAR x, y: REAL; function: Unary;

BEGIN
  ASSERT(ABS(Math.pi - 3.1415927) < eps);
  ASSERT(ABS(Math.e - 2.7182817) < eps);
  function := Math.sqrt; ASSERT(ABS(function(4.0) - 2.0) < eps);
  ASSERT(ABS(Math.power(2.0, 3.0) - 8.0) < eps);
  ASSERT(ABS(Math.exp(1.0) - Math.e) < eps);
  ASSERT(ABS(Math.ln(Math.e) - 1.0) < eps);
  ASSERT(ABS(Math.log(100.0, 10.0) - 2.0) < eps);
  ASSERT(Math.round(0.5) = 1.0);
  ASSERT(Math.round(-0.5) = 0.0);
  ASSERT(ABS(Math.sin(Math.pi / 2.0) - 1.0) < eps);
  ASSERT(ABS(Math.cos(Math.pi) + 1.0) < eps);
  ASSERT(ABS(Math.tan(Math.pi / 4.0) - 1.0) < eps);
  ASSERT(ABS(Math.arcsin(0.5) - Math.pi / 6.0) < eps);
  ASSERT(ABS(Math.arccos(0.5) - Math.pi / 3.0) < eps);
  ASSERT(ABS(Math.arctan(1.0) - Math.pi / 4.0) < eps);
  ASSERT(ABS(Math.arctan2(1.0, -1.0) - 3.0 * Math.pi / 4.0) < eps);
  ASSERT(ABS(Math.sinh(1.0) - 1.1752012) < eps);
  ASSERT(ABS(Math.cosh(1.0) - 1.5430806) < eps);
  ASSERT(ABS(Math.tanh(1.0) - 0.7615942) < eps);
  ASSERT(ABS(Math.arcsinh(1.1752012) - 1.0) < eps);
  ASSERT(ABS(Math.arccosh(1.5430806) - 1.0) < eps);
  ASSERT(ABS(Math.arctanh(0.7615942) - 1.0) < eps);
  x := Math.sqrt(-1.0); ASSERT(x # x);
  y := Math.exp(100.0); ASSERT((y > 0.0) & (y = y));
  x := Math.sin(-0.0); ASSERT((1.0 / x) < 0.0);
  Out.String("ok"); Out.Ln
END LibraryMath.
