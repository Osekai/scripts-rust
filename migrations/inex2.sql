-- Adminer 4.8.1 MySQL 12.1.2-MariaDB dump

SET NAMES utf8;
SET time_zone = '+00:00';
SET foreign_key_checks = 0;
SET sql_mode = 'NO_AUTO_VALUE_ON_ZERO';

DROP VIEW IF EXISTS `All_Users`;
CREATE TABLE `All_Users` (`User_ID` int(11), `Name` varchar(27));


SET NAMES utf8mb4;

DROP TABLE IF EXISTS `Badges_Data`;
CREATE TABLE `Badges_Data` (
  `ID` int(11) NOT NULL,
  `Name` varchar(100) NOT NULL,
  `Image_URL` varchar(100) NOT NULL,
  PRIMARY KEY (`ID`,`Name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Badges_Users`;
CREATE TABLE `Badges_Users` (
  `Badge_ID` int(11) NOT NULL,
  `User_ID` int(11) NOT NULL,
  `Description` varchar(2000) DEFAULT NULL,
  `Date_Awarded` datetime DEFAULT NULL,
  PRIMARY KEY (`Badge_ID`,`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Beatmaps_Data`;
CREATE TABLE `Beatmaps_Data` (
  `Beatmap_ID` int(11) NOT NULL,
  `Beatmapset_ID` int(11) NOT NULL,
  `Mapper_ID` int(11) NOT NULL,
  `Gamemode` varchar(5) DEFAULT NULL,
  `Song_Title` varchar(80) NOT NULL,
  `Song_Artist` varchar(80) NOT NULL,
  `Mapper_Name` varchar(27) DEFAULT NULL,
  `Difficulty_Rating` double DEFAULT NULL,
  `Difficulty_Name` varchar(80) DEFAULT NULL,
  `Download_Unavailable` int(11) DEFAULT NULL,
  `Status` varchar(255) DEFAULT 'ranked',
  PRIMARY KEY (`Beatmap_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Beatmaps_Packs`;
CREATE TABLE `Beatmaps_Packs` (
  `Beatmapset_ID` int(11) NOT NULL,
  `Pack_ID` varchar(10) NOT NULL,
  PRIMARY KEY (`Beatmapset_ID`,`Pack_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Common_Comments`;
CREATE TABLE `Common_Comments` (
  `ID` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `Target_ID` int(11) DEFAULT NULL,
  `Target_Table` varchar(40) DEFAULT NULL,
  `User_ID` int(11) DEFAULT NULL,
  `Parent_Comment_ID` int(11) DEFAULT NULL,
  `Text` text DEFAULT NULL,
  `Date` datetime DEFAULT NULL,
  `Is_Pinned` int(11) DEFAULT NULL,
  `Deleted` int(11) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Common_Countries`;
CREATE TABLE `Common_Countries` (
  `Country_Code` varchar(11) NOT NULL,
  `Name` varchar(100) NOT NULL,
  PRIMARY KEY (`Country_Code`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Common_Mods`;
CREATE TABLE `Common_Mods` (
  `ID` varchar(4) NOT NULL,
  `Name` varchar(50) NOT NULL,
  `Type` varchar(255) DEFAULT 'difficultyincrease',
  `Description` varchar(255) DEFAULT NULL,
  `Gamemode` varchar(255) DEFAULT NULL,
  UNIQUE KEY `ID` (`ID`,`Name`,`Gamemode`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Common_Votes`;
CREATE TABLE `Common_Votes` (
  `Target_ID` int(11) NOT NULL,
  `Target_Table` varchar(50) NOT NULL,
  `User_ID` int(11) NOT NULL,
  PRIMARY KEY (`Target_ID`,`Target_Table`,`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `copy_Medals_Configuration`;
CREATE TABLE `copy_Medals_Configuration` (
  `Medal_ID` int(11) NOT NULL,
  `Video_URL` varchar(200) DEFAULT NULL,
  `First_Achieved_Date` datetime DEFAULT NULL,
  `First_Achieved_User_ID` int(11) DEFAULT NULL,
  `Is_Solution_Found` int(11) DEFAULT NULL,
  `Is_Lazer` int(11) DEFAULT NULL,
  `Is_Restricted` int(11) DEFAULT NULL,
  `Solution` varchar(2000) DEFAULT NULL,
  `Date_Released` datetime DEFAULT NULL,
  PRIMARY KEY (`Medal_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Beatmaps`;
CREATE TABLE `Medals_Beatmaps` (
  `ID` int(11) NOT NULL AUTO_INCREMENT,
  `Medal_ID` int(11) DEFAULT NULL,
  `Beatmap_ID` int(11) DEFAULT NULL,
  `Beatmap_Submitted_User_ID` int(11) DEFAULT NULL,
  `Beatmap_Submitted_Date` datetime DEFAULT NULL,
  `Note` varchar(2000) DEFAULT NULL,
  `Note_Submitted_User_ID` int(11) DEFAULT NULL,
  `Note_Submitted_Date` datetime DEFAULT NULL,
  `Deleted` int(11) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Configuration`;
CREATE TABLE `Medals_Configuration` (
  `Medal_ID` int(11) NOT NULL,
  `Video_URL` varchar(200) DEFAULT NULL,
  `First_Achieved_Date` datetime DEFAULT NULL,
  `First_Achieved_User_ID` int(11) DEFAULT NULL,
  `Is_Solution_Found` int(11) DEFAULT NULL,
  `Supports_Lazer` int(11) DEFAULT NULL,
  `Is_Restricted` int(11) DEFAULT NULL,
  `Solution` varchar(2000) DEFAULT NULL,
  `Date_Released` datetime DEFAULT NULL,
  `Supports_Stable` int(11) NOT NULL DEFAULT 1,
  PRIMARY KEY (`Medal_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Data`;
CREATE TABLE `Medals_Data` (
  `Medal_ID` int(11) NOT NULL,
  `Name` varchar(50) DEFAULT NULL,
  `Link` varchar(70) DEFAULT NULL,
  `Description` varchar(500) DEFAULT NULL,
  `Gamemode` varchar(8) DEFAULT NULL,
  `Grouping` varchar(30) DEFAULT NULL,
  `Instructions` varchar(500) DEFAULT NULL,
  `Ordering` int(11) DEFAULT NULL,
  `Frequency` float DEFAULT NULL,
  `Count_Achieved_By` int(11) DEFAULT NULL,
  PRIMARY KEY (`Medal_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Favourites`;
CREATE TABLE `Medals_Favourites` (
  `User_ID` int(11) NOT NULL,
  `Medal_ID` int(11) NOT NULL,
  PRIMARY KEY (`Medal_ID`,`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Solutions_Beatmaps_Packs`;
CREATE TABLE `Medals_Solutions_Beatmaps_Packs` (
  `Medal_ID` int(11) NOT NULL,
  `Pack_ID` varchar(8) NOT NULL,
  `Gamemode` varchar(5) NOT NULL,
  `Maps_Count` int(11) DEFAULT NULL,
  `Maps_Length` int(11) DEFAULT NULL,
  `Name` varchar(200) NOT NULL,
  `Link` varchar(400) NOT NULL,
  PRIMARY KEY (`Medal_ID`,`Pack_ID`,`Gamemode`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Solutions_Mods`;
CREATE TABLE `Medals_Solutions_Mods` (
  `Medal_ID` int(11) NOT NULL,
  `Mod` varchar(4) NOT NULL,
  PRIMARY KEY (`Medal_ID`,`Mod`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP VIEW IF EXISTS `Merged_Users`;
CREATE TABLE `Merged_Users` (`User_ID` int(11), `Name` varchar(27));


DROP TABLE IF EXISTS `Rankings_Script_History`;
CREATE TABLE `Rankings_Script_History` (
  `ID` int(11) NOT NULL,
  `Type` varchar(30) DEFAULT NULL,
  `Time` timestamp NULL DEFAULT NULL,
  `Count_Current` int(11) DEFAULT NULL,
  `Count_Total` int(11) DEFAULT NULL,
  `Elapsed_Seconds` int(11) DEFAULT NULL,
  `Elapsed_Last_Update` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Rankings_Users`;
CREATE TABLE `Rankings_Users` (
  `ID` int(11) NOT NULL,
  `Accuracy_Catch` decimal(5,2) DEFAULT NULL,
  `Accuracy_Mania` decimal(5,2) DEFAULT NULL,
  `Accuracy_Standard` decimal(5,2) DEFAULT NULL,
  `Accuracy_Stdev` decimal(5,2) DEFAULT NULL,
  `Accuracy_Taiko` decimal(5,2) DEFAULT NULL,
  `Count_Badges` int(11) DEFAULT NULL,
  `Count_Maps_Loved` int(11) DEFAULT NULL,
  `Count_Maps_Ranked` int(11) DEFAULT NULL,
  `Count_Medals` int(11) DEFAULT NULL,
  `Count_Replays_Watched` int(11) DEFAULT NULL,
  `Count_Subscribers` int(11) DEFAULT NULL,
  `Country_Code` varchar(3) DEFAULT NULL,
  `Is_Restricted` int(11) DEFAULT NULL,
  `Level_Catch` int(11) DEFAULT NULL,
  `Level_Mania` int(11) DEFAULT NULL,
  `Level_Standard` int(11) DEFAULT NULL,
  `Level_Stdev` int(11) DEFAULT NULL,
  `Level_Taiko` int(11) DEFAULT NULL,
  `Name` varchar(27) DEFAULT NULL,
  `PP_Catch` decimal(8,2) DEFAULT NULL,
  `PP_Mania` decimal(8,2) DEFAULT NULL,
  `PP_Standard` decimal(8,2) DEFAULT NULL,
  `PP_Stdev` decimal(8,2) DEFAULT NULL,
  `PP_Taiko` decimal(8,2) DEFAULT NULL,
  `PP_Total` decimal(8,2) DEFAULT NULL,
  `Rank_Global_Catch` int(11) DEFAULT NULL,
  `Rank_Global_Mania` int(11) DEFAULT NULL,
  `Rank_Global_Standard` int(11) DEFAULT NULL,
  `Rank_Global_Taiko` int(11) DEFAULT NULL,
  `Rarest_Medal_Achieved` datetime DEFAULT NULL,
  `Rarest_Medal_ID` int(11) DEFAULT NULL,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Rankings_Users_Medals`;
CREATE TABLE `Rankings_Users_Medals` (
  `User_ID` int(11) NOT NULL,
  `Medal_ID` int(4) NOT NULL,
  `Achieved_At` datetime DEFAULT NULL,
  PRIMARY KEY (`User_ID`,`Medal_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Blacklist_Words`;
CREATE TABLE `System_Blacklist_Words` (
  `Word` varchar(64) NOT NULL,
  PRIMARY KEY (`Word`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Phinx`;
CREATE TABLE `System_Phinx` (
  `version` bigint(20) NOT NULL,
  `migration_name` varchar(100) DEFAULT NULL,
  `start_time` timestamp NULL DEFAULT NULL,
  `end_time` timestamp NULL DEFAULT NULL,
  `breakpoint` tinyint(1) NOT NULL DEFAULT 0,
  PRIMARY KEY (`version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Roles_Assignments`;
CREATE TABLE `System_Roles_Assignments` (
  `User_ID` int(11) NOT NULL,
  `Role_ID` int(11) NOT NULL,
  PRIMARY KEY (`User_ID`,`Role_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Roles_Roles`;
CREATE TABLE `System_Roles_Roles` (
  `ID` int(11) NOT NULL AUTO_INCREMENT,
  `Name_Short` varchar(100) DEFAULT NULL,
  `Name_Long` varchar(200) DEFAULT NULL,
  `Colour` varchar(500) DEFAULT NULL,
  `Permissions` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`Permissions`)),
  `Blocked_Permissions` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`Blocked_Permissions`)),
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Sessions`;
CREATE TABLE `System_Sessions` (
  `Key` varchar(64) NOT NULL,
  `User_ID` int(11) DEFAULT NULL,
  PRIMARY KEY (`Key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Users`;
CREATE TABLE `System_Users` (
  `User_ID` int(11) NOT NULL,
  `Name` varchar(27) DEFAULT NULL,
  `Joined_Date` datetime DEFAULT NULL,
  PRIMARY KEY (`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `System_Users_Settings`;
CREATE TABLE `System_Users_Settings` (
  `User_ID` int(11) NOT NULL,
  `Settings` longtext DEFAULT NULL,
  PRIMARY KEY (`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `All_Users`;
CREATE ALGORITHM=UNDEFINED SQL SECURITY DEFINER VIEW `All_Users` AS select `Rankings_Users`.`ID` AS `User_ID`,`Rankings_Users`.`Name` AS `Name` from `Rankings_Users` union all select `System_Users`.`User_ID` AS `User_ID`,`System_Users`.`Name` AS `Name` from `System_Users`;

DROP TABLE IF EXISTS `Merged_Users`;
CREATE ALGORITHM=UNDEFINED SQL SECURITY DEFINER VIEW `Merged_Users` AS select `Rankings_Users`.`ID` AS `User_ID`,`Rankings_Users`.`Name` AS `Name` from `Rankings_Users` union select `System_Users`.`User_ID` AS `User_ID`,`System_Users`.`Name` AS `Name` from `System_Users`;

-- 2025-12-02 23:10:17