//! Garupa protobuf message schemas, ported field-for-field from
//! GarupaSpeedTracker's `src/types/garupaSchema/*`.

use super::schema::{field, ProtoType, Schema};

/// Declares the ubiquitous protobuf `entries: repeated Message` wrapper.
macro_rules! list_schema {
    ($name:ident => $entry:ident) => {
        pub static $name: Schema = Schema {
            fields: &[(1, field("entries", ProtoType::Message(&$entry), true))],
        };
    };
}

/// Declares protobuf's encoded key/value entry used by map fields.
macro_rules! map_entry_schema {
    ($name:ident => $key_type:ident, $value:ident) => {
        pub static $name: Schema = Schema {
            fields: &[
                (1, field("key", ProtoType::$key_type, false)),
                (2, field("value", ProtoType::Message(&$value), false)),
            ],
        };
    };
}

// ============================================================================
// Ranking user, shared by monthly and event rankings
// ============================================================================

pub static USER_DECK_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("deckId", ProtoType::Int, false)),
        (2, field("deckName", ProtoType::String, false)),
        (3, field("leader", ProtoType::Int, false)),
        (4, field("member1", ProtoType::Int, false)),
        (5, field("member2", ProtoType::Int, false)),
        (6, field("member3", ProtoType::Int, false)),
        (7, field("member4", ProtoType::Int, false)),
        (8, field("bondsEffectIds", ProtoType::Int, true)),
        (10, field("deckType", ProtoType::String, false)),
    ],
};

pub static USER_APPEND_PARAMETER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("performance", ProtoType::Int, false)),
        (4, field("technique", ProtoType::Int, false)),
        (5, field("visual", ProtoType::Int, false)),
        (6, field("characterPotentialPerformance", ProtoType::Int, false)),
        (7, field("characterPotentialTechnique", ProtoType::Int, false)),
        (8, field("characterPotentialVisual", ProtoType::Int, false)),
        (9, field("characterBonusPerformance", ProtoType::Int, false)),
        (10, field("characterBonusTechnique", ProtoType::Int, false)),
        (11, field("characterBonusVisual", ProtoType::Int, false)),
    ],
};

pub static USER_SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("exp", ProtoType::Int, false)),
        (5, field("createdAt", ProtoType::Long, false)),
        (6, field("addExp", ProtoType::Int, false)),
        (7, field("trainingStatus", ProtoType::String, false)),
        (8, field("duplicateCount", ProtoType::Int, false)),
        (9, field("illust", ProtoType::String, false)),
        (10, field("skillExp", ProtoType::Int, false)),
        (11, field("skillLevel", ProtoType::Int, false)),
        (
            12,
            field("userAppendParameter", ProtoType::Message(&USER_APPEND_PARAMETER_SCHEMA), false),
        ),
        (13, field("limitBreakRank", ProtoType::Int, false)),
    ],
};

list_schema!(USER_SITUATION_LIST_SCHEMA => USER_SITUATION_SCHEMA);

pub static USER_PROFILE_SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("situationId", ProtoType::Int, false)),
        (3, field("illust", ProtoType::String, false)),
        (4, field("viewProfileSituationStatus", ProtoType::String, false)),
    ],
};

pub static USER_PROFILE_DEGREE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("profileDegreeType", ProtoType::String, false)),
        (3, field("degreeId", ProtoType::Int, false)),
    ],
};

map_entry_schema!(USER_PROFILE_DEGREE_MAP_ENTRY_SCHEMA => String, USER_PROFILE_DEGREE_SCHEMA);

list_schema!(USER_PROFILE_DEGREE_MAP_SCHEMA => USER_PROFILE_DEGREE_MAP_ENTRY_SCHEMA);

pub static RANKING_USER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("name", ProtoType::String, false)),
        (2, field("ownFlg", ProtoType::Bool, false)),
        (3, field("rankLevel", ProtoType::Int, false)),
        (4, field("introduction", ProtoType::String, false)),
        (5, field("rank", ProtoType::Int, false)),
        (6, field("point", ProtoType::Int, false)),
        (7, field("userId", ProtoType::Int, false)),
        (8, field("degreeId", ProtoType::Int, false)),
        (9, field("userDeck", ProtoType::Message(&USER_DECK_SCHEMA), false)),
        (
            10,
            field("userSituationList", ProtoType::Message(&USER_SITUATION_LIST_SCHEMA), false),
        ),
        (
            11,
            field("userProfileSituation", ProtoType::Message(&USER_PROFILE_SITUATION_SCHEMA), false),
        ),
        (
            12,
            field("userProfileDegreeMap", ProtoType::Message(&USER_PROFILE_DEGREE_MAP_SCHEMA), false),
        ),
    ],
};

list_schema!(RANKING_USER_LIST_SCHEMA => RANKING_USER_SCHEMA);

// ============================================================================
// Monthly ranking
// ============================================================================

pub static USER_MONTHLY_RANKING_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (
            1,
            field("monthlyRankingPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (
            2,
            field("monthlyRankingPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (
            3,
            field(
                "monthlyRankingPointBorderUsers",
                ProtoType::Message(&RANKING_USER_LIST_SCHEMA),
                false,
            ),
        ),
    ],
};

pub static MASTER_MONTHLY_RANKING_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("monthlyRankingId", ProtoType::Int, false)),
        (3, field("fromRank", ProtoType::Int, false)),
        (4, field("toRank", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
    ],
};

pub static MASTER_MONTHLY_RANKING_GRADE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("monthlyRankingId", ProtoType::Int, false)),
        (3, field("gradeAheadType", ProtoType::String, false)),
        (4, field("pt", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
        (8, field("rankingThresholdFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_MONTHLY_RANKING_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("monthlyRankingId", ProtoType::Int, false)),
        (2, field("monthlyRankingName", ProtoType::String, false)),
        (3, field("assetBundleName", ProtoType::String, false)),
        (4, field("bgmAssetBundleName", ProtoType::String, false)),
        (5, field("bgmFileName", ProtoType::String, false)),
        (6, field("startAt", ProtoType::Long, false)),
        (7, field("endAt", ProtoType::Long, false)),
        (8, field("enableFlg", ProtoType::Bool, false)),
        (9, field("publicStartAt", ProtoType::Long, false)),
        (10, field("publicEndAt", ProtoType::Long, false)),
        (11, field("distributionStartAt", ProtoType::Long, false)),
        (12, field("distributionEndAt", ProtoType::Long, false)),
        (13, field("receptionEndAt", ProtoType::Long, false)),
        (14, field("aggregateEndAt", ProtoType::Long, false)),
        (
            101,
            field("rewards", ProtoType::Message(&MASTER_MONTHLY_RANKING_REWARD_SCHEMA), true),
        ),
        (102, field("grades", ProtoType::Message(&MASTER_MONTHLY_RANKING_GRADE_SCHEMA), true)),
    ],
};

list_schema!(MASTER_MONTHLY_RANKING_LIST_SCHEMA => MASTER_MONTHLY_RANKING_SCHEMA);

// ============================================================================
// Event ranking, one response schema per event type
// ============================================================================

pub static USER_MEDLEY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (
            1,
            field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (
            5,
            field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (6, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_LIVE_TRY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (
            3,
            field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
    ],
};

pub static USER_STORY_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_CHALLENGE_MUSIC_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_CHALLENGE_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (
            1,
            field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (
            3,
            field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (
            101,
            field(
                "challengeMusicRankings",
                ProtoType::Message(&USER_CHALLENGE_MUSIC_RANKING_RESPONSE_SCHEMA),
                true,
            ),
        ),
    ],
};

pub static USER_MISSION_LIVE_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("borderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_TEAM_LIVE_FESTIVAL_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("nearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (2, field("topUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (
            3,
            field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
    ],
};

pub static USER_VERSUS_MUSIC_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("scoreNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (3, field("scoreTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (4, field("scoreBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
    ],
};

pub static USER_VERSUS_EVENT_RANKING_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (
            1,
            field("eventPointNearUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
        (2, field("eventPointTopUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false)),
        (
            3,
            field(
                "versusMusicRankings",
                ProtoType::Message(&USER_VERSUS_MUSIC_RANKING_RESPONSE_SCHEMA),
                true,
            ),
        ),
        (
            4,
            field("eventPointBorderUsers", ProtoType::Message(&RANKING_USER_LIST_SCHEMA), false),
        ),
    ],
};

/// Maps a protobuf event type string to its ranking response schema.
pub static EVENT_TYPE_SCHEMAS: &[(&str, &Schema)] = &[
    ("medley", &USER_MEDLEY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("challenge", &USER_CHALLENGE_EVENT_RANKING_RESPONSE_SCHEMA),
    ("versus", &USER_VERSUS_EVENT_RANKING_RESPONSE_SCHEMA),
    ("live_try", &USER_LIVE_TRY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("story", &USER_STORY_EVENT_RANKING_RESPONSE_SCHEMA),
    ("mission_live", &USER_MISSION_LIVE_EVENT_RANKING_RESPONSE_SCHEMA),
    ("team_live_festival", &USER_TEAM_LIVE_FESTIVAL_EVENT_RANKING_RESPONSE_SCHEMA),
];

// ============================================================================
// Event master
// ============================================================================

pub static MASTER_EVENT_POINT_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("eventId", ProtoType::Int, false)),
        (3, field("point", ProtoType::Long, false)),
        (4, field("rewardType", ProtoType::String, false)),
        (5, field("rewardId", ProtoType::Int, false)),
        (6, field("rewardQuantity", ProtoType::Int, false)),
        (7, field("recommendFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_EVENT_RANKING_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("eventId", ProtoType::Int, false)),
        (3, field("fromRank", ProtoType::Int, false)),
        (4, field("toRank", ProtoType::Int, false)),
        (5, field("rewardType", ProtoType::String, false)),
        (6, field("rewardId", ProtoType::Int, false)),
        (7, field("rewardQuantity", ProtoType::Int, false)),
        (8, field("recommendFlg", ProtoType::Bool, false)),
    ],
};

pub static MASTER_EVENT_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("eventId", ProtoType::Int, false)),
        (2, field("eventType", ProtoType::String, false)),
        (3, field("eventName", ProtoType::String, false)),
        (4, field("assetBundleName", ProtoType::String, false)),
        (5, field("startAt", ProtoType::Long, false)),
        (6, field("endAt", ProtoType::Long, false)),
        (7, field("enableFlg", ProtoType::Bool, false)),
        (8, field("publicStartAt", ProtoType::Long, false)),
        (9, field("publicEndAt", ProtoType::Long, false)),
        (10, field("distributionStartAt", ProtoType::Long, false)),
        (11, field("distributionEndAt", ProtoType::Long, false)),
        (12, field("bgmAssetBundleName", ProtoType::String, false)),
        (13, field("bgmFileName", ProtoType::String, false)),
        (14, field("aggregateEndAt", ProtoType::Long, false)),
        (15, field("eventExchangesEndAt", ProtoType::Long, false)),
        (16, field("receptionEndAt", ProtoType::Long, false)),
        (18, field("previousEventId", ProtoType::Int, false)),
        (
            101,
            field("pointRewards", ProtoType::Message(&MASTER_EVENT_POINT_REWARD_SCHEMA), true),
        ),
        (
            102,
            field("rankingRewards", ProtoType::Message(&MASTER_EVENT_RANKING_REWARD_SCHEMA), true),
        ),
    ],
};

list_schema!(MASTER_EVENT_LIST_SCHEMA => MASTER_EVENT_SCHEMA);

// ============================================================================
// Application
// ============================================================================

pub static APPLICATION_PLATFORM_STATUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("platformId", ProtoType::Int, false)),
        (2, field("status", ProtoType::String, false)),
    ],
};

/// Flat application health/version response, not entries-wrapped.
pub static APPLICATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("clientVersion", ProtoType::String, false)),
        (2, field("dataVersion", ProtoType::String, false)),
        (3, field("appStatus", ProtoType::String, false)),
        (4, field("appType", ProtoType::String, false)),
        (5, field("serverName", ProtoType::String, false)),
        (7, field("liveStatus", ProtoType::String, false)),
        (8, field("gachaStatus", ProtoType::String, false)),
        (9, field("shopStatus", ProtoType::String, false)),
        (10, field("masterVersion", ProtoType::String, false)),
        (11, field("checksum", ProtoType::String, false)),
        (
            12,
            field("platformMaintenance", ProtoType::Message(&APPLICATION_PLATFORM_STATUS_SCHEMA), true),
        ),
    ],
};

// ============================================================================
// Music
// ============================================================================

pub static MUSIC_MISSION_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("missionType", ProtoType::String, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
    ],
};

pub static MUSIC_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("musicName", ProtoType::String, false)),
        (3, field("bgmAssetBundleName", ProtoType::String, false)),
        (4, field("jacketAssetBundleName", ProtoType::String, false)),
        (5, field("lyricist", ProtoType::String, false)),
        (6, field("composer", ProtoType::String, false)),
        (7, field("musicType", ProtoType::String, false)),
        (9, field("arranger", ProtoType::String, false)),
        (10, field("keyword", ProtoType::String, false)),
        (11, field("releaseFlg", ProtoType::Int, false)),
        (12, field("releaseCondition", ProtoType::String, false)),
        (13, field("missionRewards", ProtoType::Message(&MUSIC_MISSION_REWARD_SCHEMA), true)),
        (14, field("assetBundleName", ProtoType::String, false)),
        (15, field("sortOrder", ProtoType::Int, false)),
        (16, field("startAt", ProtoType::Long, false)),
        (17, field("endAt", ProtoType::Long, false)),
        (20, field("musicNameKana", ProtoType::String, false)),
        (21, field("category", ProtoType::String, false)),
        (22, field("seq", ProtoType::Int, false)),
    ],
};

list_schema!(MUSIC_LIST_SCHEMA => MUSIC_SCHEMA);

// ============================================================================
// Song difficulty and live difficulty master data
// ============================================================================

/// Score thresholds and chart metadata for one song/difficulty pair.
pub static MUSIC_MULTI_LIVE_SCORE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("musicDifficulty", ProtoType::String, false)),
        (3, field("multiLiveDifficultyId", ProtoType::Int, false)),
        (4, field("scoreS", ProtoType::Int, false)),
        (5, field("scoreA", ProtoType::Int, false)),
        (6, field("scoreB", ProtoType::Int, false)),
        (7, field("scoreC", ProtoType::Int, false)),
        (8, field("multiLiveDifficultyType", ProtoType::String, false)),
        (9, field("scoreSS", ProtoType::Int, false)),
        (10, field("scoreSSS", ProtoType::Int, false)),
    ],
};

map_entry_schema!(MUSIC_MULTI_LIVE_SCORE_MAP_ENTRY_SCHEMA => Int, MUSIC_MULTI_LIVE_SCORE_SCHEMA);

pub static MUSIC_DIFFICULTY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicId", ProtoType::Int, false)),
        (2, field("difficulty", ProtoType::String, false)),
        (3, field("playLevel", ProtoType::Int, false)),
        (
            4,
            field(
                "multiLiveScoreMap",
                ProtoType::Message(&MUSIC_MULTI_LIVE_SCORE_MAP_ENTRY_SCHEMA),
                true,
            ),
        ),
        (5, field("notesQuantity", ProtoType::Int, false)),
        (6, field("scoreS", ProtoType::Int, false)),
        (7, field("scoreA", ProtoType::Int, false)),
        (8, field("scoreB", ProtoType::Int, false)),
        (9, field("scoreC", ProtoType::Int, false)),
        (10, field("scoreSS", ProtoType::Int, false)),
        (11, field("scoreSSS", ProtoType::Int, false)),
        (12, field("enableSpecialNotes", ProtoType::Int, false)),
    ],
};

list_schema!(MUSIC_DIFFICULTY_LIST_SCHEMA => MUSIC_DIFFICULTY_SCHEMA);

pub static MULTI_LIVE_DIFFICULTY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("difficulty", ProtoType::String, false)),
        (3, field("requiredTotalParam", ProtoType::Int, false)),
        (4, field("bonusRate", ProtoType::Float, false)),
        (5, field("seq", ProtoType::Int, false)),
        (6, field("multiLiveDifficultyType", ProtoType::String, false)),
        (7, field("matchingLogicId", ProtoType::Int, false)),
        (8, field("roomName", ProtoType::String, false)),
        (9, field("requiredColorCode", ProtoType::String, false)),
    ],
};

map_entry_schema!(MULTI_LIVE_DIFFICULTY_MAP_ENTRY_SCHEMA => Int, MULTI_LIVE_DIFFICULTY_SCHEMA);

list_schema!(MULTI_LIVE_DIFFICULTY_MAP_SCHEMA => MULTI_LIVE_DIFFICULTY_MAP_ENTRY_SCHEMA);

pub static WEEKLY_MULTI_LIVE_DIFFICULTY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("id", ProtoType::Int, false)),
        (2, field("difficulty", ProtoType::String, false)),
        (3, field("requiredTotalParam", ProtoType::Int, false)),
        (4, field("bonusRate", ProtoType::Float, false)),
        (5, field("seq", ProtoType::Int, false)),
        (6, field("dayOfWeek", ProtoType::Int, false)),
        (7, field("description", ProtoType::String, false)),
        (8, field("resourceName", ProtoType::String, false)),
        (9, field("multiLiveDifficultyType", ProtoType::String, false)),
        (10, field("matchingLogicId", ProtoType::Int, false)),
    ],
};

map_entry_schema!(WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_ENTRY_SCHEMA => Int, WEEKLY_MULTI_LIVE_DIFFICULTY_SCHEMA);

list_schema!(WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_SCHEMA => WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_ENTRY_SCHEMA);

// ============================================================================
// Area, bond, action, and exchange master data
// ============================================================================

pub static AREA_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaItemId", ProtoType::Int, false)),
        (2, field("categoryId", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("areaItemName", ProtoType::String, false)),
        (5, field("areaId", ProtoType::Int, false)),
        (6, field("spawnPoint", ProtoType::String, false)),
        (7, field("life", ProtoType::Int, false)),
        (8, field("performance", ProtoType::Float, false)),
        (9, field("technique", ProtoType::Float, false)),
        (10, field("visual", ProtoType::Float, false)),
        (11, field("valueType", ProtoType::String, false)),
        (12, field("targetAttributes", ProtoType::String, true)),
        (13, field("resourceId", ProtoType::Int, false)),
        (14, field("description", ProtoType::String, false)),
        (15, field("characterIds", ProtoType::Int, true)),
        (16, field("targetBandIds", ProtoType::Int, true)),
        (17, field("flavorText", ProtoType::String, false)),
        (18, field("seq", ProtoType::Int, false)),
        (19, field("areaItemCategoryType", ProtoType::String, false)),
    ],
};

map_entry_schema!(AREA_ITEM_MAP_ENTRY_SCHEMA => Int, AREA_ITEM_SCHEMA);

list_schema!(AREA_ITEM_MAP_SCHEMA => AREA_ITEM_MAP_ENTRY_SCHEMA);

pub static AREA_ITEM_SPAWN_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("spawnPoint", ProtoType::String, false)),
        (2, field("spawnName", ProtoType::String, false)),
        (3, field("seq", ProtoType::Int, false)),
        (4, field("areaId", ProtoType::Int, false)),
        (5, field("inject", ProtoType::String, false)),
        (6, field("showUnsetObjectFlg", ProtoType::Int, false)),
    ],
};

map_entry_schema!(AREA_ITEM_SPAWN_MAP_ENTRY_SCHEMA => String, AREA_ITEM_SPAWN_SCHEMA);

list_schema!(AREA_ITEM_SPAWN_MAP_SCHEMA => AREA_ITEM_SPAWN_MAP_ENTRY_SCHEMA);

pub static BONDS_LEVEL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("bondsId", ProtoType::Int, false)),
        (2, field("level", ProtoType::Int, false)),
        (3, field("bondsName", ProtoType::String, false)),
        (4, field("bondsEffectId", ProtoType::Int, false)),
    ],
};

map_entry_schema!(BONDS_LEVEL_MAP_ENTRY_SCHEMA => Int, BONDS_LEVEL_SCHEMA);

pub static BONDS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("bondsId", ProtoType::Int, false)),
        (2, field("description", ProtoType::String, false)),
        (3, field("tag", ProtoType::String, false)),
        (4, field("characters", ProtoType::Int, true)),
        (5, field("bondsLevel", ProtoType::Message(&BONDS_LEVEL_MAP_ENTRY_SCHEMA), true)),
    ],
};

map_entry_schema!(BONDS_MAP_ENTRY_SCHEMA => Int, BONDS_SCHEMA);

list_schema!(BONDS_MAP_SCHEMA => BONDS_MAP_ENTRY_SCHEMA);

pub static BONDS_EFFECT_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("bondsEffectId", ProtoType::Int, false)),
        (2, field("bondsEffectName", ProtoType::String, false)),
        (3, field("description", ProtoType::String, false)),
        (4, field("bondsId", ProtoType::Int, false)),
        (5, field("level", ProtoType::Int, false)),
        (6, field("valueType", ProtoType::String, false)),
        (7, field("life", ProtoType::Int, false)),
        (8, field("performance", ProtoType::Int, false)),
        (9, field("technique", ProtoType::Int, false)),
        (10, field("visual", ProtoType::Int, false)),
        (11, field("skillEffect", ProtoType::Int, false)),
        (12, field("scope", ProtoType::String, false)),
        (13, field("targetCharacters", ProtoType::Int, true)),
    ],
};

map_entry_schema!(BONDS_EFFECT_MAP_ENTRY_SCHEMA => Int, BONDS_EFFECT_SCHEMA);

list_schema!(BONDS_EFFECT_MAP_SCHEMA => BONDS_EFFECT_MAP_ENTRY_SCHEMA);

pub static ACTION_SET_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("actionSetId", ProtoType::Int, false)),
        (2, field("areaId", ProtoType::Int, false)),
        (3, field("characterIds", ProtoType::Int, true)),
        (4, field("actionSetType", ProtoType::String, false)),
        (5, field("areaItemId", ProtoType::Int, false)),
        (6, field("areaName", ProtoType::String, false)),
        (7, field("seasonSpecialId", ProtoType::Int, false)),
        (8, field("balloonText", ProtoType::String, false)),
        (9, field("startSeason", ProtoType::String, false)),
        (10, field("endSeason", ProtoType::String, false)),
    ],
};

map_entry_schema!(ACTION_SET_MAP_ENTRY_SCHEMA => Int, ACTION_SET_SCHEMA);

list_schema!(ACTION_SET_MAP_SCHEMA => ACTION_SET_MAP_ENTRY_SCHEMA);

pub static PLAYER_RESOURCE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("resourceId", ProtoType::Int, false)),
        (2, field("resourceType", ProtoType::String, false)),
        (3, field("quantity", ProtoType::Int, false)),
        (4, field("lbBonus", ProtoType::Int, false)),
        (5, field("firstGet", ProtoType::Bool, false)),
        (6, field("duplicatedCount", ProtoType::Int, false)),
    ],
};

list_schema!(PLAYER_RESOURCE_LIST_SCHEMA => PLAYER_RESOURCE_SCHEMA);

pub static MUSIC_SHOP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("musicShopId", ProtoType::Int, false)),
        (2, field("shopId", ProtoType::Int, false)),
        (3, field("shopCategory", ProtoType::String, false)),
        (4, field("seq", ProtoType::Int, false)),
        (5, field("musicId", ProtoType::Int, false)),
        (6, field("amount", ProtoType::Int, false)),
        (7, field("costs", ProtoType::Message(&PLAYER_RESOURCE_LIST_SCHEMA), false)),
    ],
};

map_entry_schema!(MUSIC_SHOP_MAP_ENTRY_SCHEMA => Int, MUSIC_SHOP_SCHEMA);

list_schema!(MUSIC_SHOP_MAP_SCHEMA => MUSIC_SHOP_MAP_ENTRY_SCHEMA);

pub static DEGREE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("degreeId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("baseImageName", ProtoType::String, false)),
        (4, field("rank", ProtoType::String, false)),
        (5, field("degreeName", ProtoType::String, false)),
        (6, field("degreeType", ProtoType::String, false)),
        (7, field("iconImageName", ProtoType::String, false)),
        (8, field("description", ProtoType::String, false)),
    ],
};

map_entry_schema!(DEGREE_MAP_ENTRY_SCHEMA => Int, DEGREE_SCHEMA);

list_schema!(DEGREE_MAP_SCHEMA => DEGREE_MAP_ENTRY_SCHEMA);

// ============================================================================
// Character
// ============================================================================

pub static CHARACTER_PROFILE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("profileType", ProtoType::String, false)),
        (3, field("voiceActorName", ProtoType::String, false)),
        (4, field("school", ProtoType::String, false)),
        (5, field("grade", ProtoType::String, false)),
        (6, field("birthday", ProtoType::Long, false)),
        (7, field("zodiacSign", ProtoType::String, false)),
        (8, field("favoriteFood", ProtoType::String, false)),
        (9, field("dislikedFood", ProtoType::String, false)),
        (10, field("hobby", ProtoType::String, false)),
        (11, field("introduction", ProtoType::String, false)),
        (12, field("height", ProtoType::Int, false)),
        (13, field("class", ProtoType::String, false)),
        (14, field("schoolType", ProtoType::String, false)),
    ],
};

pub static CHARACTER_COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("costumeVariation", ProtoType::Int, false)),
        (3, field("costumeType", ProtoType::String, false)),
        (4, field("costumeSubType", ProtoType::String, false)),
        (5, field("assetName", ProtoType::String, false)),
        (6, field("assetBundleName", ProtoType::String, false)),
        (7, field("season", ProtoType::String, false)),
    ],
};

list_schema!(CHARACTER_COSTUME_LIST_SCHEMA => CHARACTER_COSTUME_SCHEMA);

pub static CHARACTER_COSTUME_SEASON_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("season", ProtoType::String, false)),
        (2, field("costumes", ProtoType::Message(&CHARACTER_COSTUME_LIST_SCHEMA), false)),
    ],
};

pub static CHARACTER_COSTUME_SEASON_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("season", ProtoType::Message(&CHARACTER_COSTUME_SEASON_SCHEMA), false))],
};

pub static CHARACTER_EPISODE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("episodeId", ProtoType::Int, false)),
        (2, field("episodeType", ProtoType::String, false)),
        (4, field("assetBundleName", ProtoType::String, false)),
        (11, field("episodeName", ProtoType::String, false)),
        (12, field("releaseFlg", ProtoType::Int, false)),
        (13, field("episodeNumber", ProtoType::Int, false)),
    ],
};

pub static CHARACTER_EPISODE_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("episodes", ProtoType::Message(&CHARACTER_EPISODE_SCHEMA), true))],
};

pub static CHARACTER_LIVE2D_COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("costumeVariation", ProtoType::Int, false)),
        (3, field("costumeId", ProtoType::Int, false)),
        (4, field("assetName", ProtoType::String, false)),
        (5, field("assetBundleName", ProtoType::String, false)),
    ],
};

pub static CHARACTER_LIVE2D_ENTRY_SCHEMA: Schema = Schema {
    fields: &[(1, field("costumes", ProtoType::Message(&CHARACTER_LIVE2D_COSTUME_SCHEMA), true))],
};

pub static CHARACTER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("characterType", ProtoType::String, false)),
        (3, field("characterName", ProtoType::String, false)),
        (4, field("characterNameKana", ProtoType::String, false)),
        (7, field("givenName", ProtoType::String, false)),
        (8, field("familyName", ProtoType::String, false)),
        (9, field("bandId", ProtoType::Int, false)),
        (10, field("resourceName", ProtoType::String, false)),
        (11, field("index", ProtoType::Int, false)),
        (12, field("profile", ProtoType::Message(&CHARACTER_PROFILE_SCHEMA), false)),
        (
            13,
            field("costumeSeasons", ProtoType::Message(&CHARACTER_COSTUME_SEASON_ENTRY_SCHEMA), true),
        ),
        (14, field("episodes", ProtoType::Message(&CHARACTER_EPISODE_ENTRY_SCHEMA), true)),
        (16, field("color", ProtoType::String, false)),
        (
            17,
            field("live2dCostumes", ProtoType::Message(&CHARACTER_LIVE2D_ENTRY_SCHEMA), true),
        ),
        (20, field("attribute", ProtoType::String, false)),
    ],
};

list_schema!(CHARACTER_LIST_SCHEMA => CHARACTER_SCHEMA);

// ============================================================================
// Band
// ============================================================================

pub static BAND_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("bandId", ProtoType::Int, false)),
        (2, field("bandName", ProtoType::String, false)),
        (3, field("enableFlg", ProtoType::Int, false)),
        (4, field("color", ProtoType::String, false)),
        (5, field("member1", ProtoType::Int, false)),
        (6, field("member2", ProtoType::Int, false)),
        (7, field("member3", ProtoType::Int, false)),
        (8, field("member4", ProtoType::Int, false)),
        (9, field("member5", ProtoType::Int, false)),
        (10, field("seq", ProtoType::Int, false)),
        (11, field("bandType", ProtoType::String, false)),
        (13, field("bandNameKana", ProtoType::String, false)),
        (14, field("sortOrder", ProtoType::Int, false)),
    ],
};

list_schema!(BAND_LIST_SCHEMA => BAND_SCHEMA);

// ============================================================================
// Area
// ============================================================================

pub static AREA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaId", ProtoType::Int, false)),
        (2, field("areaType", ProtoType::String, false)),
        (3, field("areaName", ProtoType::String, false)),
        (4, field("description", ProtoType::String, false)),
        (8, field("seq", ProtoType::Int, false)),
        (13, field("fromSeason", ProtoType::String, false)),
        (14, field("toSeason", ProtoType::String, false)),
    ],
};

list_schema!(AREA_LIST_SCHEMA => AREA_SCHEMA);

// ============================================================================
// Gacha
// ============================================================================

pub static GACHA_RATE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("rarity", ProtoType::Int, false)),
        (3, field("rate", ProtoType::Float, false)),
        (4, field("rateType", ProtoType::Int, false)),
    ],
};

pub static GACHA_TYPE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("costType", ProtoType::String, false)),
        (3, field("cost", ProtoType::Int, false)),
        (4, field("sortOrder", ProtoType::Int, false)),
        (5, field("enableFlg", ProtoType::Int, false)),
        (6, field("type", ProtoType::String, false)),
        (7, field("paidFlg", ProtoType::Int, false)),
        (9, field("costTotal", ProtoType::Int, false)),
        (12, field("discountFlg", ProtoType::Int, false)),
        (13, field("freeFlg", ProtoType::Int, false)),
    ],
};

pub static GACHA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("gachaId", ProtoType::Int, false)),
        (2, field("gachaName", ProtoType::String, false)),
        (3, field("gachaType", ProtoType::String, false)),
        (4, field("startAt", ProtoType::Long, false)),
        (5, field("endAt", ProtoType::Long, false)),
        (7, field("rates", ProtoType::Message(&GACHA_RATE_SCHEMA), true)),
        (8, field("types", ProtoType::Message(&GACHA_TYPE_SCHEMA), true)),
        (9, field("description", ProtoType::String, false)),
        (10, field("sortOrder", ProtoType::Int, false)),
        (11, field("assetBundleName", ProtoType::String, false)),
        (13, field("endText", ProtoType::String, false)),
        (19, field("gachaCategory", ProtoType::String, false)),
        (35, field("pickupText", ProtoType::String, false)),
        (38, field("limitFlg", ProtoType::Int, false)),
        (40, field("stampFlg", ProtoType::Int, false)),
        (43, field("bonusFlg", ProtoType::Int, false)),
    ],
};

list_schema!(GACHA_LIST_SCHEMA => GACHA_SCHEMA);

// ============================================================================
// Item
// ============================================================================

pub static ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("itemId", ProtoType::Int, false)),
        (2, field("itemName", ProtoType::String, false)),
        (3, field("itemType", ProtoType::Int, false)),
        (4, field("description", ProtoType::String, false)),
        (5, field("seq", ProtoType::Int, false)),
    ],
};

list_schema!(ITEM_LIST_SCHEMA => ITEM_SCHEMA);

// ============================================================================
// Skill
// ============================================================================

pub static SKILL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("skillId", ProtoType::Int, false)),
        (2, field("skillLevel", ProtoType::Int, false)),
        (3, field("effectValue", ProtoType::Float, false)),
        (4, field("skillName", ProtoType::String, false)),
        (5, field("description", ProtoType::String, false)),
        (6, field("skillType", ProtoType::String, false)),
    ],
};

list_schema!(SKILL_LIST_SCHEMA => SKILL_SCHEMA);

// ============================================================================
// Stamp
// ============================================================================

pub static STAMP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("stampId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("assetName", ProtoType::String, false)),
        (4, field("stampType", ProtoType::String, false)),
        (7, field("startAt", ProtoType::Long, false)),
        (8, field("endAt", ProtoType::Long, false)),
    ],
};

list_schema!(STAMP_LIST_SCHEMA => STAMP_SCHEMA);

// ============================================================================
// Login bonus
// ============================================================================

pub static LOGIN_BONUS_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("loginBonusId", ProtoType::Int, false)),
        (2, field("day", ProtoType::Int, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
        (7, field("seq", ProtoType::Int, false)),
        (8, field("presentType", ProtoType::String, false)),
    ],
};

pub static LOGIN_BONUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("loginBonusId", ProtoType::Int, false)),
        (2, field("loginBonusType", ProtoType::String, false)),
        (3, field("loginBonusName", ProtoType::String, false)),
        (4, field("campaignName", ProtoType::String, false)),
        (5, field("receiveFlg", ProtoType::Int, false)),
        (6, field("startAt", ProtoType::Long, false)),
        (7, field("rewards", ProtoType::Message(&LOGIN_BONUS_REWARD_SCHEMA), true)),
        (8, field("imageAssetBundleName", ProtoType::String, false)),
    ],
};

list_schema!(LOGIN_BONUS_LIST_SCHEMA => LOGIN_BONUS_SCHEMA);

// ============================================================================
// Costume
// ============================================================================

pub static COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("costumeId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("assetBundleName", ProtoType::String, false)),
        (5, field("costumeName", ProtoType::String, false)),
        (6, field("sortOrder", ProtoType::Int, false)),
        (7, field("sdAssetBundleName", ProtoType::String, false)),
        (9, field("startAt", ProtoType::Long, false)),
        (10, field("characterId", ProtoType::Int, false)),
    ],
};

list_schema!(COSTUME_LIST_SCHEMA => COSTUME_SCHEMA);

// ============================================================================
// Situation master, the game's name for cards
// ============================================================================

pub static SITUATION_APPEND_PARAMETER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("situationId", ProtoType::Int, false)),
        (2, field("level", ProtoType::Int, false)),
        (3, field("performance", ProtoType::Int, false)),
        (4, field("technique", ProtoType::Int, false)),
        (5, field("visual", ProtoType::Int, false)),
    ],
};

pub static SITUATION_LEVEL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("level", ProtoType::Int, false)),
        (
            2,
            field("appendParameter", ProtoType::Message(&SITUATION_APPEND_PARAMETER_SCHEMA), false),
        ),
    ],
};

/// Reward granted by a card episode or special training. Field 1 is the item
/// id, absent for pure-currency rewards like stars; field 4 is an unconfirmed
/// sequence marker that is always 1 in live dumps.
pub static SITUATION_EPISODE_REWARD_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("rewardId", ProtoType::Int, false)),
        (2, field("rewardType", ProtoType::String, false)),
        (3, field("rewardQuantity", ProtoType::Int, false)),
        (4, field("seq", ProtoType::Int, false)),
    ],
};

list_schema!(SITUATION_EPISODE_REWARD_LIST_SCHEMA => SITUATION_EPISODE_REWARD_SCHEMA);

/// A card episode, either standard or memorial. Fields 5-7 are the stat
/// bonuses the episode grants; fields 9 and 10 are its item and star reward
/// lists.
pub static SITUATION_EPISODE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("episodeId", ProtoType::Int, false)),
        (2, field("episodeType", ProtoType::String, false)),
        (3, field("episodeNumber", ProtoType::Int, false)),
        (4, field("assetBundleName", ProtoType::String, false)),
        (5, field("bonusPerformance", ProtoType::Int, false)),
        (6, field("bonusTechnique", ProtoType::Int, false)),
        (7, field("bonusVisual", ProtoType::Int, false)),
        (8, field("maxLevel", ProtoType::Int, false)),
        (
            9,
            field("rewards", ProtoType::Message(&SITUATION_EPISODE_REWARD_LIST_SCHEMA), false),
        ),
        (
            10,
            field("starRewards", ProtoType::Message(&SITUATION_EPISODE_REWARD_LIST_SCHEMA), false),
        ),
        (11, field("episodeName", ProtoType::String, false)),
        (12, field("releaseFlg", ProtoType::Int, false)),
    ],
};

list_schema!(SITUATION_EPISODE_LIST_SCHEMA => SITUATION_EPISODE_SCHEMA);

/// Special-training data present on cards that can be trained. Fields 4-6
/// are the stat bonuses granted and field 7 the item rewards; the official
/// field names are inferred from a live dump.
pub static SITUATION_TRAINING_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("situationId", ProtoType::Int, false)),
        (2, field("characterIndex", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("performance", ProtoType::Int, false)),
        (5, field("technique", ProtoType::Int, false)),
        (6, field("visual", ProtoType::Int, false)),
        (
            7,
            field("rewards", ProtoType::Message(&SITUATION_EPISODE_REWARD_LIST_SCHEMA), false),
        ),
    ],
};

pub static SITUATION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("situationId", ProtoType::Int, false)),
        (2, field("situationType", ProtoType::Int, false)),
        (3, field("rarity", ProtoType::Int, false)),
        (5, field("attribute", ProtoType::String, false)),
        (7, field("skillId", ProtoType::Int, false)),
        (8, field("levels", ProtoType::Message(&SITUATION_LEVEL_SCHEMA), true)),
        (10, field("cardName", ProtoType::String, false)),
        (11, field("maxLevel", ProtoType::Int, false)),
        (12, field("resourceName", ProtoType::String, false)),
        (13, field("sdAssetName", ProtoType::String, false)),
        (14, field("episodes", ProtoType::Message(&SITUATION_EPISODE_LIST_SCHEMA), false)),
        (15, field("training", ProtoType::Message(&SITUATION_TRAINING_SCHEMA), false)),
        (16, field("characterIndex", ProtoType::Int, false)),
        (17, field("releaseAt", ProtoType::Long, false)),
        (18, field("skillId2", ProtoType::Int, false)),
        (19, field("flag2", ProtoType::Int, false)),
        (20, field("illustType", ProtoType::String, false)),
        (24, field("extra", ProtoType::String, false)),
        (25, field("seq", ProtoType::Int, false)),
    ],
};

list_schema!(SITUATION_LIST_SCHEMA => SITUATION_SCHEMA);

// ============================================================================
// Shop
// ============================================================================

/// Shop where area items and music are exchanged. Field 2 is a per-type order
/// hint discovered from a live dump; its exact meaning is unconfirmed.
pub static SHOP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("shopId", ProtoType::Int, false)),
        (2, field("seq", ProtoType::Int, false)),
        (3, field("shopName", ProtoType::String, false)),
        (4, field("description", ProtoType::String, false)),
        (5, field("shopType", ProtoType::String, false)),
    ],
};

list_schema!(SHOP_LIST_SCHEMA => SHOP_SCHEMA);

// ============================================================================
// User profile, deck, situation
// ============================================================================

pub static USER_PROFILE_STATS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("rank", ProtoType::Int, false)),
        (3, field("level", ProtoType::Int, false)),
        (4, field("exp", ProtoType::Int, false)),
        (5, field("liveCount", ProtoType::Int, false)),
        (7, field("stamina", ProtoType::Int, false)),
        (9, field("staminaMax", ProtoType::Int, false)),
        (16, field("friendCount", ProtoType::Int, false)),
        (17, field("playCount", ProtoType::Int, false)),
    ],
};

pub static USER_PROFILE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("uuid", ProtoType::String, false)),
        (3, field("userName", ProtoType::String, false)),
        (4, field("clientVersion", ProtoType::String, false)),
        (5, field("platform", ProtoType::String, false)),
        (6, field("device", ProtoType::String, false)),
        (7, field("osVersion", ProtoType::String, false)),
        (9, field("userCode", ProtoType::String, false)),
        (10, field("tutorialStatus", ProtoType::String, false)),
        (11, field("comment", ProtoType::String, false)),
        (12, field("profileFrame", ProtoType::String, false)),
        (13, field("lastLoginAt", ProtoType::Long, false)),
    ],
};

pub static USER_PROFILE_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("profile", ProtoType::Message(&USER_PROFILE_SCHEMA), false)),
        (2, field("stats", ProtoType::Message(&USER_PROFILE_STATS_SCHEMA), false)),
    ],
};

list_schema!(USER_DECK_LIST_SCHEMA => USER_DECK_SCHEMA);

// ============================================================================
// User title, stamps, areas, items, presents, gacha
// ============================================================================

/// Flat equipped-title response, not entries-wrapped.
pub static USER_TITLE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("titleId", ProtoType::Int, false)),
        (2, field("assetName", ProtoType::String, false)),
        (3, field("startAt", ProtoType::Long, false)),
        (4, field("endAt", ProtoType::Long, false)),
        (6, field("seq", ProtoType::Int, false)),
        (7, field("colorType", ProtoType::String, false)),
        (8, field("frameColor", ProtoType::String, false)),
        (9, field("copyrightAssetName", ProtoType::String, false)),
        (10, field("rightsAssetName", ProtoType::String, false)),
        (11, field("enableFlg", ProtoType::Int, false)),
    ],
};

pub static USER_STAMP_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("stampId", ProtoType::Int, false)),
        (3, field("seq", ProtoType::Int, false)),
    ],
};

list_schema!(USER_STAMP_LIST_SCHEMA => USER_STAMP_SCHEMA);

pub static USER_AREA_STATUS_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaItemId", ProtoType::Int, false)),
        (2, field("status", ProtoType::String, false)),
    ],
};

pub static USER_AREA_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("areaId", ProtoType::Int, false)),
        (2, field("areaItems", ProtoType::Message(&USER_AREA_STATUS_ITEM_SCHEMA), true)),
    ],
};

list_schema!(USER_AREA_LIST_SCHEMA => USER_AREA_SCHEMA);

pub static USER_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("itemId", ProtoType::Int, false)),
        (3, field("count", ProtoType::Int, false)),
    ],
};

list_schema!(USER_ITEM_LIST_SCHEMA => USER_ITEM_SCHEMA);

pub static USER_PRESENT_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("presentId", ProtoType::Int, false)),
        (2, field("userId", ProtoType::Long, false)),
        (3, field("rewardType", ProtoType::String, false)),
        (4, field("rewardId", ProtoType::Int, false)),
        (5, field("rewardCount", ProtoType::Int, false)),
        (6, field("message", ProtoType::String, false)),
        (7, field("receivedAt", ProtoType::Long, false)),
        (8, field("createdAt", ProtoType::Long, false)),
    ],
};

/// Present box summary. Field 1 is a max-slot sentinel that is i64::MAX on a
/// live dump, field 2 the current slot count, field 3 an unread flag. Names are
/// best guesses from a single empty-box response.
pub static USER_PRESENT_BOX_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("maxSlot", ProtoType::Long, false)),
        (2, field("slotCount", ProtoType::Int, false)),
        (3, field("unreadCount", ProtoType::Int, false)),
    ],
};

pub static USER_PRESENT_LIST_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("entries", ProtoType::Message(&USER_PRESENT_SCHEMA), true)),
        (2, field("presentBox", ProtoType::Message(&USER_PRESENT_BOX_SCHEMA), false)),
    ],
};

pub static USER_GACHA_ENTRY_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("gachaId", ProtoType::Int, false)),
        (3, field("count", ProtoType::Int, false)),
    ],
};

list_schema!(USER_GACHA_LIST_SCHEMA => USER_GACHA_ENTRY_SCHEMA);

// ============================================================================
// User episodes
// ============================================================================

/// Provisional episode unlock schema. A live probe of the configured user
/// returned an empty payload, so these fields await a user with episodes.
pub static USER_EPISODE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("episodeId", ProtoType::Int, false)),
        (3, field("status", ProtoType::String, false)),
    ],
};

list_schema!(USER_EPISODE_LIST_SCHEMA => USER_EPISODE_SCHEMA);

// ============================================================================
// User missions, login bonuses, costumes, characters
// ============================================================================

/// A mission progress row for the configured user. Field names are inferred
/// from a live dump: field 3 is always 1 in the sampled data (likely a type or
/// flag), field 4 is the current progress, field 5 the status string, and
/// field 6 a constant value whose meaning is unconfirmed.
pub static USER_MISSION_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("missionId", ProtoType::Int, false)),
        (3, field("missionType", ProtoType::Int, false)),
        (4, field("progress", ProtoType::Int, false)),
        (5, field("status", ProtoType::String, false)),
        (6, field("missionValue", ProtoType::Int, false)),
    ],
};

list_schema!(USER_MISSION_LIST_SCHEMA => USER_MISSION_SCHEMA);

/// A login bonus progress row for the configured user. Field 3 is the number
/// of rewards received so far (3 or 1 in the live dump).
pub static USER_LOGIN_BONUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("loginBonusId", ProtoType::Int, false)),
        (3, field("receiveCount", ProtoType::Int, false)),
    ],
};

list_schema!(USER_LOGIN_BONUS_LIST_SCHEMA => USER_LOGIN_BONUS_SCHEMA);

/// A costume owned by the configured user.
pub static USER_COSTUME_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("costumeId", ProtoType::Int, false)),
    ],
};

list_schema!(USER_COSTUME_LIST_SCHEMA => USER_COSTUME_SCHEMA);

/// An owned-character row from the individual user-character endpoint.
pub static USER_CHARACTER_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userCharacterId", ProtoType::Long, false)),
        (2, field("userId", ProtoType::Long, false)),
        (3, field("characterId", ProtoType::Int, false)),
        (4, field("closeness", ProtoType::Int, false)),
        (5, field("costumeId", ProtoType::Int, false)),
    ],
};

list_schema!(USER_CHARACTER_LIST_SCHEMA => USER_CHARACTER_SCHEMA);

/// One per-difficulty score from the complete suite user snapshot.
pub static USER_MUSIC_SCORE_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("musicId", ProtoType::Int, false)),
        (3, field("musicDifficulty", ProtoType::String, false)),
        (4, field("soloHighScore", ProtoType::Int, false)),
        (5, field("maxCombo", ProtoType::Int, false)),
        (6, field("soloScoreRank", ProtoType::String, false)),
        (7, field("clearStatus", ProtoType::String, false)),
    ],
};

list_schema!(USER_MUSIC_SCORE_LIST_SCHEMA => USER_MUSIC_SCORE_SCHEMA);

map_entry_schema!(USER_MUSIC_SCORE_MAP_ENTRY_SCHEMA => Int, USER_MUSIC_SCORE_LIST_SCHEMA);

list_schema!(USER_MUSIC_SCORE_MAP_SCHEMA => USER_MUSIC_SCORE_MAP_ENTRY_SCHEMA);

/// Aggregate clear counts grouped by difficulty. The per-song status API uses
/// `USER_MUSIC_SCORE_SCHEMA`; this map mirrors the game's profile counters.
pub static USER_MUSIC_CLEAR_INFO_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("clearedMusicCount", ProtoType::Int, false)),
        (2, field("fullComboMusicCount", ProtoType::Int, false)),
        (3, field("allPerfectMusicCount", ProtoType::Int, false)),
    ],
};

map_entry_schema!(USER_MUSIC_CLEAR_INFO_MAP_ENTRY_SCHEMA => String, USER_MUSIC_CLEAR_INFO_SCHEMA);

list_schema!(USER_MUSIC_CLEAR_INFO_MAP_SCHEMA => USER_MUSIC_CLEAR_INFO_MAP_ENTRY_SCHEMA);

/// Character rank data is carried by the complete suite user snapshot.
pub static USER_CHARACTER_RANK_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("rank", ProtoType::Int, false)),
        (2, field("exp", ProtoType::Long, false)),
        (3, field("addExp", ProtoType::Long, false)),
        (4, field("nextExp", ProtoType::Long, false)),
        (5, field("totalExp", ProtoType::Long, false)),
        (6, field("releasedPotentialLevel", ProtoType::Long, false)),
    ],
};

/// An enabled area item from the complete suite user snapshot.
pub static USER_AREA_ITEM_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("userId", ProtoType::Long, false)),
        (2, field("areaItemId", ProtoType::Int, false)),
        (3, field("areaItemCategory", ProtoType::Int, false)),
        (4, field("level", ProtoType::Int, false)),
    ],
};

map_entry_schema!(USER_AREA_ITEM_MAP_ENTRY_SCHEMA => Int, USER_AREA_ITEM_SCHEMA);

list_schema!(USER_AREA_ITEM_MAP_SCHEMA => USER_AREA_ITEM_MAP_ENTRY_SCHEMA);

map_entry_schema!(USER_CHARACTER_RANK_MAP_ENTRY_SCHEMA => Int, USER_CHARACTER_RANK_SCHEMA);

list_schema!(USER_CHARACTER_RANK_MAP_SCHEMA => USER_CHARACTER_RANK_MAP_ENTRY_SCHEMA);

/// The three released potential dimensions for one character.
pub static USER_CHARACTER_POTENTIAL_LEVEL_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("performanceLevel", ProtoType::Int, false)),
        (2, field("techniqueLevel", ProtoType::Int, false)),
        (3, field("visualLevel", ProtoType::Int, false)),
    ],
};

map_entry_schema!(USER_CHARACTER_POTENTIAL_LEVEL_MAP_ENTRY_SCHEMA => Int, USER_CHARACTER_POTENTIAL_LEVEL_SCHEMA);

list_schema!(USER_CHARACTER_POTENTIAL_LEVEL_MAP_SCHEMA => USER_CHARACTER_POTENTIAL_LEVEL_MAP_ENTRY_SCHEMA);

/// A character bonus granted by character missions.
pub static USER_CHARACTER_MISSION_BONUS_SCHEMA: Schema = Schema {
    fields: &[
        (1, field("characterId", ProtoType::Int, false)),
        (2, field("characterBonusType", ProtoType::String, false)),
        (3, field("performance", ProtoType::Float, false)),
        (4, field("technique", ProtoType::Float, false)),
        (5, field("visual", ProtoType::Float, false)),
    ],
};

list_schema!(USER_CHARACTER_MISSION_BONUS_LIST_SCHEMA => USER_CHARACTER_MISSION_BONUS_SCHEMA);

map_entry_schema!(USER_CHARACTER_MISSION_BONUS_MAP_ENTRY_SCHEMA => Int, USER_CHARACTER_MISSION_BONUS_LIST_SCHEMA);

list_schema!(USER_CHARACTER_MISSION_BONUS_MAP_SCHEMA => USER_CHARACTER_MISSION_BONUS_MAP_ENTRY_SCHEMA);

/// The complete suite snapshot fields used by the production user APIs.
/// Field 22 contains enabled area items, field 54 contains per-song scores,
/// field 400 contains character ranks, field 401 contains the three-dimensional
/// potential levels, field 409 contains aggregate music clear counts, and field
/// 456 contains character mission bonuses.
pub static SUITE_USER_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (22, field("userAreaItemMap", ProtoType::Message(&USER_AREA_ITEM_MAP_SCHEMA), false)),
        (
            54,
            field("userMusicScoreMap", ProtoType::Message(&USER_MUSIC_SCORE_MAP_SCHEMA), false),
        ),
        (
            409,
            field(
                "userMusicClearInfoMap",
                ProtoType::Message(&USER_MUSIC_CLEAR_INFO_MAP_SCHEMA),
                false,
            ),
        ),
        (
            400,
            field("userCharacterRankMap", ProtoType::Message(&USER_CHARACTER_RANK_MAP_SCHEMA), false),
        ),
        (
            401,
            field(
                "userCharacterPotentialLevelMap",
                ProtoType::Message(&USER_CHARACTER_POTENTIAL_LEVEL_MAP_SCHEMA),
                false,
            ),
        ),
        (
            456,
            field(
                "userCharacterMissionBonusMap",
                ProtoType::Message(&USER_CHARACTER_MISSION_BONUS_MAP_SCHEMA),
                false,
            ),
        ),
    ],
};

/// Selected fields from the game's full `/suite/master` snapshot. The suite
/// payload contains many more message families; keeping this descriptor
/// focused makes the public aggregate endpoint stable and avoids pretending
/// that undocumented fields are complete.
pub static SUITE_MASTER_RESPONSE_SCHEMA: Schema = Schema {
    fields: &[
        (
            2,
            field("musicDifficulties", ProtoType::Message(&MUSIC_DIFFICULTY_LIST_SCHEMA), false),
        ),
        (
            28,
            field(
                "multiLiveDifficulties",
                ProtoType::Message(&MULTI_LIVE_DIFFICULTY_MAP_SCHEMA),
                false,
            ),
        ),
        (9, field("areaItems", ProtoType::Message(&AREA_ITEM_MAP_SCHEMA), false)),
        (10, field("bonds", ProtoType::Message(&BONDS_MAP_SCHEMA), false)),
        (11, field("bondEffects", ProtoType::Message(&BONDS_EFFECT_MAP_SCHEMA), false)),
        (14, field("actionSets", ProtoType::Message(&ACTION_SET_MAP_SCHEMA), false)),
        (30, field("musicShops", ProtoType::Message(&MUSIC_SHOP_MAP_SCHEMA), false)),
        (47, field("degrees", ProtoType::Message(&DEGREE_MAP_SCHEMA), false)),
        (
            49,
            field(
                "weeklyMultiLiveDifficulties",
                ProtoType::Message(&WEEKLY_MULTI_LIVE_DIFFICULTY_MAP_SCHEMA),
                false,
            ),
        ),
    ],
};
