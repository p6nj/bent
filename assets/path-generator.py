p = lambda x: print(x, end=" ")


def line(coords: tuple[tuple[int]]):
    p(f"M {coords[0][0]} {coords[0][1]} L {coords[1][0]} {coords[1][1]}")


n = 100
line(((0, 0), (n, n)))
line(((n, 0), (0, n)))
for i in range(1, n):
    line(((i, 0), (n - i, n)))
    line(((0, i), (n, n - i)))

print()
