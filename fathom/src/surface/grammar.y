%token Type Hole Name Number String true false match overlap
%start Term
%%
Pattern: Name | "_" | String | Number | true | false;
AnnPattern: Pattern | "(" Pattern ":" LetTerm ")";
Term: LetTerm | LetTerm ":" LetTerm;
LetTerm: FunTerm | "let" Pattern MaybeAnnLetTerm "=" Term ";" LetTerm;
MaybeAnnLetTerm: ":" LetTerm | /* Empty */;

FunTerm
: AppTerm 
| AppTerm "->" FunTerm 
| "fun" AnnPattern "->" FunTerm
| "fun" AnnPattern "=>" LetTerm
;

AppTerm: AppTerm AtomicTerm | AtomicTerm;

AtomicTerm:
 "(" Term ")"
| Name
| "_"
| Hole
| match AtomicTerm "{" PatternSeq "}"
| Type
| String
| Number
| true
| false
| "{" "}"
| "{" NonEmptyAnn "}"
| "{" NonEmptyAssign "}"
| "{" NonEmptyBackArrow "}"
| overlap "{" NonEmptyBackArrow "}"
| AtomicTerm "." Name
| "[" TermSeq "]"
;

PatternSeq
: Pattern "=>" Term "," PatternSeq
| Pattern "=>" Term
| /* Empty */ 
;

TermSeq
: Term "," TermSeq
| Term
| /* Empty */
;


NonEmptyAnn: Name ":" Term NonEmptyAnnSeq;
MaybeNonEmptyAnn: Name ":" Term NonEmptyAnnSeq | /* Empty */;
NonEmptyAnnSeq: "," MaybeNonEmptyAnn | /* Empty */;

NonEmptyAssign: Name "=" Term NonEmptyAssignSeq;
MaybeNonEmptyAssign: Name "=" Term NonEmptyAssignSeq | /* Empty */;
NonEmptyAssignSeq: "," MaybeNonEmptyAssign | /* Empty */;

NonEmptyBackArrow: Name "<-" Term NonEmptyBackArrowSeq;
MaybeNonEmptyBackArrow: Name "<-" Term NonEmptyBackArrowSeq | /* Empty */;
NonEmptyBackArrowSeq: "," MaybeNonEmptyBackArrow | /* Empty */;
