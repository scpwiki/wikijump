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
 * @package Wikidot_Cron
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Periodically creates downloadable site backups (on request).
 *
 */
class HandleBackupRequestsJob implements SchedulerJob {
	
	public function run(){
		
		// check!
		$c = new Criteria();
		$c->add("status", null);
		$c->addOrderDescending("backup_id");

		$sb = DB_SiteBackupPeer::instance()->selectOne($c); // select only one!
		
		if(!$sb){
			return;
		}

		$db = Database::connection();
		$sb->setStatus("started");
		$sb->save();
		
		$db->begin();
		
		try{
			$b = new Backuper();
			$b->setConfig($sb);
			$b->backup();

			// check 
			
			$sb->setStatus("completed");
			$sb->setDate(new ODate());
			$sb->setRand($b->getRand());
			
			$sb->save();
		}catch(Exception $e){
			$sb->setStatus("failed");
			$sb->save();	
		}
		$db->commit();
			
	}
	
}
