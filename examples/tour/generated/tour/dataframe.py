"""pandas end to end: a dev dependency from pyproject.toml, used with zero
wrapper code. Needs `uv sync` first, then: gan run src/tour/dataframe.gan
"""

import builtins
import pandas
import pandas as pd


def sales() -> pandas.DataFrame:
    """A demo frame — host types appear in specs as `$pandas.DataFrame()`.

    >>> sales().shape
    (6, 4)
"""
    return pd.DataFrame({"product": ["keyboard", "mouse", "monitor", "cable", "keyboard", "monitor"], "region": ["east", "east", "west", "west", "east", "west"], "units": [12, 30, 7, 45, 8, 5], "price": [100.0, 25.5, 300.0, 9.9, 100.0, 300.0]})


def report(df: pandas.DataFrame) -> pandas.DataFrame:
    """Revenue per product, best first — a pandas fluent chain as a pipeline.

## Parameters

  - df: The sales frame.
"""
    return _summarize(_with_revenue(df))


def _with_revenue(df):
    return df.assign(revenue=(lambda d: d["units"] * d["price"]))


def _summarize(df):
    return df.groupby("product", as_index=False).agg({"units": "sum", "revenue": "sum"}).sort_values("revenue", ascending=False)


def main() -> None:
    """Runs the chapter."""
    df = sales()
    print(f"rows = {builtins.len(df)}, columns = {df.columns.tolist()}")
    top = report(df)
    print("\nrevenue by product:")
    print(top.to_string(index=False))
    print("\nover 500:")
    print(top.query("revenue > 500").to_string(index=False))
    total = top.revenue.sum()
    print(f"\ntotal revenue = {total}")
    east_units = df.query("region == 'east'").units.sum()
    return print(f"east units    = {east_units}")


if __name__ == "__main__":
    main()
