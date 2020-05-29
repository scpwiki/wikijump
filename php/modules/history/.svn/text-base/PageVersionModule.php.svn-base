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

class PageVersionModule extends SmartyModule {
	
	public function build($runData){
		$revisionId = $runData->getParameterList()->getParameterValue("revision_id");
		
		$revision = DB_PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
		if($revision == null){
			throw new ProcessException(_("Revision error"), "revision_error");	
		}
		
		$metadata = $revision->getMetadata();
		
		$source = $revision->getSourceText();	
		
		$tr = new WikiTransformation();
		$content = $tr->processSource($source);
		
		$runData->contextAdd("pageContent", $content);
		$runData->contextAdd("revision", $revision);
		$runData->contextAdd("metadata", $metadata);
		$runData->ajaxResponseAdd("title", $metadata->getTitle());

	}
	
}
