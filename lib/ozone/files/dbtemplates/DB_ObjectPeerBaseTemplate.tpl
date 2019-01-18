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
 
/**
 * Base peer class mapped to the database table <{$tableName}>.
 */
class DB_<{$className}>PeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='<{$tableName}>';
		$this->objectName='<{$objectName}>';
		$this->primaryKeyName = '<{$primaryKeyName}>';
		$this->fieldNames = array(<{foreach name="columns" from=$columns item=col}> '<{$col->getName()}>' <{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
		$this->fieldTypes = array(<{foreach name="columns" from=$columns item=col}> '<{$col->getName()}>' => '<{$col->getType()}>'<{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
		$this->defaultValues = array(<{foreach name="columns" from=$defaultValues key=col item=val}> '<{$col}>' => '<{$val}>'<{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_<{$className}>Peer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}
