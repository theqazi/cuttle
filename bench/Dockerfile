# Cuttle bench: Phase A in a clean Linux container.
#
# Runs `bench.py` (model-driven SWE+Secure suite) inside a fresh Python
# environment with the deps that aren't in macOS's /usr/bin/python3
# (pyyaml, jinja2). Useful for:
#
#   - Adversarial Phase A iteration without exposing the host machine
#     to model-generated exploit code.
#   - Reproducible bench runs across machines / CI.
#   - Unblocking the deferred yaml_load and jinja_render tasks once
#     they're added.
#
# NOT a replacement for Phase C: cuttle-sandbox is macOS-Seatbelt-
# specific, so Docker on Mac (which runs a Linux VM) cannot exercise
# cuttle's containment claim. Phase C must stay native macOS.
#
# Build:
#     docker build -t cuttle-bench bench/
#
# Run (pass API key as env, don't bake into image):
#     docker run --rm \
#       -e ANTHROPIC_API_KEY \
#       -v "$(pwd)/bench:/bench" \
#       -v "$(pwd)/bench/results:/bench/results" \
#       cuttle-bench \
#       python3 /bench/bench.py --runs 5 --model claude-haiku-4-5-20251001 \
#         --out /bench/results/results.json

FROM python:3.13-slim

# git is required by the command_injection task's setup; ca-certificates
# is required for the anthropic SDK's TLS handshake.
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir \
    anthropic==0.39.0 \
    pyyaml==6.0.2 \
    jinja2==3.1.4

# Default working dir matches the volume mount target; the runtime
# `docker run -v $(pwd)/bench:/bench` populates this with the live
# tree from the host so iterating on bench.py doesn't require a rebuild.
WORKDIR /bench

# Pre-create the results dir so the volume mount doesn't fail if the
# host hasn't created it yet.
RUN mkdir -p /bench/results

# No ENTRYPOINT; pass the bench command at `docker run` time so the
# same image can run different sweeps (--task, --model, --runs).
CMD ["python3", "bench.py", "--help"]
