DROP TABLE IF EXISTS data.Mail;
DROP TABLE IF EXISTS data.AbilityFeatureRow;;
DROP TABLE IF EXISTS data.DropMoney;
DROP TABLE IF EXISTS data.DropOptionValue;
DROP TABLE IF EXISTS data.ChatPoint;
DROP TABLE IF EXISTS data.SecretDungeonElementMask;
DROP TABLE IF EXISTS data.DropDist;
DROP TABLE IF EXISTS data.GameLog;
DROP TABLE IF EXISTS data.PCStat;
DROP TABLE IF EXISTS data.NpcAdjustmentAttackPower;
DROP TABLE IF EXISTS data.LifeTierBalance;
DROP TABLE IF EXISTS data.Dispose;
DROP TABLE IF EXISTS data.ExcessTemplate;
DROP TABLE IF EXISTS data.SecretDungeonElementSpawn;
DROP TABLE IF EXISTS data.LifeFishingTiming;

ALTER TABLE data.GameMsg_English RENAME KEY TO Id;
ALTER TABLE data.GameMsg_English RENAME MSG TO Message;

ALTER TABLE data.ArkPassive RENAME PCClass TO ClassId;
ALTER TABLE data.Skill RENAME LearnClass TO ClassId;
ALTER TABLE data.Skill RENAME LearnSkillPoint TO ClassId;
ALTER TABLE data.Skill RENAME LearnAwakening TO IsAwakening;
ALTER TABLE data.Skill RENAME LearnSuperSkill TO IsSuperSkill;
ALTER TABLE data.PC RENAME TO PlayerCharacter;