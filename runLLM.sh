#!/usr/bin/env bash
set -euo pipefail

jq -n \
	--arg model "summarizer" \
	--rawfile prompt prompt.txt \
	--rawfile article /tmp/input.html \
	'{
		model: $model,
		messages: [
			{ role: "system", content: $prompt },
			{ role: "user", content: $article }
		],
		stream: false
	}' |
curl --fail-with-body \
	http://localhost:11434/api/chat \
	-H "Content-Type: application/json" \
	--data-binary @-
