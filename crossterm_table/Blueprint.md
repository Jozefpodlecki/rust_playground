# 🛠️ Blueprint  
**Lost Ark Simulator**

This is a console-based application that simulates a raid encounter as it happens in **Lost Ark**.

After specifying a raid or boss template along with a list of player classes, the application:

- Spawns a **dedicated thread per player class**, simulating their individual combat behavior (cooldowns, buffs, attack logic, etc.).
- Spawns a **boss thread** to manage state, mechanics, invulnerability phases, DR windows, name changes, and minion spawns.
- Launches a **collector thread** to gather and aggregate combat data (e.g., total damage, individual DPS, active buffs), simulating a log collector or DPS meter.

---

## 👥 Party Composition

Each party consists of **4 players**:
- **3 DPS**
- **1 Support**

---

## 🗡️ DPS Classes in Lost Ark

- Deathblade  
- Shadowhunter  
- Sorceress  
- Wardancer  
- Scrapper  
- Soulfist  
- Glaivier  
- Striker  
- Gunslinger  
- Artillerist  
- Deadeye  
- Sharpshooter  
- Machinist  
- Arcanist  
- Reaper  
- Summoner  
- Slayer  
- Aeromancer  
- Scouter *(aka Machinist)*  
- Destroyer  
- Berserker  
- Gunlancer

---

## 💫 Support Classes in Lost Ark

- **Bard**  
- **Paladin**  
- **Artist**
