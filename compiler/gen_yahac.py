#!/usr/bin/env python3
"""Build yahac.yaha: a Yahalang-to-C compiler written in Yahalang.

Yahalang has no comments, names, or functions, so the compiler is described
here in a tiny structured form and lowered mechanically to Yahalang. The
result is an ordinary Yahalang program; this script is only its "assembler".

Memory model of the generated program
-------------------------------------
The Yahalang stack only exposes its top two values, so the program keeps a
single integer array M on the stack and uses it as flat memory. Between
statements the stack top is always M.

* An expression is lowered to code that consumes M and leaves its value:
  [.., M] -> [.., v]  ("cons"), or keeps M below it: [.., M] -> [.., M, v]
  ("keep" = ppuru + cons). Binary a OP b is keep(a), swap, cons(b), OP.
* M[i] = v is: ppuru ppuru keep(i) swap cons(v) "=", leaving [.., M].
* M grows with "yada ppuru" (append), which keeps the array reference.

Usage: gen_yahac.py [--lang ko|en|ja] [-o yahac.yaha]
"""

import argparse
import pathlib

HERE = pathlib.Path(__file__).resolve().parent

UTTERANCES = {
    "hayaha": ("hayaha", "하야하", "ハヤハ"),
    "haa": ("haa", "하아", "ハア"),
    "ura": ("ura", "우라", "ウラ"),
    "yaha": ("yaha", "야하", "ヤハ"),
    "ppuru": ("ppuru", "뿌루", "プル"),
    "ppururu": ("ppururu", "뿌루루", "プルル"),
    "iiiiyaha": ("iiiiyaha", "이이이야하", "イイイヤハ"),
    "urara": ("urara", "우라라", "ウララ"),
    "iya": ("iya", "이야", "イヤ"),
    "yada": ("yada", "야다", "ヤダ"),
    "hat": ("hat", "핫", "ハッ"),
    "yat": ("yat", "얏", "ヤッ"),
}
LANGS = {"en": 0, "ko": 1, "ja": 2}

# Token kinds used by the compiled lexer and parser. EOF is 0.
KIND = {
    "hayaha": 1, "haa": 2, "ura": 3, "yaha": 4, "ppuru": 5, "ppururu": 6,
    "iiiiyaha": 7, "urara": 8, "iya": 9, "yada": 10, "hat": 11, "yat": 12,
}
SYMBOLS = {";": 13, "[": 14, "]": 15, ",": 16, "@": 17, "=": 18, "#": 19}
SEMI, LBR, RBR, COMMA = 13, 14, 15, 16

# ---------------------------------------------------------------------------
# Constants: cheapest token sequence that pushes n (dynamic programming).

_const_cache = {}


def _const_table(limit):
    cost = [n + 2 for n in range(limit + 1)]
    how = [("direct", n) for n in range(limit + 1)]
    for n in range(2, limit + 1):
        if n % 2 == 0 and cost[n // 2] + 2 < cost[n]:
            cost[n], how[n] = cost[n // 2] + 2, ("double", n // 2)
        root = int(n ** 0.5)
        if root * root == n and root > 1 and cost[root] + 2 < cost[n]:
            cost[n], how[n] = cost[root] + 2, ("square", root)
        for a in range(2, min(40, int(n ** 0.5)) + 1):
            if n % a == 0 and cost[a] + cost[n // a] + 1 < cost[n]:
                cost[n], how[n] = cost[a] + cost[n // a] + 1, ("mul", a, n // a)
        for r in range(1, 13):
            if r < n and cost[n - r] + r + 3 < cost[n]:
                cost[n], how[n] = cost[n - r] + r + 3, ("add", n - r, r)
    return how


_how = _const_table(70000)


def const(n):
    if n < 0:
        return const(-n) + ["ura", "yat", ";"]
    if n in _const_cache:
        return _const_cache[n]
    h = _how[n]
    if h[0] == "direct":
        out = [("num", n)]
    elif h[0] == "double":
        out = const(h[1]) + ["ppuru", "yaha"]
    elif h[0] == "square":
        out = const(h[1]) + ["ppuru", "hat"]
    elif h[0] == "mul":
        out = const(h[1]) + const(h[2]) + ["hat"]
    else:
        out = const(h[1]) + [("num", h[2]), "yaha"]
    _const_cache[n] = out
    return out


# ---------------------------------------------------------------------------
# Expressions

OPS = {
    "+": ["yaha"], "-": ["ura", "yaha", ";"], "*": ["hat"],
    "/": ["ura", "hat", ";"], "%": ["ura", "hat", "hat", ";"],
    "==": ["haa", "hat"], "<": ["haa", "iya"], ">": ["haa", "yaha"],
}


class X:
    def __init__(self, node):
        self.node = node

    def _b(self, op, other, swap=False):
        a, b = (lift(other), self.node) if swap else (self.node, lift(other))
        return X(("b", op, a, b))

    def __add__(self, o): return self._b("+", o)
    def __radd__(self, o): return self._b("+", o, True)
    def __sub__(self, o): return self._b("-", o)
    def __rsub__(self, o): return self._b("-", o, True)
    def __mul__(self, o): return self._b("*", o)
    def __rmul__(self, o): return self._b("*", o, True)
    def __floordiv__(self, o): return self._b("/", o)
    def __mod__(self, o): return self._b("%", o)
    def eq(self, o): return self._b("==", o)
    def lt(self, o): return self._b("<", o)
    def gt(self, o): return self._b(">", o)
    def ne(self, o): return self.eq(o).eq(0)
    def le(self, o): return self.gt(o).eq(0)
    def ge(self, o): return self.lt(o).eq(0)


def lift(v):
    if isinstance(v, X):
        return v.node
    return ("c", v)


def mem(e):
    return X(("m", lift(e)))


READ_CHAR = X(("rc",))
LENGTH = X(("len",))


def cons(n):
    kind = n[0]
    if kind == "c":
        return ["ura", "ppuru"] + const(n[1])
    if kind == "m":
        return keep(n[1]) + ["@"]
    if kind == "rc":
        return ["ura", "ppuru", "urara", "iiiiyaha"]
    if kind == "len":
        return ["yada", "ppururu"]
    _, op, a, b = n
    if b[0] == "c":
        return cons(a) + const(b[1]) + OPS[op]
    return keep(a) + ["iya", "ppuru"] + cons(b) + OPS[op]


def keep(n):
    if n[0] == "c":
        return const(n[1])
    if n[0] == "rc":
        return ["urara", "iiiiyaha"]
    return ["ppuru"] + cons(n)


# ---------------------------------------------------------------------------
# Statements: each is a list of tokens taking [.., M] to [.., M].
# "\n" tokens only affect layout.


def block(body):
    out = []
    for stmt in body:
        out += stmt
    return out


def store(addr, value):
    a, v = lift(addr), lift(value)
    if v[0] == "c":
        return ["ppuru"] + keep(a) + const(v[1]) + ["=", "\n"]
    return ["ppuru", "ppuru"] + keep(a) + ["iya", "ppuru"] + cons(v) + ["=", "\n"]


def append(value):
    return keep(lift(value)) + ["yada", "ppuru", "\n"]


def putc(value):
    return keep(lift(value)) + ["iiiiyaha", "urara", "\n"]


def if_(cond, *body):
    return keep(lift(cond)) + ["iya", "yaha", "yaha", "\n"] + block(body) + ["ura", "yaha", "yaha", "\n"]


def if_else(cond, then, otherwise):
    return (keep(lift(cond)) + const(0) + ["haa", "hat", "ppuru"] + const(0) + ["haa", "hat"]
            + ["iya", "yaha", "yaha", "iya", "ppuru", "\n"] + block(then)
            + ["iya", "ppuru", "ura", "yaha", "yaha", "\n", "iya", "yaha", "yaha", "\n"]
            + block(otherwise) + ["ura", "yaha", "yaha", "\n"])


def while_(cond, *body):
    c = keep(lift(cond))
    return (c + ["iya", "yada", "yada", "ura", "ppuru", "\n"] + block(body)
            + c + ["ura", "yada", "yada", "ura", "ppuru", "\n"])


def fail():
    """Stop with a runtime error (division by zero) and no output."""
    return const(1) + const(0) + ["ura", "hat", ";", "\n"]


# ---------------------------------------------------------------------------
# Program layout

NVARS = 40
_vars = {}


def var(name):
    if name not in _vars:
        _vars[name] = len(_vars)
        assert len(_vars) <= NVARS
    return mem(_vars[name])


def let(name, value):
    var(name)
    return store(_vars[name], value)


def inc(name, by=1):
    return let(name, var(name) + by)


ENTRY = 12  # kind, lang, length, up to 8 code points, padding
TAB = NVARS
TABLE = []
for lang in range(3):
    for name, spellings in UTTERANCES.items():
        word = [ord(ch) for ch in spellings[lang]]
        TABLE += ([KIND[name], lang, len(word)] + word + [0] * ENTRY)[:ENTRY]
BLK = TAB + len(TABLE)
MAX_DEPTH = 128
SRC = BLK + MAX_DEPTH + 2


def emit(text):
    return block([append(ord(ch)) for ch in text])


def emit_number(name):
    """Append the decimal digits of a nonnegative variable to the output."""
    n = var(name)
    return block([
        let("pw", 1),
        while_((n // var("pw")).gt(9), let("pw", var("pw") * 10)),
        while_(var("pw"),
               append(n // var("pw") % 10 + 48),
               let("pw", var("pw") // 10)),
    ])


def read_number():
    """At t just after hayaha: count ppuru tokens into num and expect yaha."""
    t = var("t")
    return block([
        let("num", 0),
        while_(mem(t).eq(KIND["ppuru"]), inc("num"), inc("t")),
        if_(mem(t).ne(KIND["yaha"]), fail()),
        inc("t"),
    ])


def choose(selector, cases):
    """Emit the C text of the case whose kind equals mem[t]; fail otherwise."""
    stmts = [let("f", 0)]
    for kind, text in cases:
        stmts.append(if_(selector.eq(kind), emit(text), let("f", 1)))
    stmts.append(if_(var("f").eq(0), fail()))
    stmts.append(inc("t"))
    return block(stmts)


def program():
    pos, ch, t, k, k1, k2 = (var(n) for n in ("pos", "ch", "t", "k", "k1", "k2"))
    K = KIND
    is_block_marker = k1.eq(k2) * (k1.eq(K["yada"]) + k1.eq(K["yaha"]))

    lex_word = block([
        let("blen", 0),
        let("e", 0), let("eend", 36),
        if_(var("lang").ge(0), let("e", var("lang") * 12), let("eend", var("e") + 12)),
        while_(var("e").lt(var("eend")),
               let("base", var("e") * ENTRY + TAB),
               let("len", mem(var("base") + 2)),
               if_(var("len").gt(var("blen")),
                   let("j", 0), let("ok", 1),
                   while_(var("ok") * var("j").lt(var("len")),
                          let("ok", mem(pos + var("j")).eq(mem(var("base") + 3 + var("j")))),
                          inc("j")),
                   if_(var("ok"),
                       let("best", mem(var("base"))),
                       let("blen", var("len")),
                       let("blang", mem(var("base") + 1)))),
               inc("e")),
        if_(var("blen").eq(0), fail()),
        let("lang", var("blang")),
        append(var("best")),
        let("pos", pos + var("blen")),
    ])

    lexer = block([
        let("pos", SRC), let("lang", -1),
        while_(pos.lt(var("end")),
               let("ch", mem(pos)), let("h", 0),
               if_(ch.eq(32) + ch.eq(9) + ch.eq(10), inc("pos"), let("h", 1)),
               if_(ch.eq(13),
                   if_(mem(pos + 1).ne(10), fail()),
                   inc("pos", 2), let("h", 1)),
               *[if_(ch.eq(ord(sym)), append(code), inc("pos"), let("h", 1))
                 for sym, code in SYMBOLS.items()],
               if_(var("h").eq(0), lex_word)),
    ])

    instruction = block([
        inc("t"),
        let("cur", mem(t)),
        if_(k.eq(K["hayaha"]), read_number(), emit("P("), emit_number("num"), emit(");\n")),
        if_(k.eq(K["yaha"]), emit("A();\n")),
        if_(k.eq(K["hat"]), emit("M();\n")),
        if_(k.eq(K["ppuru"]), emit("U();\n")),
        if_(k.eq(K["iya"]), inc("t"), emit("W();\n")),
        if_(k.eq(K["yat"]), emit("H();\n")),
        if_(k.eq(K["yada"]), choose(var("cur"), [
            (K["hayaha"], "B();\n"), (K["ppururu"], "n();\n"), (K["iya"], "V();\n"),
            (K["hat"], "Y();\n"), (K["ppuru"], "a();\n"), (K["ura"], "p();\n"),
            (K["urara"], "c();\n"), (K["iiiiyaha"], "Z();\n")])),
        if_(k.eq(K["urara"]), choose(var("cur"), [
            (K["iiiiyaha"], "K();\n"), (K["yada"], "T();\n")])),
        if_(k.eq(K["haa"]), choose(var("cur"), [
            (K["yaha"], "G();\n"), (K["iya"], "l();\n"), (K["hat"], "E();\n")])),
        if_(k.eq(K["ura"]),
            if_(var("cur").eq(K["hat"]) * mem(t + 1).eq(K["hat"]), inc("t"), let("cur", -7), emit("R();\n")),
            choose(var("cur"), [
                (K["yaha"], "S();\n"), (K["hat"], "D();\n"), (K["yat"], "N();\n"),
                (K["ppuru"], "X();\n"), (-7, "")])),
        if_(k.eq(K["iiiiyaha"]),
            let("f", 0),
            if_(var("cur").eq(K["ppururu"]), inc("t"), emit("O();\n"), let("f", 1)),
            if_(var("cur").eq(K["urara"]), inc("t"), emit("Q();\n"), let("f", 1)),
            if_(var("f").eq(0), emit("I();\n"))),
        if_(k.eq(LBR), if_else(var("cur").eq(RBR),
            [inc("t"), emit("B();\n")],
            [emit("{static const Yi q[]={"),
             let("f", 1),
             while_(var("f"),
                    if_(mem(t).ne(K["hayaha"]), fail()),
                    inc("t"),
                    read_number(),
                    emit_number("num"),
                    let("f", mem(t).eq(COMMA)),
                    if_(var("f"), emit(","), inc("t"))),
             if_(mem(t).ne(RBR), fail()),
             inc("t"),
             emit("};J(q,sizeof q/sizeof*q);}\n")])),
        if_(k.eq(SYMBOLS["@"]), emit("V();\n")),
        if_(k.eq(SYMBOLS["="]), emit("Y();\n")),
        if_(k.eq(SYMBOLS["#"]), emit("n();\n")),
        if_(k.eq(K["ppururu"]) + k.eq(RBR) + k.eq(COMMA), fail()),
    ])

    parser = block([
        let("t", var("tok")), let("sp", 0), let("loops", 0),
        while_(mem(t),
               let("k", mem(t)), let("k1", mem(t + 1)), let("k2", mem(t + 2)), let("h", 0),
               if_(k.eq(SEMI), inc("t"), let("h", 1)),
               if_(var("h").eq(0) * k.eq(K["ura"]) * is_block_marker,
                   if_(var("sp").eq(0), fail()),
                   if_(mem(var("sp") + (BLK - 1)).ne(k1), fail()),
                   inc("sp", -1),
                   if_(k1.eq(K["yada"]), inc("loops", -1)),
                   emit("}\n"), inc("t", 3), let("h", 1)),
               if_(var("h").eq(0) * k.eq(K["ura"]) * k1.eq(K["iya"]),
                   if_(var("loops").eq(0), fail()),
                   emit("break;\n"), inc("t", 2), let("h", 1)),
               if_(var("h").eq(0) * k.eq(K["iya"]) * k1.ne(K["ppuru"]),
                   if_(var("sp").ge(MAX_DEPTH), fail()),
                   if_(is_block_marker.eq(0), fail()),
                   store(var("sp") + BLK, k1),
                   inc("sp"),
                   if_(k1.eq(K["yada"]), inc("loops"), emit("while(C()){\n")),
                   if_(k1.eq(K["yaha"]), emit("if(F()){\n")),
                   inc("t", 3), let("h", 1)),
               if_(var("h").eq(0), instruction)),
        if_(var("sp"), fail()),
    ])

    runtime = (HERE / "runtime.c").read_text()
    epilogue = "H();return 0;}\n"

    output = block([
        *[putc(ord(c)) for c in runtime],
        let("i", var("out")),
        while_(var("i").lt(LENGTH), putc(mem(var("i"))), inc("i")),
        *[putc(ord(c)) for c in epilogue],
    ])

    # Allocate all variables before computing the layout check.
    body = block([
        let("ch", READ_CHAR),
        while_(ch + 1, append(ch), let("ch", READ_CHAR)),
        let("end", LENGTH),
        *[append(-1) for _ in range(16)],
        let("tok", LENGTH),
        lexer,
        *[append(0) for _ in range(4)],
        let("out", LENGTH),
        parser,
        output,
        ["yat", "\n"],
    ])

    # Prologue: create M with NVARS zero slots, the spelling table, and the
    # block stack, using only the raw stack.
    def zeros(count):
        return (const(count) + ["iya", "yada", "yada", "\n", "iya", "ppuru"] + const(0)
                + ["yada", "ppuru", "iya", "ppuru"] + const(1)
                + ["ura", "yaha", ";", "\n", "ura", "yada", "yada", "ura", "ppuru", "\n"])

    prologue = ["yada", "hayaha", "\n"] + zeros(NVARS)
    prologue += block([const(v) + ["yada", "ppuru", "\n"] for v in TABLE])
    prologue += zeros(SRC - BLK)
    return prologue + body


def render(tokens, lang):
    index = LANGS[lang]
    lines, line = [], []
    for tok in tokens:
        if tok == "\n":
            if line:
                lines.append(" ".join(line))
            line = []
        elif isinstance(tok, tuple):
            ppuru = UTTERANCES["ppuru"][index]
            line.append(" ".join(x for x in (UTTERANCES["hayaha"][index], ppuru * tok[1],
                                             UTTERANCES["yaha"][index]) if x))
        elif tok in UTTERANCES:
            line.append(UTTERANCES[tok][index])
        else:
            line.append(tok)
    if line:
        lines.append(" ".join(line))
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--lang", choices=LANGS, default="ko")
    parser.add_argument("-o", "--output", default=str(HERE / "yahac.yaha"))
    args = parser.parse_args()
    text = render(program(), args.lang)
    pathlib.Path(args.output).write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
