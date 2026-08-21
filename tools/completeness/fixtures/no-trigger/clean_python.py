# Fixture: clean Python file — should produce zero findings.


def add(a: int, b: int) -> int:
    return a + b


def multiply(a: int, b: int) -> int:
    return a * b


def test_add():
    assert add(2, 3) == 5
