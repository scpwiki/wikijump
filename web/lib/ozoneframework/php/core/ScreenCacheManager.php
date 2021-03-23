<?php

namespace Ozone\Framework;



use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ScreenCache;
use Wikidot\DB\ScreenCachePeer;

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

	public function cachedLayout($runData, $screenCacheSettings){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "layout");
		$c->add("user_authenticated", $runData->isUserAuthenticated());

		$timeout = $screenCacheSettings->getLayoutTimeout($runData);
		// it was in seconds. make date with maximum time allowed
		$date = new ODate();
		$date->subtractSeconds($timeout);
		$c->add("date_updated", $date, ">");

		$sc = ScreenCachePeer::instance()->selectOne($c);
		if($sc != null){
			return $sc->getContent();
		}
		return null;
	}

	public function cachedScreen($runData, $screenCacheSettings){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "screen");
		$c->add("user_authenticated", $runData->isUserAuthenticated());

		$timeout = $screenCacheSettings->getScreenTimeout($runData);
		// it was in seconds. make date with maximum time allowed
		$date = new ODate();
		$date->subtractSeconds($timeout);
		$c->add("date_updated", $date, ">");

		$sc = ScreenCachePeer::instance()->selectOne($c);
		if($sc != null){
			return $sc->getContent();
		}
		return null;
	}

	public function updateCachedLayout($runData, $content){
		// delete any previous cache content for this request
		$this->deleteCachedLayout($runData);

		$sc = new ScreenCache();
		$sc->setTemplate($runData->getScreenTemplate());
		$sc->setDateUpdated(new ODate());
		$sc->setType("layout");
		$sc->setUserAuthenticated($runData->isUserAuthenticated());
		$sc->setRequestUri($runData->getRequestUri());
		$sc->setContent($content);

		$sc->save();

	}

	public function updateCachedScreen($runData, $content){
		// delete any previous cache content for this request
		$this->deleteCachedScreen($runData);

		$sc = new ScreenCache();
		$sc->setTemplate($runData->getScreenTemplate());
		$sc->setDateUpdated(new ODate());
		$sc->setType("screen");
		$sc->setUserAuthenticated($runData->isUserAuthenticated());
		$sc->setRequestUri($runData->getRequestUri());
		$sc->setContent($content);

		echo $content;

		$sc->save();
	}

	public function deleteCachedLayout($runData){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "layout");
		$c->add("user_authenticated", $runData->isUserAuthenticated());

		ScreenCachePeer::instance()->delete($c);
	}

	public function deleteCachedScreen($runData){
		$c = new Criteria();
		$c->add("template", $runData->getScreenTemplate());
		$c->add("request_uri", $runData->getRequestUri());
		$c->add("type", "screen");
		$c->add("user_authenticated", $runData->isUserAuthenticated());

		ScreenCachePeer::instance()->delete($c);
	}

	public function clearCache($template=null){

	}
}
