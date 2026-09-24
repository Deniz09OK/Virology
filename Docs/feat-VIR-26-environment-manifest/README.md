# VIR-26 — Setup Environment Manifest Reproductible

**Auteur :** Deniz Ok
**Date :** 2026-09-24
**MITRE :** N/A — ticket infrastructure / reproductibilité

## Contexte

Ce ticket documente la procédure complète pour monter l'environnement de lab du projet Virology.
L'objectif est qu'un membre de l'équipe puisse recréer le lab from scratch en suivant ce README.

---

## Specs de la VM

| Paramètre           | Valeur                          |
|---------------------|---------------------------------|
| Nom                 | virology-target                 |
| OS                  | Windows 11 25H2 Enterprise Eval |
| vCPU                | 2                               |
| RAM                 | 6 Go                            |
| Disque              | 60 Go (dynamique)               |
| Réseau              | Host-Only — vboxnet0            |
| IP fixe             | 192.168.56.111                  |
| C2 host (attaquant) | 192.168.56.112                  |

> **Note réseau** : Les IPs `192.168.56.x` sont gérées localement par VirtualBox (réseau host-only).
> Elles sont **indépendantes** du réseau physique (wifi maison, réseau école, etc.) et ne changent pas d'un endroit à l'autre.
> Si ton interface vboxnet0 utilise une plage différente, adapte les IPs dans le `.env` en conséquence.

---

## 1. Créer la VM dans VirtualBox

### Option A — Interface graphique (recommandé)

1. Télécharger l'ISO Windows 11 25H2 Enterprise Evaluation :
   👉 https://www.microsoft.com/fr-fr/evalcenter/download-windows-11-enterprise

2. Ouvrir VirtualBox → **Nouvelle**
3. Nom : `virology-target`, Type : `Microsoft Windows`, Version : `Windows 11 (64-bit)`
4. RAM : `6144 Mo`, vCPU : `2`
5. Disque : `60 Go`, format VDI, allocation dynamique
6. Réseau → **Adaptateur 1** → `Réseau hôte uniquement` → `vboxnet0`
7. Démarrer la VM et installer Windows depuis l'ISO

### Option B — Ligne de commande (optionnel)

```bash
VBoxManage createvm --name "virology-target" --ostype Windows11_64 --register

VBoxManage modifyvm "virology-target" \
  --memory 6144 \
  --cpus 2 \
  --nic1 hostonly \
  --hostonlyadapter1 vboxnet0

VBoxManage createhd \
  --filename "$HOME/VirtualBox VMs/virology-target/virology-target.vdi" \
  --size 61440

VBoxManage storagectl "virology-target" \
  --name "SATA Controller" --add sata --controller IntelAhci

VBoxManage storageattach "virology-target" \
  --storagectl "SATA Controller" \
  --port 0 --device 0 --type hdd \
  --medium "$HOME/VirtualBox VMs/virology-target/virology-target.vdi"

VBoxManage storageattach "virology-target" \
  --storagectl "SATA Controller" \
  --port 1 --device 0 --type dvddrive \
  --medium /chemin/vers/win11-enterprise.iso
```

---

## 2. Configurer vboxnet0

### Option A — Interface graphique

1. VirtualBox → **Fichier** → **Gestionnaire de réseau hôte**
2. Vérifier que `vboxnet0` existe avec :
   - IP : `192.168.56.1`
   - Masque : `255.255.255.0`
   - DHCP : **désactivé**

### Option B — CLI

```bash
VBoxManage hostonlyif create
VBoxManage hostonlyif ipconfig vboxnet0 --ip 192.168.56.1 --netmask 255.255.255.0
VBoxManage dhcpserver remove --netname HostInterfaceNetworking-vboxnet0 2>/dev/null || true
```

---

## 3. Assigner l'IP fixe dans la VM

Dans Windows (sur la VM) :

1. `Paramètres` → `Réseau et Internet` → `Ethernet` → `Modifier les options de l'adaptateur`
2. Clic droit sur l'interface → `Propriétés` → `Internet Protocol Version 4`
3. IP fixe :
   - Adresse : `192.168.56.111`
   - Masque : `255.255.255.0`
   - Passerelle : *(laisser vide)*
   - DNS : *(laisser vide)*

---

## 4. Snapshot — État de base

Prendre un snapshot **avant** tout test offensif pour pouvoir restaurer proprement.

### Option A — Interface graphique

1. VM démarrée (ou éteinte) → **Machine** → **Prendre un instantané**
2. Nom : `BASE — Windows 11 clean`

### Option B — CLI

```bash
VBoxManage snapshot "virology-target" take "BASE -- Windows 11 clean" \
  --description "Etat propre post-install"
```

---

## 5. Setup hôte (machine attaquante)

Sur ta machine (Windows) :

```bash
# Installer rustup si pas déjà fait
# https://rustup.rs

# Ajouter la cible Windows MSVC
rustup target add x86_64-pc-windows-msvc

# Vérifier
rustup target list --installed
```

---

## 6. Restaurer le snapshot

À répéter avant chaque session de test offensif pour repartir d'un état propre.

### Option A — Interface graphique

1. Éteindre la VM
2. Onglet **Instantanés** → Sélectionner `BASE — Windows 11 clean`
3. Clic droit → **Restaurer**

### Option B — CLI

```bash
VBoxManage controlvm "virology-target" poweroff 2>/dev/null || true
VBoxManage snapshot "virology-target" restore "BASE -- Windows 11 clean"
```

---

## 7. Checklist pré-démo

- [ ] VM `virology-target` démarrée
- [ ] IP `192.168.56.111` accessible depuis le host (`ping 192.168.56.111`)
- [ ] Snapshot `BASE` pris
- [ ] `rustup target list --installed` contient `x86_64-pc-windows-msvc`
- [ ] `cert.pem` et `key.pem` absents du repo (vérifier `.gitignore`)

---

## Sécurité

- Le beacon et le serveur tournent **uniquement** sur le réseau host-only (`192.168.56.0/24`) — jamais exposé sur le réseau réel.
- Ne jamais commiter `cert.pem`, `key.pem`, ou des credentials réels sur GitHub.
- Ce lab est réalisé dans un cadre pédagogique Epitech, sur un réseau fermé et consenti.

---

## Comment tester

```bash
# Depuis le host, vérifier la connectivité avec la VM
ping 192.168.56.111

# Vérifier que le serveur C2 est joignable depuis la VM
# (depuis PowerShell sur la VM)
Test-NetConnection -ComputerName 192.168.56.112 -Port 443
```

---

*Ticket : VIR-26 | Projet : Virology | Promo MSc 2027 — Epitech Nancy*
