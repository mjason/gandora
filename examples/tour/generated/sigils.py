import re

word_re = re.compile("\\w+")


def main():
    print(repr(["gandora", "elixir", "python"]))
    print("no need to escape \"quotes\" here")
    print(repr(word_re.findall("hello, 世界 world!")))
    total = (sum(i * i for i in range(10)))
    print(f"sum of squares: {total}")
    evens = (lambda xs: [x for x in xs if x % 2 == 0])([1, 2, 3, 4, 5, 6])
    return print(repr(evens))


if __name__ == "__main__":
    main()
