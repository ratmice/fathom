foo = item true;

bar = bool_elim item foo { item true, item false } : item Bool;

baz = item bar;