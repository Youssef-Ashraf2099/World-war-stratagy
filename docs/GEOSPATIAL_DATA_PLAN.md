# 🗺️ NATURAL EARTH DATA INTEGRATION PLAN

**Status:** Analysis Complete | **Recommendation:** Use in v0.1 (Small Extract)

---

## 📊 WHAT YOU HAVE

Your `/assets/data` folder contains **Natural Earth Shapefiles** — professional GIS datasets:

| Dataset | Contains | Files |
|---------|----------|-------|
| **ne_110m_admin_0_countries** | 195 sovereign nations | 6 files |
| **ne_110m_admin_1_states_provinces** | Sub-national regions | 6 files |
| **ne_110m_populated_places** | Major cities | 6 files |
| **ne_110m_admin_0_boundary_lines_land** | Border geometry | 6 files |
| **ne_110m_admin_0_sovereignty** | Territorial disputes | 6 files |

**Total: 30 files of world geopolitical data**

---

## 🔍 SHAPEFILE FORMAT

Each dataset = 5-6 related files:
- `.shp` = Geometry (coordinates, polygons)
- `.dbf` = Database (attributes, metadata)
- `.shx` = Index (fast lookups)
- `.prj` = Map projection (WGS84 world standard)
- `.cpg` = Character encoding

---

## 💡 HOW TO USE IN SPIRITS OF STEEL

### **OPTION A: v0.1 (Lightweight - RECOMMENDED)**

**Extract nation data as JSON:**

```json
[
  {
    "id": 1,
    "name": "United States",
    "population": 331000000,
    "gdp": 21000000000000,
    "continent": "North America",
    "code": "US"
  },
  {
    "id": 2,
    "name": "China",
    "population": 1411000000,
    "gdp": 14730000000000,
    "continent": "Asia",
    "code": "CN"
  }
]
```

**Effort:** 4-6 hours total
**Result:** 195 real nations with authentic starting values

---

### **OPTION B: v0.2+ (Full GIS Integration)**

Parse shapefiles at runtime:
- Load nation boundaries as polygons
- Procedurally subdivide into provinces
- Assign resources by geography
- Create trade route networks

**Effort:** 30-40 hours
**Result:** Perfect geographic fidelity

---

## 🎯 RECOMMENDED: HYBRID v0.1 + v0.2

### **v0.1: Quick Extract (This Week)**

```
1. Download QGIS (free, qgis.org)
2. Open: ne_110m_admin_0_countries.shp
3. Export key attributes to CSV:
   - NAME, CONTINENT, POP_EST, GDP_MD_EST, ISO_A2
4. Convert CSV → JSON (Python, 10 min)
5. Load in game: src/main.cpp
```

**Time:** 4 hours  
**Payoff:** Massive — 195 real nations, authentic starting conditions

---

### **v0.2: Full Integration (Next Month)**

- Load shapefile geometries
- Procedurally subdivide nations into provinces
- Map geography to resources (climate → food, mountains → defense)
- Build adjacency graph from real borders

**Time:** 30-40 hours  
**Payoff:** Geographic accuracy + emergent geopolitics

---

## 📋 IMPLEMENTATION STEPS (CONCRETE)

### **Step 1: Data Extraction (2 hours)**
```bash
# Use free tool: QGIS
# Open shapefile → Export to CSV
# Columns: NAME, POP_EST, GDP_MD_EST, CONTINENT, ISO_A2
```

### **Step 2: Convert to JSON (30 min)**
```python
import csv, json
with open('countries.csv') as f:
    nations = list(csv.DictReader(f))
with open('nations.json', 'w') as out:
    json.dump(nations, out, indent=2)
```

### **Step 3: Load in Game (1.5 hours)**
```cpp
// Edit: src/main.cpp
nlohmann::json nationsData = 
    nlohmann::json::parse(std::ifstream("src/game/scenarios/nations.json"));

for (auto& n : nationsData) {
    Nation nation(nextID++);
    nation.name = n["NAME"];
    nation.population = stoi(n["POP_EST"]);
    nation.gdp = stod(n["GDP_MD_EST"]) * 1e6;  // Convert to actual GDP
    world.addNation(nation);
}
```

### **Step 4: Test (30 min)**
```cpp
// Run simulation with real nation data
// 195 nations, authentic starting values
```

**Total: 4 hours for complete v0.1 integration**

---

## 📁 FILE STRUCTURE

Add to project:

```
src/game/
├── configs/
│   └── nations.json           ← All 195 nations (extracted)
├── scenarios/
│   ├── scenario_template.json
│   ├── earth_1900.json        ← Historical start
│   ├── earth_modern.json      ← 2023 borders
│   └── earth_test.json        ← 3-nation test
```

---

## ✅ WHAT THIS GIVES YOU

**With Real Data:**
- ✅ 195 authentic nations
- ✅ Historical accuracy
- ✅ Player recognition ("That's MY country!")
- ✅ Realistic power asymmetry
- ✅ Geography-driven gameplay

**Without It:**
- ❌ Generic procedural world
- ❌ Less player investment
- ❌ Harder to balance
- ❌ No historical scenarios

---

## 🚀 FINAL DECISION

| Aspect | Answer |
|--------|--------|
| **Use in v0.1?** | ✅ YES (just nations) |
| **Use in v0.2?** | ✅ YES (add geometry) |
| **Time cost?** | 4-6 hours for v0.1 |
| **Worth it?** | 🟢 **ABSOLUTELY** |
| **Skip it?** | Possible, but lose authenticity |

---

## 📌 RECOMMENDATION

**Spend 4 hours extracting real nation data. It's the cheapest authenticity boost available.**

Next: Create Python extraction script → Load nations → Run simulation with real world data.

---

**Status:** Ready to implement. Data is in place. Just needs extraction and loading. 🌍
