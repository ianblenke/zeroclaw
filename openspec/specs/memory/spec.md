# Memory Specification

## Purpose

Define requirements for the memory subsystem: trait contract, backend implementations (SQLite, Markdown, Lucid, Hybrid, Cortex, None, PostgreSQL, Qdrant), backend profiles, embeddings, chunking, vector operations, response caching, snapshots, hygiene, and CLI.

## Scope

- Files: 18 files in `src/memory/` (~8,841 LOC total)
- Risk tier: MEDIUM (memory stores persist agent knowledge; incorrect recall affects agent behavior)
- Existing tests: 234 across 18 files
  - `src/memory/traits.rs` — 5 tests
  - `src/memory/sqlite.rs` — 69 tests
  - `src/memory/markdown.rs` — 11 tests
  - `src/memory/lucid.rs` — 6 tests
  - `src/memory/hybrid.rs` — 3 tests
  - `src/memory/cortex.rs` — 2 tests
  - `src/memory/none.rs` — 1 test
  - `src/memory/postgres.rs` — 4 tests (3 sync + 1 async config validation)
  - `src/memory/qdrant.rs` — 4 tests
  - `src/memory/embeddings.rs` — 20 tests
  - `src/memory/chunker.rs` — 19 tests
  - `src/memory/vector.rs` — 30 tests
  - `src/memory/response_cache.rs` — 17 tests
  - `src/memory/mod.rs` — 18 tests
  - `src/memory/snapshot.rs` — 7 tests
  - `src/memory/hygiene.rs` — 5 tests
  - `src/memory/cli.rs` — 6 tests
  - `src/memory/backend.rs` — 7 tests (backend profile classification/selection)

## Requirements

### REQ-MEM-001: Memory Trait Contract

All memory backends MUST implement the `Memory` trait with store, recall, get, list, forget, count, and health_check methods. Category display and serialization MUST use expected formats.

#### Scenario: MemoryCategory Display outputs expected values

- WHEN `MemoryCategory` variants are formatted with Display
- THEN MUST produce expected string representations
- Test: `memory_category_display_outputs_expected_values` in `src/memory/traits.rs`

#### Scenario: MemoryCategory serialization uses snake_case

- WHEN `MemoryCategory` variants are serialized and deserialized via serde
- THEN MUST use snake_case keys
- Test: `memory_category_serde_uses_snake_case` in `src/memory/traits.rs`

#### Scenario: Default reindex implementation bails

- WHEN the default `reindex` method is called on a Memory implementor that does not override it
- THEN MUST return an error
- Test: `memory_reindex_default_bails` in `src/memory/traits.rs`

#### Scenario: Custom category serde round-trip

- WHEN a `MemoryCategory::Custom` variant is serialized then deserialized
- THEN MUST preserve the custom string value
- Test: `memory_category_custom_serde_roundtrip` in `src/memory/traits.rs`

#### Scenario: MemoryEntry round-trip preserves optional fields

- WHEN a `MemoryEntry` with optional fields set is serialized then deserialized
- THEN MUST preserve all optional fields (session_id, score, embedding, etc.)
- Test: `memory_entry_roundtrip_preserves_optional_fields` in `src/memory/traits.rs`

---

### REQ-MEM-002: SQLite Memory Backend

`SqliteMemory` MUST provide persistent memory storage using SQLite with full-text search (FTS5), session filtering, embedding-based recall, content hashing, and safe handling of special characters.

#### Scenario: Backend reports expected name

- WHEN `name()` is called on SqliteMemory
- THEN MUST return the expected backend name string
- Test: `sqlite_name` in `src/memory/sqlite.rs`

#### Scenario: Health check reports healthy

- WHEN `health_check()` is called on a valid SqliteMemory instance
- THEN MUST return true
- Test: `sqlite_health` in `src/memory/sqlite.rs`

#### Scenario: Store and get entry

- WHEN an entry is stored with a key and content
- THEN MUST be retrievable via `get()` with matching key and content
- Test: `sqlite_store_and_get` in `src/memory/sqlite.rs`

#### Scenario: Store upserts existing key

- WHEN an entry is stored with an existing key
- THEN MUST update the content (upsert behavior)
- Test: `sqlite_store_upsert` in `src/memory/sqlite.rs`

#### Scenario: Keyword recall

- WHEN entries are stored and recall is called with a matching keyword
- THEN MUST return entries containing that keyword
- Test: `sqlite_recall_keyword` in `src/memory/sqlite.rs`

#### Scenario: Multi-keyword recall

- WHEN recall is called with multiple keywords
- THEN MUST return entries matching the combined query
- Test: `sqlite_recall_multi_keyword` in `src/memory/sqlite.rs`

#### Scenario: Recall with no match

- WHEN recall is called with a query that matches nothing
- THEN MUST return an empty result set
- Test: `sqlite_recall_no_match` in `src/memory/sqlite.rs`

#### Scenario: Forget entry

- WHEN `forget()` is called with an existing key
- THEN MUST remove the entry and return true
- Test: `sqlite_forget` in `src/memory/sqlite.rs`

#### Scenario: Forget nonexistent entry

- WHEN `forget()` is called with a nonexistent key
- THEN MUST return false without error
- Test: `sqlite_forget_nonexistent` in `src/memory/sqlite.rs`

#### Scenario: List all entries

- WHEN `list()` is called without a category filter
- THEN MUST return all stored entries
- Test: `sqlite_list_all` in `src/memory/sqlite.rs`

#### Scenario: List by category

- WHEN `list()` is called with a category filter
- THEN MUST return only entries matching that category
- Test: `sqlite_list_by_category` in `src/memory/sqlite.rs`

#### Scenario: Count on empty database

- WHEN `count()` is called on an empty database
- THEN MUST return zero
- Test: `sqlite_count_empty` in `src/memory/sqlite.rs`

#### Scenario: Get nonexistent key

- WHEN `get()` is called with a nonexistent key
- THEN MUST return None
- Test: `sqlite_get_nonexistent` in `src/memory/sqlite.rs`

#### Scenario: Database persistence across instances

- WHEN a new SqliteMemory instance is created for the same path
- THEN MUST see entries stored by the previous instance
- Test: `sqlite_db_persists` in `src/memory/sqlite.rs`

#### Scenario: Category round-trip

- WHEN entries are stored with various MemoryCategory values and recalled
- THEN MUST preserve the category on retrieval
- Test: `sqlite_category_roundtrip` in `src/memory/sqlite.rs`

#### Scenario: FTS5 BM25 ranking

- WHEN recall is performed with FTS5
- THEN MUST return results ordered by BM25 relevance score
- Test: `fts5_bm25_ranking` in `src/memory/sqlite.rs`

#### Scenario: FTS5 multi-word query

- WHEN recall is called with a multi-word query
- THEN MUST match entries containing all query terms
- Test: `fts5_multi_word_query` in `src/memory/sqlite.rs`

#### Scenario: Recall with empty query

- WHEN recall is called with an empty string
- THEN MUST return an empty result set
- Test: `recall_empty_query_returns_empty` in `src/memory/sqlite.rs`

#### Scenario: Recall with whitespace-only query

- WHEN recall is called with a whitespace-only query
- THEN MUST return an empty result set
- Test: `recall_whitespace_query_returns_empty` in `src/memory/sqlite.rs`

#### Scenario: Content hash determinism

- WHEN content_hash is called with the same input twice
- THEN MUST return identical hashes
- Test: `content_hash_deterministic` in `src/memory/sqlite.rs`

#### Scenario: Content hash different inputs

- WHEN content_hash is called with different inputs
- THEN MUST return different hashes
- Test: `content_hash_different_inputs` in `src/memory/sqlite.rs`

#### Scenario: Schema has FTS5 table

- WHEN the database is initialized
- THEN MUST have a FTS5 virtual table in the schema
- Test: `schema_has_fts5_table` in `src/memory/sqlite.rs`

#### Scenario: Schema has embedding cache table

- WHEN the database is initialized
- THEN MUST have an embedding_cache table
- Test: `schema_has_embedding_cache` in `src/memory/sqlite.rs`

#### Scenario: Memories table has embedding column

- WHEN the database schema is inspected
- THEN the memories table MUST have an embedding BLOB column
- Test: `schema_memories_has_embedding_column` in `src/memory/sqlite.rs`

#### Scenario: FTS5 syncs on insert

- WHEN a new entry is inserted
- THEN MUST synchronize the FTS5 index for that entry
- Test: `fts5_syncs_on_insert` in `src/memory/sqlite.rs`

#### Scenario: FTS5 syncs on delete

- WHEN an entry is deleted
- THEN MUST remove the entry from the FTS5 index
- Test: `fts5_syncs_on_delete` in `src/memory/sqlite.rs`

#### Scenario: FTS5 syncs on update

- WHEN an entry is updated (upserted)
- THEN MUST update the FTS5 index to reflect new content
- Test: `fts5_syncs_on_update` in `src/memory/sqlite.rs`

#### Scenario: Open with timeout succeeds when fast

- WHEN a database is opened with a timeout and responds quickly
- THEN MUST succeed without error
- Test: `open_with_timeout_succeeds_when_fast` in `src/memory/sqlite.rs`

#### Scenario: Open with timeout does not change store/recall behavior

- WHEN a database is opened with a timeout
- THEN store and recall MUST behave identically to a non-timeout open
- Test: `open_with_timeout_store_recall_unchanged` in `src/memory/sqlite.rs`

#### Scenario: With embedder noop

- WHEN SqliteMemory is created with_embedder using a noop provider
- THEN MUST construct without error
- Test: `with_embedder_noop` in `src/memory/sqlite.rs`

#### Scenario: Reindex rebuilds FTS

- WHEN reindex is called
- THEN MUST rebuild the FTS5 index so all entries are searchable
- Test: `reindex_rebuilds_fts` in `src/memory/sqlite.rs`

#### Scenario: Recall respects limit

- WHEN recall is called with a limit
- THEN MUST return at most that many entries
- Test: `recall_respects_limit` in `src/memory/sqlite.rs`

#### Scenario: Recall results have scores

- WHEN recall returns entries
- THEN each entry MUST have a relevance score set
- Test: `recall_results_have_scores` in `src/memory/sqlite.rs`

#### Scenario: Recall with quotes in query

- WHEN recall is called with a query containing quote characters
- THEN MUST not panic or return SQL errors
- Test: `recall_with_quotes_in_query` in `src/memory/sqlite.rs`

#### Scenario: Recall with asterisk in query

- WHEN recall is called with a query containing asterisk characters
- THEN MUST not panic or return SQL errors
- Test: `recall_with_asterisk_in_query` in `src/memory/sqlite.rs`

#### Scenario: Recall with parentheses in query

- WHEN recall is called with a query containing parentheses
- THEN MUST not panic or return SQL errors
- Test: `recall_with_parentheses_in_query` in `src/memory/sqlite.rs`

#### Scenario: Recall with SQL injection attempt

- WHEN recall is called with a query containing SQL injection patterns
- THEN MUST safely handle the input without data leakage or errors
- Test: `recall_with_sql_injection_attempt` in `src/memory/sqlite.rs`

#### Scenario: Store empty content

- WHEN an entry with empty content is stored
- THEN MUST succeed and be retrievable
- Test: `store_empty_content` in `src/memory/sqlite.rs`

#### Scenario: Store empty key

- WHEN an entry with an empty key is stored
- THEN MUST succeed and be retrievable
- Test: `store_empty_key` in `src/memory/sqlite.rs`

#### Scenario: Store very long content

- WHEN an entry with very long content is stored
- THEN MUST succeed and be retrievable
- Test: `store_very_long_content` in `src/memory/sqlite.rs`

#### Scenario: Store unicode and emoji content

- WHEN an entry with unicode and emoji characters is stored and retrieved
- THEN MUST preserve the content exactly
- Test: `store_unicode_and_emoji` in `src/memory/sqlite.rs`

#### Scenario: Store content with newlines and tabs

- WHEN an entry with newlines and tabs is stored and retrieved
- THEN MUST preserve the whitespace characters
- Test: `store_content_with_newlines_and_tabs` in `src/memory/sqlite.rs`

#### Scenario: Recall single character query

- WHEN recall is called with a single character query
- THEN MUST return matching entries or empty without error
- Test: `recall_single_character_query` in `src/memory/sqlite.rs`

#### Scenario: Recall limit zero

- WHEN recall is called with limit zero
- THEN MUST return an empty result set
- Test: `recall_limit_zero` in `src/memory/sqlite.rs`

#### Scenario: Recall limit one

- WHEN recall is called with limit one
- THEN MUST return at most one entry
- Test: `recall_limit_one` in `src/memory/sqlite.rs`

#### Scenario: Recall matches by key not just content

- WHEN recall is called with a query matching a key string
- THEN MUST also match entries by key, not only by content
- Test: `recall_matches_by_key_not_just_content` in `src/memory/sqlite.rs`

#### Scenario: Recall with unicode query

- WHEN recall is called with a unicode query string
- THEN MUST return matching entries without error
- Test: `recall_unicode_query` in `src/memory/sqlite.rs`

#### Scenario: Schema idempotent on reopen

- WHEN the database is reopened with schema initialization
- THEN schema creation MUST be idempotent (no errors or data loss)
- Test: `schema_idempotent_reopen` in `src/memory/sqlite.rs`

#### Scenario: Schema triple open

- WHEN the database is opened three times in succession
- THEN MUST succeed each time without corruption
- Test: `schema_triple_open` in `src/memory/sqlite.rs`

#### Scenario: Forget then recall produces no ghost results

- WHEN an entry is forgotten and then recalled by its content
- THEN MUST return no results (no ghost FTS entries)
- Test: `forget_then_recall_no_ghost_results` in `src/memory/sqlite.rs`

#### Scenario: Forget and re-store same key

- WHEN an entry is forgotten then re-stored with the same key
- THEN MUST store the new content and be retrievable
- Test: `forget_and_re_store_same_key` in `src/memory/sqlite.rs`

#### Scenario: Reindex on empty database

- WHEN reindex is called on an empty database
- THEN MUST return zero and succeed
- Test: `reindex_empty_db` in `src/memory/sqlite.rs`

#### Scenario: Reindex twice is safe

- WHEN reindex is called twice
- THEN MUST be idempotent and produce the same state
- Test: `reindex_twice_is_safe` in `src/memory/sqlite.rs`

#### Scenario: Content hash of empty string

- WHEN content_hash is called with an empty string
- THEN MUST return a valid deterministic hash
- Test: `content_hash_empty_string` in `src/memory/sqlite.rs`

#### Scenario: Content hash of unicode input

- WHEN content_hash is called with unicode input
- THEN MUST return a valid deterministic hash
- Test: `content_hash_unicode` in `src/memory/sqlite.rs`

#### Scenario: Content hash of long input

- WHEN content_hash is called with very long input
- THEN MUST return a valid hash of fixed length
- Test: `content_hash_long_input` in `src/memory/sqlite.rs`

#### Scenario: Category round-trip with custom spaces

- WHEN a Custom category with spaces is stored and retrieved
- THEN MUST preserve the custom category string including spaces
- Test: `category_roundtrip_custom_with_spaces` in `src/memory/sqlite.rs`

#### Scenario: Category round-trip with empty custom

- WHEN a Custom category with an empty string is stored and retrieved
- THEN MUST preserve the empty custom category
- Test: `category_roundtrip_empty_custom` in `src/memory/sqlite.rs`

#### Scenario: List entries with custom category

- WHEN entries with custom categories are stored and listed by category
- THEN MUST filter correctly by custom category
- Test: `list_custom_category` in `src/memory/sqlite.rs`

#### Scenario: List on empty database

- WHEN list is called on an empty database
- THEN MUST return an empty result set
- Test: `list_empty_db` in `src/memory/sqlite.rs`

#### Scenario: Store and recall with session_id

- WHEN entries are stored with a session_id and recalled with a session filter
- THEN MUST return only entries matching that session
- Test: `store_and_recall_with_session_id` in `src/memory/sqlite.rs`

#### Scenario: Recall without session filter returns all

- WHEN recall is called without a session filter
- THEN MUST return entries from all sessions
- Test: `recall_no_session_filter_returns_all` in `src/memory/sqlite.rs`

#### Scenario: Cross-session recall isolation

- WHEN two different sessions store entries and one session filters recall
- THEN MUST return only entries for the requested session
- Test: `cross_session_recall_isolation` in `src/memory/sqlite.rs`

#### Scenario: List with session filter

- WHEN list is called with a session filter
- THEN MUST return only entries for that session
- Test: `list_with_session_filter` in `src/memory/sqlite.rs`

#### Scenario: Schema migration idempotent on reopen

- WHEN an older schema database is reopened
- THEN schema migration MUST be idempotent and non-destructive
- Test: `schema_migration_idempotent_on_reopen` in `src/memory/sqlite.rs`

#### Scenario: Concurrent writes with no data loss

- WHEN multiple concurrent writes are issued
- THEN MUST complete without data loss
- Test: `sqlite_concurrent_writes_no_data_loss` in `src/memory/sqlite.rs`

#### Scenario: Concurrent read/write with no panic

- WHEN concurrent reads and writes are issued simultaneously
- THEN MUST complete without panics or errors
- Test: `sqlite_concurrent_read_write_no_panic` in `src/memory/sqlite.rs`

#### Scenario: Reindex preserves data

- WHEN reindex is called on a populated database
- THEN MUST preserve all existing entries and their content
- Test: `sqlite_reindex_preserves_data` in `src/memory/sqlite.rs`

#### Scenario: Reindex is idempotent

- WHEN reindex is called multiple times on the same data
- THEN MUST produce identical results each time
- Test: `sqlite_reindex_idempotent` in `src/memory/sqlite.rs`

---

### REQ-MEM-003: Markdown Memory Backend

`MarkdownMemory` MUST provide file-based memory storage using markdown files with keyword-based recall and category-based listing.

#### Scenario: Backend reports expected name

- WHEN `name()` is called on MarkdownMemory
- THEN MUST return the expected backend name string
- Test: `markdown_name` in `src/memory/markdown.rs`

#### Scenario: Health check reports healthy

- WHEN `health_check()` is called
- THEN MUST return true
- Test: `markdown_health_check` in `src/memory/markdown.rs`

#### Scenario: Store core category entry

- WHEN an entry with Core category is stored
- THEN MUST persist to the core memory markdown file
- Test: `markdown_store_core` in `src/memory/markdown.rs`

#### Scenario: Store daily category entry

- WHEN an entry with Daily category is stored
- THEN MUST persist to the daily memory markdown file
- Test: `markdown_store_daily` in `src/memory/markdown.rs`

#### Scenario: Recall by keyword

- WHEN recall is called with a keyword matching stored content
- THEN MUST return matching entries
- Test: `markdown_recall_keyword` in `src/memory/markdown.rs`

#### Scenario: Recall with no match

- WHEN recall is called with a keyword that matches nothing
- THEN MUST return an empty result set
- Test: `markdown_recall_no_match` in `src/memory/markdown.rs`

#### Scenario: Count entries

- WHEN entries are stored and count is called
- THEN MUST return the correct number of entries
- Test: `markdown_count` in `src/memory/markdown.rs`

#### Scenario: List by category

- WHEN list is called with a category filter
- THEN MUST return only entries matching that category
- Test: `markdown_list_by_category` in `src/memory/markdown.rs`

#### Scenario: Forget is a no-op

- WHEN forget is called
- THEN MUST return false (markdown backend does not support deletion)
- Test: `markdown_forget_is_noop` in `src/memory/markdown.rs`

#### Scenario: Empty recall on fresh workspace

- WHEN recall is called on a workspace with no stored entries
- THEN MUST return an empty result set
- Test: `markdown_empty_recall` in `src/memory/markdown.rs`

#### Scenario: Empty count on fresh workspace

- WHEN count is called on a workspace with no stored entries
- THEN MUST return zero
- Test: `markdown_empty_count` in `src/memory/markdown.rs`

---

### REQ-MEM-004: Lucid Memory Backend

`LucidMemory` MUST provide memory with semantic search by bridging to the lucid-memory CLI while keeping a local SQLite fallback. It MUST handle CLI failures gracefully with cooldown and timeout logic.

#### Scenario: Backend reports expected name

- WHEN `name()` is called on LucidMemory
- THEN MUST return the expected backend name string
- Test: `lucid_name` in `src/memory/lucid.rs`

#### Scenario: Store succeeds when lucid CLI is missing

- WHEN an entry is stored and the lucid CLI binary is not available
- THEN MUST still store to the local SQLite backend without error
- Test: `store_succeeds_when_lucid_missing` in `src/memory/lucid.rs`

#### Scenario: Recall merges lucid and local results

- WHEN recall is called and both lucid CLI and local SQLite have results
- THEN MUST merge and deduplicate results from both sources
- Test: `recall_merges_lucid_and_local_results` in `src/memory/lucid.rs`

#### Scenario: Recall handles lucid cold start delay within timeout

- WHEN the lucid CLI has a cold start delay within the configured timeout
- THEN MUST wait and include the lucid results
- Test: `recall_handles_lucid_cold_start_delay_within_timeout` in `src/memory/lucid.rs`

#### Scenario: Recall skips lucid when local hits are enough

- WHEN local SQLite has enough high-quality hits
- THEN MAY skip the lucid CLI call for performance
- Test: `recall_skips_lucid_when_local_hits_are_enough` in `src/memory/lucid.rs`

#### Scenario: Failure cooldown avoids repeated lucid calls

- WHEN the lucid CLI fails
- THEN MUST enter a cooldown period and skip subsequent lucid calls until cooldown expires
- Test: `failure_cooldown_avoids_repeated_lucid_calls` in `src/memory/lucid.rs`

---

### REQ-MEM-005: Hybrid Memory Backend

`SqliteQdrantHybridMemory` MUST combine SQLite and Qdrant backends, storing to both and merging recall results with Qdrant ranking. It MUST fall back to SQLite when Qdrant is unavailable.

#### Scenario: Store keeps SQLite data when Qdrant sync fails

- WHEN an entry is stored and Qdrant sync fails
- THEN MUST still persist the entry in SQLite
- Test: `store_keeps_sqlite_when_qdrant_sync_fails` in `src/memory/hybrid.rs`

#### Scenario: Recall joins Qdrant ranking with SQLite rows

- WHEN recall is called and both backends respond
- THEN MUST merge results using Qdrant ranking with SQLite metadata
- Test: `recall_joins_qdrant_ranking_with_sqlite_rows` in `src/memory/hybrid.rs`

#### Scenario: Recall falls back to SQLite when Qdrant fails

- WHEN recall is called and Qdrant returns an error
- THEN MUST fall back to SQLite-only results
- Test: `recall_falls_back_to_sqlite_when_qdrant_fails` in `src/memory/hybrid.rs`

---

### REQ-MEM-006: Cortex Memory Backend

`CortexMemMemory` MUST bridge to the cortex-mem CLI for memory operations while keeping a local SQLite backend as fallback.

#### Scenario: Backend reports expected name

- WHEN `name()` is called on CortexMemMemory
- THEN MUST return the expected backend name string
- Test: `cortex_backend_reports_expected_name` in `src/memory/cortex.rs`

#### Scenario: Store keeps local data when bridge command fails

- WHEN an entry is stored and the cortex-mem CLI bridge command fails
- THEN MUST still persist the entry in the local SQLite backend
- Test: `cortex_backend_keeps_local_store_when_bridge_command_fails` in `src/memory/cortex.rs`

---

### REQ-MEM-007: None Memory Backend

`NoneMemory` MUST provide a no-op memory backend that returns empty results for all operations.

#### Scenario: All operations are no-ops

- WHEN any memory operation (store, recall, get, list, forget, count, health_check) is performed
- THEN MUST return success with empty/zero/true results as appropriate
- Test: `none_memory_is_noop` in `src/memory/none.rs`

---

### REQ-MEM-008: PostgreSQL Memory Backend

`PostgresMemory` MUST provide PostgreSQL-based memory storage with connection pooling, identifier validation, and category mapping.

#### Scenario: Valid identifiers pass validation

- WHEN valid PostgreSQL identifiers are provided for schema/table names
- THEN MUST pass validation without error
- Test: `valid_identifiers_pass_validation` in `src/memory/postgres.rs`

#### Scenario: Invalid identifiers are rejected

- WHEN invalid PostgreSQL identifiers are provided (empty, starts with digit, contains hyphens)
- THEN MUST reject with an error
- Test: `invalid_identifiers_are_rejected` in `src/memory/postgres.rs`

#### Scenario: Category parsing maps known and custom values

- WHEN category strings are parsed
- THEN MUST map known strings (core, daily, conversation) to typed variants and unknown strings to Custom
- Test: `parse_category_maps_known_and_custom_values` in `src/memory/postgres.rs`

#### Scenario: Construction does not panic inside tokio runtime

- WHEN `PostgresMemory::new` is called inside a tokio runtime with an unreachable endpoint
- THEN MUST not panic (should return a connect error)
- Test: `new_does_not_panic_inside_tokio_runtime` in `src/memory/postgres.rs`

---

### REQ-MEM-009: Qdrant Memory Backend

`QdrantMemory` MUST provide vector database storage via Qdrant with category mapping, payload serialization, and lazy initialization.

#### Scenario: Category to string maps known categories

- WHEN known MemoryCategory variants are converted to string
- THEN MUST produce expected lowercase string representations
- Test: `category_to_str_maps_known_categories` in `src/memory/qdrant.rs`

#### Scenario: Parse category maps known and custom values

- WHEN category strings are parsed back to MemoryCategory
- THEN MUST map known strings to typed variants and unknown strings to Custom
- Test: `parse_category_maps_known_and_custom_values` in `src/memory/qdrant.rs`

#### Scenario: Memory payload serializes correctly

- WHEN a memory payload with all fields is serialized to JSON
- THEN MUST produce the expected JSON structure
- Test: `memory_payload_serializes_correctly` in `src/memory/qdrant.rs`

#### Scenario: Memory payload skips None session_id

- WHEN a memory payload with no session_id is serialized
- THEN MUST omit the session_id field from the JSON output
- Test: `memory_payload_skips_none_session_id` in `src/memory/qdrant.rs`

---

### REQ-MEM-010: Embedding Providers

Embedding provider functions MUST support multiple backends (OpenAI, Ollama, local, OpenRouter) with configurable endpoints and dimensions. The factory MUST produce the correct provider or fall back to NoopEmbeddingProvider.

#### Scenario: Noop provider name

- WHEN `name()` is called on NoopEmbeddingProvider
- THEN MUST return the expected name string
- Test: `noop_name` in `src/memory/embeddings.rs`

#### Scenario: Noop embed returns empty

- WHEN `embed()` is called on NoopEmbeddingProvider
- THEN MUST return an empty vector
- Test: `noop_embed_returns_empty` in `src/memory/embeddings.rs`

#### Scenario: Noop embed_one returns error

- WHEN `embed_one()` is called on NoopEmbeddingProvider
- THEN MUST return an error (single embedding not supported for noop)
- Test: `noop_embed_one_returns_error` in `src/memory/embeddings.rs`

#### Scenario: Noop embed empty batch

- WHEN `embed()` is called on NoopEmbeddingProvider with an empty batch
- THEN MUST return an empty vector
- Test: `noop_embed_empty_batch` in `src/memory/embeddings.rs`

#### Scenario: Noop embed multiple texts

- WHEN `embed()` is called on NoopEmbeddingProvider with multiple texts
- THEN MUST return an empty vector for all texts
- Test: `noop_embed_multiple_texts` in `src/memory/embeddings.rs`

#### Scenario: Factory creates noop for "none"

- WHEN the embedding factory is called with provider "none"
- THEN MUST return a NoopEmbeddingProvider
- Test: `factory_none` in `src/memory/embeddings.rs`

#### Scenario: Factory creates OpenAI provider

- WHEN the embedding factory is called with provider "openai"
- THEN MUST return an OpenAI-backed embedding provider
- Test: `factory_openai` in `src/memory/embeddings.rs`

#### Scenario: Factory creates OpenRouter provider

- WHEN the embedding factory is called with provider "openrouter"
- THEN MUST return an OpenRouter-backed embedding provider with correct base URL
- Test: `factory_openrouter` in `src/memory/embeddings.rs`

#### Scenario: Factory creates custom URL provider

- WHEN the embedding factory is called with a custom base URL
- THEN MUST return a provider configured with that URL
- Test: `factory_custom_url` in `src/memory/embeddings.rs`

#### Scenario: Factory returns noop for empty string

- WHEN the embedding factory is called with an empty provider string
- THEN MUST return a NoopEmbeddingProvider
- Test: `factory_empty_string_returns_noop` in `src/memory/embeddings.rs`

#### Scenario: Factory returns noop for unknown provider

- WHEN the embedding factory is called with an unknown provider name
- THEN MUST return a NoopEmbeddingProvider
- Test: `factory_unknown_provider_returns_noop` in `src/memory/embeddings.rs`

#### Scenario: Factory with custom empty URL

- WHEN the embedding factory is called with a custom provider but empty URL
- THEN MUST handle gracefully (return noop or error)
- Test: `factory_custom_empty_url` in `src/memory/embeddings.rs`

#### Scenario: Factory with OpenAI but no API key

- WHEN the embedding factory is called with openai provider but no API key
- THEN MUST handle gracefully
- Test: `factory_openai_no_api_key` in `src/memory/embeddings.rs`

#### Scenario: OpenAI trailing slash is stripped

- WHEN an OpenAI base URL with a trailing slash is provided
- THEN MUST strip the trailing slash from the stored URL
- Test: `openai_trailing_slash_stripped` in `src/memory/embeddings.rs`

#### Scenario: OpenAI custom dimensions

- WHEN custom dimensions are specified for OpenAI provider
- THEN MUST report the custom dimension count
- Test: `openai_dimensions_custom` in `src/memory/embeddings.rs`

#### Scenario: Embeddings URL for OpenRouter

- WHEN the embeddings URL is computed for an OpenRouter provider
- THEN MUST use the correct OpenRouter API path
- Test: `embeddings_url_openrouter` in `src/memory/embeddings.rs`

#### Scenario: Embeddings URL for standard OpenAI

- WHEN the embeddings URL is computed for a standard OpenAI provider
- THEN MUST use the standard /v1/embeddings path
- Test: `embeddings_url_standard_openai` in `src/memory/embeddings.rs`

#### Scenario: Embeddings URL with v1 base does not duplicate path

- WHEN the base URL already ends with /v1
- THEN MUST not duplicate the /v1 segment in the embeddings URL
- Test: `embeddings_url_base_with_v1_no_duplicate` in `src/memory/embeddings.rs`

#### Scenario: Embeddings URL with non-v1 API path uses raw suffix

- WHEN the base URL has a non-v1 API path (explicit full endpoint)
- THEN MUST use the raw path with /embeddings suffix
- Test: `embeddings_url_non_v1_api_path_uses_raw_suffix` in `src/memory/embeddings.rs`

#### Scenario: Embeddings URL with custom full endpoint

- WHEN the base URL contains a full custom endpoint path
- THEN MUST use that path directly
- Test: `embeddings_url_custom_full_endpoint` in `src/memory/embeddings.rs`

---

### REQ-MEM-011: Chunker

`Chunk` functions MUST split documents into semantically meaningful chunks for embedding, respecting markdown structure (headings, blank lines, line breaks).

#### Scenario: Empty text produces no chunks

- WHEN an empty string is chunked
- THEN MUST return an empty vector
- Test: `empty_text` in `src/memory/chunker.rs`

#### Scenario: Single short paragraph

- WHEN a short text without headings is chunked
- THEN MUST return a single chunk containing the full text
- Test: `single_short_paragraph` in `src/memory/chunker.rs`

#### Scenario: Heading sections

- WHEN a text with heading-separated sections is chunked
- THEN MUST produce separate chunks at heading boundaries
- Test: `heading_sections` in `src/memory/chunker.rs`

#### Scenario: Respects max_tokens

- WHEN a large section exceeds max_tokens
- THEN MUST split into sub-chunks that respect the token limit
- Test: `respects_max_tokens` in `src/memory/chunker.rs`

#### Scenario: Preserves heading in split sections

- WHEN a section with a heading is split due to size
- THEN MUST preserve the heading text in the chunk metadata
- Test: `preserves_heading_in_split_sections` in `src/memory/chunker.rs`

#### Scenario: Indexes are sequential

- WHEN multiple chunks are produced
- THEN MUST have sequential zero-based index values
- Test: `indexes_are_sequential` in `src/memory/chunker.rs`

#### Scenario: Chunk count is reasonable

- WHEN a text of known size is chunked
- THEN MUST produce a reasonable number of chunks (not excessively many or few)
- Test: `chunk_count_reasonable` in `src/memory/chunker.rs`

#### Scenario: Headings only with no body

- WHEN text consists of headings with no body content
- THEN MUST produce chunks for each heading without error
- Test: `headings_only_no_body` in `src/memory/chunker.rs`

#### Scenario: Deeply nested headings

- WHEN text has deeply nested headings (h4+)
- THEN MUST split only on top-level headings and treat deep headings as content
- Test: `deeply_nested_headings_ignored` in `src/memory/chunker.rs`

#### Scenario: Very long single line without newlines

- WHEN text is a single very long line with no newlines
- THEN MUST still split into chunks within the token limit
- Test: `very_long_single_line_no_newlines` in `src/memory/chunker.rs`

#### Scenario: Only newlines and whitespace

- WHEN text consists only of newlines and whitespace
- THEN MUST return an empty vector (no meaningful chunks)
- Test: `only_newlines_and_whitespace` in `src/memory/chunker.rs`

#### Scenario: max_tokens zero

- WHEN max_tokens is set to zero
- THEN MUST handle gracefully (produce chunks without infinite loop)
- Test: `max_tokens_zero` in `src/memory/chunker.rs`

#### Scenario: max_tokens one

- WHEN max_tokens is set to one
- THEN MUST produce one-token chunks without error
- Test: `max_tokens_one` in `src/memory/chunker.rs`

#### Scenario: Unicode content

- WHEN text contains unicode characters (CJK, emoji, etc.)
- THEN MUST chunk correctly without corrupting characters
- Test: `unicode_content` in `src/memory/chunker.rs`

#### Scenario: FTS5 special characters in content

- WHEN text contains FTS5 special characters (quotes, asterisks, etc.)
- THEN MUST include them in chunks without error
- Test: `fts5_special_chars_in_content` in `src/memory/chunker.rs`

#### Scenario: Multiple blank lines between paragraphs

- WHEN text has multiple blank lines between paragraphs
- THEN MUST split at blank-line boundaries and not produce empty chunks
- Test: `multiple_blank_lines_between_paragraphs` in `src/memory/chunker.rs`

#### Scenario: Heading at end of text

- WHEN text ends with a heading and no body content after it
- THEN MUST produce a chunk for that heading
- Test: `heading_at_end_of_text` in `src/memory/chunker.rs`

#### Scenario: Single heading with no content

- WHEN text is a single heading with no body
- THEN MUST produce one chunk containing the heading
- Test: `single_heading_no_content` in `src/memory/chunker.rs`

#### Scenario: No content loss

- WHEN text is chunked and chunks are concatenated
- THEN all original content MUST be present (no data loss)
- Test: `no_content_loss` in `src/memory/chunker.rs`

---

### REQ-MEM-012: Vector Operations

Vector operations MUST provide cosine similarity, hybrid merge, serialization, and scoring utilities with correct mathematical behavior for edge cases.

#### Scenario: Cosine similarity of identical vectors

- WHEN two identical vectors are compared
- THEN MUST return 1.0 (perfect similarity)
- Test: `cosine_identical_vectors` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of orthogonal vectors

- WHEN two orthogonal vectors are compared
- THEN MUST return 0.0 (no similarity)
- Test: `cosine_orthogonal_vectors` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of similar vectors

- WHEN two similar but not identical vectors are compared
- THEN MUST return a value between 0.0 and 1.0
- Test: `cosine_similar_vectors` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of empty vectors

- WHEN two empty vectors are compared
- THEN MUST return 0.0
- Test: `cosine_empty_returns_zero` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of mismatched length vectors

- WHEN two vectors of different lengths are compared
- THEN MUST return 0.0 (invalid comparison)
- Test: `cosine_mismatched_lengths` in `src/memory/vector.rs`

#### Scenario: Cosine similarity with zero vector

- WHEN a zero vector is compared with any other vector
- THEN MUST return 0.0 (undefined similarity clamped)
- Test: `cosine_zero_vector` in `src/memory/vector.rs`

#### Scenario: Cosine similarity with NaN values

- WHEN a vector containing NaN is compared
- THEN MUST return 0.0 (invalid values produce zero)
- Test: `cosine_nan_returns_zero` in `src/memory/vector.rs`

#### Scenario: Cosine similarity with infinity values

- WHEN a vector containing infinity is compared
- THEN MUST return 0.0 or a finite value (no propagation of infinity)
- Test: `cosine_infinity_returns_zero_or_finite` in `src/memory/vector.rs`

#### Scenario: Cosine similarity with negative values

- WHEN vectors with negative values are compared
- THEN MUST compute correct cosine similarity
- Test: `cosine_negative_values` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of opposite vectors is clamped

- WHEN two opposite vectors are compared
- THEN MUST return a clamped value (at or near -1.0, clamped to 0.0 if range is [0,1])
- Test: `cosine_opposite_vectors_clamped` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of high-dimensional vectors

- WHEN high-dimensional vectors are compared
- THEN MUST compute correctly without overflow
- Test: `cosine_high_dimensional` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of single-element vectors

- WHEN single-element vectors are compared
- THEN MUST return correct similarity
- Test: `cosine_single_element` in `src/memory/vector.rs`

#### Scenario: Cosine similarity of both-zero vectors

- WHEN two zero vectors are compared
- THEN MUST return 0.0
- Test: `cosine_both_zero_vectors` in `src/memory/vector.rs`

#### Scenario: Vector bytes round-trip

- WHEN a vector is serialized to bytes and deserialized back
- THEN MUST produce the original vector
- Test: `vec_bytes_roundtrip` in `src/memory/vector.rs`

#### Scenario: Vector bytes empty

- WHEN an empty vector is serialized and deserialized
- THEN MUST produce an empty vector
- Test: `vec_bytes_empty` in `src/memory/vector.rs`

#### Scenario: Bytes to vec with non-aligned input truncates

- WHEN a byte slice whose length is not a multiple of 4 is deserialized
- THEN MUST truncate to the nearest valid float boundary
- Test: `bytes_to_vec_non_aligned_truncates` in `src/memory/vector.rs`

#### Scenario: Bytes to vec with three bytes returns empty

- WHEN a byte slice of length 3 (less than one f32) is deserialized
- THEN MUST return an empty vector
- Test: `bytes_to_vec_three_bytes_returns_empty` in `src/memory/vector.rs`

#### Scenario: Vector bytes round-trip preserves special values

- WHEN vectors with special float values (infinity, subnormals) are serialized and deserialized
- THEN MUST preserve all values exactly
- Test: `vec_bytes_roundtrip_special_values` in `src/memory/vector.rs`

#### Scenario: Vector bytes round-trip preserves NaN bit patterns

- WHEN a vector with NaN is serialized and deserialized
- THEN MUST preserve the NaN bit pattern
- Test: `vec_bytes_roundtrip_nan_preserves_bits` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with vector-only results

- WHEN hybrid_merge is called with vector results but no keyword results
- THEN MUST return results ranked by vector score
- Test: `hybrid_merge_vector_only` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with keyword-only results

- WHEN hybrid_merge is called with keyword results but no vector results
- THEN MUST return results ranked by keyword score
- Test: `hybrid_merge_keyword_only` in `src/memory/vector.rs`

#### Scenario: Hybrid merge deduplicates

- WHEN hybrid_merge receives the same entry from both sources
- THEN MUST deduplicate and combine scores
- Test: `hybrid_merge_deduplicates` in `src/memory/vector.rs`

#### Scenario: Hybrid merge respects limit

- WHEN hybrid_merge is called with a limit
- THEN MUST return at most that many results
- Test: `hybrid_merge_respects_limit` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with empty inputs

- WHEN hybrid_merge is called with empty inputs for both sources
- THEN MUST return an empty vector
- Test: `hybrid_merge_empty_inputs` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with limit zero

- WHEN hybrid_merge is called with limit zero
- THEN MUST return an empty vector
- Test: `hybrid_merge_limit_zero` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with zero weights

- WHEN hybrid_merge is called with zero weights
- THEN MUST handle gracefully (no division by zero)
- Test: `hybrid_merge_zero_weights` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with negative keyword scores

- WHEN hybrid_merge receives entries with negative BM25 scores
- THEN MUST handle normalization correctly
- Test: `hybrid_merge_negative_keyword_scores` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with duplicate IDs in same source

- WHEN the same source contains duplicate IDs
- THEN MUST deduplicate within the source
- Test: `hybrid_merge_duplicate_ids_in_same_source` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with large BM25 normalization

- WHEN hybrid_merge receives very large BM25 scores
- THEN MUST normalize correctly without overflow
- Test: `hybrid_merge_large_bm25_normalization` in `src/memory/vector.rs`

#### Scenario: Hybrid merge with single item

- WHEN hybrid_merge receives a single item
- THEN MUST return that item with correct score
- Test: `hybrid_merge_single_item` in `src/memory/vector.rs`

---

### REQ-MEM-013: Response Cache

`ResponseCache` MUST provide LRU-based response caching with TTL expiration, deterministic cache keys, hit counting, and token savings tracking.

#### Scenario: Cache key is deterministic

- WHEN cache_key is called with the same inputs twice
- THEN MUST return identical keys
- Test: `cache_key_deterministic` in `src/memory/response_cache.rs`

#### Scenario: Cache key varies by model

- WHEN cache_key is called with different model names but same prompt
- THEN MUST return different keys
- Test: `cache_key_varies_by_model` in `src/memory/response_cache.rs`

#### Scenario: Cache key varies by system prompt

- WHEN cache_key is called with different system prompts but same user prompt
- THEN MUST return different keys
- Test: `cache_key_varies_by_system_prompt` in `src/memory/response_cache.rs`

#### Scenario: Cache key varies by prompt

- WHEN cache_key is called with different user prompts but same model
- THEN MUST return different keys
- Test: `cache_key_varies_by_prompt` in `src/memory/response_cache.rs`

#### Scenario: Put and get

- WHEN a response is cached and then retrieved
- THEN MUST return the cached response
- Test: `put_and_get` in `src/memory/response_cache.rs`

#### Scenario: Cache miss returns None

- WHEN a non-cached key is looked up
- THEN MUST return None
- Test: `miss_returns_none` in `src/memory/response_cache.rs`

#### Scenario: Expired entry returns None

- WHEN a cached entry has expired past its TTL
- THEN MUST return None
- Test: `expired_entry_returns_none` in `src/memory/response_cache.rs`

#### Scenario: Hit count is incremented

- WHEN a cached entry is accessed multiple times
- THEN the hit count MUST increment
- Test: `hit_count_incremented` in `src/memory/response_cache.rs`

#### Scenario: Tokens saved is calculated

- WHEN cached entries are accessed
- THEN the tokens_saved stat MUST reflect the cumulative token savings
- Test: `tokens_saved_calculated` in `src/memory/response_cache.rs`

#### Scenario: LRU eviction when capacity exceeded

- WHEN the cache exceeds max_entries
- THEN MUST evict the least-recently-used entry
- Test: `lru_eviction` in `src/memory/response_cache.rs`

#### Scenario: Clear wipes all entries

- WHEN clear is called
- THEN MUST remove all cached entries and return the count removed
- Test: `clear_wipes_all` in `src/memory/response_cache.rs`

#### Scenario: Stats on empty cache

- WHEN stats is called on an empty cache
- THEN MUST return zero for all counters
- Test: `stats_empty_cache` in `src/memory/response_cache.rs`

#### Scenario: Overwrite same key

- WHEN a new response is cached with an existing key
- THEN MUST overwrite the previous response
- Test: `overwrite_same_key` in `src/memory/response_cache.rs`

#### Scenario: Unicode prompt handling

- WHEN a prompt with unicode characters is used as cache key input
- THEN MUST produce a valid key and store/retrieve correctly
- Test: `unicode_prompt_handling` in `src/memory/response_cache.rs`

#### Scenario: LRU eviction keeps most recent

- WHEN multiple entries are cached and eviction occurs
- THEN MUST retain the most recently accessed entries
- Test: `lru_eviction_keeps_most_recent` in `src/memory/response_cache.rs`

#### Scenario: Cache handles zero max_entries

- WHEN the cache is created with max_entries of zero
- THEN MUST handle gracefully (no panics, entries may be immediately evicted)
- Test: `cache_handles_zero_max_entries` in `src/memory/response_cache.rs`

#### Scenario: Concurrent reads do not panic

- WHEN multiple concurrent reads are performed
- THEN MUST complete without panics
- Test: `cache_concurrent_reads_no_panic` in `src/memory/response_cache.rs`

---

### REQ-MEM-014: Memory Factory

Memory factory functions MUST construct the correct backend from configuration, resolve embedding configurations, and handle migration scenarios.

#### Scenario: Factory creates SQLite backend

- WHEN config specifies "sqlite" backend
- THEN MUST construct a SqliteMemory instance
- Test: `factory_sqlite` in `src/memory/mod.rs`

#### Scenario: Factory creates Markdown backend

- WHEN config specifies "markdown" backend
- THEN MUST construct a MarkdownMemory instance
- Test: `factory_markdown` in `src/memory/mod.rs`

#### Scenario: Factory creates Lucid backend

- WHEN config specifies "lucid" backend
- THEN MUST construct a LucidMemory instance
- Test: `factory_lucid` in `src/memory/mod.rs`

#### Scenario: Factory creates Cortex-Mem backend

- WHEN config specifies "cortex-mem" backend
- THEN MUST construct a CortexMemMemory instance
- Test: `factory_cortex_mem` in `src/memory/mod.rs`

#### Scenario: Factory creates SQLite+Qdrant hybrid backend

- WHEN config specifies "sqlite_qdrant_hybrid" backend
- THEN MUST construct a SqliteQdrantHybridMemory instance
- Test: `factory_sqlite_qdrant_hybrid` in `src/memory/mod.rs`

#### Scenario: Factory creates None backend

- WHEN config specifies "none" backend
- THEN MUST construct a NoneMemory instance
- Test: `factory_none_uses_noop_memory` in `src/memory/mod.rs`

#### Scenario: Factory falls back to Markdown for unknown backend

- WHEN config specifies an unknown backend string
- THEN MUST fall back to MarkdownMemory
- Test: `factory_unknown_falls_back_to_markdown` in `src/memory/mod.rs`

#### Scenario: Migration factory creates Lucid

- WHEN migration factory is called with "lucid"
- THEN MUST create a LucidMemory for migration
- Test: `migration_factory_lucid` in `src/memory/mod.rs`

#### Scenario: Migration factory creates Cortex-Mem

- WHEN migration factory is called with "cortex-mem"
- THEN MUST create a CortexMemMemory for migration
- Test: `migration_factory_cortex_mem` in `src/memory/mod.rs`

#### Scenario: Migration factory rejects None backend

- WHEN migration factory is called with "none"
- THEN MUST reject with an error (cannot migrate to/from None)
- Test: `migration_factory_none_is_rejected` in `src/memory/mod.rs`

#### Scenario: Effective backend name prefers storage override

- WHEN both base config and storage override specify backend names
- THEN MUST prefer the storage override
- Test: `effective_backend_name_prefers_storage_override` in `src/memory/mod.rs`

#### Scenario: Factory rejects Postgres without database URL

- WHEN config specifies "postgres" backend without a database URL
- THEN MUST reject with a configuration error
- Test: `factory_postgres_without_db_url_is_rejected` in `src/memory/mod.rs`

#### Scenario: Factory rejects hybrid without Qdrant URL

- WHEN config specifies "sqlite_qdrant_hybrid" without a Qdrant URL
- THEN MUST reject with a configuration error
- Test: `factory_hybrid_requires_qdrant_url` in `src/memory/mod.rs`

#### Scenario: Resolve embedding config uses base config when model is not a hint

- WHEN embedding config is resolved and the model is a literal name (not a routing hint)
- THEN MUST use the base embedding config directly
- Test: `resolve_embedding_config_uses_base_config_when_model_is_not_hint` in `src/memory/mod.rs`

#### Scenario: Resolve embedding config uses matching route with API key override

- WHEN embedding config is resolved with a routing hint that matches a route
- THEN MUST use the route's base URL and API key
- Test: `resolve_embedding_config_uses_matching_route_with_api_key_override` in `src/memory/mod.rs`

#### Scenario: Resolve embedding config falls back when hint is missing

- WHEN embedding config is resolved with a routing hint that has no matching route
- THEN MUST fall back to the base embedding config
- Test: `resolve_embedding_config_falls_back_when_hint_is_missing` in `src/memory/mod.rs`

#### Scenario: Resolve embedding config falls back when route is invalid

- WHEN embedding config is resolved with a routing hint that matches an invalid route
- THEN MUST fall back to the base embedding config
- Test: `resolve_embedding_config_falls_back_when_route_is_invalid` in `src/memory/mod.rs`

#### Scenario: Assistant autosave key detection matches legacy patterns

- WHEN `is_assistant_autosave_key` is called with known legacy key patterns
- THEN MUST return true for matching patterns and false for non-matching ones
- Test: `assistant_autosave_key_detection_matches_legacy_patterns` in `src/memory/mod.rs`

---

### REQ-MEM-015: Snapshots

Snapshot functions MUST support export and import of memory entries with round-trip fidelity.

#### Scenario: Parse snapshot basic format

- WHEN a well-formed snapshot string is parsed
- THEN MUST extract all key-content pairs correctly
- Test: `parse_snapshot_basic` in `src/memory/snapshot.rs`

#### Scenario: Parse empty snapshot

- WHEN an empty string is parsed as snapshot
- THEN MUST return an empty vector
- Test: `parse_snapshot_empty` in `src/memory/snapshot.rs`

#### Scenario: Parse snapshot with multiline content

- WHEN a snapshot entry has multiline content
- THEN MUST preserve the full multiline content in the parsed result
- Test: `parse_snapshot_multiline_content` in `src/memory/snapshot.rs`

#### Scenario: Export with no database returns zero

- WHEN export is called on a workspace with no database
- THEN MUST return zero entries exported
- Test: `export_no_db_returns_zero` in `src/memory/snapshot.rs`

#### Scenario: Export and hydrate round-trip

- WHEN entries are exported to a snapshot and then hydrated back
- THEN MUST preserve all entry content through the round-trip
- Test: `export_and_hydrate_roundtrip` in `src/memory/snapshot.rs`

#### Scenario: should_hydrate only when needed

- WHEN should_hydrate is called
- THEN MUST return true only when a snapshot file exists and no database is present
- Test: `should_hydrate_only_when_needed` in `src/memory/snapshot.rs`

#### Scenario: Hydrate with no snapshot returns zero

- WHEN hydrate is called on a workspace with no snapshot file
- THEN MUST return zero entries hydrated
- Test: `hydrate_no_snapshot_returns_zero` in `src/memory/snapshot.rs`

---

### REQ-MEM-016: Memory Hygiene

Hygiene functions MUST perform scheduled cleanup (archive/purge) of old entries based on configured cadence and retention policies.

#### Scenario: Archives old daily memory files

- WHEN daily memory files are older than the archive threshold
- THEN MUST move them to the archive directory
- Test: `archives_old_daily_memory_files` in `src/memory/hygiene.rs`

#### Scenario: Archives old session files

- WHEN session files are older than the archive threshold
- THEN MUST move them to the archive directory
- Test: `archives_old_session_files` in `src/memory/hygiene.rs`

#### Scenario: Skips second run within cadence window

- WHEN run_if_due is called twice within the configured cadence window
- THEN MUST skip the second run
- Test: `skips_second_run_within_cadence_window` in `src/memory/hygiene.rs`

#### Scenario: Purges old memory archives

- WHEN archived memory files are older than the purge threshold
- THEN MUST delete them permanently
- Test: `purges_old_memory_archives` in `src/memory/hygiene.rs`

#### Scenario: Prunes old conversation rows in SQLite

- WHEN conversation rows in SQLite are older than the retention period
- THEN MUST delete them from the database
- Test: `prunes_old_conversation_rows_in_sqlite_backend` in `src/memory/hygiene.rs`

---

### REQ-MEM-017: Memory CLI

CLI command handler MUST support list, get, stats, clear, and reindex operations with correct category parsing and content formatting.

#### Scenario: Parse category maps known variants

- WHEN parse_category is called with known category strings (core, daily, conversation)
- THEN MUST return the corresponding typed MemoryCategory variant
- Test: `parse_category_known_variants` in `src/memory/cli.rs`

#### Scenario: Parse category falls back to Custom for unknown strings

- WHEN parse_category is called with an unknown category string
- THEN MUST return MemoryCategory::Custom with the string value
- Test: `parse_category_custom_fallback` in `src/memory/cli.rs`

#### Scenario: Truncate content leaves short text unchanged

- WHEN truncate_content is called with text shorter than the max length
- THEN MUST return the text unchanged
- Test: `truncate_content_short_text_unchanged` in `src/memory/cli.rs`

#### Scenario: Truncate content truncates long text

- WHEN truncate_content is called with text longer than the max length
- THEN MUST truncate and add ellipsis
- Test: `truncate_content_long_text_truncated` in `src/memory/cli.rs`

#### Scenario: Truncate content uses first line for multiline text

- WHEN truncate_content is called with multiline text
- THEN MUST use only the first line
- Test: `truncate_content_multiline_uses_first_line` in `src/memory/cli.rs`

#### Scenario: Truncate content handles empty string

- WHEN truncate_content is called with an empty string
- THEN MUST return an empty string
- Test: `truncate_content_empty_string` in `src/memory/cli.rs`

---

### REQ-MEM-018: Backend Profile Classification

Backend profile functions MUST classify backend strings into typed kinds, provide metadata profiles for each backend, and maintain a stable selection order for onboarding.

#### Scenario: Classify known backend strings

- WHEN classify_memory_backend is called with known backend strings (sqlite, lucid, postgres, etc.)
- THEN MUST return the correct MemoryBackendKind variant
- Test: `classify_known_backends` in `src/memory/backend.rs`

#### Scenario: Classify unknown backend string

- WHEN classify_memory_backend is called with an unrecognized string
- THEN MUST return MemoryBackendKind::Unknown
- Test: `classify_unknown_backend` in `src/memory/backend.rs`

#### Scenario: Hybrid profile is SQLite-based

- WHEN the profile for sqlite_qdrant_hybrid is retrieved
- THEN MUST report sqlite_based=true and uses_sqlite_hygiene=true
- Test: `hybrid_profile_is_sqlite_based` in `src/memory/backend.rs`

#### Scenario: Selectable backends are ordered for onboarding

- WHEN selectable_memory_backends is called
- THEN MUST return backends in the expected onboarding presentation order
- Test: `selectable_backends_are_ordered_for_onboarding` in `src/memory/backend.rs`

#### Scenario: Lucid profile is SQLite-based optional backend

- WHEN the profile for lucid is retrieved
- THEN MUST report sqlite_based=true, optional_dependency=true, uses_sqlite_hygiene=true
- Test: `lucid_profile_is_sqlite_based_optional_backend` in `src/memory/backend.rs`

#### Scenario: Cortex profile is SQLite-based optional backend

- WHEN the profile for cortex-mem is retrieved
- THEN MUST report sqlite_based=true, optional_dependency=true, uses_sqlite_hygiene=true
- Test: `cortex_profile_is_sqlite_based_optional_backend` in `src/memory/backend.rs`

#### Scenario: Unknown profile preserves extensibility defaults

- WHEN the profile for an unknown backend is retrieved
- THEN MUST return the custom profile with auto_save_default=true and uses_sqlite_hygiene=false
- Test: `unknown_profile_preserves_extensibility_defaults` in `src/memory/backend.rs`

---

## Coverage Notes

Total explicit test count: **234 tests** across **18 source files**.

| Requirement | File(s) | Test Count |
|---|---|---|
| REQ-MEM-001 | `traits.rs` | 5 |
| REQ-MEM-002 | `sqlite.rs` | 69 |
| REQ-MEM-003 | `markdown.rs` | 11 |
| REQ-MEM-004 | `lucid.rs` | 6 |
| REQ-MEM-005 | `hybrid.rs` | 3 |
| REQ-MEM-006 | `cortex.rs` | 2 |
| REQ-MEM-007 | `none.rs` | 1 |
| REQ-MEM-008 | `postgres.rs` | 4 |
| REQ-MEM-009 | `qdrant.rs` | 4 |
| REQ-MEM-010 | `embeddings.rs` | 20 |
| REQ-MEM-011 | `chunker.rs` | 19 |
| REQ-MEM-012 | `vector.rs` | 30 |
| REQ-MEM-013 | `response_cache.rs` | 17 |
| REQ-MEM-014 | `mod.rs` | 18 |
| REQ-MEM-015 | `snapshot.rs` | 7 |
| REQ-MEM-016 | `hygiene.rs` | 5 |
| REQ-MEM-017 | `cli.rs` | 6 |
| REQ-MEM-018 | `backend.rs` | 7 |
| **Total** | **18 files** | **234** |

## Mock Strategy

- SQLite: `tempfile::TempDir` for database isolation
- Markdown: `tempfile::TempDir` for file isolation
- PostgreSQL/Qdrant: Feature-gated; unit tests validate config parsing only
- Embeddings: Mock embedding provider returning fixed vectors
- Cortex: Mock cortex-mem binary or test helper constructor
- Lucid: Fake/delayed/failing shell scripts written to tempdir for CLI simulation
- Hybrid: `MockMemory` struct with configurable fail/recall behavior
- Response Cache: `tempfile::TempDir` for database isolation with configurable TTL
