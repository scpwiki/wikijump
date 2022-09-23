<?php

namespace Ozone\Framework;


/**
 * Simple db-based screen cache manager.
 *
 */


class ScreenCacheManager {

	public static $manager;

	public static function instance(){
		if(self::$manager == null){
			self::$manager = new ScreenCacheManager();
		}
		return self::$manager;
	}

}
