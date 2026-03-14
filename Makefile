# ── Claude Code OAuth credential extraction ──────────────────
# Linux: reads from ~/.claude/credentials.json
# macOS: reads from Keychain via `security` command
CLAUDE_CREDS_FILE := $(HOME)/.claude/credentials.json
ifeq ($(shell uname),Darwin)
CLAUDE_CREDS_JSON := $(shell security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null)
else
CLAUDE_CREDS_JSON := $(shell cat $(CLAUDE_CREDS_FILE) 2>/dev/null)
endif
export CLAUDE_CODE_OAUTH_TOKEN := $(shell echo '$(CLAUDE_CREDS_JSON)' | python3 -c "import sys,json; print(json.load(sys.stdin).get('claudeAiOauth',{}).get('accessToken',''))" 2>/dev/null)
export CLAUDE_CODE_OAUTH_REFRESH_TOKEN := $(shell echo '$(CLAUDE_CREDS_JSON)' | python3 -c "import sys,json; print(json.load(sys.stdin).get('claudeAiOauth',{}).get('refreshToken',''))" 2>/dev/null)

.PHONY: up logs down auth

auth:
	@if [ -z "$(CLAUDE_CODE_OAUTH_TOKEN)" ]; then \
		echo "No Claude Code credentials found. Run: claude auth"; \
		exit 1; \
	fi
	@echo "Claude Code OAuth credentials loaded."

up:
	make -C haven up

logs:
	make -C haven logs

down:
	make -C haven down

