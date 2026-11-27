C2_HOST ?= 127.0.0.1
C2_PORT ?= 8080

.PHONY: help server implant implant-windows test-curl check-strings

help:
	@echo "Cibles disponibles :"
	@echo "  make server            Lance le serveur C2 (port $(C2_PORT))"
	@echo "  make implant           Compile et lance l'implant (local, $(C2_HOST):$(C2_PORT))"
	@echo "  make implant-windows   Cross-compile implant.exe pour Windows (x86_64-msvc)"
	@echo "  make test-curl         Teste le ping-pong via curl"
	@echo "  make check-strings     Vérifie que les chaînes sont obfusquées dans le binaire"
	@echo ""
	@echo "Variables :"
	@echo "  C2_HOST=$(C2_HOST)  C2_PORT=$(C2_PORT)"
	@echo ""
	@echo "Exemple test host -> VM :"
	@echo "  C2_HOST=192.168.41.1 make implant-windows"

server:
	RUST_LOG=info C2_PORT=$(C2_PORT) cargo run -p c2_server

implant:
	cargo run -p implant

implant-windows:
	cargo build --target x86_64-pc-windows-msvc -p implant
	@echo "[+] Binaire : target/x86_64-pc-windows-msvc/debug/implant.exe"

test-curl:
	curl -s -X POST http://$(C2_HOST):$(C2_PORT)/ping \
		-H "Content-Type: application/json" \
		-d '{"id":42,"ts":1700000000}' | python3 -m json.tool

check-strings:
	@echo "=== Chaînes suspectes dans le binaire ==="
	strings target/debug/implant 2>/dev/null | grep -E "/ping|$(C2_HOST)|Microsoft-Crypto" || echo "[+] Aucune chaîne sensible en clair"
