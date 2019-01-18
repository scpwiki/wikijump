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

WIKIDOT.modules.HelloWorldModule = {};

WIKIDOT.modules.HelloWorldModule.listener = {
	click: function(e){
		alert('clicked');
		var p = new Object();
		p.action = "NewSiteAction";
		p.event = "..."
		OZONE.ajax.requestModule("test/HelloWorld2Module", p,WIKIDOT.modules.HelloWorldModule.callbacks.click );
	}	
	
}

WIKIDOT.modules.HelloWorldModule.callbacks = {
	
	click: function(r){
		$("hello-world-2-box").innerHTML = r.body;
		
	}	
}