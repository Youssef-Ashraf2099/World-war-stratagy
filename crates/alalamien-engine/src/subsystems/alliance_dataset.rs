//! Alliance Dataset - Predefined Blocs for V0.4
//!
//! Categories:
//! 1. SECURITY & MILITARY (War/Defense) — ONE PER NATION MAX
//! 2. ECONOMIC & TRADE (Commerce) — ONE PER NATION MAX  
//! 3. SCIENTIFIC & RESEARCH (Tech) — ONE PER NATION MAX
//! 4. CULTURAL & IDEOLOGICAL (Soft Power) — ONE PER NATION MAX
//! 5. REGIONAL & GEOGRAPHIC (Proximity) — ONE PER NATION MAX
//!
//! Rules:
//! - Nation can join max ONE alliance per category
//! - Nation cannot join same alliance twice
//! - Alliance membership reduces threat calculation with allies
//! - Affects which nations are considered "enemies of my enemy"

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedAlliance {
    pub id: String,
    pub name: String,
    pub category: AllianceCategory,
    pub doctrine: AllianceDoctrine,
    pub ai_behavior: AIBehavior,
    pub primary_perk: String,
    pub perk_description: String,
    pub secondary_perks: Vec<String>,
    pub real_world_mirror: String,
    pub founding_requirement: AllianceRequirement,
    pub join_requirement: AllianceRequirement,
    pub cohesion_decay_rate: f64,
    pub threat_reduction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceCategory {
    Military,
    Economic,
    Scientific,
    Cultural,
    Regional,
}

impl AllianceCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Military => "military",
            Self::Economic => "economic",
            Self::Scientific => "scientific",
            Self::Cultural => "cultural",
            Self::Regional => "regional",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceDoctrine {
    DefensiveAgreement,
    OffensivePact,
    EconomicBloc,
    ResearchConsortium,
    BalanceOfPower,
}

impl AllianceDoctrine {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DefensiveAgreement => "defensive_agreement",
            Self::OffensivePact => "offensive_pact",
            Self::EconomicBloc => "economic_bloc",
            Self::ResearchConsortium => "research_consortium",
            Self::BalanceOfPower => "balance_of_power",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehavior {
    Defensive,
    Mercenary,
    Ideological,
    Pragmatic,
    Hegemonic,
}

impl AIBehavior {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Defensive => "defensive",
            Self::Mercenary => "mercenary",
            Self::Ideological => "ideological",
            Self::Pragmatic => "pragmatic",
            Self::Hegemonic => "hegemonic",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceRequirement {
    pub min_gdp: f64,
    pub min_military_strength: f64,
    pub min_legitimacy: f64,
    pub required_resources: Vec<(String, f64)>,
    pub diplomatic_score_min: i32,
}

// ============================================================================
// SECURITY & MILITARY ALLIANCES
// ============================================================================

pub fn security_alliances() -> Vec<PredefinedAlliance> {
    vec![
        PredefinedAlliance {
            id: "SSP".to_string(),
            name: "Sovereign Shield Pact".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Defensive,
            primary_perk: "Iron Umbrella".to_string(),
            perk_description: "20% boost to defense when fighting within member's borders. Additional 10% if defending against non-member aggressor.".to_string(),
            secondary_perks: vec![
                "Early warning system: Detect enemy army movement 2 tiles away".to_string(),
                "Reinforcement speed: 15% faster deployment to allied borders".to_string(),
            ],
            real_world_mirror: "NATO".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 50000.0,
                min_military_strength: 1000.0,
                min_legitimacy: 60.0,
                required_resources: vec![("iron".to_string(), 500.0), ("oil".to_string(), 300.0)],
                diplomatic_score_min: 0,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 20000.0,
                min_military_strength: 400.0,
                min_legitimacy: 50.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            cohesion_decay_rate: 1.0,
            threat_reduction: 0.35,
        },
        PredefinedAlliance {
            id: "TSH".to_string(),
            name: "The Steel Hegemony".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::OffensivePact,
            ai_behavior: AIBehavior::Hegemonic,
            primary_perk: "Mass Mobilization".to_string(),
            perk_description: "Reduces recruitment cost of infantry units by 30%. Members gain access to leader's military doctrine bonuses.".to_string(),
            secondary_perks: vec![
                "Concentrated force: +5% damage per ally that has declared war on same target".to_string(),
                "Supply efficiency: -20% supply consumption for member armies".to_string(),
            ],
            real_world_mirror: "Warsaw Pact".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 60000.0,
                min_military_strength: 1500.0,
                min_legitimacy: 50.0,
                required_resources: vec![("iron".to_string(), 800.0)],
                diplomatic_score_min: -50,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 15000.0,
                min_military_strength: 300.0,
                min_legitimacy: 40.0,
                required_resources: vec![],
                diplomatic_score_min: 0,
            },
            cohesion_decay_rate: 2.0,
            threat_reduction: 0.25,
        },
        PredefinedAlliance {
            id: "VF".to_string(),
            name: "Vanguard Front".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Quick Response".to_string(),
            perk_description: "Air units move 50% faster to any ally currently under attack. Immediate rapid-response bonuses.".to_string(),
            secondary_perks: vec![
                "Coordination bonus: +10% combat effectiveness when 2+ allies fight together".to_string(),
                "Intelligence sharing: Fog of war partially lifted for all members".to_string(),
            ],
            real_world_mirror: "SEATO".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 35000.0,
                min_military_strength: 800.0,
                min_legitimacy: 55.0,
                required_resources: vec![("oil".to_string(), 400.0)],
                diplomatic_score_min: 5,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 18000.0,
                min_military_strength: 350.0,
                min_legitimacy: 45.0,
                required_resources: vec![],
                diplomatic_score_min: 5,
            },
            cohesion_decay_rate: 1.5,
            threat_reduction: 0.40,
        },
        PredefinedAlliance {
            id: "AC".to_string(),
            name: "Aegis Command".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Defensive,
            primary_perk: "Naval Superiority".to_string(),
            perk_description: "Shared vision of all coastal waters owned by members. Naval units gain +15% combat in member territory.".to_string(),
            secondary_perks: vec![
                "Blockade immunity: Member trade routes cannot be fully blockaded".to_string(),
                "Marine reinforcement: 25% faster naval unit deployment".to_string(),
            ],
            real_world_mirror: "ANZUS".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 45000.0,
                min_military_strength: 900.0,
                min_legitimacy: 65.0,
                required_resources: vec![("trade_ports".to_string(), 5.0)],
                diplomatic_score_min: 20,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 22000.0,
                min_military_strength: 450.0,
                min_legitimacy: 55.0,
                required_resources: vec![],
                diplomatic_score_min: 15,
            },
            cohesion_decay_rate: 1.2,
            threat_reduction: 0.45,
        },
        PredefinedAlliance {
            id: "TIP".to_string(),
            name: "The Iron Phalanx".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::OffensivePact,
            ai_behavior: AIBehavior::Hegemonic,
            primary_perk: "Blitz Doctrine".to_string(),
            perk_description: "Units gain +25% damage during first 5 turns of new war. Penalties decrease each turn after.".to_string(),
            secondary_perks: vec![
                "Coordinated assault: Combined armies move 20% faster".to_string(),
                "Shock tactics: 40% faster initial assault, -50% penalty if defending instead".to_string(),
            ],
            real_world_mirror: "Axis Powers".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 70000.0,
                min_military_strength: 1800.0,
                min_legitimacy: 40.0,
                required_resources: vec![("iron".to_string(), 1000.0), ("oil".to_string(), 700.0)],
                diplomatic_score_min: -70,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 25000.0,
                min_military_strength: 600.0,
                min_legitimacy: 35.0,
                required_resources: vec![],
                diplomatic_score_min: -50,
            },
            cohesion_decay_rate: 2.5,
            threat_reduction: 0.20,
        },
        PredefinedAlliance {
            id: "UP".to_string(),
            name: "United Peacekeepers".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Defensive,
            primary_perk: "Conflict Mediation".to_string(),
            perk_description: "Can deploy neutral peacekeeping forces to disputed regions. Peace proposals between members cost -30% legitimacy.".to_string(),
            secondary_perks: vec![
                "Humanitarian aid: War weariness reduced by 25% between members".to_string(),
                "Neutral ground: Wars between members resolve faster (50% duration reduction)".to_string(),
            ],
            real_world_mirror: "UN Peacekeeping".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 55000.0,
                min_military_strength: 1100.0,
                min_legitimacy: 75.0,
                required_resources: vec![],
                diplomatic_score_min: 30,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 20000.0,
                min_military_strength: 400.0,
                min_legitimacy: 65.0,
                required_resources: vec![],
                diplomatic_score_min: 25,
            },
            cohesion_decay_rate: 0.8,
            threat_reduction: 0.50,
        },
        PredefinedAlliance {
            id: "CW".to_string(),
            name: "Commonwealth Accord".to_string(),
            category: AllianceCategory::Military,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Global Reach".to_string(),
            perk_description: "Members can project power across continents. Armies deployed to distant allies gain +10% combat effectiveness.".to_string(),
            secondary_perks: vec![
                "Colonial advantage: More provinces can be colonized by members".to_string(),
                "Naval networks: Trade routes gain +20% efficiency connecting members".to_string(),
            ],
            real_world_mirror: "Commonwealth of Nations".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 65000.0,
                min_military_strength: 1400.0,
                min_legitimacy: 70.0,
                required_resources: vec![("trade_ports".to_string(), 8.0)],
                diplomatic_score_min: 20,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 24000.0,
                min_military_strength: 500.0,
                min_legitimacy: 60.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            cohesion_decay_rate: 1.3,
            threat_reduction: 0.38,
        },
    ]
}

// ============================================================================
// ECONOMIC ALLIANCES
// ============================================================================

pub fn economic_alliances() -> Vec<PredefinedAlliance> {
    vec![
        PredefinedAlliance {
            id: "GEB".to_string(),
            name: "Global Exchange Bloc".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Open Borders".to_string(),
            perk_description: "Trade units generate +50% Gold when traveling between members. Reduced tariffs (-25% trade cost).".to_string(),
            secondary_perks: vec![
                "Free movement: Citizens can migrate between member territories freely".to_string(),
                "Shared market: Pool of available goods increased for all members".to_string(),
            ],
            real_world_mirror: "European Union".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 50000.0,
                min_military_strength: 400.0,
                min_legitimacy: 70.0,
                required_resources: vec![("trade_ports".to_string(), 6.0)],
                diplomatic_score_min: 30,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 18000.0,
                min_military_strength: 200.0,
                min_legitimacy: 60.0,
                required_resources: vec![],
                diplomatic_score_min: 20,
            },
            cohesion_decay_rate: 1.0,
            threat_reduction: 0.40,
        },
        PredefinedAlliance {
            id: "ORS".to_string(),
            name: "O.R.E. Syndicate".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Mercenary,
            primary_perk: "Market Corner".to_string(),
            perk_description: "Members collectively control resource prices. Set price floor for rare minerals +30%. Non-member purchases cost 40% more.".to_string(),
            secondary_perks: vec![
                "Supply leverage: Can embargo resources to enemies during war".to_string(),
                "Member support: Members buy from each other at -20% cost".to_string(),
            ],
            real_world_mirror: "OPEC".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 35000.0,
                min_military_strength: 300.0,
                min_legitimacy: 50.0,
                required_resources: vec![("oil".to_string(), 600.0)],
                diplomatic_score_min: 0,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 15000.0,
                min_military_strength: 150.0,
                min_legitimacy: 40.0,
                required_resources: vec![("oil".to_string(), 200.0)],
                diplomatic_score_min: -10,
            },
            cohesion_decay_rate: 2.5,
            threat_reduction: 0.15,
        },
        PredefinedAlliance {
            id: "SCU".to_string(),
            name: "Silk & Circuit Union".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Infrastructure Hub".to_string(),
            perk_description: "Reduces cost of building roads, rails, ports by 25%. Infrastructure connects member territories (shared logistics network).".to_string(),
            secondary_perks: vec![
                "Development fund: Members pool resources for infrastructure growth".to_string(),
                "Tech transfer: Industrial technology spreads faster between allies".to_string(),
            ],
            real_world_mirror: "BRICS".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 40000.0,
                min_military_strength: 500.0,
                min_legitimacy: 55.0,
                required_resources: vec![("iron".to_string(), 400.0)],
                diplomatic_score_min: 10,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 16000.0,
                min_military_strength: 200.0,
                min_legitimacy: 45.0,
                required_resources: vec![],
                diplomatic_score_min: 5,
            },
            cohesion_decay_rate: 1.2,
            threat_reduction: 0.30,
        },
        PredefinedAlliance {
            id: "MG".to_string(),
            name: "Merchant's Guild".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Mercenary,
            primary_perk: "Mercenary Fund".to_string(),
            perk_description: "Can hire neutral military units using 15% less Gold. Access to mercenary armies no other alliance can recruit.".to_string(),
            secondary_perks: vec![
                "Trade insurance: Insurance against piracy costs -40%".to_string(),
                "Banking network: Can transfer wealth between members instantly".to_string(),
            ],
            real_world_mirror: "Hanseatic League".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 45000.0,
                min_military_strength: 250.0,
                min_legitimacy: 60.0,
                required_resources: vec![("trade_ports".to_string(), 7.0)],
                diplomatic_score_min: 15,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 17000.0,
                min_military_strength: 100.0,
                min_legitimacy: 50.0,
                required_resources: vec![("trade_ports".to_string(), 2.0)],
                diplomatic_score_min: 5,
            },
            cohesion_decay_rate: 1.5,
            threat_reduction: 0.25,
        },
        PredefinedAlliance {
            id: "GA".to_string(),
            name: "The Gilded Accord".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Hegemonic,
            primary_perk: "Financial Leverage".to_string(),
            perk_description: "Can freeze enemy bank accounts during war (reduces their Gold income by 40%). Set economic sanctions on non-allies.".to_string(),
            secondary_perks: vec![
                "Capital accumulation: Members earn +15% Gold from all sources".to_string(),
                "Debt leverage: Can force nations to join via economic pressure".to_string(),
            ],
            real_world_mirror: "G7".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 80000.0,
                min_military_strength: 600.0,
                min_legitimacy: 65.0,
                required_resources: vec![],
                diplomatic_score_min: 25,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 35000.0,
                min_military_strength: 300.0,
                min_legitimacy: 55.0,
                required_resources: vec![],
                diplomatic_score_min: 20,
            },
            cohesion_decay_rate: 1.8,
            threat_reduction: 0.35,
        },
        PredefinedAlliance {
            id: "NU".to_string(),
            name: "Nordic Union".to_string(),
            category: AllianceCategory::Economic,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Cooperative Prosperity".to_string(),
            perk_description: "Members share profits from all industries. Combined GDP grows faster (+10% for members). Equal wealth distribution bonus.".to_string(),
            secondary_perks: vec![
                "Social stability: War weariness reduced by 20% in member population".to_string(),
                "Equal opportunity: All members have equal voice in alliance decisions".to_string(),
            ],
            real_world_mirror: "Nordic Union".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 30000.0,
                min_military_strength: 300.0,
                min_legitimacy: 80.0,
                required_resources: vec![],
                diplomatic_score_min: 40,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 12000.0,
                min_military_strength: 100.0,
                min_legitimacy: 70.0,
                required_resources: vec![],
                diplomatic_score_min: 35,
            },
            cohesion_decay_rate: 0.6,
            threat_reduction: 0.50,
        },
    ]
}

// ============================================================================
// SCIENTIFIC ALLIANCES
// ============================================================================

pub fn scientific_alliances() -> Vec<PredefinedAlliance> {
    vec![
        PredefinedAlliance {
            id: "AH".to_string(),
            name: "Apex Helix".to_string(),
            category: AllianceCategory::Scientific,
            doctrine: AllianceDoctrine::ResearchConsortium,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Biological Defense".to_string(),
            perk_description: "Immune to plague/disease effects on armies. Wounded troops heal 30% faster. Bioweapon research locked (peaceful doctrine).".to_string(),
            secondary_perks: vec![
                "Health infrastructure: Medical buildings provide +20% unit recovery".to_string(),
                "Research sharing: Biology/medicine tech spreads freely between members".to_string(),
            ],
            real_world_mirror: "WHO".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 35000.0,
                min_military_strength: 200.0,
                min_legitimacy: 75.0,
                required_resources: vec![("rare_earths".to_string(), 80.0)],
                diplomatic_score_min: 30,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 13000.0,
                min_military_strength: 100.0,
                min_legitimacy: 65.0,
                required_resources: vec![],
                diplomatic_score_min: 25,
            },
            cohesion_decay_rate: 0.8,
            threat_reduction: 0.45,
        },
        PredefinedAlliance {
            id: "ZP".to_string(),
            name: "The Zenith Project".to_string(),
            category: AllianceCategory::Scientific,
            doctrine: AllianceDoctrine::ResearchConsortium,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Space Race Advancement".to_string(),
            perk_description: "Unlock 'Science Victory' conditions. Space program costs -40%. Can't wage wars (research-only doctrine).".to_string(),
            secondary_perks: vec![
                "Orbital advantages: Satellites provide global vision and research bonuses".to_string(),
                "Tech acceleration: All science/tech research 25% faster".to_string(),
            ],
            real_world_mirror: "NASA/ESA".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 70000.0,
                min_military_strength: 100.0,
                min_legitimacy: 80.0,
                required_resources: vec![("rare_earths".to_string(), 200.0)],
                diplomatic_score_min: 40,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 25000.0,
                min_military_strength: 50.0,
                min_legitimacy: 70.0,
                required_resources: vec![],
                diplomatic_score_min: 35,
            },
            cohesion_decay_rate: 0.5,
            threat_reduction: 0.50,
        },
        PredefinedAlliance {
            id: "GGF".to_string(),
            name: "Gene-Guard Front".to_string(),
            category: AllianceCategory::Scientific,
            doctrine: AllianceDoctrine::ResearchConsortium,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Advanced Genetic Enhancement".to_string(),
            perk_description: "Humanitarian facade: -50% legitimacy damage from wars. Secret benefit: Elite genetic soldiers cost less and are stronger (+20%).".to_string(),
            secondary_perks: vec![
                "Population growth: Recruitment pools increase 15% faster in member territories".to_string(),
                "Forbidden research: Access to unique biotech units other alliances cannot get".to_string(),
            ],
            real_world_mirror: "Red Cross".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 40000.0,
                min_military_strength: 400.0,
                min_legitimacy: 50.0,
                required_resources: vec![("rare_earths".to_string(), 120.0)],
                diplomatic_score_min: 10,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 15000.0,
                min_military_strength: 150.0,
                min_legitimacy: 40.0,
                required_resources: vec![],
                diplomatic_score_min: 0,
            },
            cohesion_decay_rate: 2.0,
            threat_reduction: 0.20,
        },
        PredefinedAlliance {
            id: "ARR".to_string(),
            name: "Aether Research Ring".to_string(),
            category: AllianceCategory::Scientific,
            doctrine: AllianceDoctrine::ResearchConsortium,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Advanced Weapons Research".to_string(),
            perk_description: "Experimental weapons tech (plasma, lasers, energy weapons) +30% faster. Can deploy exclusive high-tech units.".to_string(),
            secondary_perks: vec![
                "Physics breakthrough: New unit types unlock from particle research".to_string(),
                "Energy efficiency: Military units consume -20% fuel".to_string(),
            ],
            real_world_mirror: "CERN".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 65000.0,
                min_military_strength: 800.0,
                min_legitimacy: 72.0,
                required_resources: vec![("rare_earths".to_string(), 180.0), ("oil".to_string(), 400.0)],
                diplomatic_score_min: 30,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 28000.0,
                min_military_strength: 350.0,
                min_legitimacy: 62.0,
                required_resources: vec![],
                diplomatic_score_min: 20,
            },
            cohesion_decay_rate: 1.2,
            threat_reduction: 0.35,
        },
        PredefinedAlliance {
            id: "EG".to_string(),
            name: "Earth's Guardian Coalition".to_string(),
            category: AllianceCategory::Scientific,
            doctrine: AllianceDoctrine::ResearchConsortium,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Environmental Management".to_string(),
            perk_description: "Environmental disasters less severe for members. Agricultural output +15% (sustainable farming). Industry pollution reduced.".to_string(),
            secondary_perks: vec![
                "Climate resilience: Bonus vs natural disasters, famine".to_string(),
                "Green technology: Renewable energy sources available".to_string(),
            ],
            real_world_mirror: "IPCC".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 32000.0,
                min_military_strength: 200.0,
                min_legitimacy: 78.0,
                required_resources: vec![],
                diplomatic_score_min: 45,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 11000.0,
                min_military_strength: 80.0,
                min_legitimacy: 68.0,
                required_resources: vec![],
                diplomatic_score_min: 40,
            },
            cohesion_decay_rate: 0.7,
            threat_reduction: 0.48,
        },
    ]
}

// ============================================================================
// CULTURAL ALLIANCES
// ============================================================================

pub fn cultural_alliances() -> Vec<PredefinedAlliance> {
    vec![
        PredefinedAlliance {
            id: "LB".to_string(),
            name: "Liberal Brotherhood".to_string(),
            category: AllianceCategory::Cultural,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Democratic Strength".to_string(),
            perk_description: "Government stability increases (less coup risk). Population happiness +20%. Citizens less likely to defect.".to_string(),
            secondary_perks: vec![
                "Free press: Better intel on enemy locations (+15% spy accuracy)".to_string(),
                "Civil society: More productive workforce (+10% production)".to_string(),
            ],
            real_world_mirror: "Liberal Democracies".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 42000.0,
                min_military_strength: 500.0,
                min_legitimacy: 80.0,
                required_resources: vec![],
                diplomatic_score_min: 40,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 16000.0,
                min_military_strength: 200.0,
                min_legitimacy: 70.0,
                required_resources: vec![],
                diplomatic_score_min: 35,
            },
            cohesion_decay_rate: 0.9,
            threat_reduction: 0.42,
        },
        PredefinedAlliance {
            id: "CD".to_string(),
            name: "Communist Collective".to_string(),
            category: AllianceCategory::Cultural,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "United Workers".to_string(),
            perk_description: "Production bases grow 25% faster (collective mobilization). Unified economic planning between members.".to_string(),
            secondary_perks: vec![
                "Class consciousness: Population more resistant to capitalist propaganda".to_string(),
                "Worker's paradise: Lower wage costs for military recruitment".to_string(),
            ],
            real_world_mirror: "Soviet Bloc".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 38000.0,
                min_military_strength: 700.0,
                min_legitimacy: 50.0,
                required_resources: vec![],
                diplomatic_score_min: -20,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 14000.0,
                min_military_strength: 250.0,
                min_legitimacy: 40.0,
                required_resources: vec![],
                diplomatic_score_min: -25,
            },
            cohesion_decay_rate: 2.2,
            threat_reduction: 0.28,
        },
        PredefinedAlliance {
            id: "IC".to_string(),
            name: "Islamic Conference".to_string(),
            category: AllianceCategory::Cultural,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Ummah Solidarity".to_string(),
            perk_description: "War against Islamic members considered grave offense. Members gain religious unity bonus (+25% morale).".to_string(),
            secondary_perks: vec![
                "Shared pilgrimage sites: +15% happiness and trade between members".to_string(),
                "Islamic law unity: Coordinated justice system, less crime".to_string(),
            ],
            real_world_mirror: "Organisation of Islamic Cooperation".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 35000.0,
                min_military_strength: 600.0,
                min_legitimacy: 65.0,
                required_resources: vec![],
                diplomatic_score_min: 20,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 13000.0,
                min_military_strength: 200.0,
                min_legitimacy: 55.0,
                required_resources: vec![],
                diplomatic_score_min: 15,
            },
            cohesion_decay_rate: 1.4,
            threat_reduction: 0.38,
        },
        PredefinedAlliance {
            id: "AU".to_string(),
            name: "African Unity Front".to_string(),
            category: AllianceCategory::Cultural,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Pan-African Solidarity".to_string(),
            perk_description: "Colonial land claims reversed (recovering lost territory). Joint development programs (+12% growth in agriculture).".to_string(),
            secondary_perks: vec![
                "Anti-imperialism: Defensive bonus against non-African aggressors (+25%)".to_string(),
                "Cultural preservation: Heritage sites provide unique bonuses".to_string(),
            ],
            real_world_mirror: "African Union".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 25000.0,
                min_military_strength: 400.0,
                min_legitimacy: 70.0,
                required_resources: vec![],
                diplomatic_score_min: 25,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 10000.0,
                min_military_strength: 150.0,
                min_legitimacy: 60.0,
                required_resources: vec![],
                diplomatic_score_min: 20,
            },
            cohesion_decay_rate: 1.1,
            threat_reduction: 0.40,
        },
        PredefinedAlliance {
            id: "RA".to_string(),
            name: "Royal Alliance".to_string(),
            category: AllianceCategory::Cultural,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Ideological,
            primary_perk: "Divine Legitimacy".to_string(),
            perk_description: "Succession protected (no instability on regime change). Monarchies gain +30% legitimacy in peaceful periods.".to_string(),
            secondary_perks: vec![
                "Royal marriage: Can create political unions with other monarchies".to_string(),
                "Throne defense: Members will defend each other's dynasties".to_string(),
            ],
            real_world_mirror: "Monarchist Alliances".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 30000.0,
                min_military_strength: 400.0,
                min_legitimacy: 75.0,
                required_resources: vec![],
                diplomatic_score_min: 30,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 12000.0,
                min_military_strength: 150.0,
                min_legitimacy: 65.0,
                required_resources: vec![],
                diplomatic_score_min: 25,
            },
            cohesion_decay_rate: 1.0,
            threat_reduction: 0.44,
        },
    ]
}

// ============================================================================
// REGIONAL ALLIANCES
// ============================================================================

pub fn regional_alliances() -> Vec<PredefinedAlliance> {
    vec![
        PredefinedAlliance {
            id: "MS".to_string(),
            name: "Mediterranean Summit".to_string(),
            category: AllianceCategory::Regional,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Maritime Trade Routes".to_string(),
            perk_description: "Coastal members gain +40% trade efficiency. Naval units move 20% faster in regional waters.".to_string(),
            secondary_perks: vec![
                "Port cooperation: Shared harbors and shipyards between members".to_string(),
                "Piracy suppression: Joint naval patrols protect member ships".to_string(),
            ],
            real_world_mirror: "Mediterranean Partnership".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 28000.0,
                min_military_strength: 300.0,
                min_legitimacy: 60.0,
                required_resources: vec![("trade_ports".to_string(), 4.0)],
                diplomatic_score_min: 15,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 10000.0,
                min_military_strength: 100.0,
                min_legitimacy: 50.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            cohesion_decay_rate: 1.3,
            threat_reduction: 0.32,
        },
        PredefinedAlliance {
            id: "CARS".to_string(),
            name: "Central Asian Regional Stability".to_string(),
            category: AllianceCategory::Regional,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Silk Road Cooperation".to_string(),
            perk_description: "Overland trade routes cost 30% less. Caravan units move 30% faster across member territories.".to_string(),
            secondary_perks: vec![
                "Border stability: Reduced conflicts along member boundaries".to_string(),
                "Water rights: Fair distribution of regional water resources".to_string(),
            ],
            real_world_mirror: "Central Asian Organization".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 22000.0,
                min_military_strength: 300.0,
                min_legitimacy: 55.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 9000.0,
                min_military_strength: 100.0,
                min_legitimacy: 45.0,
                required_resources: vec![],
                diplomatic_score_min: 5,
            },
            cohesion_decay_rate: 1.5,
            threat_reduction: 0.28,
        },
        PredefinedAlliance {
            id: "EAPC".to_string(),
            name: "East Asian Prosperity Circle".to_string(),
            category: AllianceCategory::Regional,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Regional Economic Hub".to_string(),
            perk_description: "Manufacturing efficiency +25% in member territories. High-tech production unlocked for industrial regions.".to_string(),
            secondary_perks: vec![
                "Tech corridor: Innovation spreads 30% faster".to_string(),
                "Supply chain integration: Factories work together across borders".to_string(),
            ],
            real_world_mirror: "East Asian Economic Community".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 55000.0,
                min_military_strength: 700.0,
                min_legitimacy: 65.0,
                required_resources: vec![("trade_ports".to_string(), 5.0)],
                diplomatic_score_min: 20,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 20000.0,
                min_military_strength: 300.0,
                min_legitimacy: 55.0,
                required_resources: vec![],
                diplomatic_score_min: 15,
            },
            cohesion_decay_rate: 1.1,
            threat_reduction: 0.36,
        },
        PredefinedAlliance {
            id: "SAARC".to_string(),
            name: "South Asian Economic Association".to_string(),
            category: AllianceCategory::Regional,
            doctrine: AllianceDoctrine::EconomicBloc,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Spice Trade Mastery".to_string(),
            perk_description: "Agricultural and luxury trade +35% value. Spice routes controlled by members generate continuous Gold.".to_string(),
            secondary_perks: vec![
                "Culinary unity: Population happiness from shared food culture".to_string(),
                "Textile industry: Production chain efficiency +20%".to_string(),
            ],
            real_world_mirror: "SAARC".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 32000.0,
                min_military_strength: 400.0,
                min_legitimacy: 60.0,
                required_resources: vec![("food".to_string(), 500.0)],
                diplomatic_score_min: 15,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 12000.0,
                min_military_strength: 150.0,
                min_legitimacy: 50.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            cohesion_decay_rate: 1.8,
            threat_reduction: 0.26,
        },
        PredefinedAlliance {
            id: "LARC".to_string(),
            name: "Latin American Regional Council".to_string(),
            category: AllianceCategory::Regional,
            doctrine: AllianceDoctrine::BalanceOfPower,
            ai_behavior: AIBehavior::Pragmatic,
            primary_perk: "Continental Independence".to_string(),
            perk_description: "Defensive bonus against non-regional powers (+20%). Colonial historical sites provide unique bonuses.".to_string(),
            secondary_perks: vec![
                "Amazon cooperation: Joint environmental management of rainforests".to_string(),
                "Regional trade corridor: Internal trade routes 40% cheaper".to_string(),
            ],
            real_world_mirror: "CELAC".to_string(),
            founding_requirement: AllianceRequirement {
                min_gdp: 26000.0,
                min_military_strength: 350.0,
                min_legitimacy: 62.0,
                required_resources: vec![],
                diplomatic_score_min: 15,
            },
            join_requirement: AllianceRequirement {
                min_gdp: 10000.0,
                min_military_strength: 120.0,
                min_legitimacy: 52.0,
                required_resources: vec![],
                diplomatic_score_min: 10,
            },
            cohesion_decay_rate: 1.4,
            threat_reduction: 0.34,
        },
    ]
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

pub fn all_alliances() -> Vec<PredefinedAlliance> {
    let mut all = Vec::new();
    all.extend(security_alliances());
    all.extend(economic_alliances());
    all.extend(scientific_alliances());
    all.extend(cultural_alliances());
    all.extend(regional_alliances());
    all
}

pub fn alliances_by_category(category: AllianceCategory) -> Vec<PredefinedAlliance> {
    all_alliances()
        .into_iter()
        .filter(|a| a.category == category)
        .collect()
}

pub fn alliance_by_id(id: &str) -> Option<PredefinedAlliance> {
    all_alliances().into_iter().find(|a| a.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_alliances_loaded() {
        let alliances = all_alliances();
        assert!(alliances.len() >= 25, "Should have 25+ predefined alliances");
        println!("✓ Loaded {} alliances", alliances.len());
    }

    #[test]
    fn test_alliance_categories_complete() {
        let military = alliances_by_category(AllianceCategory::Military);
        let economic = alliances_by_category(AllianceCategory::Economic);
        let scientific = alliances_by_category(AllianceCategory::Scientific);
        let cultural = alliances_by_category(AllianceCategory::Cultural);
        let regional = alliances_by_category(AllianceCategory::Regional);

        assert!(military.len() >= 7, "Military: {}", military.len());
        assert!(economic.len() >= 6, "Economic: {}", economic.len());
        assert!(scientific.len() >= 5, "Scientific: {}", scientific.len());
        assert!(cultural.len() >= 5, "Cultural: {}", cultural.len());
        assert!(regional.len() >= 5, "Regional: {}", regional.len());
    }

    #[test]
    fn test_alliance_uniqueness() {
        let alliances = all_alliances();
        let ids: Vec<String> = alliances.iter().map(|a| a.id.clone()).collect();
        for id in &ids {
            assert_eq!(ids.iter().filter(|i| *i == id).count(), 1, "Duplicate: {}", id);
        }
    }

    #[test]
    fn test_alliance_perks_documented() {
        let alliances = all_alliances();
        for alliance in alliances {
            assert!(!alliance.primary_perk.is_empty());
            assert!(!alliance.perk_description.is_empty());
        }
    }
}
