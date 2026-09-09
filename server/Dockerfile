FROM registry.access.redhat.com/ubi10/ubi-minimal:10.2

COPY --from=ghcr.io/astral-sh/uv:0.12.6 /uv /usr/local/bin/uv
# The PyCOLMAP CUDA wheel links directly to these X11 runtime libraries.
# Git and GCC are required to build commit-pinned Python dependencies.
RUN microdnf install -y --setopt=install_weak_deps=0 gcc-c++ git-core libSM libX11 libXext sed \
    && microdnf clean all
WORKDIR /app
COPY pyproject.toml uv.lock .python-version ./

ARG CUDA_VERSION=12.6
ENV UV_LINK_MODE=copy
# Resolve CUDA 13 from the existing lock, retaining compatible package pins.
RUN --mount=type=cache,target=/root/.cache/uv \
    case "${CUDA_VERSION}" in \
        12.6) ;; \
        13.0) sed -i 's|https://download.pytorch.org/whl/cu126|https://download.pytorch.org/whl/cu130|' pyproject.toml \
            && uv lock ;; \
        *) echo "Unsupported CUDA_VERSION: ${CUDA_VERSION}; expected 12.6 or 13.0" >&2; exit 1 ;; \
    esac \
    && uv sync --locked --no-dev

COPY src ./src
COPY LICENSE THIRDPARTY.md ./

ARG SHRIMPLY_SERVER_GIT_HASH=""
ENV SHRIMPLY_SERVER_GIT_HASH=${SHRIMPLY_SERVER_GIT_HASH}
LABEL org.opencontainers.image.source="https://github.com/soirihiroka/shrimply" \
    org.opencontainers.image.revision=${SHRIMPLY_SERVER_GIT_HASH}

HEALTHCHECK --interval=10s --timeout=3s --retries=3 --start-period=5m \
    CMD ["/app/.venv/bin/python", "-c", "import os, urllib.request; urllib.request.urlopen('http://127.0.0.1:' + os.environ.get('SHRIMPLY_SERVER_PORT', '8787') + '/', timeout=3).close()"]

CMD ["uv", "run", "--locked", "--no-dev", "--exact", "src/main.py"]
