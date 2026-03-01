# Security Auth & Detection Specification

## Purpose
Define the behavioral contract for ZeroClaw's authentication, secret management, and threat detection subsystems: OTP validation, device pairing, encrypted secret storage, credential leak detection, prompt injection defense, syscall anomaly detection, file link guards, sensitive path detection, domain matching, adversarial suffix detection, and audit logging.

## Scope
- Files: `src/security/otp.rs` (6 tests), `src/security/pairing.rs` (33 tests), `src/security/secrets.rs` (40 tests), `src/security/leak_detector.rs` (14 tests), `src/security/prompt_guard.rs` (10 tests), `src/security/syscall_anomaly.rs` (11 tests), `src/security/file_link_guard.rs` (3 tests), `src/security/sensitive_paths.rs` (5 tests), `src/security/domain_matcher.rs` (7 tests), `src/security/perplexity.rs` (4 tests), `src/security/audit.rs` (9 tests)
- Total tests: 142
- Risk tier: HIGH (authentication and threat detection boundary)

## Requirements

---

### OTP Validation (`src/security/otp.rs`)

### REQ-SEC-OTP-001: Valid TOTP codes MUST be accepted within time window
Codes for current and adjacent time steps (+/-1) MUST be accepted.

#### Scenario: Valid code accepted
- WHEN validate_at(code, timestamp) is called with current-step code
- THEN it returns Ok(true)
- Test: `valid_totp_code_is_accepted` in `src/security/otp.rs`

### REQ-SEC-OTP-002: Replayed TOTP codes MUST be rejected
A code used once MUST be cached and rejected on subsequent use within cache validity period.

#### Scenario: Replay rejected
- WHEN the same valid code is validated twice
- THEN first returns Ok(true), second returns Ok(false)
- Test: `replayed_totp_code_is_rejected` in `src/security/otp.rs`

### REQ-SEC-OTP-003: Expired TOTP codes MUST be rejected
Codes from far-past time steps MUST not validate.

#### Scenario: Expired code
- WHEN validate_at(stale_code, now + 300s) is called
- THEN it returns Ok(false)
- Test: `expired_totp_code_is_rejected` in `src/security/otp.rs`

### REQ-SEC-OTP-003A: Wrong TOTP codes MUST be rejected
Codes that do not match the expected value MUST not validate.

#### Scenario: Wrong code
- WHEN an incorrect code is submitted
- THEN it returns Ok(false)
- Test: `wrong_totp_code_is_rejected` in `src/security/otp.rs`

### REQ-SEC-OTP-004: OTP secret MUST be generated on first use and reused on reload
First init generates secret + URI; subsequent loads reuse the encrypted secret.

#### Scenario: Secret persistence
- WHEN OtpValidator is created twice from same dir
- THEN first returns Some(uri), second returns None, codes match
- Test: `secret_is_generated_and_reused` in `src/security/otp.rs`

### REQ-SEC-OTP-005: otpauth URI MUST contain correct issuer and format
The URI MUST follow otpauth://totp/ZeroClaw:zeroclaw?secret=...&issuer=ZeroClaw&period=N format.

#### Scenario: URI format
- WHEN otpauth_uri() is called
- THEN it contains "otpauth://totp/ZeroClaw" and "issuer=ZeroClaw"
- Test: `otpauth_uri_format_is_correct` in `src/security/otp.rs`

---

### Device Pairing (`src/security/pairing.rs`)

### REQ-SEC-PAIR-001: PairingGuard MUST generate a code when no tokens exist
When pairing is required and no prior tokens exist, a one-time code MUST be generated.

#### Scenario: New guard generates code
- WHEN PairingGuard is created with require_pairing=true and no existing tokens
- THEN pairing_code() returns Some(code) with 6 digits
- Test: `new_guard_generates_code_when_no_tokens` in `src/security/pairing.rs`

### REQ-SEC-PAIR-002: PairingGuard MUST NOT generate code when tokens exist
When tokens already exist, no pairing code is needed.

#### Scenario: Tokens suppress code
- WHEN PairingGuard is created with existing tokens
- THEN pairing_code() returns None
- Test: `new_guard_no_code_when_tokens_exist` in `src/security/pairing.rs`

### REQ-SEC-PAIR-003: PairingGuard MUST NOT generate code when pairing is disabled
When require_pairing=false, no code should be generated.

#### Scenario: Disabled pairing
- WHEN PairingGuard is created with require_pairing=false
- THEN pairing_code() returns None
- Test: `new_guard_no_code_when_pairing_disabled` in `src/security/pairing.rs`

### REQ-SEC-PAIR-004: Correct pairing code MUST yield a bearer token
Submitting the correct one-time code MUST produce a persistent bearer token.

#### Scenario: Correct code pairs
- WHEN try_pair(correct_code) is called
- THEN it returns Ok(Some(token))
- Test: `try_pair_correct_code` in `src/security/pairing.rs`

### REQ-SEC-PAIR-005: Wrong pairing code MUST be rejected
Incorrect codes MUST be rejected without yielding a token.

#### Scenario: Wrong code rejected
- WHEN try_pair(wrong_code) is called
- THEN it returns Ok(None)
- Test: `try_pair_wrong_code` in `src/security/pairing.rs`

#### Scenario: Empty code rejected
- WHEN try_pair("") is called
- THEN it returns Err (lockout) or Ok(None)
- Test: `try_pair_empty_code` in `src/security/pairing.rs`

### REQ-SEC-PAIR-006: Bearer token authentication MUST validate correctly
Valid tokens MUST be accepted; invalid tokens MUST be rejected.

#### Scenario: Valid token accepted
- WHEN is_authenticated(valid_token) is called
- THEN it returns true
- Test: `is_authenticated_with_valid_token` in `src/security/pairing.rs`

#### Scenario: Pre-hashed token accepted
- WHEN is_authenticated is called with a pre-hashed token
- THEN it returns true
- Test: `is_authenticated_with_prehashed_token` in `src/security/pairing.rs`

#### Scenario: Invalid token rejected
- WHEN is_authenticated(invalid_token) is called
- THEN it returns false
- Test: `is_authenticated_with_invalid_token` in `src/security/pairing.rs`

#### Scenario: Pairing disabled allows all
- WHEN require_pairing=false and is_authenticated(any_token) is called
- THEN it returns true
- Test: `is_authenticated_when_pairing_disabled` in `src/security/pairing.rs`

### REQ-SEC-PAIR-007: Token storage MUST return hashed values
tokens() MUST return SHA-256 hashed versions of bearer tokens.

#### Scenario: Tokens returns hashes
- WHEN tokens() is called after pairing
- THEN returned values are 64-char hex strings
- Test: `tokens_returns_hashes` in `src/security/pairing.rs`

### REQ-SEC-PAIR-008: Pair-then-authenticate MUST work end-to-end
A token obtained through pairing MUST authenticate successfully.

#### Scenario: Full flow
- WHEN pair produces a token and is_authenticated checks it
- THEN authentication succeeds
- Test: `pair_then_authenticate` in `src/security/pairing.rs`

### REQ-SEC-PAIR-009: Device management MUST support listing and revocation
Paired devices MUST be listable and individually revocable.

#### Scenario: Device roundtrip
- WHEN devices are paired, listed, and revoked
- THEN listing shows devices and revocation removes the correct one
- Test: `paired_devices_and_revoke_device_roundtrip` in `src/security/pairing.rs`

### REQ-SEC-PAIR-010: Authentication MUST update device last-seen timestamp
Legacy devices without metadata MUST get timestamps updated on authentication.

#### Scenario: Last seen update
- WHEN a legacy device authenticates
- THEN its last_seen timestamp is updated
- Test: `authenticate_updates_legacy_device_last_seen` in `src/security/pairing.rs`

### REQ-SEC-PAIR-011: Token hashing MUST be deterministic and collision-resistant

#### Scenario: Hash length and format
- WHEN hash_token is called
- THEN it produces a 64-char hex string
- Test: `hash_token_produces_64_hex_chars` in `src/security/pairing.rs`

#### Scenario: Hash determinism
- WHEN hash_token is called twice with same input
- THEN results are identical
- Test: `hash_token_is_deterministic` in `src/security/pairing.rs`

#### Scenario: Hash uniqueness
- WHEN hash_token is called with different inputs
- THEN results differ
- Test: `hash_token_differs_for_different_inputs` in `src/security/pairing.rs`

### REQ-SEC-PAIR-012: is_token_hash MUST distinguish hashes from plaintext

#### Scenario: Hash detection
- WHEN is_token_hash is called with hash vs plaintext
- THEN it correctly distinguishes them
- Test: `is_token_hash_detects_hash_vs_plaintext` in `src/security/pairing.rs`

### REQ-SEC-PAIR-013: Public bind detection MUST classify network addresses

#### Scenario: Localhost is not public
- WHEN is_public_bind is called with localhost variants
- THEN it returns false
- Test: `localhost_variants_not_public` in `src/security/pairing.rs`

#### Scenario: 0.0.0.0 is public
- WHEN is_public_bind("0.0.0.0") is called
- THEN it returns true
- Test: `zero_zero_is_public` in `src/security/pairing.rs`

#### Scenario: Real IP is public
- WHEN is_public_bind is called with a real IP
- THEN it returns true
- Test: `real_ip_is_public` in `src/security/pairing.rs`

### REQ-SEC-PAIR-014: Constant-time equality MUST compare strings safely

#### Scenario: Same strings
- WHEN constant_time_eq("abc", "abc") is called
- THEN it returns true
- Test: `constant_time_eq_same` in `src/security/pairing.rs`

#### Scenario: Different strings
- WHEN constant_time_eq("abc", "abd") is called
- THEN it returns false
- Test: `constant_time_eq_different` in `src/security/pairing.rs`

### REQ-SEC-PAIR-015: Code and token generation MUST use cryptographic randomness

#### Scenario: Code format
- WHEN generate_code() is called
- THEN it returns a 6-digit numeric string
- Test: `generate_code_is_6_digits` in `src/security/pairing.rs`

#### Scenario: Code non-determinism
- WHEN generate_code() is called multiple times
- THEN results differ with overwhelming probability
- Test: `generate_code_is_not_deterministic` in `src/security/pairing.rs`

#### Scenario: Token format
- WHEN generate_token() is called
- THEN it returns a prefixed hex payload string
- Test: `generate_token_has_prefix_and_hex_payload` in `src/security/pairing.rs`

### REQ-SEC-PAIR-016: Brute force protection MUST lock out after max attempts

#### Scenario: Lockout after max attempts
- WHEN max failed pairing attempts are exceeded
- THEN try_pair returns Err with remaining lockout seconds
- Test: `brute_force_lockout_after_max_attempts` in `src/security/pairing.rs`

#### Scenario: Correct code resets attempts
- WHEN a correct code is submitted after failed attempts
- THEN the failure counter resets
- Test: `correct_code_resets_failed_attempts` in `src/security/pairing.rs`

#### Scenario: Lockout returns remaining seconds
- WHEN locked out, try_pair returns Err with remaining seconds
- THEN the value reflects time remaining
- Test: `lockout_returns_remaining_seconds` in `src/security/pairing.rs`

#### Scenario: Successful pair resets only requesting client
- WHEN one client succeeds, another client's failure state is unaffected
- THEN per-client lockout is isolated
- Test: `successful_pair_resets_only_requesting_client_state` in `src/security/pairing.rs`

#### Scenario: Failed attempt state is bounded
- WHEN many unique clients fail
- THEN the tracked state is bounded by max_clients
- Test: `failed_attempt_state_is_bounded_by_max_clients` in `src/security/pairing.rs`

#### Scenario: Expired client states are pruned
- WHEN client states expire
- THEN they are cleaned up on subsequent operations
- Test: `failed_attempt_sweep_prunes_expired_clients` in `src/security/pairing.rs`

#### Scenario: Lockout is per-client
- WHEN one client is locked out
- THEN another client can still attempt pairing
- Test: `lockout_is_per_client` in `src/security/pairing.rs`

---

### Secret Store (`src/security/secrets.rs`)

### REQ-SEC-SECRET-001: Encrypt/decrypt MUST roundtrip correctly
Encrypted values MUST decrypt back to the original plaintext.

#### Scenario: Basic roundtrip
- WHEN a secret is encrypted and then decrypted
- THEN the original plaintext is recovered
- Test: `encrypt_decrypt_roundtrip` in `src/security/secrets.rs`

#### Scenario: Empty string returns empty
- WHEN an empty string is encrypted
- THEN it returns empty without encryption prefix
- Test: `encrypt_empty_returns_empty` in `src/security/secrets.rs`

#### Scenario: Plaintext passthrough on decrypt
- WHEN a non-encrypted value is decrypted
- THEN it passes through unchanged
- Test: `decrypt_plaintext_passthrough` in `src/security/secrets.rs`

#### Scenario: Disabled store returns plaintext
- WHEN the store is disabled and encrypt is called
- THEN it returns the plaintext unchanged
- Test: `disabled_store_returns_plaintext` in `src/security/secrets.rs`

### REQ-SEC-SECRET-002: Encryption detection MUST identify prefixed values

#### Scenario: Prefix detection
- WHEN is_encrypted is called with enc2: and enc: prefixed values
- THEN it returns true; for unprefixed values, false
- Test: `is_encrypted_detects_prefix` in `src/security/secrets.rs`

### REQ-SEC-SECRET-003: Key file MUST be created lazily on first encrypt

#### Scenario: Key creation
- WHEN the first encryption occurs
- THEN the key file is created on disk
- Test: `key_file_created_on_first_encrypt` in `src/security/secrets.rs`

### REQ-SEC-SECRET-004: Encryption MUST produce unique ciphertext (nonce reuse prevention)

#### Scenario: Non-deterministic encryption
- WHEN the same value is encrypted twice
- THEN the ciphertexts differ
- Test: `encrypting_same_value_produces_different_ciphertext` in `src/security/secrets.rs`

### REQ-SEC-SECRET-005: Different stores with same key MUST interoperate

#### Scenario: Cross-store interop
- WHEN two SecretStore instances share the same directory
- THEN one can decrypt what the other encrypted
- Test: `different_stores_same_dir_interop` in `src/security/secrets.rs`

### REQ-SEC-SECRET-006: Unicode and long secrets MUST roundtrip

#### Scenario: Unicode roundtrip
- WHEN a Unicode secret is encrypted and decrypted
- THEN the original Unicode string is recovered
- Test: `unicode_secret_roundtrip` in `src/security/secrets.rs`

#### Scenario: Long secret roundtrip
- WHEN a long secret (1000+ chars) is encrypted and decrypted
- THEN the original is recovered
- Test: `long_secret_roundtrip` in `src/security/secrets.rs`

### REQ-SEC-SECRET-007: Tampered or corrupt ciphertext MUST be detected

#### Scenario: Corrupt hex
- WHEN ciphertext hex is invalid
- THEN decrypt returns error
- Test: `corrupt_hex_returns_error` in `src/security/secrets.rs`

#### Scenario: Tampered ciphertext
- WHEN ciphertext bytes are modified
- THEN decrypt returns error
- Test: `tampered_ciphertext_detected` in `src/security/secrets.rs`

#### Scenario: Wrong key
- WHEN a different key is used for decryption
- THEN decrypt returns error
- Test: `wrong_key_detected` in `src/security/secrets.rs`

#### Scenario: Truncated ciphertext
- WHEN ciphertext is truncated
- THEN decrypt returns error
- Test: `truncated_ciphertext_returns_error` in `src/security/secrets.rs`

### REQ-SEC-SECRET-008: Legacy XOR format MUST still decrypt for backward compat

#### Scenario: Legacy decrypt
- WHEN a legacy enc: prefixed value is decrypted
- THEN the original plaintext is recovered
- Test: `legacy_xor_decrypt_still_works` in `src/security/secrets.rs`

### REQ-SEC-SECRET-009: Migration detection and execution MUST work

#### Scenario: Migration detection
- WHEN needs_migration is called with enc: and enc2: values
- THEN enc: returns true, enc2: returns false
- Test: `needs_migration_detects_legacy_prefix` in `src/security/secrets.rs`

#### Scenario: Secure prefix detection
- WHEN is_secure_encrypted is called
- THEN only enc2: returns true
- Test: `is_secure_encrypted_detects_enc2_only` in `src/security/secrets.rs`

#### Scenario: decrypt_and_migrate with enc2 (no migration needed)
- WHEN decrypt_and_migrate is called with enc2: value
- THEN returns (plaintext, None)
- Test: `decrypt_and_migrate_returns_none_for_enc2` in `src/security/secrets.rs`

#### Scenario: decrypt_and_migrate with plaintext (no migration needed)
- WHEN decrypt_and_migrate is called with plaintext
- THEN returns (plaintext, None)
- Test: `decrypt_and_migrate_returns_none_for_plaintext` in `src/security/secrets.rs`

#### Scenario: decrypt_and_migrate upgrades legacy XOR
- WHEN decrypt_and_migrate is called with enc: value
- THEN returns (plaintext, Some(enc2:...))
- Test: `decrypt_and_migrate_upgrades_legacy_xor` in `src/security/secrets.rs`

#### Scenario: Migration handles Unicode
- WHEN decrypt_and_migrate is called with Unicode enc: value
- THEN migration succeeds
- Test: `decrypt_and_migrate_handles_unicode` in `src/security/secrets.rs`

#### Scenario: Migration handles empty secret
- WHEN decrypt_and_migrate is called with empty enc: value
- THEN migration succeeds
- Test: `decrypt_and_migrate_handles_empty_secret` in `src/security/secrets.rs`

#### Scenario: Migration handles long secret
- WHEN decrypt_and_migrate is called with long enc: value
- THEN migration succeeds
- Test: `decrypt_and_migrate_handles_long_secret` in `src/security/secrets.rs`

#### Scenario: Migration fails on corrupt legacy hex
- WHEN decrypt_and_migrate is called with corrupt enc: value
- THEN returns error
- Test: `decrypt_and_migrate_fails_on_corrupt_legacy_hex` in `src/security/secrets.rs`

#### Scenario: Migration with wrong key
- WHEN decrypt_and_migrate is called with a different store's key
- THEN produces garbage or fails
- Test: `decrypt_and_migrate_wrong_key_produces_garbage_or_fails` in `src/security/secrets.rs`

#### Scenario: Migration produces unique ciphertext
- WHEN migration runs twice on same input
- THEN migrated values differ (nonce uniqueness)
- Test: `migration_produces_different_ciphertext_each_time` in `src/security/secrets.rs`

#### Scenario: Migrated value is tamper-resistant
- WHEN a migrated enc2: value is tampered
- THEN decrypt fails
- Test: `migrated_value_is_tamper_resistant` in `src/security/secrets.rs`

### REQ-SEC-SECRET-010: Internal crypto primitives MUST be correct

#### Scenario: XOR cipher roundtrip
- WHEN xor_cipher is applied twice with same key
- THEN original data is recovered
- Test: `xor_cipher_roundtrip` in `src/security/secrets.rs`

#### Scenario: XOR cipher with empty key
- WHEN xor_cipher is called with empty key
- THEN data passes through unchanged
- Test: `xor_cipher_empty_key` in `src/security/secrets.rs`

#### Scenario: Hex encode/decode roundtrip
- WHEN data is hex encoded and decoded
- THEN original bytes are recovered
- Test: `hex_roundtrip` in `src/security/secrets.rs`

#### Scenario: Hex decode odd length fails
- WHEN hex_decode is called with odd-length string
- THEN it returns error
- Test: `hex_decode_odd_length_fails` in `src/security/secrets.rs`

#### Scenario: Hex decode invalid chars fails
- WHEN hex_decode is called with non-hex chars
- THEN it returns error
- Test: `hex_decode_invalid_chars_fails` in `src/security/secrets.rs`

### REQ-SEC-SECRET-011: Windows icacls helper MUST validate and format usernames

#### Scenario: Empty username rejected
- WHEN build_windows_icacls_grant_arg("") is called
- THEN it returns None
- Test: `windows_icacls_grant_arg_rejects_empty_username` in `src/security/secrets.rs`

#### Scenario: Username is trimmed
- WHEN build_windows_icacls_grant_arg(" user ") is called
- THEN it returns the trimmed value
- Test: `windows_icacls_grant_arg_trims_username` in `src/security/secrets.rs`

#### Scenario: Valid characters preserved
- WHEN build_windows_icacls_grant_arg("domain\\user") is called
- THEN it returns the value with valid characters
- Test: `windows_icacls_grant_arg_preserves_valid_characters` in `src/security/secrets.rs`

### REQ-SEC-SECRET-012: Random key generation MUST produce secure keys

#### Scenario: Key length correct
- WHEN generate_random_key is called
- THEN it returns a key of ChaCha20-Poly1305 key length
- Test: `generate_random_key_correct_length` in `src/security/secrets.rs`

#### Scenario: Key not all zeros
- WHEN generate_random_key is called
- THEN it returns non-zero bytes
- Test: `generate_random_key_not_all_zeros` in `src/security/secrets.rs`

#### Scenario: Two keys differ
- WHEN generate_random_key is called twice
- THEN results differ
- Test: `two_random_keys_differ` in `src/security/secrets.rs`

#### Scenario: No UUID fixed bits
- WHEN generate_random_key is called
- THEN it has no UUID v4 fixed bit patterns
- Test: `generate_random_key_has_no_uuid_fixed_bits` in `src/security/secrets.rs`

### REQ-SEC-SECRET-013: Key file MUST have restricted permissions on Unix

#### Scenario: Key file permissions (Unix only)
- WHEN a key file is created on Unix
- THEN its permissions are owner-only (0o600)
- Test: `key_file_has_restricted_permissions` in `src/security/secrets.rs`

---

### Leak Detection (`src/security/leak_detector.rs`)

### REQ-SEC-LEAK-001: Known API key patterns MUST be detected regardless of sensitivity
Structurally identifiable keys (Stripe, OpenAI, Anthropic, AWS, GitHub) MUST trigger at any sensitivity level.

#### Scenario: Clean content passes
- WHEN LeakDetector scans safe content
- THEN it returns Clean
- Test: `clean_content_passes` in `src/security/leak_detector.rs`

#### Scenario: Stripe key detected
- WHEN LeakDetector scans content with Stripe key
- THEN it returns Detected with patterns containing "Stripe"
- Test: `detects_stripe_keys` in `src/security/leak_detector.rs`

#### Scenario: AWS credentials detected
- WHEN LeakDetector scans content with AWS access keys
- THEN it returns Detected with patterns containing "AWS"
- Test: `detects_aws_credentials` in `src/security/leak_detector.rs`

#### Scenario: Structural keys at zero sensitivity
- WHEN sensitivity=0.0 and content has a Stripe key
- THEN it still detects
- Test: `structural_api_key_detected_regardless_of_sensitivity` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-002: Generic secret patterns MUST obey sensitivity threshold
Generic patterns (password=, secret=, token=) MUST only fire when sensitivity > 0.5.

#### Scenario: Low sensitivity skips generic
- WHEN sensitivity=0.3 and content has "secret=aaaa..."
- THEN result is Clean
- Test: `low_sensitivity_skips_generic` in `src/security/leak_detector.rs`

#### Scenario: Threshold boundary
- WHEN sensitivity == 0.5 (exact threshold)
- THEN generic rules do NOT fire
- Test: `sensitivity_at_threshold_does_not_fire_generic` in `src/security/leak_detector.rs`

#### Scenario: Just above threshold fires generic
- WHEN sensitivity is just above 0.5
- THEN generic rules fire
- Test: `sensitivity_just_above_threshold_fires_generic` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-003: PEM private keys MUST be detected and redacted
Content containing PEM key blocks MUST be flagged and the block replaced with [REDACTED_PRIVATE_KEY].

#### Scenario: Private key detection
- WHEN content contains RSA private key block
- THEN Detected with redacted containing [REDACTED_PRIVATE_KEY]
- Test: `detects_private_keys` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-004: JWT tokens MUST be detected and redacted
Content with eyJ...eyJ... pattern MUST be flagged.

#### Scenario: JWT detection
- WHEN content contains a JWT
- THEN Detected with patterns containing "JWT"
- Test: `detects_jwt_tokens` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-004A: Database connection URLs MUST be detected

#### Scenario: Database URL detection
- WHEN content contains a database connection URL
- THEN Detected with patterns containing database type
- Test: `detects_database_urls` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-005: High-entropy tokens MUST be detected above sensitivity threshold
Random-looking tokens >=24 chars with mixed alphanumeric MUST trigger when sensitivity > 0.5.

#### Scenario: High entropy token
- WHEN sensitivity=0.9 and content has random 40-char token
- THEN Detected with "High-entropy token"
- Test: `high_entropy_token_is_detected_and_redacted` in `src/security/leak_detector.rs`

#### Scenario: Natural language not flagged
- WHEN sensitivity=0.9 and content is natural text
- THEN it is not flagged as high-entropy
- Test: `natural_language_text_is_not_flagged_as_high_entropy` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-005A: Shannon entropy MUST distinguish patterns

#### Scenario: Entropy differentiation
- WHEN shannon_entropy is called on repetitive vs random data
- THEN random data has higher entropy
- Test: `shannon_entropy_distinguishes_repetitive_from_random_tokens` in `src/security/leak_detector.rs`

### REQ-SEC-LEAK-006: GitHub tokens MUST be detected
GitHub PATs and classic tokens (ghp_, gho_, ghu_, ghs_, ghr_, github_pat_) MUST be caught.

#### Scenario: GitHub token
- WHEN content contains "ghp_" followed by 36+ alphanumeric chars
- THEN Detected with patterns containing "GitHub"
- Test: `detects_github_tokens` in `src/security/leak_detector.rs`

---

### Prompt Injection Defense (`src/security/prompt_guard.rs`)

### REQ-SEC-GUARD-001: Safe messages MUST pass scan
Normal conversational messages MUST return GuardResult::Safe.

#### Scenario: Safe message
- WHEN guard.scan("What is the weather?") is called
- THEN it returns Safe
- Test: `safe_messages_pass` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-002: System override attempts MUST be detected
Patterns like "ignore previous instructions" MUST trigger detection.

#### Scenario: System override
- WHEN guard.scan("Ignore previous instructions...") is called
- THEN it returns Suspicious
- Test: `detects_system_override` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-002A: Role confusion attacks MUST be detected

#### Scenario: Role confusion
- WHEN guard.scan(role confusion pattern) is called
- THEN it returns Suspicious
- Test: `detects_role_confusion` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-002B: Secret extraction attempts MUST be detected

#### Scenario: Secret extraction
- WHEN guard.scan(secret extraction pattern) is called
- THEN it returns Suspicious
- Test: `detects_secret_extraction` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-002C: Jailbreak attempts MUST be detected

#### Scenario: Jailbreak attempts
- WHEN guard.scan(jailbreak pattern) is called
- THEN it returns Suspicious
- Test: `detects_jailbreak_attempts` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-003: Block mode MUST block when score exceeds sensitivity
When action=Block and score > sensitivity threshold, MUST return Blocked.

#### Scenario: Block mode
- WHEN guard with Block action and 0.5 sensitivity scans injection
- THEN it returns Blocked
- Test: `blocking_mode_works` in `src/security/prompt_guard.rs`

#### Scenario: High sensitivity catches more
- WHEN guard with high vs low sensitivity scans borderline injection
- THEN high-sensitivity guard catches more
- Test: `high_sensitivity_catches_more` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-004: GuardAction::from_str() MUST parse known actions
"block" -> Block, "sanitize" -> Sanitize, anything else -> Warn.

#### Scenario: Parse guard actions
- WHEN GuardAction::from_str("block"), from_str("sanitize"), from_str("unknown") are called
- THEN they return Block, Sanitize, Warn respectively
- Test: `guard_action_from_str_parses_known_values` in `src/security/prompt_guard.rs`

### REQ-SEC-GUARD-005: Static signatures MUST use Aho-Corasick for linear-time scanning
Known injection signatures MUST be detected via Aho-Corasick matcher.

#### Scenario: Static signature detection
- WHEN content contains "reveal your system prompt"
- THEN detected patterns include "aho_corasick_injection_signature"
- Test: `detects_aho_corasick_static_signatures` in `src/security/prompt_guard.rs`

#### Scenario: Large repeated payload scans in linear time
- WHEN a large repeated injection payload is scanned
- THEN it completes within reasonable time
- Test: `large_repeated_payload_scans_in_linear_time_path` in `src/security/prompt_guard.rs`

---

### File Link Guard (`src/security/file_link_guard.rs`)

### REQ-SEC-LINK-001: Single-link files MUST NOT be flagged
Regular files with nlink=1 MUST return false from has_multiple_hard_links.

#### Scenario: Single link
- WHEN has_multiple_hard_links is called on a regular file
- THEN it returns false
- Test: `single_link_file_is_not_flagged` in `src/security/file_link_guard.rs`

### REQ-SEC-LINK-002: Hard-linked files MUST be flagged
Files with nlink > 1 MUST return true from has_multiple_hard_links.

#### Scenario: Hard link detected
- WHEN a file has a hard link and has_multiple_hard_links is called
- THEN it returns true (on supported filesystems)
- Test: `hard_link_file_is_flagged_when_supported` in `src/security/file_link_guard.rs`

### REQ-SEC-LINK-003: Symlinks MUST NOT trigger hard link guard
Symlinks create separate inodes and MUST NOT be flagged by has_multiple_hard_links.

#### Scenario: Symlink not flagged
- WHEN a symlink is created and has_multiple_hard_links is checked on the target
- THEN nlink remains 1 (symlink does not increment hard link count)
- Test: `symlink_does_not_trigger_hard_link_guard` in `src/security/file_link_guard.rs`

---

### Sensitive Path Detection (`src/security/sensitive_paths.rs`)

### REQ-SEC-SENS-001: Known sensitive filenames MUST be detected
.env, .npmrc, credentials.json, id_rsa, etc. MUST return true.

#### Scenario: Exact filenames
- WHEN is_sensitive_file_path(".env") and ("credentials.json") are called
- THEN they return true
- Test: `detects_sensitive_exact_filenames` in `src/security/sensitive_paths.rs`

### REQ-SEC-SENS-002: Sensitive suffixes and path components MUST be detected
.pem, .key, .p12, and paths containing .ssh, .aws, .gnupg MUST return true.

#### Scenario: Suffixes and components
- WHEN is_sensitive_file_path("tls/cert.pem") and (".aws/credentials") are called
- THEN they return true
- Test: `detects_sensitive_suffixes_and_components` in `src/security/sensitive_paths.rs`

### REQ-SEC-SENS-003: Regular source files MUST NOT be flagged
Normal development files like .rs, .md MUST return false.

#### Scenario: Regular files
- WHEN is_sensitive_file_path("src/main.rs") is called
- THEN it returns false
- Test: `ignores_regular_paths` in `src/security/sensitive_paths.rs`

### REQ-SEC-SENS-004: .env.* variations MUST be detected
Files matching the .env.* pattern (like .env.production, .env.local) MUST be flagged.

#### Scenario: Env variations
- WHEN is_sensitive_file_path(".env.production") and (".env.local") are called
- THEN they return true
- Test: `detects_env_variations` in `src/security/sensitive_paths.rs`

### REQ-SEC-SENS-005: Detection MUST be case-insensitive
"ID_RSA" and ".ENV" MUST be detected regardless of case.

#### Scenario: Case insensitive
- WHEN is_sensitive_file_path("ID_RSA") and (".ENV") are called
- THEN they return true
- Test: `detection_is_case_insensitive` in `src/security/sensitive_paths.rs`

---

### Domain Matcher (`src/security/domain_matcher.rs`)

### REQ-SEC-DOM-001: Exact domain matches MUST work
Direct domain strings MUST match exactly.

#### Scenario: Exact match
- WHEN matcher has "accounts.google.com" and is_gated("accounts.google.com") is called
- THEN it returns true
- Test: `exact_match_works` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-002: Wildcard domain matches MUST work
"*.chase.com" MUST match "www.chase.com" but NOT "chase.com".

#### Scenario: Wildcard match
- WHEN matcher has "*.chase.com" and is_gated("www.chase.com") and is_gated("chase.com")
- THEN www returns true, bare domain returns false
- Test: `wildcard_match_works` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-003: Category presets MUST expand and match
"banking" category MUST expand to all banking domain patterns.

#### Scenario: Category expansion
- WHEN matcher is built with category "banking"
- THEN is_gated("login.paypal.com") returns true
- Test: `category_preset_expands_and_matches` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-003A: Non-matching domains MUST return false

#### Scenario: Non-matching domain
- WHEN is_gated is called with a non-matching domain
- THEN it returns false
- Test: `non_matching_domain_returns_false` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-004: Invalid patterns MUST be rejected
Patterns with spaces, consecutive dots, or invalid chars MUST fail validation.

#### Scenario: Invalid pattern
- WHEN "bad domain.com" is provided
- THEN constructor returns Err
- Test: `malformed_domain_pattern_is_rejected` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-004A: Unknown categories MUST be rejected

#### Scenario: Unknown category
- WHEN an unknown category string is provided
- THEN constructor returns Err
- Test: `unknown_category_is_rejected` in `src/security/domain_matcher.rs`

### REQ-SEC-DOM-005: URLs with ports/paths MUST normalize to bare domain
Domains extracted from full URLs (with port, path, query) MUST match patterns correctly.

#### Scenario: URL normalization
- WHEN is_gated("https://accounts.google.com:443/login?next=/") is called
- THEN it extracts "accounts.google.com" and matches
- Test: `url_with_port_and_path_normalizes` in `src/security/domain_matcher.rs`

---

### Syscall Anomaly Detection (`src/security/syscall_anomaly.rs`)

### REQ-SEC-SYSCALL-001: Syscall signal parsing MUST extract fields correctly

#### Scenario: Numeric audit syscall
- WHEN a line contains audit syscall=59
- THEN parse_syscall_signal extracts the syscall name via number mapping
- Test: `parse_syscall_signal_extracts_numeric_audit_syscall` in `src/security/syscall_anomaly.rs`

#### Scenario: Seccomp denied line
- WHEN a line contains "seccomp: denied syscall=openat"
- THEN parse_syscall_signal marks it as denied
- Test: `parse_syscall_signal_marks_denied_from_seccomp_line` in `src/security/syscall_anomaly.rs`

#### Scenario: Symbolic syscall name
- WHEN a line contains "syscall=__NR_openat"
- THEN parse_syscall_signal extracts "openat"
- Test: `parse_syscall_signal_extracts_symbolic_name` in `src/security/syscall_anomaly.rs`

#### Scenario: Space-separated syscall number
- WHEN a line contains "system call nr 59"
- THEN parse_syscall_signal extracts the syscall via number mapping
- Test: `parse_syscall_signal_extracts_space_separated_number` in `src/security/syscall_anomaly.rs`

#### Scenario: Hex syscall number
- WHEN a line contains "syscall=0x3b"
- THEN parse_syscall_signal extracts the syscall via hex number mapping
- Test: `parse_syscall_signal_extracts_hex_syscall_number` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-002: Unknown syscalls MUST trigger alerts
When a syscall is observed that is not in the baseline, UnknownSyscall alert MUST fire.

#### Scenario: Unknown syscall alert
- WHEN output contains "audit: syscall=openat" and openat is not in baseline
- THEN alerts include UnknownSyscall
- Test: `detector_alerts_on_unknown_syscall` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-003: Denied rate spikes MUST trigger alerts
When denied events exceed max_denied_events_per_minute, DeniedRateExceeded alert MUST fire.

#### Scenario: Rate spike
- WHEN multiple denied events arrive within one minute
- THEN alerts include DeniedRateExceeded
- Test: `detector_alerts_on_denied_rate_spike` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-004: Disabled detector MUST return no alerts
When enabled=false, inspect_command_output MUST return empty.

#### Scenario: Disabled mode
- WHEN detector is disabled and output has syscall events
- THEN alerts are empty
- Test: `detector_respects_disabled_mode` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-005: Alert cooldown MUST suppress repeated identical alerts
Same alert key MUST not fire again within cooldown period.

#### Scenario: Cooldown suppression
- WHEN same alert triggers twice within cooldown
- THEN second call returns no alerts for that key
- Test: `detector_applies_alert_cooldown` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-006: Alert rate limit MUST cap alerts per minute

#### Scenario: Alerts per minute limit
- WHEN many alerts fire within one minute
- THEN excess alerts are suppressed
- Test: `detector_limits_alerts_per_minute` in `src/security/syscall_anomaly.rs`

### REQ-SEC-SYSCALL-007: Default baseline MUST cover common syscalls

#### Scenario: Default baseline coverage
- WHEN default baseline is checked against common mapped syscalls
- THEN they are present
- Test: `default_baseline_covers_common_mapped_syscalls` in `src/security/syscall_anomaly.rs`

---

### Adversarial Suffix Detection (`src/security/perplexity.rs`)

### REQ-SEC-PERP-001: Disabled filter MUST return None
When enable_perplexity_filter=false, detect_adversarial_suffix MUST return None.

#### Scenario: Disabled
- WHEN filter is disabled
- THEN returns None
- Test: `filter_disabled_returns_none` in `src/security/perplexity.rs`

### REQ-SEC-PERP-002: GCG-like adversarial suffixes MUST be flagged
Prompts with high-entropy garbage suffixes MUST trigger detection.

#### Scenario: GCG-like suffix
- WHEN prompt ends with "!!a$$z_x9"
- THEN returns Some(assessment) with suspicious_token_count >= 1
- Test: `detects_known_gcg_like_suffix` in `src/security/perplexity.rs`

### REQ-SEC-PERP-003: Natural language prompts MUST NOT be flagged
Normal English prompts MUST return None.

#### Scenario: Natural language
- WHEN prompt is normal text
- THEN returns None
- Test: `natural_language_prompt_is_not_flagged` in `src/security/perplexity.rs`

### REQ-SEC-PERP-004: Detection latency MUST stay under 50ms for typical prompts

#### Scenario: Latency bound
- WHEN a typical prompt is scanned
- THEN detection completes under 50ms
- Test: `latency_stays_under_fifty_ms_for_typical_prompt` in `src/security/perplexity.rs`

---

### Audit Logging (`src/security/audit.rs`)

### REQ-SEC-AUDIT-001: AuditEvent MUST generate unique IDs
Each AuditEvent::new() call MUST produce a unique event_id.

#### Scenario: Unique IDs
- WHEN two AuditEvents are created
- THEN their event_ids differ
- Test: `audit_event_new_creates_unique_id` in `src/security/audit.rs`

### REQ-SEC-AUDIT-002: AuditEvent builder MUST chain actor, action, result, security
Builder methods MUST set the corresponding fields.

#### Scenario: Actor builder
- WHEN with_actor is called
- THEN actor fields are populated
- Test: `audit_event_with_actor` in `src/security/audit.rs`

#### Scenario: Action builder
- WHEN with_action is called
- THEN action fields are populated
- Test: `audit_event_with_action` in `src/security/audit.rs`

#### Scenario: JSON serialization
- WHEN an AuditEvent is serialized to JSON
- THEN it produces valid JSON with all fields
- Test: `audit_event_serializes_to_json` in `src/security/audit.rs`

### REQ-SEC-AUDIT-003: Disabled logger MUST NOT create files
When enabled=false, AuditLogger.log() MUST be a no-op.

#### Scenario: Disabled logger
- WHEN logger is disabled and log() is called
- THEN no file is created
- Test: `audit_logger_disabled_does_not_create_file` in `src/security/audit.rs`

### REQ-SEC-AUDIT-004: Enabled logger MUST write and rotate
Events MUST be written as JSON lines; rotation MUST trigger when file exceeds max_size_mb.

#### Scenario: Write event
- WHEN enabled logger writes an event
- THEN audit log file exists with parseable JSON
- Test: `audit_logger_writes_event_when_enabled` in `src/security/audit.rs`

#### Scenario: Structured command event
- WHEN log_command_event is called with command execution data
- THEN the log contains structured entry
- Test: `audit_log_command_event_writes_structured_entry` in `src/security/audit.rs`

#### Scenario: Rotation
- WHEN log exceeds max size
- THEN a .1.log backup is created
- Test: `audit_rotation_creates_numbered_backup` in `src/security/audit.rs`

### REQ-SEC-AUDIT-005: log_command() backward-compat helper MUST delegate to log_command_event
The log_command() method MUST produce the same structured entry as log_command_event().

#### Scenario: Compat helper
- WHEN log_command() is called with channel, command, risk_level, etc.
- THEN the log file contains the same structured entry
- Test: `log_command_compat_writes_same_as_log_command_event` in `src/security/audit.rs`

## Mock Strategy
- OTP tests use tempfile::tempdir() for encrypted secret storage
- Audit tests use tempfile::TempDir for isolated log files
- Leak/prompt guard tests are pure (no I/O)
- Syscall anomaly tests use tempfile::tempdir() for anomaly log
- File link guard tests use tempfile::tempdir() for filesystem operations

## Coverage Notes
- `src/security/otp.rs`: 6 tests
- `src/security/pairing.rs`: 33 tests
- `src/security/secrets.rs`: 40 tests (includes 1 Unix-only test)
- `src/security/leak_detector.rs`: 14 tests
- `src/security/prompt_guard.rs`: 10 tests
- `src/security/syscall_anomaly.rs`: 11 tests
- `src/security/file_link_guard.rs`: 3 tests (1 Unix-only)
- `src/security/sensitive_paths.rs`: 5 tests
- `src/security/domain_matcher.rs`: 7 tests
- `src/security/perplexity.rs`: 4 tests
- `src/security/audit.rs`: 9 tests
- Total: 142 tests across 11 files
