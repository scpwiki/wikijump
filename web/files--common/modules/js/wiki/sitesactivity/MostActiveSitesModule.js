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

WIKIDOT.modules.MostActiveSitesModule = {};

WIKIDOT.modules.MostActiveSitesModule.vars = {}

WIKIDOT.modules.MostActiveSitesModule.listeners = {
	changeTime: function(e, range){
		if(!WIKIDOT.modules.MostActiveSitesModule.vars['box']){
			var box = YAHOO.util.Event.getTarget(e);
			do{
				box = box.parentNode;
			}while(box && box.className != 'most-active-sites-box')
			WIKIDOT.modules.MostActiveSitesModule.vars['box'] = box;
		}
		
		OZONE.ajax.requestModule('wiki/sitesactivity/MostActiveSitesModule', {range: range}, WIKIDOT.modules.MostActiveSitesModule.callbacks.changeTime);
		
	}	
	
}

WIKIDOT.modules.MostActiveSitesModule.callbacks = {
	changeTime: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		WIKIDOT.modules.MostActiveSitesModule.vars['box'].innerHTML = r.body.replace(/<div[^>]>/, '').replace(/<\/div>\s*$/, '');
	}	
}
