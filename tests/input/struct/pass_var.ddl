//! Test referring to aliases in struct fields.

struct Pair {
    first: U8,
    second: U8,
}

MyPair = Pair;

struct PairPair {
    first: Pair,
    second: MyPair,
}