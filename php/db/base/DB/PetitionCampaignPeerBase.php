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
 * @version \$Id\$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

namespace DB;

use BaseDBPeer;



 
/**
 * Base peer class mapped to the database table petition_campaign.
 */
class PetitionCampaignPeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='petition_campaign';
		$this->objectName='DB_PetitionCampaign';
		$this->primaryKeyName = 'campaign_id';
		$this->fieldNames = array( 'campaign_id' ,  'site_id' ,  'name' ,  'identifier' ,  'active' ,  'number_signatures' ,  'deleted' ,  'collect_address' ,  'collect_city' ,  'collect_state' ,  'collect_zip' ,  'collect_country' ,  'collect_comments' ,  'show_city' ,  'show_state' ,  'show_zip' ,  'show_country' ,  'show_comments' ,  'thank_you_page' );
		$this->fieldTypes = array( 'campaign_id' => 'serial',  'site_id' => 'int',  'name' => 'varchar(256)',  'identifier' => 'varchar(256)',  'active' => 'boolean',  'number_signatures' => 'int',  'deleted' => 'boolean',  'collect_address' => 'boolean',  'collect_city' => 'boolean',  'collect_state' => 'boolean',  'collect_zip' => 'boolean',  'collect_country' => 'boolean',  'collect_comments' => 'boolean',  'show_city' => 'boolean',  'show_state' => 'boolean',  'show_zip' => 'boolean',  'show_country' => 'boolean',  'show_comments' => 'boolean',  'thank_you_page' => 'varchar(256)');
		$this->defaultValues = array( 'active' => 'true',  'number_signatures' => '0',  'deleted' => 'false',  'collect_address' => 'true',  'collect_city' => 'true',  'collect_state' => 'true',  'collect_zip' => 'true',  'collect_country' => 'true',  'collect_comments' => 'true',  'show_city' => 'true',  'show_state' => 'true',  'show_zip' => 'false',  'show_country' => 'true',  'show_comments' => 'false');
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_PetitionCampaignPeer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}