"""Conservative structural quality checks for educational Python programs.

Only report findings supported by the AST. We deliberately do not guess that
unfamiliar algorithms are slower or worse than one particular reference.
"""
from __future__ import annotations

import ast
from dataclasses import dataclass


@dataclass(frozen=True)
class Finding:
    code: str
    line: int
    message: str
    penalty: int


class QualityVisitor(ast.NodeVisitor):
    """Detect provably unreachable/redundant code without executing the source."""

    def __init__(self, protected_lines: int = 0):
        self.protected_lines = protected_lines
        self.findings: list[Finding] = []
        self._reported: set[tuple[str, int]] = set()

    def report(self, code: str, node: ast.AST, message: str, penalty: int) -> None:
        line = getattr(node, 'lineno', 0)
        if line <= self.protected_lines or (code, line) in self._reported:
            return
        self._reported.add((code, line))
        self.findings.append(Finding(code, line, message, penalty))

    @staticmethod
    def literal_boolean(expr: ast.expr) -> bool | None:
        # Only literal constants are safe to resolve statically. Do not use
        # eval(), and do not assume variables keep constant values.
        if isinstance(expr, ast.Constant) and type(expr.value) in (bool, int, float, str, type(None)):
            return bool(expr.value)
        if isinstance(expr, ast.Compare) and len(expr.ops) == 1 and len(expr.comparators) == 1:
            # A comparison between immutable literals can be proven constant.
            # Only equality and order of same-type numeric literals are handled;
            # never call user-defined __eq__ or __lt__ during inspection.
            left, right = expr.left, expr.comparators[0]
            if isinstance(left, ast.Constant) and isinstance(right, ast.Constant):
                a, b = left.value, right.value
                if type(a) in (int, float, str, bool, type(None)) and type(a) is type(b):
                    operator = expr.ops[0]
                    if isinstance(operator, ast.Eq):
                        return a == b
                    if isinstance(operator, ast.NotEq):
                        return a != b
                    if type(a) in (int, float, str):
                        if isinstance(operator, ast.Lt):
                            return a < b
                        if isinstance(operator, ast.Gt):
                            return a > b
                        if isinstance(operator, ast.LtE):
                            return a <= b
                        if isinstance(operator, ast.GtE):
                            return a >= b
        if isinstance(expr, ast.UnaryOp) and isinstance(expr.op, ast.Not):
            value = QualityVisitor.literal_boolean(expr.operand)
            return not value if value is not None else None
        return None

    @staticmethod
    def is_terminator(stmt: ast.stmt) -> bool:
        return isinstance(stmt, (ast.Return, ast.Raise, ast.Break, ast.Continue))

    def scan_block(self, body: list[ast.stmt], *, docstring: bool = False) -> None:
        dead = False
        for index, stmt in enumerate(body):
            if dead:
                self.report('UNREACHABLE', stmt, 'Недостижимая инструкция после выхода из блока', 12)
                # Don't recurse into already-dead code: otherwise a single
                # unreachable subtree would be penalized repeatedly.
                continue
            if isinstance(stmt, ast.Expr) and isinstance(stmt.value, ast.Constant):
                if not (docstring and index == 0 and isinstance(stmt.value.value, str)):
                    self.report('NO_EFFECT', stmt, 'Выражение не имеет эффекта', 7)
            self.visit(stmt)
            dead = self.is_terminator(stmt)

    def visit_Module(self, node: ast.Module) -> None:
        self.scan_block(node.body, docstring=True)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self.scan_block(node.body, docstring=True)
        for expr in node.decorator_list + node.args.defaults + [x for x in node.args.kw_defaults if x]:
            self.visit(expr)

    visit_AsyncFunctionDef = visit_FunctionDef

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        self.scan_block(node.body, docstring=True)

    def visit_If(self, node: ast.If) -> None:
        constant = self.literal_boolean(node.test)
        if constant is False:
            self.report('DEAD_BRANCH', node, 'Условие всегда False: ветка не выполнится', 18)
            # Report size, but don't duplicate nested findings within dead block.
            for stmt in node.body[1:]:
                self.report('UNREACHABLE', stmt, 'Код в заведомо недостижимой ветке', 10)
            self.scan_block(node.orelse)
        elif constant is True:
            self.report('CONSTANT_CONDITION', node, 'Условие всегда True: проверка избыточна', 8)
            for stmt in node.orelse:
                self.report('UNREACHABLE', stmt, 'Недостижимая ветка else', 10)
            self.scan_block(node.body)
        else:
            self.visit(node.test)
            self.scan_block(node.body)
            self.scan_block(node.orelse)

    def visit_While(self, node: ast.While) -> None:
        # while True + break is a normal control-flow idiom, not dead code.
        if self.literal_boolean(node.test) is False:
            self.report('DEAD_LOOP', node, 'Цикл while False никогда не выполнится', 18)
            for stmt in node.body[1:]:
                self.report('UNREACHABLE', stmt, 'Тело заведомо недостижимого цикла', 10)
        else:
            self.visit(node.test)
            self.scan_block(node.body)
        self.scan_block(node.orelse)

    def visit_For(self, node: ast.For) -> None:
        self.visit(node.iter)
        self.scan_block(node.body)
        self.scan_block(node.orelse)

    visit_AsyncFor = visit_For

    def visit_With(self, node: ast.With) -> None:
        self.scan_block(node.body)

    visit_AsyncWith = visit_With

    def visit_Try(self, node: ast.Try) -> None:
        self.scan_block(node.body)
        for handler in node.handlers:
            self.scan_block(handler.body)
        self.scan_block(node.orelse)
        self.scan_block(node.finalbody)

    visit_TryStar = visit_Try

    def visit_Match(self, node: ast.Match) -> None:
        for case in node.cases:
            self.scan_block(case.body)


def inspect(source: str, starter_source: str = '') -> list[Finding]:
    """Return explicit findings; never inspect game-authored complete prefix lines."""
    protected = starter_source.count('\n') if starter_source and source.startswith(starter_source) else 0
    visitor = QualityVisitor(protected)
    tree = ast.parse(source)
    visitor.visit(tree)
    return visitor.findings + _unused_straight_line_assignments(tree, protected)


def _pure_expression(expr: ast.AST) -> bool:
    """Conservative: absence of calls, attribute access and mutation is required."""
    allowed = (ast.Constant, ast.Name, ast.BinOp, ast.UnaryOp, ast.BoolOp,
               ast.Compare, ast.IfExp, ast.Tuple, ast.List, ast.Set, ast.Dict,
               ast.Load, ast.Add, ast.Sub, ast.Mult, ast.Div, ast.FloorDiv,
               ast.Mod, ast.Pow, ast.BitAnd, ast.BitOr, ast.BitXor,
               ast.LShift, ast.RShift, ast.USub, ast.UAdd, ast.Not,
               ast.And, ast.Or, ast.Eq, ast.NotEq, ast.Lt, ast.Gt,
               ast.LtE, ast.GtE, ast.Is, ast.IsNot, ast.In, ast.NotIn)
    return all(isinstance(node, allowed) for node in ast.walk(expr))


def _reads(expr: ast.AST) -> set[str]:
    return {node.id for node in ast.walk(expr)
            if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Load)}


def _unused_straight_line_assignments(tree: ast.Module, protected: int) -> list[Finding]:
    """Backward liveness for simple top-level assignments.

    Other statements act as observable barriers. Never mark input(), calls,
    subscripts, attribute assignments, or assignments in control-flow as dead.
    Dynamic namespace inspection also disables this optimization.
    """
    if any(isinstance(n, (ast.Global, ast.Nonlocal)) or
           (isinstance(n, ast.Call) and isinstance(n.func, ast.Name)
            and n.func.id in ('eval', 'exec', 'globals', 'locals', 'vars'))
           for n in ast.walk(tree)):
        return []
    live: set[str] = set()
    result: list[Finding] = []
    for stmt in reversed(tree.body):
        target = None
        value = None
        if (isinstance(stmt, ast.Assign) and len(stmt.targets) == 1
                and isinstance(stmt.targets[0], ast.Name)):
            target, value = stmt.targets[0], stmt.value
        elif isinstance(stmt, ast.AnnAssign) and isinstance(stmt.target, ast.Name):
            target, value = stmt.target, stmt.value
        if target is not None:
            if value is None:
                continue
            if _pure_expression(value):
                if target.id not in live:
                    if stmt.lineno > protected:
                        result.append(Finding('UNUSED_CALCULATION', stmt.lineno,
                                              f'Значение {target.id} вычисляется, но не влияет на результат', 10))
                    # A dead assignment does not make its dependencies live.
                    continue
                live.discard(target.id)
                live.update(_reads(value))
                continue
        # A nontrivial statement may observe values through code we do not
        # understand. Treat all reads as live and avoid speculative penalties.
        live.update(_reads(stmt))
    return result


def numeric_answer_hardcoded(source: str) -> bool:
    """Prove a *narrow* case: the program only prints a numeric literal or
    a name bound to such a literal. Do not reject arithmetic shortcuts.
    This is a ranking hint, not an invalidation of a correct result.
    """
    tree = ast.parse(source)
    known: dict[str, int | float] = {}
    outputs = 0
    for stmt in tree.body:
        if (isinstance(stmt, ast.Assign) and len(stmt.targets) == 1 and
            isinstance(stmt.targets[0], ast.Name) and isinstance(stmt.value, ast.Constant)
            and type(stmt.value.value) in (int, float)):
            known[stmt.targets[0].id] = stmt.value.value
            continue
        if (isinstance(stmt, ast.Expr) and isinstance(stmt.value, ast.Call) and
            isinstance(stmt.value.func, ast.Name) and stmt.value.func.id == 'print' and
            len(stmt.value.args) == 1 and not stmt.value.keywords):
            arg = stmt.value.args[0]
            if isinstance(arg, ast.Constant) and type(arg.value) in (int, float):
                outputs += 1
                continue
            if isinstance(arg, ast.Name) and arg.id in known:
                outputs += 1
                continue
        return False
    return outputs == 1


