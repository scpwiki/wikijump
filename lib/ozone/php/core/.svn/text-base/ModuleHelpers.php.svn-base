<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Ozone
 * @package Ozone_Web
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Module utility helper class.
 *
 */
class ModuleHelpers {
	
	public static function findModuleClass($template){
		$classFilename = PathManager :: moduleClass($template);
		
		if (file_exists($classFilename)) {
			$moduleClassPath = $classFilename;
			$tmp1 = explode('/', $template);
			$size = sizeof($tmp1);
			$moduleClassName = $tmp1[$size -1];

		} else {
			$tmppath = PathManager :: moduleClassDir();
			// generate list of possible classes:
			$template;
			$path44 = explode('/', $template);

			for ($i = sizeof($path44) - 1; $i >= 0; $i --) {

				$tmppath2 = "";
				for ($k = 0; $k < $i; $k ++) {
					$tmppath2 .= $path44[$k]."/";
				}
				$tmppath2 .= "DefaultModule.php";
				$classFiles[] = $tmppath2;
			}

			foreach ($classFiles as $classFile) {
				if (file_exists($tmppath.$classFile)) {
					$moduleClassPath = $tmppath.$classFile;
					$moduleClassName = "DefaultModule";
					break;
				}
			}

		}
		return array($moduleClassName, $moduleClassPath);

	}

}
