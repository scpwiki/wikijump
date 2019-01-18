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
 * Base class mapped to the database table <{$tableName}>.
 */
class DB_<{$className}>Base extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='<{$tableName}>';
		$this->peerName = '<{$peerName}>';
		$this->primaryKeyName = '<{$primaryKeyName}>';
		$this->fieldNames = array(<{foreach name="columns" from=$columns item=col}> '<{$col->getName()}>' <{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
		
		//$this->fieldDefaultValues=
	}


	<{foreach from=$masterRelations item=mrel}>

	public function get<{$mrel.foreignTmp}>s($criteria0=null){
		if($criteria0 == null){
			$criteria = new Criteria();
		} else {
			$criteria = clone($criteria0);
		}
		$criteria->addAnd("<{$mrel.foreignKeyName}>",$this->fieldValues['<{$mrel.localKeyName}>'] );
	
		$foreignPeerClassName = 'DB_<{$mrel.foreignTmp}>Peer';
		
		$result = $fpeer->selectByCriteria($criteria);
		return $result;
	}
	
	public function get<{$mrel.foreignTmp}>($criteria0 = null){
		if($criteria0 == null && is_array($this->prefetched)){
			if(in_array('<{$mrel.foreignTableName}>', $this->prefetched)){
				if(in_array('<{$mrel.foreignTableName}>', $this->prefetchedObjects)){
					return $this->prefetchedObjects['<{$mrel.foreignTableName}>'];
				} else {
					$obj = new DB_<{$mrel.foreignTmp}>($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['<{$mrel.foreignTableName}>'] = $obj;
					return $obj;
					
				}
			}
		}
		$foreignPeerClassName = 'DB_<{$mrel.foreignTmp}>Peer';
		$fpeer = new $foreignPeerClassName();
		
		if($criteria0 == null){
			$criteria = new Criteria();
		} else {
			$criteria = clone($criteria0);
		}
		
		$criteria->addAnd("<{$mrel.foreignKeyName}>",$this->fieldValues['<{$mrel.localKeyName}>'] );
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
	public function add<{$mrel.foreignTmp}>($referencingObject){
		$referencingObject->setFieldValue('<{$mrel.foreignKeyName}>', $this->fieldValues['<{$mrel.localKeyName}>']);
		// not save!
	}	
	
	<{/foreach}>

	<{foreach from=$foreignRelations item=frel}>
	<{if $frel.customFunction!=null}>
	public function get<{$frel.customFunction}>(){
	<{else}>
	public function get<{$frel.foreignTmp}>(){
	<{/if}>
		if(is_array($this->prefetched)){
			if(in_array('<{$frel.foreignTableName}>', $this->prefetched)){
				if(in_array('<{$frel.foreignTableName}>', $this->prefetchedObjects)){
					return $this->prefetchedObjects['<{$frel.foreignTableName}>'];
				} else {
					
					$obj = new DB_<{$frel.foreignTmp}>($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['<{$frel.foreignTableName}>'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB_<{$frel.foreignTmp}>Peer';	
		$fpeer = new $foreignPeerClassName();
		
		$criteria = new Criteria();
		
		$criteria->add("<{$frel.foreignKeyName}>", $this->fieldValues['<{$frel.localKeyName}>']);
		
		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}
	
	<{if $frel.customFunction!=null}>
	public function set<{$frel.customFunction}>(){
	<{else}>
	public function set<{$frel.foreignTmp}>($primaryObject){
	<{/if}>
		$this->fieldValues['<{$frel.localKeyName}>'] = $primaryObject->getFieldValue('<{$frel.foreignKeyName}>');
	}
	<{/foreach}>
	
	
	<{foreach name="gsetters" from=$columns item=col}>
	
	public function get<{$col->getPropertyNameFirstCapitalized()}>() {
		return $this->getFieldValue('<{$col->getName()}>');
	}
	
	public function set<{$col->getPropertyNameFirstCapitalized()}>($v1, $raw=false) {
		$this->setFieldValue('<{$col->getName()}>', $v1, $raw); 
	}
	
	<{/foreach}>
	
	

}

