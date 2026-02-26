## 🎯 **SPIRITS OF STEEL — EXTENDED SIMULATION DESIGN DOCUMENT (GDD v1.0)**

***

# 1️⃣ OVERVIEW

**Project Name:** Spirits of Steel\
**Genre:** Grand Strategy / Geopolitical Simulation\
**Perspective:** 2D Strategic Map (Primary), No Tactical Micro-View\
**Core Identity:** *A deterministic geopolitical simulator driven by systemic pressure rather than scripted gameplay.*

This is **not a war game**.\
This is a **statecraft simulator** where war is one of many tools.

The player governs a Nation navigating:

* Resource asymmetry

* Population pressure

* Internal ideology shifts

* Trade dependencies

* Military deterrence

* Alliance blocs

* Random destabilization events

The game simulates **macro-historical behavior**, not micromanagement.

***

## DESIGN PHILOSOPHY

| Principle              | Meaning                                    |
| ---------------------- | ------------------------------------------ |
| Deterministic Systems  | Every outcome traceable to variables       |
| No Magic Numbers       | Everything derived from economy/population |
| War is Expensive       | Economic collapse > battlefield loss       |
| Map = Data Structure   | The world is a graph, not an image         |
| Player Shapes Pressure | You influence — not control — history      |

***

# 2️⃣ CORE LOOP & SYSTEM ARCHITECTURE

***

## ⏱️ MASTER GAME LOOP

```text
while(gameRunning)
{
    world.tick();           // Economy + Demographics
    diplomacy.tick();       // Relations recalculation
    logistics.tick();       // Resource movement
    military.tick();        // Army upkeep + conflicts
    stability.tick();       // Revolts / ideology shifts
    eventEngine.tick();     // Random/global triggers

    render();
}
```

***

## ⧠ WORLD MODEL

The world is a **Graph**, not a grid.

```text
Province {
    ResourceType resource;
    Population pop;
    Infrastructure infra;
    Owner nation;
    Satisfaction localMood;
}
```

Adjacency defines trade, invasion routes, culture spread.

***

## ⧮ SIMULATION LAYERS (RUN EACH TICK)

| Layer              | Function                 |
| ------------------ | ------------------------ |
| Demographic Engine | Birth, death, migration  |
| Economic Engine    | Production & consumption |
| Political Engine   | Ideology drift           |
| Diplomatic Engine  | Trust decay & alignment  |
| Military Engine    | Force projection         |
| Entropy Engine     | Chaos events             |

***

## ⚙️ KEY FORMULA (GAME FEEL DRIVER)

```text
National Power =
(Economy × Stability × Logistics)
+ (Military Deterrence × Alliances)
− Internal Pressure
```

War is won **before it starts**.

***

# 3️⃣ NATIONS & GOVERNANCE MODEL

***

## 🏛️ NATION STRUCTURE

```text
Nation {
    GovernanceType system;
    Legitimacy legitimacy;
    EconomicModel economyType;
    MilitaryDoctrine doctrine;
    Population demographics;
    Stability cohesion;
}
```

***

## ⚖️ GOVERNANCE TYPES

These are **behavior modifiers**, not flavor.

| System        | Strength            | Weakness        |
| ------------- | ------------------- | --------------- |
| Democracy     | High innovation     | Slow decisions  |
| Authoritarian | Fast mobilization   | Revolt risk     |
| Technocracy   | Economic efficiency | Low morale      |
| Oligarchy     | Stable elites       | Corruption drag |
| Fragile State | Cheap expansion     | Collapse spiral |

***

## 🧠 LEGITIMACY SYSTEM (CRITICAL)

Legitimacy replaces “Happiness”.

```text
Legitimacy =
Economic Growth
+ Military Success
+ Resource Access
− Inequality
− War Losses
− Trade Dependency
```

If Legitimacy < 35 → protests\
If Legitimacy < 20 → civil war roll.

***

## 🔥 CIVIL WAR MECHANIC

Civil wars **split simulation actors**, not scripted rebels.

New Nation spawned dynamically:

```text
Rebel State inherits:
    % of army
    % of provinces
    ideological mutation
```

This keeps simulation emergent.

***

# 4️⃣ MILITARY SYSTEM — NOT UNIT SPAM

***

## 🪖 ARMIES ARE LOGISTICS ENTITIES

You don't build units.

You build:

* Supply chains

* Industrial base

* Doctrine

***

## FORCE MODEL

```text
ArmyStrength =
Industrial Output
× Supply Integrity
× Doctrine Modifier
× Morale
```

***

## DOCTRINE TYPES

| Doctrine           | Effect                        |
| ------------------ | ----------------------------- |
| Mass Mobilization  | Cheap armies, economic damage |
| Professional Force | Small but elite               |
| Mechanized         | Resource heavy                |
| Asymmetric         | Good for weak nations         |
| Defensive Depth    | Hard to invade                |

***

## WAR IS A RESOURCE COLLISION

Combat resolution:

```text
Outcome =
(Logistics Score × Readiness × Terrain)
vs
(Enemy Supply Collapse Rate)
```

Battles don’t matter.\
**Supply death spirals decide wars.**

***

# 5️⃣ ECONOMY & RESOURCE DISTRIBUTION

***

## RESOURCE MODEL (HARD GEOGRAPHY)

Each province has **one dominant resource**.

| Resource    | Drives            |
| ----------- | ----------------- |
| Food        | Population growth |
| Iron        | Military          |
| Oil         | Mobility          |
| Rare Earths | Tech              |
| Water       | Stability         |
| Trade Ports | GDP multiplier    |

***

## TRADE IS MANDATORY

No nation is self-sufficient.

Trade routes:

```text
If Route Broken:
    Industry efficiency −40%
    Stability −15
```

This forces diplomacy.

***

# 6️⃣ ALLIANCE SYSTEM — MULTI-LAYER GEOPOLITICAL FRAMEWORK (REVISED)

Alliances are **not a binary flag**.\
They are **structured contracts** with scope, obligations, and internal tension.

You are modeling **power coordination**, not friendship.

***

# 🧭 ALLIANCE = A TREATY OBJECT

Instead of:

```cpp
bool allied[A][B];
```

You implement:

```cpp
Treaty {
    TreatyType type;
    vector<NationID> members;

    ObligationMatrix obligations;
    TrustMatrix trust;
    ExpirationCondition expiry;

    float cohesion;          // internal unity
    float commandIntegration;
}
```

An alliance is a **living system** that can strengthen or decay.

***

# 🧱 6.1 TYPES OF ALLIANCES

Each alliance type affects **different simulation layers**.

## ⚔️ 1. WAR ALLIANCE (Mutual Defense Pact)

Purpose: Deterrence & coordinated warfare.

### Effects:

* Auto-join defensive wars

* Shared military intelligence

* Supply corridors between members

* Joint operations bonus

### Formula Bonus:

```cpp
CombinedForceModifier =
(logisticsCompatibility × doctrineSimilarity × cohesion)
```

### Risk:

If one member starts reckless wars → alliance cohesion drops.

***

## 💰 2. ECONOMIC ALLIANCE (Trade Bloc)

Purpose: Resource stabilization.

### Effects:

* Tariff removal

* Shared infrastructure investment

* Internal market multiplier

* Crisis cushioning

```cpp
TradeEfficiency += SharedInfrastructure × PolicyAlignment
```

### Risk:

Strong members dominate weak economies → resentment grows.

***

## 🧪 3. TECHNOLOGICAL ALLIANCE (Research Compact)

Purpose: Accelerated development.

### Effects:

* Shared tech unlock progression

* Reduced research cost

* Knowledge diffusion

```cpp
ResearchSpeed =
Base + (AverageEducationLevel × DataSharingIndex)
```

### Risk:

Espionage probability increases between members.

***

## 🛢️ 4. RESOURCE COALITION (Strategic Cartel)

Purpose: Control supply chains (Oil, Food, Minerals).

### Effects:

* Export quotas

* Price manipulation

* Supply guarantees to members

### Risk:

External nations may sanction or intervene.

***

## 🕊️ 5. POLITICAL UNION (Ideological Bloc)

Purpose: Maintain regime stability.

### Effects:

* Propaganda support

* Coup protection

* Legitimacy boost

### Risk:

If one collapses → domino destabilization.

***

# 🏛️ 6.2 PRE-CREATED ALLIANCES (WORLD START STATE)

At game start, the world already contains alliances.

These are generated using:

```cpp
AllianceSeed =
IdeologySimilarity
+ TradeDependency
+ HistoricConflictMemory
```

Examples:

* Regional defense coalitions

* Resource cartels

* Former empires' economic zones

The player must **interact with an existing geopolitical structure**, not a blank map.

***

# 📥 6.3 JOINING AN ALLIANCE (APPLICATION PROCESS)

Joining is a **multi-variable evaluation**, not a yes/no.

When the player applies, members run:

```cpp
AcceptanceScore =
Trust(player)
+ StrategicNeed(player)
+ EconomicBenefit(player)
− IdeologicalDistance(player)
− RiskOfEntrapment(player)
```

***

## Possible Outcomes:

| Result            | Meaning                       |
| ----------------- | ----------------------------- |
| Accepted          | Full integration              |
| Conditional Entry | Must reduce army / open trade |
| Observer Status   | No defense guarantee yet      |
| Rejected          | Relations damaged             |
| Vetoed            | One member blocked entry      |

***

# 🧠 6.4 INTERNAL ALLIANCE POLITICS

Alliances have **their own internal simulation**.

Each tick:

```cpp
cohesion -= policyDisagreements
cohesion -= unequalBurdenSharing
cohesion += sharedThreatLevel
```

Low cohesion leads to:

* Members ignoring obligations

* Delayed military response

* Economic fragmentation

At critical failure:

```cpp
AllianceFragmentationEvent triggered
```

***

# ⚖️ 6.5 BURDEN SHARING SYSTEM (PREVENTS EXPLOITS)

Members must contribute proportionally.

```cpp
ExpectedContribution =
GDPShare × MilitaryCapacity × ResourceControl
```

If a nation under-contributes:

* Trust penalty

* Influence loss

* Possible expulsion vote

***

# 🪓 6.6 LEAVING OR BREAKING AN ALLIANCE

Leaving is costly.

```cpp
ExitShock =
TradeLoss + SecurityVacuum + LegitimacyDrop
```

This prevents "join-for-benefits then leave" abuse.

***

# 🎲 6.7 ALLIANCE-LEVEL EVENTS

Events now target alliances, not just nations.

Examples:

| Event                    | Impact               |
| ------------------------ | -------------------- |
| Joint Military Failure   | cohesion −20         |
| Trade Dispute            | internal sanctions   |
| Leadership Struggle      | influence shift      |
| Shared Tech Breakthrough | research spike       |
| Member Coup              | alliance instability |

***

# 🧮 6.8 ALLIANCE INFLUENCE HIERARCHY

Not all members are equal.

Each alliance calculates:

```cpp
InfluenceScore =
EconomicWeight
+ MilitaryWeight
+ FinancialContribution
+ DiplomaticCentrality
```

High influence nations:

* Propose policies

* Veto wars

* Shape doctrine

This naturally creates **power struggles inside alliances**.

***

# 🖥️ 6.9 PLAYER INTERACTION MODEL

Player actions inside alliance:

* Propose joint military operation

* Request aid

* Sanction member

* Push policy vote

* Attempt leadership shift

* Secretly undermine cohesion

***

# 🧩 6.10 WHY THIS SYSTEM ADDS DEPTH

Without this:\
Alliances are buffs.

With this:\
Alliances become **secondary arenas of gameplay**.

The player must manage:

* External rivals

* Internal alliance politics

* Economic interdependence

* Strategic credibility

You now simulate **real geopolitical friction**.

***

# ✅ IMPLEMENTATION PRIORITY (DO THIS EARLY)

1️⃣ Treaty object structure\
2️⃣ Acceptance scoring model\
3️⃣ Cohesion decay system\
4️⃣ Contribution tracking\
5️⃣ Alliance event hooks

Do **not** add UI before these exist.

***

## Next Expansion Options (choose direction):

* A) Intelligence / espionage layer affecting alliances

* B) Domestic politics interacting with foreign blocs

* C) AI behavior model for nations

* D) Economic simulation deep dive (markets, debt, sanctions)

Pick one — that defines the next architectural milestone.

***

# 7️⃣ EVENT ENGINE — CONTROLLED CHAOS

Events are **weighted by world state**, not random.

Example:

```text
If Food Shortage + Low Stability:
    Famine probability = HIGH
```

Event types:

* Economic crises

* Coups

* Tech breakthroughs

* Border incidents

* Migration waves

* Sanctions

* Proxy wars

***

# 8️⃣ GRAPHICS DIRECTION (KEEP IT SIMPLE)

***

## VISUAL GOAL

Readable. Cold. Analytical.

No visual noise.

***

## STYLE RULES

| Element | Direction                |
| ------- | ------------------------ |
| Map     | Flat colors by ownership |
| Borders | Hard geometric lines     |
| Units   | Numeric overlays only    |
| Cities  | Dots, not models         |
| War     | Lines, not animations    |
| UI      | Terminal-inspired panels |

***

## WHY THIS MATTERS

Graphics must **serve data visibility**.

You are building a **decision surface**, not immersion.

***

# Additional mandatory things i forgot about it

Yes. The current design is strong structurally, but it is still missing **three critical simulation layers** required for the game to feel alive rather than mechanical.

Right now you simulate:

* States

* Resources

* Diplomacy

* War

But real geopolitical motion is driven by **friction inside nations and information flow between them**.

Without these, the game risks becoming a spreadsheet war simulator.

You need to add the following systems.

***

# 1️⃣ SOCIETAL STRATIFICATION SYSTEM (THE REAL STABILITY DRIVER)

Currently you model “Population” as a number.\
That is not enough. Stability emerges from **competing internal groups**.

You must simulate **Population Blocs**.

## Add This Structure

```cpp
SocialBloc {
    string name;              // Industrial Workers, Rural, Elites, Military Class
    float sizeRatio;
    float wealth;
    float politicalPower;
    float satisfaction;
    IdeologyVector ideology;
}
```

Each Nation contains 4–6 blocs.

***

## Why This Matters

Policies now affect groups differently.

Example:

* Increase military spending → Military Bloc +20 satisfaction

* Raise taxes → Industrial Bloc −15 satisfaction

* Trade liberalization → Elites +, Workers −

Instability is no longer random:

```cpp
Instability =
VarianceBetweenBlocSatisfaction × InequalityIndex
```

Civil wars now emerge from **class fracture**, not a flat happiness drop.

***

# 2️⃣ INFORMATION & ESPIONAGE LAYER (THE HIDDEN GAME)

Grand strategy is not just tanks and trade.\
It is intelligence, influence, and miscalculation.

Add an **Information Layer** running parallel to diplomacy.

***

## Intelligence Model

```cpp
IntelProfile {
    float knowledgeLevel;     // how much you know about target
    float infiltration;
    float disinformationImpact;
    float cyberCapability;
}
```

This determines:

* Accuracy of enemy data (fog of economics)

* Ability to sabotage trade routes

* Influence alliance cohesion secretly

* Detect coups before they happen

***

## Example Mechanic

Without intel:\
You see:

> “Enemy Army Strength: Unknown”

With high intel:

> “Enemy logistics failing in 6 months.”

Now espionage becomes strategic preparation, not flavor.

***

# 3️⃣ ECONOMIC REALISM LAYER (STOP GDP FROM BEING A MAGIC NUMBER)

Right now economy is production-based.\
You must simulate **capital flow and debt** or wars have no long-term consequences.

***

## Add Financial System

```cpp
NationalFinance {
    float reserves;
    float debt;
    float creditRating;
    float inflation;
    float investorConfidence;
}
```

***

## War Now Has Delayed Cost

```cpp
If Debt > Threshold:
    IndustryEfficiency -= confidenceDrop
    AllianceTrust -= defaultRisk
```

You can win a war and lose your country 10 years later.

This creates **post-war collapse gameplay** — critical for realism.

***

# 4️⃣ TIME-SCALE DYNAMICS (PREVENTS TURN-BASED FEEL)

Different systems must react at different speeds.

| System     | Reaction Speed |
| ---------- | -------------- |
| Military   | Immediate      |
| Markets    | Months         |
| Population | Years          |
| Ideology   | Decades        |
| Alliances  | Crisis-driven  |

Implement **asynchronous decay/update rates**.

```cpp
system.update(deltaTime × responsivenessFactor);
```

This removes the “everything changes instantly” problem common in strategy games.

***

# 5️⃣ STRATEGIC INFRASTRUCTURE (GEOGRAPHY MUST MATTER MORE)

Add buildable macro-assets:

* Rail corridors

* Energy grids

* Naval chokepoints

* Data cables

* Industrial zones

These are not buildings.\
They are **national-scale modifiers** tied to provinces.

Destroying infrastructure should hurt more than losing land.

***

# 6️⃣ AI PERSONALITY ENGINE (NATIONS MUST HAVE MEMORY)

AI nations cannot be utility maximizers.\
They need identity and bias.

```cpp
StrategicCulture {
    float riskTolerance;
    float honorWeight;
    float tradePreference;
    float paranoia;
    float allianceLoyalty;
}
```

Now some nations overreact.\
Some forgive.\
Some never forget wars.

This creates emergent history.

***

# 7️⃣ CRISIS ESCALATION MODEL (THE “POINT OF NO RETURN”)

Conflicts should escalate through stages instead of instant war.

Add escalation ladder:

1. Diplomatic protest

2. Sanctions

3. Proxy conflict

4. Mobilization

5. Limited war

6. Total war

Each stage modifies economy and alliances automatically.

***

# 🔥 WHAT THESE ADDITIONS ACHIEVE

Without them:\
The player plays a strategy game.

With them:\
The player manages a **living geopolitical organism**.

You move from:

> map-painting simulator

to:

> systemic pressure simulator

***

# 📌 PRIORITY ORDER (DO NOT ADD RANDOMLY)

Implement in this order:

1️⃣ Social Bloc System\
2️⃣ Financial/Debt System\
3️⃣ Intelligence Layer\
4️⃣ Infrastructure Layer\
5️⃣ AI Strategic Culture\
6️⃣ Crisis Escalation

This sequence builds causal depth correctly.

***

If you want, the next step should be locking the **data schema for Nations + Provinces including these new layers** so implementation can start without refactoring later.

***

# 9️⃣ ENGINEERING STACK (MANDATORY FOR SUCCESS)

You are writing a simulation engine, not a game loop hack.

### Use:

* **C++17**

* **SDL2** (ONLY for rendering/input)

* **Eigen** (math)

* **entt ECS** (entity simulation)

* **ImGui** (debug panels — critical)

Avoid Unreal / Unity — they fight systemic simulation.

***

# 🎯 Recommended Frontend Stack (Purpose-Built)

Use a **thin immediate-mode UI layer in C++**.

### ✔ Rendering/Input

* **SDL2** → Windowing + input + frame timing

### ✔ UI Framework

* **Dear ImGui** → Immediate-mode control panels (perfect for simulation tooling)

### ✔ Optional Plotting

* **ImPlot** → Real-time graphs (economy, legitimacy drift, logistics strain)

This combination is widely used in:

* Game engines (internal tools)

* Financial simulators

* Robotics dashboards

* Defense simulations

Because it is **instrumentation-first**, not UX-first.

***

# 🔟 DEVELOPMENT ROADMAP

| Phase   | Goal                             |
| ------- | -------------------------------- |
| Phase 1 | Province graph + tick simulation |
| Phase 2 | Resource economy                 |
| Phase 3 | Nation behavior                  |
| Phase 4 | Diplomacy engine                 |
| Phase 5 | War logistics                    |
| Phase 6 | Event chaos                      |
| Phase 7 | UI shell                         |
| Phase 8 | Balancing                        |

***

# FINAL REALITY CHECK

This is **not a small project**.

You are building:

> A geopolitical simulation engine disguised as a strategy game.

If done correctly, even a 2D prototype will feel alive.

***

##
