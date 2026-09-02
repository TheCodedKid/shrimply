FROM fedora:44 AS builder
WORKDIR /src

RUN dnf install -y curl git make patchelf rustup && dnf clean all

RUN rustup-init -y --default-toolchain none --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# ffmepg from RPMFusion
RUN dnf install -y \
        "https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm" \
        "https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm" \
    && dnf clean all

COPY Makefile ./
RUN make deps-fedora DNF="dnf -y"


RUN rpm -q gtk4-devel && pkg-config --exists gtk4 \
    && echo "gtk4-devel OK: $(pkg-config --modversion gtk4)"
    && make qt-native-deps

# Add CUDA from Nvidia source
RUN . /etc/os-release && \
    curl -fsSL -o /etc/yum.repos.d/cuda-fedora.repo \
        "https://developer.download.nvidia.com/compute/cuda/repos/fedora${VERSION_ID}/x86_64/cuda-fedora${VERSION_ID}.repo" && \
    dnf install -y cuda-toolkit && \
    dnf clean all

COPY . .


RUN rustup show

# Git submodules might be missing or outdated before the build. Happened on Arch, not sure if it happens on Fedora
RUN test -e external/cuda-oxide/crates/cargo-oxide/Cargo.toml || { \
        echo "Git submodules are missing from the build context." >&2; \
        echo "Run 'git submodule update --init --recursive' before 'docker build'." >&2; \
        exit 1; \
    }

# This is something that happens on fresh installs of CUDA-Oxide due to cargo trying to build it from Shrimply's workspace (where there is no library target)
# We build directly againts the submodule that is pinned with the nightly version
RUN rustup run nightly-2026-04-03 cargo install --path external/cuda-oxide/crates/cargo-oxide --locked
RUN cd external/cuda-oxide/crates/rustc-codegen-cuda && \
    SYSROOT="$(rustup run nightly-2026-04-03 rustc --print sysroot)" && \
    LIBRARY_PATH="$SYSROOT/lib" LD_LIBRARY_PATH="$SYSROOT/lib" \
        rustup run nightly-2026-04-03 cargo build --lib --target host-tuple --target-dir target

ENV CUDA_OXIDE_BACKEND=/src/external/cuda-oxide/crates/rustc-codegen-cuda/target/x86_64-unknown-linux-gnu/debug/librustc_codegen_cuda.so
ENV CUDA_HOME=/usr/local/cuda
ENV CUDA_TOOLKIT_PATH=/usr/local/cuda

# We have to do this to compile since Docker doesnt have a nvidia driver. 
RUN ln -sf libcuda.so /usr/local/cuda/lib64/stubs/libcuda.so.1
ENV LD_LIBRARY_PATH=/usr/local/cuda/lib64/stubs:${LD_LIBRARY_PATH}

RUN make release qt-release
RUN make install install-qt DESTDIR=/stage PREFIX=/usr

# --- Bundle the runtime libraries
RUN mkdir -p /stage/usr/lib/shrimply && \
    for bin in shrimply shrimply-editor shrimply-mcp shrimply-qt shrimply-editor-qt; do \
        ldd "/stage/usr/bin/$bin" | awk '{print $3}' | grep '^/'; \
    done | sort -u \
      | grep -Ev '/(ld-linux[^/]*|libc|libm|libdl|libpthread|librt|libresolv)\.so' \
      | grep -Ev '/lib(GL|EGL|GLX|GLdispatch|drm|gbm)\.so' \
      | grep -Ev '/libcuda\.so|/libnvidia-' \
      | xargs -r -I{} cp -Ln --remove-destination {} /stage/usr/lib/shrimply/

# Fixes issue shrimply-editor.bin: error while loading shared libraries: libxml2.so.2: cannot open shared object file: No such file or directory
# Adds the dependencies of dependencies to the ORIGIN
RUN for lib in /stage/usr/lib/shrimply/*.so*; do \
        patchelf --set-rpath '$ORIGIN' "$lib"; \
    done

# Fixes issue: flexiblas Failed to load the BLAS fallback library. Abort!.
# NOTE: Claude helped troubleshooting this one
# This is related to FlexiBLAS linking to any real BLAS Implementation
RUN mkdir -p /stage/usr/lib/shrimply/flexiblas && \
    cp -aL /usr/lib64/flexiblas/. /stage/usr/lib/shrimply/flexiblas/ && \
    cp -a /etc/flexiblasrc /stage/usr/lib/shrimply/flexiblasrc && \
    ldd /stage/usr/lib/shrimply/flexiblas/*.so* | awk '{print $3}' | grep '^/' | sort -u \
      | grep -Ev '/(ld-linux[^/]*|libc|libm|libdl|libpthread|librt|libresolv)\.so' \
      | grep -Ev '/lib(GL|EGL|GLX|GLdispatch|drm|gbm)\.so' \
      | grep -Ev '/libcuda\.so|/libnvidia-' \
      | xargs -r -I{} cp -Ln --remove-destination {} /stage/usr/lib/shrimply/ && \
    for lib in /stage/usr/lib/shrimply/*.so*; do \
        patchelf --set-rpath '$ORIGIN' "$lib"; \
    done && \
    for lib in /stage/usr/lib/shrimply/flexiblas/*.so*; do \
        patchelf --set-rpath '$ORIGIN:$ORIGIN/..' "$lib"; \
    done

# QT's Platform Libs including all the QML Imports
RUN mkdir -p /stage/usr/lib/shrimply/qt6/plugins /stage/usr/lib/shrimply/qt6/qml && \
    cp -aL /usr/lib64/qt6/plugins/. /stage/usr/lib/shrimply/qt6/plugins/ && \
    cp -aL /usr/lib64/qt6/qml/. /stage/usr/lib/shrimply/qt6/qml/ && \
    find /stage/usr/lib/shrimply/qt6 -name '*.so*' -exec ldd {} \; \
      | awk '{print $3}' | grep '^/' | sort -u \
      | grep -Ev '/(ld-linux[^/]*|libc|libm|libdl|libpthread|librt|libresolv)\.so' \
      | grep -Ev '/lib(GL|EGL|GLX|GLdispatch|drm|gbm)\.so' \
      | grep -Ev '/libcuda\.so|/libnvidia-' \
      | xargs -r -I{} cp -Ln --remove-destination {} /stage/usr/lib/shrimply/ && \
    for lib in /stage/usr/lib/shrimply/*.so*; do \
        patchelf --set-rpath '$ORIGIN' "$lib"; \
    done && \
    find /stage/usr/lib/shrimply/qt6 -name '*.so*' | while read -r lib; do \
        depth="$(dirname "$lib" | sed 's|^/stage/usr/lib/shrimply/||' | awk -F/ '{print NF}')"; \
        uprel="$(printf '../%.0s' $(seq 1 "$depth"))"; \
        patchelf --set-rpath "\$ORIGIN:\$ORIGIN/$uprel" "$lib"; \
    done

# GI typelibs, compiled GSettings schemas, and icon themes: non-.so runtime data ldd can't see, but GTK4/libadwaita needs at runtime.
# NOTE: Claude helped troubleshooting this one as well
RUN mkdir -p /stage/usr/lib/shrimply/girepository-1.0 /stage/usr/share/glib-2.0/schemas /stage/usr/share/icons && \
    cp -a /usr/lib64/girepository-1.0/. /stage/usr/lib/shrimply/girepository-1.0/ && \
    cp -a /usr/share/glib-2.0/schemas/. /stage/usr/share/glib-2.0/schemas/ && \
    cp -a /usr/share/icons/Adwaita /stage/usr/share/icons/ && \
    cp -a /usr/share/icons/hicolor /stage/usr/share/icons/

# Bundle all previous fixes into Shrimply and shrimply-editor binaries
RUN for bin in shrimply shrimply-editor shrimply-qt shrimply-editor-qt; do \
        patchelf --set-rpath '$ORIGIN/../lib/shrimply' "/stage/usr/bin/$bin"; \
        mv "/stage/usr/bin/$bin" "/stage/usr/bin/$bin.bin"; \
        printf '#!/bin/sh\nhere="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"\nexport GI_TYPELIB_PATH="$here/../lib/shrimply/girepository-1.0"\nexport GSETTINGS_SCHEMA_DIR="$here/../share/glib-2.0/schemas"\nexport XDG_DATA_DIRS="$here/../share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"\nexport FLEXIBLASRC="$here/../lib/shrimply/flexiblasrc"\nexport FLEXIBLAS_LIBRARY_PATH="$here/../lib/shrimply/flexiblas"\nexec "$here/%s.bin" "$@"\n' "$bin" > "/stage/usr/bin/$bin"; \
        chmod 0755 "/stage/usr/bin/$bin"; \
    done
RUN patchelf --set-rpath '$ORIGIN/../lib/shrimply' /stage/usr/bin/shrimply-mcp

RUN printf '[Paths]\nPrefix = ../lib/shrimply/qt6\nPlugins = plugins\nImports = qml\nQml2Imports = qml\n' \
    > /stage/usr/bin/qt.conf

FROM scratch AS export
COPY --from=builder /stage /
