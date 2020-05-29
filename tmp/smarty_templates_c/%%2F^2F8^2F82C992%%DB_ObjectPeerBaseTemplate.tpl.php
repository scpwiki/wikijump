<?php /* Smarty version 2.6.7, created on 2008-12-06 16:56:36
         compiled from /var/www/wikidot/lib/ozone/files/dbtemplates/DB_ObjectPeerBaseTemplate.tpl */ ?>
<?php echo '<?php'; ?>

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
 * Base peer class mapped to the database table <?php echo $this->_tpl_vars['tableName']; ?>
.
 */
class DB_<?php echo $this->_tpl_vars['className']; ?>
PeerBase extends BaseDBPeer {
	public static $peerInstance;
	
	protected function internalInit(){
		$this->tableName='<?php echo $this->_tpl_vars['tableName']; ?>
';
		$this->objectName='<?php echo $this->_tpl_vars['objectName']; ?>
';
		$this->primaryKeyName = '<?php echo $this->_tpl_vars['primaryKeyName']; ?>
';
		$this->fieldNames = array(<?php $this->_foreach['columns'] = array('total' => count($_from = (array)$this->_tpl_vars['columns']), 'iteration' => 0);
if ($this->_foreach['columns']['total'] > 0):
    foreach ($_from as $this->_tpl_vars['col']):
        $this->_foreach['columns']['iteration']++;
?> '<?php echo $this->_tpl_vars['col']->getName(); ?>
' <?php if (! ($this->_foreach['columns']['iteration'] == $this->_foreach['columns']['total'])): ?>, <?php endif;  endforeach; endif; unset($_from); ?>);
		$this->fieldTypes = array(<?php $this->_foreach['columns'] = array('total' => count($_from = (array)$this->_tpl_vars['columns']), 'iteration' => 0);
if ($this->_foreach['columns']['total'] > 0):
    foreach ($_from as $this->_tpl_vars['col']):
        $this->_foreach['columns']['iteration']++;
?> '<?php echo $this->_tpl_vars['col']->getName(); ?>
' => '<?php echo $this->_tpl_vars['col']->getType(); ?>
'<?php if (! ($this->_foreach['columns']['iteration'] == $this->_foreach['columns']['total'])): ?>, <?php endif;  endforeach; endif; unset($_from); ?>);
		$this->defaultValues = array(<?php $this->_foreach['columns'] = array('total' => count($_from = (array)$this->_tpl_vars['defaultValues']), 'iteration' => 0);
if ($this->_foreach['columns']['total'] > 0):
    foreach ($_from as $this->_tpl_vars['col'] => $this->_tpl_vars['val']):
        $this->_foreach['columns']['iteration']++;
?> '<?php echo $this->_tpl_vars['col']; ?>
' => '<?php echo $this->_tpl_vars['val']; ?>
'<?php if (! ($this->_foreach['columns']['iteration'] == $this->_foreach['columns']['total'])): ?>, <?php endif;  endforeach; endif; unset($_from); ?>);
	}
	
	public static function instance(){
		if(self::$peerInstance == null){
			$className = "DB_<?php echo $this->_tpl_vars['className']; ?>
Peer";
			self::$peerInstance = new $className();
		}
		return self::$peerInstance;
	}

}