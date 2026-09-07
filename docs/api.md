# API 文档

**Penlight-Dream-API** 的 HTTP 接口说明。默认监听 `http://127.0.0.1:8080`，API 前缀 `/api`。

## 鉴权

在 `.env` 里设置 `API_KEY` 后，所有 `/api/{server}/*` 请求都必须携带密钥，否则返回 `401`：

```bash
curl -H "X-API-Key: your-secret-key" http://127.0.0.1:8080/api/jp/monthly-ranking
curl -H "Authorization: Bearer your-secret-key" http://127.0.0.1:8080/api/jp/monthly-ranking
```

`API_KEY` 留空则鉴权关闭，任何人都可访问。

## 服务器作用域

游戏数据路由挂在 `/api/{server}/...` 下。当前仅配置日服，`{server}` 只接受 `jp`，其它值返回 `400`：

```bash
curl http://127.0.0.1:8080/api/jp/music
# 400: {"result":"failed","status":400,"message":"unsupported server \"en\", only \"jp\" is configured"}
```

## 响应格式

- **列表接口**返回 `{"entries": [...]}` 包裹结构。
- **单个对象接口**返回对象本身。
- **排行报告**返回结构化对象，前列/分档线用户为裸数组。
- **错误**统一为 `{"result": "failed", "status": <http状态码>, "message": "<原因>"}`。

### 服务器元信息

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/servers` | 已配置的服务器列表，不含密钥与 UID |
| GET | `/health` | 进程健康检查，含版本、运行时长、日服可用性与客户端版本 |
| GET | `/version` | 自动探测到的游戏客户端版本 |

### 月榜

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/monthly-ranking` | 月榜期次主数据 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}/info` | 单个月榜期次主数据 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}` | 某期月榜的完整排名 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}/top` | 仅前列用户 |
| GET | `/api/{server}/monthly-ranking/{monthly_id}/border` | 仅分档线用户 |

### 活动

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/events` | 活动主数据列表 |
| GET | `/api/{server}/events/{event_id}` | 单个活动主数据 |
| GET | `/api/{server}/events/{event_id}/ranking?type=medley` | 某活动排名，`type` 缺省时从活动主数据解析，`mid` 可选用于按曲目子榜 |

### 主数据

以下 Map 主数据也支持在列表路径后追加 `/{id}` 获取单个对象：`multi-live-difficulties`、`weekly-multi-live-difficulties`、`area-items`、`area-item-spawns`、`bonds`、`bond-effects`、`action-sets`、`music-shops`、`degrees`。标识字段分别为 `id`、`id`、`areaItemId`、`spawnPoint`、`bondsId`、`bondsEffectId`、`actionSetId`、`musicShopId`、`degreeId`。例如 `GET /api/jp/area-items/1`。`area-item-spawns` 的 ID 为字符串摆放点标识，其余 ID 必须为正整数；无效整数返回 `400`，不存在返回 `404`。详情与列表共享缓存和解码结果，不访问额外的上游详情路径。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/application` | 应用版本、服务器状态、各平台维护状态 |
| GET | `/api/{server}/master-suite` | 官方完整主数据快照中的精选字段，包含谱面、多人 live、区域道具、羁绊、动作、兑换与称号数据 |
| GET | `/api/{server}/music` | 乐曲主数据 |
| GET | `/api/{server}/music/{music_id}` | 单曲主数据 |
| GET | `/api/{server}/music-difficulties` | 全部歌曲难度、等级、音符数与 S/SS/SSS 分数线 |
| GET | `/api/{server}/music/{music_id}/difficulties` | 指定歌曲的难度与多人 live 分数线 |
| GET | `/api/{server}/multi-live-difficulties` | 多人 live 难度的属性要求、倍率与类型 |
| GET | `/api/{server}/weekly-multi-live-difficulties` | 每周多人 live 难度规则 |
| GET | `/api/{server}/music-shops` | 歌曲兑换条目、兑换数量与资源消耗 |
| GET | `/api/{server}/area-items` | 区域道具属性、加成目标与适用角色/乐队 |
| GET | `/api/{server}/area-item-spawns` | 区域道具摆放点 |
| GET | `/api/{server}/bonds` | 角色羁绊、关联角色与等级解锁 |
| GET | `/api/{server}/bond-effects` | 羁绊提供的生命值、能力值与技能加成 |
| GET | `/api/{server}/action-sets` | 区域动作及其角色/道具关联 |
| GET | `/api/{server}/degrees` | 玩家资料称号/徽章主数据 |
| GET | `/api/{server}/characters` | 角色主数据，含人物设定、服装季、语音、Live2D 服装 |
| GET | `/api/{server}/characters/{character_id}` | 单角色主数据 |
| GET | `/api/{server}/characters/{character_id}/cards` | 角色对应的卡列表 |
| GET | `/api/{server}/characters/{character_id}/costumes` | 角色对应的服装列表 |
| GET | `/api/{server}/bands` | 乐队主数据 |
| GET | `/api/{server}/bands/{band_id}` | 单个乐队主数据 |
| GET | `/api/{server}/bands/{band_id}/characters` | 乐队成员列表 |
| GET | `/api/{server}/bands/{band_id}/cards` | 乐队所有成员的卡列表，按角色主数据关联卡片 |
| GET | `/api/{server}/areas` | 区域主数据 |
| GET | `/api/{server}/areas/{area_id}` | 单个区域主数据 |
| GET | `/api/{server}/gacha` | 卡池主数据 |
| GET | `/api/{server}/gacha/{gacha_id}` | 单个卡池主数据 |
| GET | `/api/{server}/items` | 道具主数据 |
| GET | `/api/{server}/items/{item_id}` | 单个道具主数据 |
| GET | `/api/{server}/skills` | 技能主数据 |
| GET | `/api/{server}/skills/normalized` | 按技能 ID 聚合的技能等级、日文文本与持续时间 |
| GET | `/api/{server}/skills/{skill_id}` | 单个规范化技能及全部等级 |
| GET | `/api/{server}/skills/{skill_id}/cards` | 使用该技能（含第二技能）的卡列表 |
| GET | `/api/{server}/stamps` | 表情主数据 |
| GET | `/api/{server}/stamps/{stamp_id}` | 单个表情主数据 |
| GET | `/api/{server}/login-bonuses` | 登录奖励主数据 |
| GET | `/api/{server}/login-bonuses/{login_bonus_id}` | 单个登录奖励活动 |
| GET | `/api/{server}/costumes` | 服装主数据 |
| GET | `/api/{server}/costumes/{costume_id}` | 单个服装主数据 |
| GET | `/api/{server}/shops` | 商店主数据 |
| GET | `/api/{server}/shops/{shop_id}` | 单个商店主数据 |
| GET | `/api/{server}/cards` | 卡主数据，含各等级能力值、技能引用、`episodes` 卡面剧情与 `training` 特训数据 |
| GET | `/api/{server}/cards/{card_id}` | 单张卡主数据 |
| GET | `/api/{server}/cards/{card_id}/levels` | 单张卡各等级能力值，返回 `{"entries": [...]}` |
| GET | `/api/{server}/cards/{card_id}/episodes` | 单张卡的剧情元数据、属性加成及奖励，返回 `{"entries": [...]}` |
| GET | `/api/{server}/cards/{card_id}/training` | 单张卡的特训等级、属性加成及奖励，返回对象 |

卡不存在时返回 `404`；卡存在但
缺少等级或剧情记录时返回 `{"entries": []}`，缺少特训数据时返回 `404`。
乐队没有匹配成员时返回空列表。上述 ID 必须大于 0，否则返回 `400`。
剧情接口提供元数据，不包含剧情脚本。

map 型主数据接口统一返回 `{"entries": [...]}`，并把游戏内部的 map key
补到对象的 `areaItemId`、`bondsId`、`bondsEffectId`、`actionSetId`、`musicShopId`、
`degreeId` 或 `spawnPoint` 字段。`master-suite` 返回同一批数据的分组对象；它只保留当前
已确认的高价值字段，未知的游戏内部字段不会透传。

#### 规范化技能

`GET /api/{server}/skills/normalized` 保留原始 `/skills` 接口不变，将游戏服返回的
`(skillId, skillLevel)` 平铺记录聚合为以下结构：

```json
{
  "entries": [
    {
      "skillId": 1,
      "skillType": "score",
      "simpleDescription": {
        "jp": "スコア10%UP"
      },
      "levels": [
        {
          "skillLevel": 1,
          "duration": 5.0,
          "description": {
            "jp": "5秒間 スコアが10%UPする"
          }
        }
      ]
    }
  ]
}
```

- `entries` 按 `skillId` 升序排列，`levels` 按 `skillLevel` 升序排列。
- `simpleDescription.jp` 来源于最高等级的非空 `skillName`；某等级的说明保留在该等级的 `description.jp`。
- `duration` 映射自游戏技能主数据的浮点字段 `effectValue`；字段缺失时为 `null`。
- 当前只连接日服，因此本地化对象暂时只有 `jp`。以后增加其他服务器时无需改变字段类型。
- `skillType` 是当前上游唯一可靠的结构化效果类型。接口不会根据描述文本猜测
  Bestdori 风格的 `activationEffect`、发动条件或 `onceEffect`。

### 用户数据

以下接口针对 `.env` 中配置的 UID。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/user/profile` | 用户资料与数值 |
| GET | `/api/{server}/user/decks` | 用户编队 |
| GET | `/api/{server}/user/situations` | 用户持有的卡 |
| GET | `/api/{server}/user/title` | 用户当前称号 |
| GET | `/api/{server}/user/stamps` | 用户表情 |
| GET | `/api/{server}/user/areas` | 用户已启用区域道具，含 `areaItemCategory` 和 `level` |
| GET | `/api/{server}/user/music-scores` | 用户所有已记录的歌曲/难度成绩 |
| GET | `/api/{server}/user/music/{music_id}/scores` | 单曲所有已记录难度的原始成绩，未记录时返回 `{"entries": []}` |
| GET | `/api/{server}/user/music/{music_id}/status?difficulty=expert` | 查询指定歌曲指定难度的通关、FC、AP 状态 |
| GET | `/api/{server}/user/music-clear-info` | 按难度汇总通关、FC、AP 数量 |
| GET | `/api/{server}/user/items` | 用户道具余额 |
| GET | `/api/{server}/user/presents` | 用户礼物与礼盒信息 |
| GET | `/api/{server}/user/gacha` | 用户卡池记录 |
| GET | `/api/{server}/user/episodes` | 用户剧情解锁 |
| GET | `/api/{server}/user/missions` | 用户任务进度 |
| GET | `/api/{server}/user/login-bonuses` | 用户登录奖励进度 |
| GET | `/api/{server}/user/costumes` | 用户拥有的服装 |
| GET | `/api/{server}/user/characters` | 用户角色等级、经验、潜力和角色任务加成 |
| GET | `/api/{server}/user/character-mission-bonuses` | 用户角色任务加成明细 |
| GET | `/api/{server}/user/area-statuses` | 用户区域状态记录 |
| GET | `/api/{server}/user/character-affinity` | 用户角色亲密度记录 |

`user/areas` 的每个区域道具现在包含 `areaItemId`、`status`、`areaItemCategory` 和 `level`。
`user/characters` 的每条角色记录现在包含 `characterId`、`rank`、`exp`、`addExp`、`nextExp`、
`totalExp`、`releasedPotentialLevel`，以及三维 `potentialLevel`（`performanceLevel`、
`techniqueLevel`、`visualLevel`）。存在角色任务加成时，还会返回 `characterMissionBonus` 数组，
其中每项包含 `characterId`、`characterBonusType`、`performance`、`technique` 和 `visual`。
`user/character-mission-bonuses` 返回同样的明细数组，数据来自官方 Suite 用户快照的
`userCharacterMissionBonusMap` 字段。

#### 歌曲成绩与 FC/AP

歌曲成绩来自官方 Suite 用户快照的 `userMusicScoreMap`。单曲状态接口的
`difficulty` 支持 `easy`、`normal`、`hard`、`expert`、`special`，例如：

```bash
curl 'http://127.0.0.1:8080/api/jp/user/music/1/scores'
curl 'http://127.0.0.1:8080/api/jp/user/music/1/status?difficulty=expert'
```

单曲 `/scores` 不要求 `difficulty`。
结果保持上游顺序，成绩条目缺失 `musicId` 时使用外层 map key 补齐；`music_id <= 0` 返回 `400`。

响应示例：

```json
{
  "musicId": 1,
  "musicDifficulty": "expert",
  "played": true,
  "clearStatus": "all_perfect",
  "isCleared": true,
  "isFullCombo": true,
  "isAllPerfect": true,
  "soloHighScore": 1234567,
  "maxCombo": 987,
  "soloScoreRank": "sss"
}
```

`clearStatus` 的官方值包括 `not_cleared`、`cleared`、`full_combo` 和 `all_perfect`；AP
同时满足 FC。若该歌曲/难度没有成绩记录，接口仍返回 `200`，其中 `played`、`isCleared`、
`isFullCombo`、`isAllPerfect` 为 `false`，成绩字段为 `null`。

### 缓存

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{server}/cache` | 缓存条目数 |
| DELETE | `/api/{server}/cache` | 清空缓存 |

主数据默认缓存 1 小时，用户数据默认 5 分钟，排名默认 30 秒，可在 `.env` 中通过 `GARUPA_CACHE_TTL_MASTER`、`GARUPA_CACHE_TTL_USER`、`GARUPA_CACHE_TTL_RANKING` 调整。缓存未命中的并发请求会合并为一次上游调用，避免突刺打爆官方 API。
