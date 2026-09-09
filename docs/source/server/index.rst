Compute Server
==============

Shrimply's optional local compute server provides model-backed transcription,
text-to-speech, video segmentation, voice conversion, 3D camera tracking, and
video generation. The server advertises its available devices and exact model
capabilities to the editor.

Run locally
-----------

The server is a separate uv project requiring Python 3.14. From the repository
root, run its locked environment with:

.. code-block:: console

   $ make dev-server

From the ``server`` directory, the equivalent command is:

.. code-block:: console

   $ uv run --locked src/main.py

Models download into their configured caches on first use. Device and memory
requirements vary substantially by model; consult the model catalog before
starting a large download.

Connect Shrimply
----------------

The local server listens at ``http://127.0.0.1:8787`` by default. In Shrimply,
open :menuselection:`Preferences --> External`. Select the local server under
:guilabel:`Inference Servers`, then choose one of the compute devices reported
by that server.

The :guilabel:`Available` row shows the features supplied by the selected
server. If a feature is missing, it will not appear in the corresponding
editor controls.

Share access
------------

Set ``SHRIMPLY_SERVER_SHARE=1`` to create a temporary public
``gradio.live`` URL while keeping the MessagePack API available:

.. code-block:: console

   $ SHRIMPLY_SERVER_SHARE=1 uv run --locked src/main.py

The public URL can invoke every compute endpoint. Share it only with trusted
users, and stop the process to remove access.

Containers
----------

The Linux amd64 images include Python and server dependencies. The Compose
configuration enables GPU access and preserves the virtual environment, uv
cache, managed Python installation, and downloaded models between runs.
Startup synchronizes the mounted environment against the image's lockfile.
The host needs an NVIDIA driver compatible with the selected CUDA version and
NVIDIA Container Toolkit.

Use the development Compose override to build locally with CUDA 12.6 (the
default) or CUDA 13.0. It inherits the same persistent mounts:

.. code-block:: console

   $ cd server
   $ docker compose -f compose.yaml -f compose.dev.yaml up --build
   $ CUDA_VERSION=13.0 docker compose -f compose.yaml -f compose.dev.yaml up --build

CI publishes both variants to ``ghcr.io/soirihiroka/shrimply-server`` from
``main``. The rolling tags are ``prerelease-cuda12.6`` and
``prerelease-cuda13.0``. Pull and run a published image without building:

.. code-block:: console

   $ docker compose pull
   $ docker compose up --no-build
   $ CUDA_VERSION=13.0 docker compose pull
   $ CUDA_VERSION=13.0 docker compose up --no-build

Each CI run also publishes UTC timestamped tags, such as
``prerelease-20260909T143000Z-cuda13.0``. To select a specific published build,
set ``SHRIMPLY_SERVER_TAG`` to its tag before pulling and starting:

.. code-block:: console

   $ export SHRIMPLY_SERVER_TAG=prerelease-20260909T143000Z-cuda13.0
   $ docker compose pull
   $ docker compose up --no-build

CUDA 13.0 builds change the PyTorch index in the image's ``pyproject.toml`` and
resolve from the existing lockfile, retaining compatible pins. The repository's
``pyproject.toml`` and ``uv.lock`` remain on CUDA 12.6. Both images retain
``pycolmap-cuda12`` for 3D tracking; the image variant identifies the PyTorch
CUDA runtime. Model weights still download on first use.

Compute features
----------------

See :doc:`services` for an overview, or open a feature directly:

* :doc:`transcription`
* :doc:`text-to-speech`
* :doc:`video-segmentation`
* :doc:`voice-conversion`
* :doc:`camera-tracking`
* :doc:`video-generation`

Troubleshooting
---------------

Keep the server process running while Shrimply uses a compute feature. Check
the selected server in :menuselection:`Preferences --> External` if a model is
missing or a connection fails. A first request can take longer while its model
downloads; later requests reuse the downloaded files.

.. toctree::
   :maxdepth: 1
   :hidden:

   services
   transcription
   text-to-speech
   video-segmentation
   voice-conversion
   camera-tracking
   video-generation
