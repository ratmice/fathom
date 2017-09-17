use lalrpop_util;

use ast::{Definition, Expr, Type};
use env::Env;
use source::BytePos;

mod lexer;
#[allow(unused_extern_crates)]
mod grammar;

use self::lexer::{Lexer, Error as LexerError, Token};

pub type ParseError<'input> = lalrpop_util::ParseError<BytePos, Token<'input>, LexerError>;

// pub enum ParseError<L, T, E> {
//     InvalidToken {
//         location: L,
//     },
//     UnrecognizedToken {
//         token: Option<(L, T, L)>,
//         expected: Vec<String>,
//     },
//     ExtraToken {
//         token: (L, T, L),
//     },
//     User {
//         error: E,
//     },
// }

pub fn parse<'input, 'env>(
    env: &'env Env,
    src: &'input str,
) -> Result<Vec<Definition>, ParseError<'input>> {
    grammar::parse_Definitions(env, Lexer::new(src))
}

pub fn parse_expr<'input, 'env>(
    env: &'env Env,
    src: &'input str,
) -> Result<Expr, ParseError<'input>> {
    grammar::parse_Expr(env, Lexer::new(src))
}

pub fn parse_ty<'input, 'env>(
    env: &'env Env,
    src: &'input str,
) -> Result<Type, ParseError<'input>> {
    grammar::parse_Type(env, Lexer::new(src))
}

#[cfg(test)]
mod tests {
    use env::Env;
    use super::*;

    #[test]
    fn parse_ty_var() {
        let src = "
            Point
        ";

        assert_snapshot!(parse_ty_var, parse_ty(&Env::default(), src));
    }

    #[test]
    fn parse_ty_empty_struct() {
        let src = "struct {}";

        assert_snapshot!(parse_ty_empty_struct, parse_ty(&Env::default(), src));
    }

    #[test]
    fn parse_simple_definition() {
        let src = "
            Offset32 = u32;
        ";

        assert_snapshot!(parse_simple_definition, parse(&Env::default(), src));
    }

    #[test]
    fn parse_definition() {
        let src = "
            Point = struct {
                x : u32be,
                y : u32be,
            };

            Array = struct {
                len : u16le,
                data : [Point; len],
            };

            Formats = union {
                struct { format : u16, data: u16 },
                struct { format : u16, point: Point },
                struct { format : u16, array: Array },
            };
        ";

        assert_snapshot!(parse_definition, parse(&Env::default(), src));
    }
}
