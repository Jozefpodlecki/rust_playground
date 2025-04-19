# 🏗️ Architecture: Real-Time Lost Ark Raid Simulation

This document defines the architecture for simulating a real-time raid encounter in Lost Ark. The simulation models actual in-game behavior, including class roles, casting times, skill effects, and encounter mechanics.

---

## 🎮 Player Simulation

- Each player runs in its own thread, simulating real-time behavior during the raid encounter.
- Players use up to **11 skills**:
  - **8 Normal Skills**
  - **1 Hyper Awakening Technique**
  - **1 Awakening Skill**
  - **1 Hyper Awakening**
  
- Skills have cast durations between **250ms and 3s**, and during this time, the thread sleeps to simulate the casting time.
- Skills are modified by **tripod configurations**, which influence the effect of each skill.


### Role Configuration

- In each raid, **roles are fixed**:
  - **3 DPS** (Damage Dealers)
  - **1 Support** (Healer, Buff Provider, etc.)
  
  The **3 DPS** roles focus on maximizing damage output and maintaining optimal skill rotations, while the **Support** class focuses on buffing the party, healing, and possibly managing some utility skills.

- **Excluded Scenarios**: This simulation focuses on **simple scenarios** where the class/player casts **11 skills in a fixed rotation** during the fight. We are excluding scenarios involving **transformation classes** or **classes that enter enhanced states**.

---

## 🧑‍🤝‍🧑 Party Structure

- Each **party consists of 4 players**:
  - 3 DPS
  - 1 Support
  
- A raid encounter may consist of multiple parties, but the structure remains the same for each party.
  
- **Party state** includes active buffs, cooldowns, shields, and other statuses. This state is shared via synchronized access (`Arc<RwLock<PartyState>>` or ECS components).

---

## 👾 Boss and Minions

- The **boss** is managed by its own thread.
- Boss behavior includes:
  - **HP gates** and **invulnerability phases** that change the state of the encounter.
  - **Minions** may appear throughout the fight to complicate the encounter, which players need to handle to manage mechanics effectively.
  - **Renaming** the boss at specific HP thresholds to signify different phases of the fight.
  
- **Minions** are tracked as part of the raid's overall encounter state and often need to be defeated to avoid being overwhelmed or to avoid certain negative effects, like damage increases for the boss or more challenging mechanics.
  
- **Boss and minion HP** are tracked and used to compute damage statistics and other relevant metrics like **average DPS**.

---

## 📨 Messaging and State Sharing

- **Communication** between player threads, boss manager, and collector is done via `crossbeam::channel` or ECS event writers.
- Shared states include:
  - **BossState**: Contains boss HP, damage reduction phases, active debuffs.
  - **PartyState**: Includes active buffs, shields, and other party-wide conditions.
  - **PlayerState**: Tracks individual player cooldowns, identity gauge, and other unique attributes.

---

## 🧠 ECS Integration

An **ECS framework** (e.g., `bevy_ecs`) can be used to:

- Model all entities (players, bosses, minions) as components.
- Process various systems, including:
  - **Skill casting and resolution**
  - **Buff expiration and application**
  - **Boss phase transitions**
  - **Damage calculation and result aggregation**

Using ECS enables scalable and modular behavior without tightly coupling logic to threads.

---

## 📊 Collector Thread

- The **collector** thread gathers all player-generated data, such as:
  - **Damage events**
  - **Skill usage logs**
  - **Buff/debuff application**
  
- It computes statistics like **DPS**, **buff uptime**, and **phase timings**.
- The thread produces structured logs or **summary reports** for analysis.
