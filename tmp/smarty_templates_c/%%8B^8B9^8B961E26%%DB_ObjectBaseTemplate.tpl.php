<?php /* Smarty version 2.6.7, created on 2008-12-06 16:56:36
         compiled from /var/www/wikidot/lib/ozone/files/dbtemplates/DB_ObjectBaseTemplate.tpl */ ?>
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
 * Base class mapped to the database table <?php echo $this->_tpl_vars['tableName']; ?>
.
 */
class DB_<?php echo $this->_tpl_vars['className']; ?>
Base extends BaseDBObject {

	protected function internalInit(){
		$this->tableName='<?php echo $this->_tpl_vars['tableName']; ?>
';
		$this->peerName = '<?php echo $this->_tpl_vars['peerName']; ?>
';
		$this->primaryKeyName = '<?php echo $this->_tpl_vars['primaryKeyName']; ?>
';
		$this->fieldNames = array(<?php $this->_foreach['columns'] = array('total' => count($_from = (array)$this->_tpl_vars['columns']), 'iteration' => 0);
if ($this->_foreach['columns']['total'] > 0):
    foreach ($_from as $this->_tpl_vars['col']):
        $this->_foreach['columns']['iteration']++;
?> '<?php echo $this->_tpl_vars['col']->getName(); ?>
' <?php if (! ($this->_foreach['columns']['iteration'] == $this->_foreach['columns']['total'])): ?>, <?php endif;  endforeach; endif; unset($_from); ?>);

		//$this->fieldDefaultValues=
	}


	<?php if (count($_from = (array)$this->_tpl_vars['masterRelations'])):
    foreach ($_from as $this->_tpl_vars['mrel']):
?>

	public function get<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
s($criteria0=null){
		if($criteria0 == null){
			$criteria = new Criteria();
		} else {
			$criteria = clone($criteria0);
		}
		$criteria->addAnd("<?php echo $this->_tpl_vars['mrel']['foreignKeyName']; ?>
",$this->fieldValues['<?php echo $this->_tpl_vars['mrel']['localKeyName']; ?>
'] );

		$foreignPeerClassName = 'DB_<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
Peer';

		$result = $fpeer->selectByCriteria($criteria);
		return $result;
	}

	public function get<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
($criteria0 = null){
		if($criteria0 == null && is_array($this->prefetched)){
			if(in_array('<?php echo $this->_tpl_vars['mrel']['foreignTableName']; ?>
', $this->prefetched)){
				if(in_array('<?php echo $this->_tpl_vars['mrel']['foreignTableName']; ?>
', $this->prefetchedObjects)){
					return $this->prefetchedObjects['<?php echo $this->_tpl_vars['mrel']['foreignTableName']; ?>
'];
				} else {
					$obj = new DB_<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['<?php echo $this->_tpl_vars['mrel']['foreignTableName']; ?>
'] = $obj;
					return $obj;

				}
			}
		}
		$foreignPeerClassName = 'DB_<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
Peer';
		$fpeer = new $foreignPeerClassName();

		if($criteria0 == null){
			$criteria = new Criteria();
		} else {
			$criteria = clone($criteria0);
		}

		$criteria->addAnd("<?php echo $this->_tpl_vars['mrel']['foreignKeyName']; ?>
",$this->fieldValues['<?php echo $this->_tpl_vars['mrel']['localKeyName']; ?>
'] );

		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}

	public function add<?php echo $this->_tpl_vars['mrel']['foreignTmp']; ?>
($referencingObject){
		$referencingObject->setFieldValue('<?php echo $this->_tpl_vars['mrel']['foreignKeyName']; ?>
', $this->fieldValues['<?php echo $this->_tpl_vars['mrel']['localKeyName']; ?>
']);
		// not save!
	}

	<?php endforeach; endif; unset($_from); ?>

	<?php if (count($_from = (array)$this->_tpl_vars['foreignRelations'])):
    foreach ($_from as $this->_tpl_vars['frel']):
?>
	<?php if ($this->_tpl_vars['frel']['customFunction'] != null): ?>
	public function get<?php echo $this->_tpl_vars['frel']['customFunction']; ?>
(){
	<?php else: ?>
	public function get<?php echo $this->_tpl_vars['frel']['foreignTmp']; ?>
(){
	<?php endif; ?>
		if(is_array($this->prefetched)){
			if(in_array('<?php echo $this->_tpl_vars['frel']['foreignTableName']; ?>
', $this->prefetched)){
				if(in_array('<?php echo $this->_tpl_vars['frel']['foreignTableName']; ?>
', $this->prefetchedObjects)){
					return $this->prefetchedObjects['<?php echo $this->_tpl_vars['frel']['foreignTableName']; ?>
'];
				} else {

					$obj = new DB_<?php echo $this->_tpl_vars['frel']['foreignTmp']; ?>
($this->sourceRow);
					$obj->setNew(false);
					//$obj->prefetched = $this->prefetched;
					//$obj->sourceRow = $this->sourceRow;
					$this->prefetchedObjects['<?php echo $this->_tpl_vars['frel']['foreignTableName']; ?>
'] = $obj;
					return $obj;
				}
			}
		}
		$foreignPeerClassName = 'DB_<?php echo $this->_tpl_vars['frel']['foreignTmp']; ?>
Peer';
		$fpeer = new $foreignPeerClassName();

		$criteria = new Criteria();

		$criteria->add("<?php echo $this->_tpl_vars['frel']['foreignKeyName']; ?>
", $this->fieldValues['<?php echo $this->_tpl_vars['frel']['localKeyName']; ?>
']);

		$result = $fpeer->selectOneByCriteria($criteria);
		return $result;
	}

	<?php if ($this->_tpl_vars['frel']['customFunction'] != null): ?>
	public function set<?php echo $this->_tpl_vars['frel']['customFunction']; ?>
(){
	<?php else: ?>
	public function set<?php echo $this->_tpl_vars['frel']['foreignTmp']; ?>
($primaryObject){
	<?php endif; ?>
		$this->fieldValues['<?php echo $this->_tpl_vars['frel']['localKeyName']; ?>
'] = $primaryObject->getFieldValue('<?php echo $this->_tpl_vars['frel']['foreignKeyName']; ?>
');
	}
	<?php endforeach; endif; unset($_from); ?>


	<?php $this->_foreach['gsetters'] = array('total' => count($_from = (array)$this->_tpl_vars['columns']), 'iteration' => 0);
if ($this->_foreach['gsetters']['total'] > 0):
    foreach ($_from as $this->_tpl_vars['col']):
        $this->_foreach['gsetters']['iteration']++;
?>

	public function get<?php echo $this->_tpl_vars['col']->getPropertyNameFirstCapitalized(); ?>
() {
		return $this->getFieldValue('<?php echo $this->_tpl_vars['col']->getName(); ?>
');
	}

	public function set<?php echo $this->_tpl_vars['col']->getPropertyNameFirstCapitalized(); ?>
($v1, $raw=false) {
		$this->setFieldValue('<?php echo $this->_tpl_vars['col']->getName(); ?>
', $v1, $raw);
	}

	<?php endforeach; endif; unset($_from); ?>



}
