<?php
class DB_<{$className}>Base extends BaseDBViewObject {

	protected function internalInit(){
		$this->tableName='<{$tableName}>';
		$this->peerName = '<{$peerName}>';
		
		$this->fieldNames = array(<{foreach name="columns" from=$columns item=col}> '<{$col}>' <{if !$smarty.foreach.columns.last}>, <{/if}><{/foreach}>);
		
	}

	
	<{foreach name="gsetters" from=$columns item=col}>
	
	public function get<{$stringHelper->propertyNameFirstCapitalized($col)}>() {
		return $this->getFieldValue('<{$col}>');
	}
	<{/foreach}>

}

