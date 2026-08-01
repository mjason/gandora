"""Runs every chapter of the tour: `gan run src/main.gan`."""

import app.cli
import app.shop
import sigils
import tour.basics
import tour.dsl
import tour.functions
import tour.interop
import tour.patterns
import tour.templates


def main():
    _section("basics", tour.basics.demo)
    _section("patterns", tour.patterns.demo)
    _section("functions", tour.functions.demo)
    _section("interop", tour.interop.demo)
    _section("templates", tour.templates.main)
    _section("metaprogramming dsl", tour.dsl.main)
    _section("macros + modules", app.cli.main)
    _section("structs + attributes", app.shop.main)
    return _section("sigils", sigils.main)


def _section(title, run):
    print(f"== {title} ==")
    run()
    return print("")


if __name__ == "__main__":
    main()
