DROP TABLE IF EXISTS `Badges_Data`;
CREATE TABLE `Badges_Data` (
  `ID` int(8) NOT NULL,
  `Name` varchar(100) NOT NULL,
  `Image_URL` varchar(100) DEFAULT NULL,
  PRIMARY KEY (`ID`,`Name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Badges_Users`;
CREATE TABLE `Badges_Users` (
  `Badge_ID` int(8) NOT NULL,
  `User_ID` int(11) NOT NULL,
  `Description` varchar(2000) DEFAULT NULL,
  `Date_Awarded` datetime DEFAULT NULL,
  PRIMARY KEY (`Badge_ID`,`User_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Medals_Data`;
CREATE TABLE `Medals_Data` (
  `Medal_ID` int(4) NOT NULL,
  `Name` varchar(50) DEFAULT NULL,
  `Link` varchar(70) DEFAULT NULL,
  `Description` varchar(500) DEFAULT NULL,
  `Gamemode` varchar(8) DEFAULT NULL,
  `Grouping` varchar(30) DEFAULT NULL,
  `Instructions` varchar(500) DEFAULT NULL,
  `Ordering` int(2) DEFAULT NULL,
  `Frequency` float DEFAULT NULL,
  `Count_Achieved_By` int(10) DEFAULT NULL,
  PRIMARY KEY (`Medal_ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


DROP TABLE IF EXISTS `Rankings_Script_History`;
CREATE TABLE `Rankings_Script_History` (
  `ID` int(8) NOT NULL,
  `Type` varchar(30) DEFAULT NULL,
  `Time` timestamp NULL DEFAULT NULL,
  `Count_Current` int(11) DEFAULT NULL,
  `Count_Total` int(11) DEFAULT NULL,
  `Elapsed_Seconds` int(20) DEFAULT NULL,
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
  `Count_Badges` int(4) DEFAULT NULL,
  `Count_Maps_Loved` int(4) DEFAULT NULL,
  `Count_Maps_Ranked` int(4) DEFAULT NULL,
  `Count_Medals` int(4) DEFAULT NULL,
  `Count_Replays_Watched` int(10) DEFAULT NULL,
  `Count_Subscribers` int(7) DEFAULT NULL,
  `Country_Code` varchar(3) DEFAULT NULL,
  `Is_Restricted` int(1) DEFAULT NULL,
  `Level_Catch` int(3) DEFAULT NULL,
  `Level_Mania` int(3) DEFAULT NULL,
  `Level_Standard` int(3) DEFAULT NULL,
  `Level_Stdev` int(3) DEFAULT NULL,
  `Level_Taiko` int(3) DEFAULT NULL,
  `Name` varchar(27) DEFAULT NULL,
  `PP_Catch` decimal(8,2) DEFAULT NULL,
  `PP_Mania` decimal(8,2) DEFAULT NULL,
  `PP_Standard` decimal(8,2) DEFAULT NULL,
  `PP_Stdev` decimal(8,2) DEFAULT NULL,
  `PP_Taiko` decimal(8,2) DEFAULT NULL,
  `PP_Total` decimal(8,2) DEFAULT NULL,
  `Rank_Global_Catch` int(20) DEFAULT NULL,
  `Rank_Global_Mania` int(20) DEFAULT NULL,
  `Rank_Global_Standard` int(20) DEFAULT NULL,
  `Rank_Global_Taiko` int(20) DEFAULT NULL,
  `Rarest_Medal_Achieved` datetime DEFAULT NULL,
  `Rarest_Medal_ID` int(4) DEFAULT NULL,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

