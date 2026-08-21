FROM python:{{cookiecutter.python_version}}-slim
WORKDIR /app
COPY py/ py/
RUN pip install uv && cd py && uv sync
CMD ["uv", "run", "uvicorn", "{{cookiecutter.svc_name}}_api.main:app", "--host", "0.0.0.0", "--port", "{{cookiecutter.port}}"]
