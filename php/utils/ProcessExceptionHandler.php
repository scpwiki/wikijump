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

/** 
 * This class is responsible for handling exceptions which are thrown
 * when processing modules/screens.
 */
class ProcessExceptionHandler {
	
	public function handleInlineModule($exception, $runData){
		// rollback the transaction
		$db = Database::connection();
		$db->rollback();
		$out.= '<div class="error-block">';
		if($exception instanceof ProcessException){
			$out.=nl2br($exception->getMessage());	
		}elseif($exception instanceof WDPermissionException){
			$out.='<div class="title">Permission error</div>';
			$out.=nl2br($exception->getMessage());	
		}else{
			$out.="An error occured when processing your request.";
			// LOG ERROR TOO!!!
			$logger = OzoneLogger::instance();
			$logger->error("Exception caught while processing inline module:\n\n".$exception->__toString());
		}
		$out.='</div>';
		return $out;
	}
	
	public function handleAjaxRequest($exception, $runData){
		$db = Database::connection();
		$db->rollback();
		if($exception instanceof ProcessException){
			$runData->ajaxResponseAdd("message",$exception->getMessage());
			$runData->ajaxResponseAdd("status", $exception->getStatus());	
		}elseif($exception instanceof WBPermissionException){
			
		}
		else{
			$runData->ajaxResponseAdd("message","An error occured when processing your request.");
			$runData->ajaxResponseAdd("status", "not_ok");	
		}
	}
	
}
