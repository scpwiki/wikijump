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
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class Backuper {
	
	private $siteId;
	
	private $backupSource = true;
	private $backupFiles = true;
	private $rand;

	public function setConfig($sb){
		$this->backupSource = $sb->getBackupSource();
		$this->backupFiles = $sb->getBackupFiles();
		$this->siteId = $sb->getSiteId();	
	}
	
	public function backup(){
		
		$site = DB_SitePeer::instance()->selectByPrimaryKey($this->siteId);
		if(!$site){
			throw new ProcessException(_("Site can not be found"));	
		}
		
		// prepare working directory
		
		$wdir = WIKIDOT_ROOT.'/tmp/sitebackups/'.$site->getUnixName().'/work';
		@exec('rm -r '.$wdir.' &> /dev/null');
		mkdirfull($wdir);

		if($this->backupSource){
			mkdirfull($wdir.'/source');
			// iterate through pages
			
			$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$pages = DB_PagePeer::instance()->select($c);
			
			foreach($pages as $page){
				$source = $page->getCurrentRevision()->getSourceText();
				$filename = $page->getUnixName().'.txt';
				$filename = str_replace(':', '_', $filename);
				file_put_contents($wdir.'/source/'.$filename, $source);	
			}
				
		}
		if($this->backupFiles){
			mkdirfull($wdir.'/files');
		/*	$c = new Criteria();
			$c->add("site_id", $site->getSiteId());
			$pages = DB_PagePeer::instance()->select($c);
			
			foreach($pages as $page){
				// get the files
				$c = new Criteria();
				$c->add("page_id", $page->getPageId());
				$files = DB_FilePeer::instance()->select($c);
				if(count($files)>0){
					$path = $wdir.'/files/'.$page->getUnixName();
					mkdirfull($path);
					foreach($files as $file){
						copy($file->getFilePath(), $path);
					}
				}
			}	*/
			
			$path0 = WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/files/';
			$cmd = "cp -r ".$path0.'*'.' '.$wdir.'/files/'.' &> /dev/null'; 
			
			@exec($cmd);
			// fix colon:
			$dirstmp = ls($wdir.'/files/', '*:*');
			foreach($dirstmp as $dd){
				@rename($wdir.'/files/'.$dd, $wdir.'/files/'.str_replace(':', '_', $dd));
			}	
					
		}
		
		// zip the content
		$cmd = 'cd '.$wdir.' && zip -r backup *';
		exec($cmd);
		
		$zipfile = $wdir.'/backup.zip';
		if(!file_exists($zipfile)){
			throw new ProcessException("Error creating backup.");
		}	
		// dest dir
		@exec('rm -r '.WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup/'.' &> /dev/null');
		$rand = md5(rand(10000,99999).time());
		$ddir = WIKIDOT_ROOT.'/web/files--sites/'.$site->getUnixName().'/backup/'.$rand.'/';
		mkdirfull($ddir);
		
		copy($zipfile, $ddir.'backup.zip');
		
		// clear the working dir
		@exec('rm -r '.escapeshellarg($wdir).' &> /dev/null');
		
		$this->rand = $rand;

	}

	public function setSiteId($siteId){
		$this->siteId = $siteId;	
	}
	
	public function setBackupSource($val){
		$this->backupSource = $val;	
	}
	public function setBackupFiles($val){
		$this->backupFiles = $val;	
	}
	
	public function getRand(){
		return $this->rand;	
	}

}
