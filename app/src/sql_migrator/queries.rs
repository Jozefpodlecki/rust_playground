pub const SETUP_SQL: &str = r#"
    INSTALL sqlite;
    LOAD sqlite;

    CREATE SCHEMA data;
    CREATE SCHEMA jss;
    CREATE SCHEMA loa;
    CREATE SCHEMA lpk;
    CREATE SCHEMA assembly;

    CREATE TABLE assembly.LOSTARK
    (
        Address INT NOT NULL,
        Opcode VARCHAR(3) NOT NULL
    );

    CREATE TABLE lpk.config
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL 
    );

    CREATE TABLE lpk.data1
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );

    CREATE TABLE lpk.data2
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );
    
    CREATE TABLE lpk.data3
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );

    CREATE TABLE lpk.data4
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );

    CREATE TABLE lpk.font
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );

    CREATE TABLE lpk.leveldata1
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );

    CREATE TABLE lpk.leveldata2
    (
        Name VARCHAR(50) NOT NULL,
        FilePath VARCHAR(100) NOT NULL
    );
"#;

pub const SELECT_TOP_1_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name LIMIT 1";

pub const SELECT_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";

pub const POST_WORK_SQL: &str = r#"
ALTER TABLE data.AbilityEngrave ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityFeature ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityMapping ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilitySpecification ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityStoneAbilityGroup ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityStoneBase ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityStoneCarveOption ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AbilityStoneUpgrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Achievement ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AchievementChat ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AchievementGrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AchievementObjective ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AcquireLimit ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AddonSkillFeature ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AdvBook ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AdvTask ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AdvTaskPastStory ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AffinityProjectile ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AFKAutomaticKick ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AncientOrb ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Announce ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AnnounceCategory ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AnnounceGetItem ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosGameObject ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosLevel ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosNormal ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosPlayerClassChangeInfo ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosRegulation ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosShop ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AosSubEvent ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArcanaCard ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassive ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassiveOption ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassiveStigmaBase ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassiveStigmaLevel ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassiveStigmaRank ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassLevel ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassMission ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ArkPassSeason ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AstraDrop ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AstraOption ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AttributeInfo ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AvatarAssembly ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.AvatarGrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BalanceLevelMapping ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldDeathmatchNodeLink ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldDeathmatchV ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldEntrance ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldFieldBoss ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldGameObject ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldMatchSchedule ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldOccupyEventReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldOccupyReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldOccupyWeekReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldRankReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldRealm ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldRegulation ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldRule ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldScoreGapBuff ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldTeamBasePoint ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlefieldTimeTable ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BattlePoint ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.BreakThroughBuff ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Calendar ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CameraContentsSetting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CameraSetting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CashShopCameraPreset ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CashShopPaidToken ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CertificatedModules ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChallengePreset ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChallengePresetAdjustmentStat ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChallengePresetItemSet ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChallengePresetSkillRune ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChangeFootStepAvatarFX ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChangeFootStepAvatarMoveSkillFX ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChaosDungeonSchedule ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChaosDungeonTier ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChaosGateContents ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CharacterCustomizing ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CharInfoStatMinMax ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ChatBox ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ClientCmd ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ClientSummonNpc ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CollectingPuzzle ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Colosseum ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ColosseumMMRReset ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ColosseumRank ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ColosseumSeasonReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ColosseumSeasonSchedule ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ColumnRecovery ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CombatAdjustment ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CombatEffect ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CombinedPresetContents ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CommissionFixedAmount ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CommonAction ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CommonActionCondition ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CommonActionEffect ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ConcertBasicMusic ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ConcertInstrument ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ConcertThumbnail ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsRequirement ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsRewardDisplay ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsSeason ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentStatus ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsTimeTable ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsTriggerSignal ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContentsUnlock ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ContinentBase ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CryptoData ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CryptoTableData ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointAcquire ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointHintList ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointMusic ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointRewardSymbol ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointTypeUI ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointUIButton ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.CumulativePointUIButtonGuage ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DirectShop ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropBase ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropBaseLimit ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropBoost ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropEntity ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropEther ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DropLimit ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DungeonExitList ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.DungeonPartyBalance ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EliteNpcAbility ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EliteNpcAbilityGroup ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Emoticon ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EmoticonPack ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EpicSkill ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventContentsReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMission ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMissionCategory ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMissionCategoryReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMissionObjective ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMissionPCAutoSetting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventMissionPCAutoSettingEntity ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.EventTokenAutoReward ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ExpeditionLevelNew ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ExpeditionMissionRewardInfo ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ExpeditionRewardNew ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.FoundationBoosting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.FoundationBoostingInfo ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GameAction ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GameCondition ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GameNote ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GoldShop ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GroupBuffIconSetting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GrowthModeMission ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GrowthModeSetting ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GrowthPeriodItem ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuideBookURL ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildActivityLevel ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildContentsList ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildDonation ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildGrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildGradeAuthority ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildLeavePenalty ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildLevel ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildMark ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildMarkLayer ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildMission ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildObjective ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildResearch ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildResource ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.GuildSkill ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.HintGroup ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.HonorCompensation ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.HonorTitle ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.HotKey ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.HotTimeEvent ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ImageOrderGameMain ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Immune ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.IntegrateContentsList ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.IntegrateDungeon ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.IntegrateDungeonSpecial ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.IntegrateWarBoard ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.IsometricCamera ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.Item ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAccessoryUpgrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAddMaxCount ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAmplificationBase ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAmplificationBonus ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAmplificationMaterial ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAssembly ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAssemblyNpc ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAssemblyPack ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemAvatarSet ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemBraceletEnchant ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemBraceletEnchantCost ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemBraceletOptionConvert ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemBraceletUpgrade ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemCalibrate ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemCalibrateAdjustBonus ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemCategoryRestriction ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemChange ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemChangeNpc ALTER COLUMN PrimaryKey RENAME TO Id;
ALTER TABLE data.ItemClassOption ALTER COLUMN PrimaryKey RENAME TO Id;

ALTER TABLE data.Ability ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityEngrave ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityFeature ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityMapping ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilitySpecification ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityStoneAbilityGroup ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityStoneBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityStoneCarveOption ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AbilityStoneUpgrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Achievement ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AchievementChat ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AchievementGrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AchievementObjective ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AcquireLimit ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AddonSkillFeature ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AdvBook ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AdvTask ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AdvTaskPastStory ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AffinityProjectile ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AFKAutomaticKick ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AncientOrb ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Announce ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AnnounceCategory ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AnnounceGetItem ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosGameObject ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosLevel ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosNormal ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosPlayerClassChangeInfo ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosRegulation ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosShop ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AosSubEvent ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArcanaCard ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassive ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassiveOption ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassiveStigmaBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassiveStigmaLevel ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassiveStigmaRank ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassLevel ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassMission ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ArkPassSeason ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AttributeInfo ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AvatarAssembly ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.AvatarGrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BalanceLevelMapping ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldDeathmatchNodeLink ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldDeathmatchV ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldEntrance ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldFieldBoss ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldGameObject ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldMatchSchedule ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldOccupyEventReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldOccupyReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldOccupyWeekReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldRankReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldRealm ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldRegulation ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldRule ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldScoreGapBuff ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldTeamBasePoint ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlefieldTimeTable ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BattlePoint ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.BreakThroughBuff ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Calendar ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CameraContentsSetting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CameraSetting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CashShopCameraPreset ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CashShopPaidToken ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChallengePreset ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChallengePresetAdjustmentStat ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChallengePresetItemSet ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChallengePresetSkillRune ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChangeFootStepAvatarFX ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChangeFootStepAvatarMoveSkillFX ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChaosDungeonSchedule ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChaosDungeonTier ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChaosGateContents ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CharacterCustomizing ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CharInfoStatMinMax ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ChatBox ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ClientCmd ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ClientSummonNpc ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CollectingPuzzle ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Colosseum ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ColosseumRank ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ColosseumSeasonSchedule ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ColumnRecovery ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CombatEffect ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CombinedPresetContents ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CommissionFixedAmount ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CommonAction ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CommonActionCondition ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CommonActionEffect ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ConcertBasicMusic ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ConcertInstrument ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ConcertThumbnail ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsRequirement ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsRewardDisplay ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsSeason ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentStatus ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsTimeTable ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsTriggerSignal ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContentsUnlock ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ContinentBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CryptoData ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CryptoTableData ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointAcquire ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointHintList ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointMusic ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointRewardSymbol ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointTypeUI ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointUIButton ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.CumulativePointUIButtonGuage ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DirectShop ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DropBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DropBoost ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DropEntity ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DungeonExitList ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.DungeonPartyBalance ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EliteNpcAbility ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EliteNpcAbilityGroup ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Emoticon ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EmoticonPack ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EpicSkill ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventContentsReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMission ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMissionCategory ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMissionCategoryReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMissionObjective ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMissionPCAutoSetting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventMissionPCAutoSettingEntity ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.EventTokenAutoReward ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ExpeditionLevelNew ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ExpeditionMissionRewardInfo ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ExpeditionRewardNew ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.FoundationBoosting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.FoundationBoostingInfo ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GameAction ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GameCondition ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GameNote ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GoldShop ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GroupBuffIconSetting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GrowthModeMission ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GrowthModeSetting ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GrowthPeriodItem ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuideBookURL ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildActivityLevel ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildDonation ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildGrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildGradeAuthority ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildMark ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildMarkLayer ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildMission ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildObjective ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildResearch ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildResource ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.GuildSkill ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.HintGroup ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.HonorCompensation ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.HonorTitle ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.HotKey ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ImageOrderGameMain ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Immune ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.IntegrateContentsList ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.IntegrateDungeon ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.IntegrateDungeonSpecial ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.IntegrateWarBoard ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.IsometricCamera ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.Item ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAccessoryUpgrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAddMaxCount ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAmplificationBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAmplificationBonus ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAmplificationMaterial ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAssembly ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAssemblyNpc ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAssemblyPack ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemAvatarSet ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemBraceletEnchant ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemBraceletEnchantCost ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemBraceletOptionConvert ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemBraceletUpgrade ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemCalibrate ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemCalibrateAdjustBonus ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemCategoryRestriction ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemChange ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemChangeNpc ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemClassOption ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemContentsFilter ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemDictionaryAcquisition ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemDictionaryCategoryInfo ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemDisassembly ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirCommon ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirMaterial ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirOption ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirOptionSet ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirSelectionBase ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemElixirSelectionEntity ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemEnhanceCommon ALTER COLUMN SecondaryKey RENAME TO SubId;
ALTER TABLE data.ItemEnhanceEffect ALTER COLUMN SecondaryKey RENAME TO SubId;

VACUUM;
"#;