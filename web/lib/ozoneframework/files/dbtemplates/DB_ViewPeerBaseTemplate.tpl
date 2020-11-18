<?php
class DB_<{$className}>PeerBase extends BaseDBViewPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='<{$tableName}>';
		$this->objectName='<{$objectName}>';
		
		$this->fieldNames = array(<{foreach name="columns" from=$columns item=col}> '<{$col}>' <{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_<{$className}>Peer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}
