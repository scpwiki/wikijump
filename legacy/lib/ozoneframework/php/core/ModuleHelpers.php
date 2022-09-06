<?php

namespace Ozone\Framework;





/**
 * Module utility helper Class.
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
