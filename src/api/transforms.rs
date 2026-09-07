//! Pure transformations of decoded game data into public API views.

use crate::error::{AppError, AppResult};
use serde_json::{json, Value};

pub(super) fn find_entry<'a>(root: &'a Value, id_field: &str, id: i64, resource_name: &str) -> AppResult<&'a Value> {
    root.get("entries")
        .and_then(Value::as_array)
        .and_then(|entries| entries.iter().find(|entry| entry.get(id_field).and_then(Value::as_i64) == Some(id)))
        .ok_or_else(|| AppError::not_found(format!("{resource_name} {id} not found")))
}

/// Builds an entries-wrapped derived view while preserving source order.
pub(super) fn filtered_entries(root: &Value, predicate: impl Fn(&Value) -> bool) -> Value {
    let entries: Vec<Value> = root
        .get("entries")
        .and_then(Value::as_array)
        .map(|entries| entries.iter().filter(|entry| predicate(entry)).cloned().collect())
        .unwrap_or_default();
    json!({ "entries": entries })
}

/// Converts a decoded protobuf map into the list form used by the public API.
/// Map keys are retained as a field when the value does not already carry the
/// identifier, which makes map-backed entries self-contained for clients.
pub(super) fn flatten_map_values(root: &Value, key_name: Option<&str>, overwrite_key: bool) -> Value {
    let entries = root
        .get("entries")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let mut value = entry.get("value")?.clone();
                    if let Some(key_name) = key_name {
                        if let (Some(key), Some(object)) = (entry.get("key"), value.as_object_mut()) {
                            if overwrite_key || !object.contains_key(key_name) {
                                object.insert(key_name.to_string(), key.clone());
                            }
                        }
                    }
                    Some(value)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({ "entries": entries })
}

/// Converts a map field inside a decoded suite response into the public list
/// form used by the user endpoints.
pub(super) fn suite_map_values(root: &Value, map_name: &str, key_name: Option<&str>) -> Value {
    root.get(map_name)
        .map(|map| flatten_map_values(map, key_name, true))
        .unwrap_or_else(|| json!({ "entries": [] }))
}

/// Flattens a map whose values are entries-wrapped lists. The map key is copied
/// into each nested item when the item does not already contain the key field.
pub(super) fn flatten_nested_map_values(root: &Value, map_name: &str, key_name: &str) -> Vec<Value> {
    let mut result = Vec::new();
    let map_entries = root.get(map_name).and_then(|map| map.get("entries")).and_then(Value::as_array);

    if let Some(map_entries) = map_entries {
        for map_entry in map_entries {
            let key = map_entry.get("key").cloned();
            let values = map_entry
                .get("value")
                .and_then(|value| value.get("entries"))
                .and_then(Value::as_array);

            if let Some(values) = values {
                for value in values {
                    let mut value = value.clone();
                    if value.get(key_name).is_none() {
                        if let (Some(key), Some(object)) = (&key, value.as_object_mut()) {
                            object.insert(key_name.to_string(), key.clone());
                        }
                    }
                    result.push(value);
                }
            }
        }
    }

    result
}

pub(super) const MUSIC_DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "expert", "special"];

pub(super) fn normalize_music_difficulty(raw: &str) -> Option<&'static str> {
    MUSIC_DIFFICULTIES
        .iter()
        .copied()
        .find(|difficulty| raw.eq_ignore_ascii_case(difficulty))
}

/// Builds the single-song status response from the decoded suite snapshot.
/// Missing rows mean that the configured user has not recorded a score for
/// that song/difficulty; they are represented as an unplayed result rather
/// than a 404 so callers can query arbitrary music IDs safely.
pub(super) fn suite_music_status_value(root: &Value, music_id: i64, difficulty: &str) -> Value {
    let score = music_scores(root, music_id)
        .rev()
        .find(|entry| entry.get("musicDifficulty").and_then(Value::as_str) == Some(difficulty));
    let clear_status = score
        .as_ref()
        .and_then(|entry| entry.get("clearStatus"))
        .and_then(Value::as_str)
        .unwrap_or("not_cleared");
    let is_cleared = matches!(clear_status, "cleared" | "full_combo" | "all_perfect");
    let is_full_combo = matches!(clear_status, "full_combo" | "all_perfect");
    let is_all_perfect = clear_status == "all_perfect";

    json!({
        "musicId": music_id,
        "musicDifficulty": difficulty,
        "played": score.is_some(),
        "clearStatus": clear_status,
        "isCleared": is_cleared,
        "isFullCombo": is_full_combo,
        "isAllPerfect": is_all_perfect,
        "soloHighScore": score.as_ref().and_then(|entry| entry.get("soloHighScore")).cloned(),
        "maxCombo": score.as_ref().and_then(|entry| entry.get("maxCombo")).cloned(),
        "soloScoreRank": score.as_ref().and_then(|entry| entry.get("soloScoreRank")).cloned(),
    })
}

/// Select scores by their explicit music ID, falling back to the enclosing
/// map key, without cloning every score in the user's snapshot.
pub(super) fn music_scores(root: &Value, music_id: i64) -> impl DoubleEndedIterator<Item = &Value> {
    let maps = root
        .get("userMusicScoreMap")
        .and_then(|map| map.get("entries"))
        .and_then(Value::as_array);
    maps.into_iter().flatten().flat_map(move |map| {
        let key = map.get("key").and_then(Value::as_i64);
        map.get("value")
            .and_then(|value| value.get("entries"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(move |score| score.get("musicId").and_then(Value::as_i64).or(key) == Some(music_id))
    })
}

/// Joins character rank entries with the separate three-dimensional potential
/// level map and character-mission-bonus map from the same suite snapshot.
pub(super) fn suite_character_rank_values(root: &Value) -> Value {
    let potential_entries = root
        .get("userCharacterPotentialLevelMap")
        .and_then(|map| map.get("entries"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mission_bonus_entries = flatten_nested_map_values(root, "userCharacterMissionBonusMap", "characterId");

    let entries = root
        .get("userCharacterRankMap")
        .and_then(|map| map.get("entries"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let key = entry.get("key")?.clone();
                    let mut value = entry.get("value")?.clone();
                    let object = value.as_object_mut()?;
                    object.insert("characterId".to_string(), key.clone());

                    if let Some(potential) = potential_entries
                        .iter()
                        .find(|entry| entry.get("key") == Some(&key))
                        .and_then(|entry| entry.get("value"))
                    {
                        object.insert("potentialLevel".to_string(), potential.clone());
                    }

                    let mission_bonuses: Vec<Value> = mission_bonus_entries
                        .iter()
                        .filter(|bonus| bonus.get("characterId") == Some(&key))
                        .cloned()
                        .collect();
                    if !mission_bonuses.is_empty() {
                        object.insert("characterMissionBonus".to_string(), Value::Array(mission_bonuses));
                    }

                    Some(value)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({ "entries": entries })
}

#[cfg(test)]
mod tests {
    use super::{flatten_map_values, flatten_nested_map_values, suite_music_status_value};
    use serde_json::json;

    #[test]
    fn map_values_keep_existing_ids_and_fill_missing_ids() {
        let root = json!({
            "entries": [
                {"key": 10, "value": {"areaItemId": 99, "areaItemName": "keep-value"}},
                {"key": 20, "value": {"areaItemName": "use-map-key"}}
            ]
        });

        let flattened = flatten_map_values(&root, Some("areaItemId"), false);
        assert_eq!(flattened["entries"][0]["areaItemId"], json!(99));
        assert_eq!(flattened["entries"][1]["areaItemId"], json!(20));
    }

    #[test]
    fn music_status_reports_ap_as_fc_and_ap() {
        let root = json!({
            "userMusicScoreMap": {
                "entries": [{
                    "key": 123,
                    "value": {
                        "entries": [{
                            "musicId": 123,
                            "musicDifficulty": "expert",
                            "soloHighScore": 1234567,
                            "maxCombo": 987,
                            "soloScoreRank": "sss",
                            "clearStatus": "all_perfect"
                        }]
                    }
                }]
            }
        });

        let status = suite_music_status_value(&root, 123, "expert");
        assert_eq!(status["played"], json!(true));
        assert_eq!(status["isCleared"], json!(true));
        assert_eq!(status["isFullCombo"], json!(true));
        assert_eq!(status["isAllPerfect"], json!(true));
        assert_eq!(status["soloHighScore"], json!(1234567));
    }

    #[test]
    fn music_status_defaults_when_difficulty_has_no_record() {
        let root = json!({
            "userMusicScoreMap": {
                "entries": [{
                    "key": 123,
                    "value": {
                        "entries": [{
                            "musicId": 123,
                            "musicDifficulty": "hard",
                            "clearStatus": "full_combo"
                        }]
                    }
                }]
            }
        });

        let status = suite_music_status_value(&root, 123, "expert");
        assert_eq!(status["played"], json!(false));
        assert_eq!(status["clearStatus"], json!("not_cleared"));
        assert_eq!(status["isFullCombo"], json!(false));
        assert_eq!(status["soloHighScore"], json!(null));
    }

    #[test]
    fn music_score_map_is_flattened() {
        let root = json!({
            "userMusicScoreMap": {
                "entries": [{
                    "key": 456,
                    "value": {
                        "entries": [{"musicDifficulty": "special"}]
                    }
                }]
            }
        });

        let entries = flatten_nested_map_values(&root, "userMusicScoreMap", "musicId");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["musicId"], json!(456));
    }
}
