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
 * @category Ozone
 * @package Ozone_Db
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

/**
 * Database view generator.
 *
 */
class DBGeneratorView{
	
	private $name;
	private $columnsNames = array();
	private $selectQuery;	
	
	public function __construct($view_xml){
		$this->name = $view_xml['name'];
		foreach ($view_xml->column as $column) {
			$cname = $column['name'];
			$this->columnNames[]=$cname;
		}
		$this->selectQuery = $view_xml->select[0];

	}
	
	public function generateSQLCreateString(){
		$out = "CREATE OR REPLACE VIEW ".$this->name." ";	
		$cquery1 = '';
		
		$isFirst = true;
		foreach($this->columnNames as $column){
			if(!$isFirst){
				$cquery1 .=", ";
			} else {
				$isFirst = false;
			}
			
			$cquery1 .= "$column";
		}
		
		$out .= " (".$cquery1 . " ) AS ". $this->selectQuery .";";
		return $out;
	}

	public function getName(){
		return $this->name;
	}
	
	public function generateClass(){
		echo "generating classes for ".$this->name." view\n";
		$smarty = new OzoneSmarty();
		$smarty->assign('stringHelper', new StringHelper());
		$smarty->left_delimiter = '<{';
		$smarty->right_delimiter = '}>';
		$smarty->assign('className', $this->getNameLowercaseFirstCapitalized());
		
		$smarty->assign('tableName', $this->name);
		
		// put columns into context
		$smarty->assign('columns', $this->columnNames);

		// peer name
		$peerName = "DB_".$this->getNameLowercaseFirstCapitalized()."Peer";
		$smarty->assign('peerName', $peerName);

		$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ViewBaseTemplate.tpl";
		$out = $smarty->fetch($templateFile);	
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'Base';
		file_put_contents(PathManager::dbClass('/base/'.$cn), $out);
		
		//see if file exists!
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized();
		if(!file_exists(PathManager::dbClass($cn))){
		
			$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ViewTemplate.tpl";
			$out = $smarty->fetch($templateFile);
			file_put_contents(PathManager::dbClass($cn), $out);
		}
		
		$objectName = "DB_".$this->getNameLowercaseFirstCapitalized();
		$smarty->assign('objectName', $objectName);
		
		$templateFilePeer = OZONE_ROOT ."/files/dbtemplates/DB_ViewPeerBaseTemplate.tpl";
		$out = $smarty->fetch($templateFilePeer);	
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'PeerBase';
		file_put_contents(PathManager::dbClass('/base/'.$cn), $out);
		
		//see if file exists!
		$cn = 'DB_'.$this->getNameLowercaseFirstCapitalized().'Peer';
		if(!file_exists(PathManager::dbClass($cn))){
			$templateFile = OZONE_ROOT ."/files/dbtemplates/DB_ViewPeerTemplate.tpl";
			$out = $smarty->fetch($templateFile);
			file_put_contents(PathManager::dbClass($cn), $out);
		}
	}

	public function getNameLowercase(){
		return underscoreToLowerCase($this->name);
	}
	
	public function getNameLowercaseFirstCapitalized(){
		return capitalizeFirstLetter(underscoreToLowerCase($this->name));
	}
	
	public function getPkColumnName(){
		return $this->pkColumnName;	
	}
	
	public function getPkColumn(){
		$pkColumnName =$this->pkColumnName;
		return $this->columns["$pkColumnName"];	
	}
}
