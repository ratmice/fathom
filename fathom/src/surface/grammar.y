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

NonEmptyAnn: Name ":" Term | NonEmptyAnn "," MaybeAnn;
MaybeAnn: Name ":" Term | /* Empty */;

NonEmptyAssign: Name "=" Term | NonEmptyAssign "," MaybeAssign;
MaybeAssign: Name "=" Term | /* Empty */;

NonEmptyBackArrow: Name "<-" Term | NonEmptyBackArrow "," MaybeBackArrow;
MaybeBackArrow: Name "<-" Term | /* Empty */;