"""The FastAPI chapter, rewritten in the Elixir style: routes declared
with the Tour.WebDsl macros instead of raw @decorate lines. Serving is
unchanged — the app is still the module attribute `app`:

    uv run uvicorn tour.webapi:app --app-dir dist

`gan run src/tour/webapi.gan` self-tests the app with TestClient.
"""

import builtins
import fastapi
import fastapi.testclient


class GanMatchError(Exception):
    pass

app = fastapi.FastAPI(title="Gandora Tour API")


def asgi():
    return app


@app.get("/")
def root():
    return {"message": "hello from gandora", "framework": "fastapi"}


@app.get("/slug/{text}")
def slug(text):
    return {"input": text, "slug": text.lower().replace(" ", "-")}


@app.get("/fact/{n}")
def fact_route(n):
    return {"n": n, "fact": _fact(builtins.int(n))}


def _fact(*_gan_args):
    match _gan_args:
        case (0,):
            return 1
        case (n,):
            return n * _fact(n - 1)
    raise GanMatchError("no clause of fact/1 matched " + repr(_gan_args))


def main() -> None:
    """Prints what each route returns."""
    client = fastapi.testclient.TestClient(app)
    resp = client.get("/")
    print(f"GET /                 -> {resp.status_code} {resp.json()}")
    resp = client.get("/slug/Hello Gandora World")
    print(f"GET /slug/...         -> {resp.status_code} {resp.json()}")
    resp = client.get("/fact/10")
    print(f"GET /fact/10          -> {resp.status_code} {resp.json()}")
    resp = client.get("/nope")
    return print(f"GET /nope             -> {resp.status_code}")


if __name__ == "__main__":
    main()
