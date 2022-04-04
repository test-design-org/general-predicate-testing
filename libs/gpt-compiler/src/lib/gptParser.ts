import ohm from 'ohm-js';
import ohmExtras from 'ohm-js/extras';

const gptGrammar = ohm.grammar(String.raw`
Gpt {
  Feature = "[" (VarDecl | IfStmt | Feature)+ "]"

  VarDecl = "var" varName ":" "bool" -- bool
          | "var" varName ":" "int" -- int
          | "var" varName ":" "num" "(" posNumber ")" -- numWithPrec
          | "var" varName ":" "num" -- num
  varName = (letter | "_") (alnum | "_")*

  IfStmt = "if" "(" Conditions ")" ("{" IfStmt* "}")? ElseIf* Else?
  ElseIf = "else" "if" "(" Conditions ")" ("{" IfStmt* "}")?
  Else = "else" "{" IfStmt* "}"

  Conditions = NonemptyListOf<Cond, boolOp>
  Cond = bool eqOp varName -- boolLhs
       | varName eqOp bool -- boolRhs
       | number unaryOp varName -- unaryLhs
       | varName unaryOp number -- unaryRhs
       | varName intervalOp Interval -- interval

  Interval = ("(" | "[") number "," number (")" | "]")

  bool = "true" | "false"
  eqOp = "=" | "!="
  intervalOp = "in" | "not in"
  unaryOp = "<=" |  ">=" | "!=" | "<" | ">" | "="
  boolOp = "&&" | "||"

  posNumber = digit+ ("." digit+)?
  number = posNumber -- pos
         | "-" posNumber -- neg
         | "Inf" -- inf
         | "-" spaces "Inf" -- negInf

  comment = "/*" (~"*/" any)* "*/"
          | "//" (~"\n" any)* "\n"?
  space += comment
}
`);

export const parseGpt = (text: string) => {
  const match = gptGrammar.match(text);
  if (match.succeeded()) {
    return ohmExtras.toAST(match);
  } else {
    throw new Error(match.message);
  }
};
