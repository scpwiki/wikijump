<?php

 
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
