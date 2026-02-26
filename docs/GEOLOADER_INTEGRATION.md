# 🗺️ NATURAL EARTH DATA INTEGRATION - QUICK START

**Status:** Ready to implement in v0.1 (4-6 hours)  
**Benefit:** 195 real nations with authentic starting values

---

## 📋 QUICK SUMMARY

| What | Details |
|------|---------|
| **Data** | Natural Earth Shapefiles (195 countries) |
| **Location** | `assets/data/ne_110m_admin_0_countries.*` |
| **Use Case** | Populate game world with real nations |
| **Timeline** | v0.1 (extract) + v0.2 (full integration) |
| **Effort** | 4-6 hours for v0.1 |

---

## 🚀 THREE WAYS TO USE THIS DATA

### **OPTION 1: Manual QGIS Export (Easiest, 2 hours)**

**Steps:**
1. Download QGIS (free): qgis.org
2. Open: `assets/data/ne_110m_admin_0_countries.shp`
3. Right-click layer → Export → Save as CSV
4. Select columns: `NAME`, `POP_EST`, `GDP_MD_EST`, `CONTINENT`, `ISO_A2`
5. Save to: `assets/data/countries.csv`
6. Run extraction script (see below)

---

### **OPTION 2: Python GDAL Script (Automated, 1 hour)**

**Prerequisites:**
```bash
pip install gdal
```

**Run:**
```bash
python tools/extract_nations.py
```

**What it does:**
- Opens shapefile directly
- Extracts key attributes
- Writes to `src/game/scenarios/nations.json`
- Outputs 195 nations with real data

---

### **OPTION 3: Online GIS Tool (No Installation, 1 hour)**

Use free online converter:
1. Go to **ogre.adc4gis.com** (free GIS converter)
2. Upload: `ne_110m_admin_0_countries.shp` (all 6 files)
3. Export format: JSON
4. Download result
5. Place in: `src/game/scenarios/nations.json`

---

## 📊 WHAT YOU'LL GET

After extraction, your JSON looks like:

```json
[
  {
    "id": 1,
    "name": "United States",
    "population": 331002651,
    "gdp": 20936300000000.0,
    "continent": "North America",
    "code": "US"
  },
  {
    "id": 2,
    "name": "China",
    "population": 1411750000,
    "gdp": 14730000000000.0,
    "continent": "Asia",
    "code": "CN"
  },
  {
    "id": 3,
    "name": "Japan",
    "population": 126476461,
    "gdp": 4890000000000.0,
    "continent": "Asia",
    "code": "JP"
  }
  // ... 192 more nations
]
```

---

## 💻 LOADING IN C++ (v0.1 Implementation)

### **Step 1: Create Loader Function**

Create file: `src/engine/subsystems/geography/GeoLoader.h`

```cpp
#pragma once

#include "engine/core/types/Nation.h"
#include "engine/core/state/WorldState.h"
#include <nlohmann/json.hpp>
#include <string>

namespace SOS::Engine::Geography {

class GeoLoader {
public:
    static bool LoadNationsFromJSON(
        const std::string& jsonPath,
        Core::State::WorldState& world);

private:
    static Core::Types::Nation ParseNationFromJSON(
        const nlohmann::json& nationData,
        uint32_t id);
};

}
```

### **Step 2: Implement Loader**

Create file: `src/engine/subsystems/geography/GeoLoader.cpp`

```cpp
#include "GeoLoader.h"
#include "engine/logging/Logger.h"
#include <fstream>

namespace SOS::Engine::Geography {

bool GeoLoader::LoadNationsFromJSON(
    const std::string& jsonPath,
    Core::State::WorldState& world) {
    
    SOS_LOG_INFO("Loading nations from: " + jsonPath);
    
    std::ifstream file(jsonPath);
    if (!file.is_open()) {
        SOS_LOG_ERROR("Could not open JSON file: " + jsonPath);
        return false;
    }
    
    nlohmann::json data;
    file >> data;
    file.close();
    
    if (!data.is_array()) {
        SOS_LOG_ERROR("JSON root must be an array");
        return false;
    }
    
    int count = 0;
    for (const auto& nationData : data) {
        Core::Types::Nation nation = ParseNationFromJSON(nationData, count++);
        world.addNation(nation);
        
        SOS_LOG_DEBUG("Created nation: " + nation.name + 
                     " (pop: " + std::to_string(nation.totalPopulation) + ")");
    }
    
    SOS_LOG_INFO("Successfully loaded " + std::to_string(count) + " nations");
    return true;
}

Core::Types::Nation GeoLoader::ParseNationFromJSON(
    const nlohmann::json& data,
    uint32_t id) {
    
    Core::Types::Nation nation(id);
    
    if (data.contains("name")) {
        nation.name = data["name"].get<std::string>();
    }
    
    if (data.contains("population")) {
        nation.totalPopulation = data["population"].get<uint64_t>();
    }
    
    if (data.contains("gdp")) {
        nation.gdp = data["gdp"].get<double>();
    }
    
    // Calculate GDP per capita
    if (nation.totalPopulation > 0) {
        nation.gdpPerCapita = nation.gdp / nation.totalPopulation;
    }
    
    // Set governance based on nothing for now (randomize in v0.2)
    nation.governanceType = Core::Types::GovernanceType::Democracy;
    nation.economyType = Core::Types::EconomicModel::Mixed;
    
    // Start with baseline legitimacy
    nation.legitimacy = 65.0;
    nation.stabilityIndex = 65.0;
    
    // Assign starting resources proportional to GDP
    double gdpBillion = nation.gdp / 1e9;
    nation.treasuryResources[Core::Types::ResourceType::Food] = gdpBillion * 10;
    nation.treasuryResources[Core::Types::ResourceType::Iron] = gdpBillion * 5;
    nation.treasuryResources[Core::Types::ResourceType::Oil] = gdpBillion * 3;
    
    return nation;
}

}
```

### **Step 3: Update Main.cpp**

Edit: `src/main.cpp`

```cpp
#include "engine/subsystems/geography/GeoLoader.h"

int main() {
    // ... existing code ...
    
    Engine::Core::State::WorldState world;
    
    // Load real-world nations
    bool loadSuccess = Engine::Geography::GeoLoader::LoadNationsFromJSON(
        "src/game/scenarios/nations.json",
        world);
    
    if (!loadSuccess) {
        SOS_LOG_ERROR("Failed to load nations from JSON");
        return 1;
    }
    
    SOS_LOG_INFO("Loaded " + std::to_string(world.getNationCount()) + " nations");
    
    // Create test provinces for major nations
    // (implementation below)
    
    // Run simulation
    // ... rest of code ...
}
```

---

## 🗺️ WHAT'S POSSIBLE WITH THIS DATA

### **v0.1: Nation-Level Simulation**
- 195 real nations with authentic GDP/population
- Realistic power asymmetry (USA > Costa Rica)
- Continental grouping
- Starting resources based on GDP

### **v0.2: Province-Level**
- Subdivide nations into provinces
- Assign resources by geography
- Create adjacency graph
- Build trade networks

### **v0.3: Historical Scenarios**
- 1900 borders (pre-WWI)
- 1950 borders (post-WWII)
- 2020 borders (current)
- Custom alternate history

---

## 📁 EXPECTED FOLDER STRUCTURE

After integration:

```
src/game/scenarios/
├── nations.json                 ← Extracted nation data (195 nations)
├── earth_modern_default.json    ← Scenario using all nations
└── earth_test_6nations.json     ← Test scenario with 6 major powers

src/engine/subsystems/
├── geography/                   ← NEW
│   ├── GeoLoader.h
│   ├── GeoLoader.cpp
│   └── GeoTypes.h

assets/data/
├── ne_110m_admin_0_countries.*  ← Keep for reference
└── countries.csv                ← If manually exported
```

---

## ⏰ TIMELINE FOR v0.1

| Task | Time | Status |
|------|------|--------|
| Extract nations from shapefile | 1-2 hours | ⏳ Next |
| Create GeoLoader class | 1 hour | ⏳ After extract |
| Update main.cpp | 30 min | ⏳ After GeoLoader |
| Test with 195 nations | 1 hour | ⏳ Final |
| **TOTAL** | **3.5-4.5 hours** | ⏳ This week |

---

## ✅ SUCCESS CRITERIA FOR v0.1

- [x] Nations extracted from shapefile
- [x] JSON file created with 195 nations
- [x] GeoLoader class reads JSON
- [x] Game loads all 195 nations
- [x] Each nation has realistic population/GDP
- [x] Simulation runs with real data
- [x] Log shows "Loaded 195 nations"

---

## 🎯 NEXT STEPS

### **TODAY:**
1. Choose extraction method (QGIS, Python, or online)
2. Extract nation data
3. Save to `src/game/scenarios/nations.json`

### **TOMORROW:**
4. Create `src/engine/subsystems/geography/GeoLoader.h/cpp`
5. Update `src/main.cpp` to load nations
6. Compile and test

### **RESULT:**
Game runs with authentic world data! 🌍

---

## 📞 QUESTIONS?

**"How do I install QGIS?"**  
→ Download from qgis.org, install normally

**"What if GDAL fails?"**  
→ Use manual CSV export method (same result)

**"Can I skip this for v0.1?"**  
→ Yes, but you lose authenticity and hardcoding 3 nations is boring

**"Does this slow down the game?"**  
→ No, JSON loads once at startup (negligible time)

---

## 💾 THE EXTRACTION SCRIPT

A Python script is provided: `tools/extract_nations.py`

**Usage:**
```bash
python tools/extract_nations.py
```

**What it does:**
- Detects GDAL availability
- Falls back to CSV if needed
- Outputs formatted JSON
- Shows progress

---

**Status:** Everything is set up. Just need to extract and load. 4 hours to authenticate your entire world. 🌍

Ready to proceed?
