FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY main.py ./
COPY telemetry ./telemetry

RUN useradd --create-home --uid 10001 --shell /usr/sbin/nologin apex \
    && chown -R apex:apex /app

USER apex
ENTRYPOINT ["python", "main.py"]
