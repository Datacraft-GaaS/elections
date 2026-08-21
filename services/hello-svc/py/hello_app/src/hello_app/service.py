from fastapi import FastAPI

app = FastAPI(title="eel-hello-app", version="0.1.0")


@app.get("/")
async def root() -> dict[str, bool]:
    return {"ok": True}
