/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.MostActiveForumsModule = {};

WIKIDOT.modules.MostActiveForumsModule.vars = {}

WIKIDOT.modules.MostActiveForumsModule.listeners = {
	changeTime: function(e, range){
		if(!WIKIDOT.modules.MostActiveForumsModule.vars['box']){
			var box = YAHOO.util.Event.getTarget(e);
			do{
				box = box.parentNode;
			}while(box && box.className != 'most-active-forums-box')
			WIKIDOT.modules.MostActiveForumsModule.vars['box'] = box;
		}
		
		OZONE.ajax.requestModule('wiki/sitesactivity/MostActiveForumsModule', {range: range}, WIKIDOT.modules.MostActiveForumsModule.callbacks.changeTime);
		
	}	
	
}

WIKIDOT.modules.MostActiveForumsModule.callbacks = {
	changeTime: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		WIKIDOT.modules.MostActiveForumsModule.vars['box'].innerHTML = r.body.replace(/<div[^>]*>/, '').replace(/<\/div>\s*$/, '');
	}	
}