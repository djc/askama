#!/usr/bin/env python
# -*- coding: utf-8 -*-

from itertools import product, chain, zip_longest, tee


# The amount of branches to generate
BRANCHES = 2 # branches

IF, ELSE_IF, ELSE, END_IF = 0, 1, 2, 3

NL = "\\n"


def handle_ws(ws, trim=True):
    return [" ", "-" if trim else "+"][ws]


def trim(s, ws):
    if ws[0]:
        s = s.lstrip()
    if ws[1]:
        s = s.rstrip()
    return s


def cond_kind(i, n):
    i += 1
    if i == 1:
        return IF # if
    elif (i == n) and (i > 1):
        return ELSE # else
    elif i > n:
        return END_IF # endif
    else:
        return ELSE_IF # else if


# From: https://docs.python.org/3/library/itertools.html#itertools-recipes
def pairwise(iterable):
    a, b = tee(iterable)
    next(b, None)
    return zip(a, b)


def write_cond(conds, active_branch, sign, invert):
    n = len(conds) - 1
    trim = sign == "-"

    lits = []
    for i in range(1, n + 2 + 1):
        ws1 = "\\n" * i
        ws2 = "\\r\\n" * i
        lits.append((ws1, str(i), ws2))

    conds = list(conds)
    for i, (pws, nws) in enumerate(conds):
        kind = cond_kind(i, n)
        b = str(i == active_branch).lower()
        cond = [f"if {b}", f"else if {b}", "else", "endif"][kind]
        cond = f"{{%{handle_ws(pws, trim)} {cond} {handle_ws(nws, trim)}%}}"
        conds[i] = cond

    it = map("".join, lits)
    it = filter(None, chain.from_iterable(zip_longest(it, conds)))
    code = "".join(it)

    expected = f"{lits[0][0]}{lits[0][1]}"
    for i, (cond, (before, after)) in enumerate(zip(conds, pairwise(lits))):
        kind = cond_kind(i, n)
        pws = cond.startswith(f"{{%{sign}")
        nws = cond.endswith(f"{sign}%}}")

        if sign == "+":
            pws = not pws
            nws = not nws
        elif invert:
            pws = True
            nws = True

        cond = i == active_branch
        prev_cond = i == (active_branch + 1)

        if prev_cond or (kind == IF):
            expected += before[2] * (not pws)
        if cond or (kind == END_IF):
            expected += after[0] * (not nws)
            expected += after[1]

    # FIXME: Askama does not include whitespace before eof
    # expected += lits[-1][2]

    return code, expected


def write_match(contents, arms, match_ws):
    before, expr, after = contents
    code = before

    pws, nws = match_ws[0]
    code += f"{{%{handle_ws(pws)} match {expr} {handle_ws(nws)}%}}"

    for (arm, expr), (pws, nws) in zip(arms, match_ws[1:-1]):
        code += f"{{%{handle_ws(pws)} when {arm} {handle_ws(nws)}%}}{expr}"

    pws, nws = match_ws[-1]
    code += f"{{%{handle_ws(pws)} endmatch {handle_ws(nws)}%}}"
    code += after

    return code


def write_match_result(active_arm, contents, arms, match_ws):
    before, expr, after = contents
    expected = ""

    expected += trim(before, (False, match_ws[0][0]))
    expected += trim(arms[active_arm][1], (match_ws[1:][active_arm][1], match_ws[1:][active_arm+1][0]))
    expected += trim(after, (match_ws[-1][1], False))
    return expected


def write_cond_tests(f, name_extra, config):
    f.write("""
macro_rules! test_template{name_extra} {{
    ($source:literal, $rendered:expr) => {{{{
        #[derive(Template)]
        #[template(source = $source, ext = "txt"{config})]
        struct CondWs;

        assert_eq!(CondWs.render().unwrap(), $rendered);
    }}}};
}}

#[rustfmt::skip]
#[test]
fn test_cond_ws{name_extra}() {{
""".format(name_extra = name_extra, config = config))

    invert = len(name_extra) > 0
    sign = "+" if invert else "-"

    for branches in range(1, BRANCHES + 1):
        for x in product([False, True], repeat=(branches+1)*2):
            # it = iter(x)
            # conds = list(zip(it, it))
            conds = list(zip(x[::2], x[1::2]))

            for i in range(branches):
                code, expected = write_cond(conds, i, sign, invert)
                f.write(f'    test_template{name_extra}!("{code}", "{expected}");\n')
                if invert and ("{%+" in code or "+%}" in code):
                    # We also check that the minus sign is trimming with suppress_whitespace
                    # option enabled.
                    code, expected = write_cond(conds, i, "-", invert)
                    f.write(f'    test_template{name_extra}!("{code}", "{expected}");\n')

        if branches != BRANCHES:
            f.write("\n")
    f.write("}\n")


def write_match_tests(f):
    f.write("""
#[rustfmt::skip]
macro_rules! test_match {
    ($source:literal, $some_rendered:expr, $none_rendered:expr) => {{
        #[derive(Template)]
        #[template(source = $source, ext = "txt")]
        struct MatchWs {
            item: Option<&'static str>,
        }

        assert_eq!(MatchWs { item: Some("foo") }.render().unwrap(), $some_rendered);
        assert_eq!(MatchWs { item: None }.render().unwrap(), $none_rendered);
    }};
}

#[rustfmt::skip]
#[test]
fn test_match_ws() {
""")

    contents = "before ", "item", "      after"
    arms = [("Some with (item)", "  foo   "), ("None", "    bar     ")]

    for x in product([False, True], repeat=len(arms)*2+1):
        x = [False, False, *x, False]
        arms_ws = list(zip(x[::2], x[1::2]))

        code = write_match(contents, arms, arms_ws)
        some_expected = write_match_result(0, contents, arms, arms_ws)
        none_expected = write_match_result(1, contents, arms, arms_ws)

        f.write(f'    test_match!("{code}", "{some_expected}", "{none_expected}");\n')

    f.write("}\n")


if __name__ == "__main__":
    with open("ws.rs", "w") as f:
        f.write("// This file is auto generated by gen_ws_tests.py\n\n")
        f.write("use askama::Template;\n")
        write_cond_tests(f, "", "")
        write_cond_tests(f, "_inverted", ', config = "test_trim.toml"')
        write_match_tests(f)
