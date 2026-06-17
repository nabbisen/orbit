# Local AI Models

## Why models?

Keyword search works without any models. Semantic search requires a
local embedding model. Reranking requires a local cross-encoder model.

## Privacy guarantee

Model files are stored on your computer. They are used only for local
inference. Documents are **never** sent to the model provider's servers.

## Registering a model

1. Download or obtain the model weights file (`.onnx` or safetensors).
2. Open the **Models** view.
3. Click **Locate** and point to the file.
4. orbok validates the file and records the dimension.

## Changing the embedding model

If you switch embedding models, existing semantic indexes become
incompatible. orbok marks them stale and reindexes incrementally.

## Recommended models

See the [orbok model compatibility list](https://github.com/nabbisen/orbok/wiki/Models)
(external link, maintained separately).
