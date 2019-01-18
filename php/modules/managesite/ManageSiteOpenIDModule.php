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
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class ManageSiteOpenIDModule extends ManageSiteBaseModule {

	public function build($runData){
		$site = $runData->getTemp("site");
		$settings = $site->getSettings();

		$runData->contextAdd("siteDomain", $site->getDomain()); 
		
		$openIdServices = array(
			array(	'pattern' => '^[a-z0-9\.\-]+\.myopenid\.com\/?$',
					'server' => 'http://www.myopenid.com/server'),
			array(	'pattern' => '^[a-z0-9\.\-]+\.getopenid\.com\/?$',
					'server' => 'https://getopenid.com/server'),
			array(	'pattern' => '^[a-z0-9\.\-]+\.livejournal\.com\/?$',
					'server' => 'http://www.livejournal.com/openid/server.bml'),
			array(	'pattern' => '^[a-z0-9\.\-]+\.vox\.com\/?$',
					'server' => 'http://www.vox.com/openid/server'),
			array(	'pattern' => '^[a-z0-9\.\-]+\.verisignlabs\.com\/?$',
					'server' => 'https://pip.verisignlabs.com/server'),
			array(	'pattern' => '^[a-z0-9\.\-]+\.openid\.pl\/?$',
					'server' => 'http://openid.pl/server'),
					array(	'pattern' => '^myid\.pl\/id\/',
					'server' => 'http://myid.pl/auth')
		);
		
		$json = new JSONService();
		$os = $json->encode($openIdServices);
		
		$runData->contextAdd("openIdServices", $os);
		
		// current settings
		$runData->contextAdd("enabled", $settings->getOpenidEnabled());
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("page_id", null);
		$ooroot = DB_OpenidEntryPeer::instance()->selectOne($c);
		
		$c = new Criteria();
		$c->add("site_id", $site->getSiteId());
		$c->add("page_id", null, "!=");
		$oos = DB_OpenidEntryPeer::instance()->select($c);
		
		$runData->contextAdd("openIdRoot", $ooroot);
		
		$runData->contextAdd("openIds", $oos);
		
	}
	
}
