<?php

namespace Ozone\Framework;





use Wikidot\Utils\GlobalProperties;

/**
 * Utility Class.
 *
 */
class Utils {
	public static function screenClassFind($template) {
		$classFilename = GlobalProperties :: $ABSOLUTE_PATH.WIKIJUMP_ROOT."/screen/".$template.".php";

		if (!file_exists($classFilename)) {
			// generate list of possible classes:
			$path44 = explode('/', $template);
			$tmppath = GlobalProperties :: $ABSOLUTE_PATH.WIKIJUMP_ROOT."/screen/";
			for ($i = sizeof($path44) - 1; $i >= 0; $i --) {

				$tmppath2 = "";
				for ($k = 0; $k < $i; $k ++) {
					$tmppath2 .= $path44[$k]."/";
				}
				$tmppath2 .= "DefaultScreen.php";

				$classFiles[] = $tmppath2;
			}

			foreach ($classFiles as $classFile) {
				if (file_exists($tmppath.$classFile)) {
					return $classFile;
				}
			}
		} else {
			return $template;
		}
	}
}
