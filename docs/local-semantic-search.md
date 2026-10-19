# Local semantic search feasibility

Investigated on 2026-10-19. Status: isolated prototype completed; app integration remains proposed.

Local embeddings fit qqnote's Rust/Tauri architecture. The most promising starting point is **potion-base-4M with FP16 weights**, backed by the existing SQLite database and exposed through a local MCP server. Its model, tokenizer, and configuration total **8.24 MB**. A preliminary budget puts the app around **17 MB**, pending a complete build with search and MCP included.

## Size and model choices

All sizes below use decimal MB. The existing release `.app` measured 5,800,070 bytes (5.53 MiB, matching the README's approximately 5.5 MB).

| Candidate | Model assets | Dimensions | Evidence |
| --- | ---: | ---: | --- |
| potion-base-2M, FP16 | 4.46 MB | 64 | Loaded and tested locally |
| potion-base-4M, FP16 | 8.24 MB | 128 | Loaded and tested locally; recommended first candidate |
| all-MiniLM-L6-v2, ARM64 INT8 ONNX | 23.49 MB including tokenizer/config | 384 | File metadata inspected; inference/runtime size not tested |

The original POTION weights are FP32: 7.56 MB for 2M and 15.12 MB for 4M, before the 684 KB tokenizer. The prototype converted their embedding tables to FP16. The released Rust loader supports FP16 and expands those weights to FP32 in memory, so this saves bundle space rather than halving inference RAM. Both POTION model cards specify the MIT license. [2M files](https://huggingface.co/minishlab/potion-base-2M/tree/main), [4M files](https://huggingface.co/minishlab/potion-base-4M/tree/main), [Rust loader](https://docs.rs/model2vec-rs/0.3.0/src/model2vec_rs/model.rs.html), [MiniLM files](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/tree/main/onnx).

The standalone Rust probe compiled to **2.54 MB**, with qqnote's release settings: size optimization, LTO, one codegen unit, stripped symbols, and abort-on-panic. Adding that entire executable size to the current app and assets gives approximately **13 MB for 2M FP16** or **17 MB for 4M FP16**. These are planning estimates: shared code can reduce the incremental cost, while MCP, indexing, and search add code. A final integrated release build is required to establish the shipping size. User notes and their index are separate from the application bundle.

The probe used:

```toml
model2vec-rs = { version = "=0.3.0", default-features = false, features = ["local-only", "onig"] }
```

Its dependency tree contained no Hugging Face HTTP client, ONNX Runtime, or Python runtime. Dynamic linking used only macOS system libraries. Model downloads happened during this investigation; a shipping app would load its packaged resource files. [Model2Vec Rust source and features](https://github.com/MinishLab/model2vec-rs).

## Local measurements

Test machine: Apple M4 Pro, 48 GiB RAM. Rust 1.99.1, model2vec-rs 0.3.0. Tests used synthetic text only. No notes database was opened.

The fixture contained 24 short notes and 16 queries, including paraphrases, related distractors, and exact identifiers. Each query had one intended note. Vectors were normalized; retrieval ranked notes by their dot product. Tokenizer parallelism was disabled. Query timing is encoding only, measured across 1,600 calls after initial use; it excludes SQLite, ranking, IPC, and UI work.

| Model | Intended note ranked first | Intended note in top 3 | Query encoding median | Initial model load |
| --- | ---: | ---: | ---: | ---: |
| 2M FP32 | 9 / 16 | 14 / 16 | 10.2 µs | 14.0 ms |
| 2M FP16 | 9 / 16 | 14 / 16 | 9.4 µs | 14.8 ms |
| 4M FP32 | 11 / 16 | 14 / 16 | 9.8 µs | 15.9 ms |
| 4M FP16 | 11 / 16 | 14 / 16 | 10.4 µs | 14.3 ms |

Converting to FP16 preserved every query's top-three list in this fixture. Encoding 10,000 short notes, constructed by repeating the fixture, took 172–179 ms. That is an embedding throughput measurement, not an end-to-end indexing or search benchmark.

For example, the 4M model connected “Who should repair the leaking faucet?” to a note about asking the landlord to fix a dripping kitchen tap. It struggled with “Why was shipping the product delayed?” versus a note about postponing a release because of a billing regression: that intended note ranked fourth.

This small fixture demonstrates feasibility and some limitations; it does not establish production retrieval quality. These models average static token vectors, which limits contextual understanding. English, Danish, mixed-language notes, long notes, negation, and technical identifiers need a larger representative evaluation before choosing the shipping model. [POTION model card](https://huggingface.co/minishlab/potion-base-2M).

The separate macOS `NLEmbedding.sentenceEmbedding(for:)` probe returned an English model, revision 1, with 512 dimensions. Danish returned `nil` on this Mac. The English model found the intended note in the top three for 6 of 16 queries in the same fixture. Availability and this limited result make it a weaker initial candidate here, despite avoiding bundled weights. [Apple API](https://developer.apple.com/documentation/naturallanguage/nlembedding/sentenceembedding(for:)).

Pinned source models:

- `minishlab/potion-base-2M`: `389b9f64be5aa4ae7a6bc6fe95ef20ce485ae5da`
- `minishlab/potion-base-4M`: `9b3cff412d30be9ae8603fe10224c224f3401869`

## Integration into qqnote

1. Load the packaged tokenizer and weights once in the Rust backend. Keep the model lazy so launching the editor remains fast.
2. Add a rebuildable SQLite cache of note chunks and normalized vectors. Store a model/version identifier, chunking version, and body hash. At 128 FP32 dimensions, 10,000 chunks need 5.12 MB for vectors alone, plus text and database overhead.
3. Mark changed notes dirty in the same transaction as saving. Embed in a background worker after a pause, coalescing repeated edits. Release the database mutex while encoding and verify the body hash before committing a result. SQLite timestamps have insufficient resolution to identify every rapid edit.
4. Chunk long notes with overlap so later passages remain searchable. Start around 200 tokens per chunk with 30-token overlap and tune against examples. Exclude Trash at query time, clear caches after permanent deletion, and reindex after restoration or import as needed.
5. Start with a linear cosine scan, then measure at realistic library sizes. Combine semantic ranking with SQLite FTS5 for names, invoice numbers, and exact phrases; fuse ranks rather than adding incomparable raw scores. The bundled SQLite dependency already enables FTS5. [SQLite FTS5 documentation](https://www.sqlite.org/fts5.html).

## Claude and Codex access

Expose read-only MCP tools such as `search_notes(query, limit)` and `get_note(id)`. Return note IDs, titles, matching passages, and timestamps so the agent can retrieve useful context and cite it. The app performs query embedding internally.

A proposed `qqnote --mcp` mode could run over stdio using the same bundled executable. Dispatch that mode before GUI setup, keyboard remapping, and normal stdout logging. It should use a separate database connection and share the same indexing rules as the app, including handling schema readiness and pending edits when the GUI is closed. The official Rust MCP SDK provides a suitable implementation route. [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk), [stdio transport](https://modelcontextprotocol.io/specification/latest/basic/transports).

Both Codex and Claude Code support local stdio MCP servers, so clients could launch that mode directly after implementation. [Official OpenAI documentation](https://learn.chatgpt.com/docs/developer-commands#codex-mcp), [Claude Code documentation](https://code.claude.com/docs/en/mcp#option-3-add-a-local-stdio-server).

Embedding and searching would remain local. Passages returned to an AI client follow that client's model and data path, including a cloud provider when one is used.

The next implementation step is a Rust search module plus SQLite cache and a broader retrieval fixture, followed by the MCP adapter and a measured integrated release build.
