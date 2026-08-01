"""
  FastAPI end to end: the app lives in a module attribute, routes attach
  with @decorate, and the generated module serves with plain uvicorn:

      uv run uvicorn tour.webapi:app --app-dir dist

  `gan run src/tour/webapi.gan` self-tests the app with TestClient.
"""

import builtins
import fastapi
import fastapi.testclient


class GanMatchError(Exception):
    pass

app = fastapi.FastAPI(title="Gandora Tour API")


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


def main():
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
